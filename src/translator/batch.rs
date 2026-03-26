//! Batch translator with rate limiting
//!
//! This module provides batch translation with rate limiting and retry logic.
//! Uses static dispatch via TranslatorImpl for better performance.
//! Supports multiple translators with load balancing and failover.

use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, error, info, warn};

use crate::core::error::{Result, TranslateError};
use crate::reporter::stats::SharedStats;
use crate::translator::common::{BatchOptions, BatchResult, LimitPolicy, TranslateResponse};
use crate::translator::{Translator, TranslatorImpl};

/// Translator entry with metadata for load balancing
#[derive(Debug)]
struct TranslatorEntry {
    translator: Arc<TranslatorImpl>,
    name: String,
    healthy: AtomicU64,
    failure_count: AtomicU64,
}

impl TranslatorEntry {
    fn new(translator: Arc<TranslatorImpl>) -> Self {
        let name = translator.name().to_string();
        Self {
            translator,
            name,
            healthy: AtomicU64::new(1),
            failure_count: AtomicU64::new(0),
        }
    }

    fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Relaxed) == 1
    }

    fn mark_healthy(&self) {
        self.healthy.store(1, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
    }

    fn mark_unhealthy(&self) {
        self.healthy.store(0, Ordering::Relaxed);
    }

    fn increment_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= 1 {
            self.mark_unhealthy();
        }
    }
}

/// Batch translator with rate limiting and multi-provider support
/// Uses static dispatch via TranslatorImpl for better performance.
pub struct BatchTranslator {
    translators: Vec<TranslatorEntry>,
    current_index: AtomicU64,
    rate_limiter: Arc<
        RwLock<
            Option<
                RateLimiter<
                    governor::state::direct::NotKeyed,
                    governor::state::InMemoryState,
                    governor::clock::DefaultClock,
                >,
            >,
        >,
    >,
    semaphore: Arc<Semaphore>,
    max_retries: usize,
    limit_policy: LimitPolicy,
    shared_stats: Option<Arc<SharedStats>>,
    batch_size: usize,
}

impl BatchTranslator {
    /// Create a new batch translator with multiple translators
    pub fn new(translators: Vec<Arc<TranslatorImpl>>, options: BatchOptions) -> Self {
        Self::new_with_stats(translators, options, None)
    }

    /// Create a new batch translator with multiple translators and shared stats
    pub fn new_with_stats(
        translators: Vec<Arc<TranslatorImpl>>,
        options: BatchOptions,
        shared_stats: Option<Arc<SharedStats>>,
    ) -> Self {
        debug!(
            translator_count = translators.len(),
            "Creating batch translator"
        );
        let limit_policy = options.limit_policy.unwrap_or_default();

        debug!(
            rate_limit = limit_policy.rate_limit,
            workers = options.workers,
            max_retries = options.max_retries,
            "Batch translator configuration"
        );

        let rate_limiter = if limit_policy.rate_limit > 0 {
            let quota = Quota::per_second(
                NonZeroU32::new(limit_policy.rate_limit.max(1)).expect("max(1) is always non-zero"),
            );
            Some(RateLimiter::direct(quota))
        } else {
            None
        };

        let semaphore = Arc::new(Semaphore::new(options.workers.max(1)));

        let translator_entries: Vec<TranslatorEntry> =
            translators.into_iter().map(TranslatorEntry::new).collect();

        info!(
            translator_count = translator_entries.len(),
            rate_limiter_enabled = rate_limiter.is_some(),
            semaphore_permits = options.workers.max(1),
            "Batch translator created with multiple providers"
        );

        Self {
            translators: translator_entries,
            current_index: AtomicU64::new(0),
            rate_limiter: Arc::new(RwLock::new(rate_limiter)),
            semaphore,
            max_retries: options.max_retries.max(1),
            limit_policy,
            shared_stats,
            batch_size: options.batch_size.max(1),
        }
    }

    /// Set shared stats for collecting translator statistics
    pub fn set_shared_stats(&mut self, shared_stats: Arc<SharedStats>) {
        self.shared_stats = Some(shared_stats);
    }

    /// Set rate limit dynamically
    pub async fn set_rate_limit(&self, requests_per_second: u32) {
        let mut limiter = self.rate_limiter.write().await;
        if requests_per_second > 0 {
            let quota = Quota::per_second(
                NonZeroU32::new(requests_per_second.max(1)).expect("max(1) is always non-zero"),
            );
            *limiter = Some(RateLimiter::direct(quota));
        } else {
            *limiter = None;
        }
    }

    /// Update limit policy
    pub async fn update_limit_policy(&self, new_policy: LimitPolicy) {
        let mut limiter = self.rate_limiter.write().await;
        if new_policy.rate_limit > 0 {
            let quota = Quota::per_second(
                NonZeroU32::new(new_policy.rate_limit.max(1)).expect("max(1) is always non-zero"),
            );
            *limiter = Some(RateLimiter::direct(quota));
        } else {
            *limiter = None;
        }
    }

    /// Select next healthy translator using simple round-robin
    fn select_translator(&self) -> Option<&TranslatorEntry> {
        let healthy_translators: Vec<&TranslatorEntry> =
            self.translators.iter().filter(|t| t.is_healthy()).collect();

        if healthy_translators.is_empty() {
            // If no healthy translators, try all translators
            let total = self.translators.len();
            let index = self.current_index.fetch_add(1, Ordering::Relaxed) as usize % total;
            return self.translators.get(index);
        }

        // Simple round-robin selection
        let index =
            self.current_index.fetch_add(1, Ordering::Relaxed) as usize % healthy_translators.len();
        healthy_translators.get(index).copied()
    }

    /// Translate a batch of texts
    ///
    /// This method processes texts in batches according to the configured batch_size.
    /// Each batch is sent to the translator as a single request, improving efficiency.
    pub async fn translate_batch(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<BatchResult> {
        let start_time = Instant::now();
        let total_count = texts.len();
        let mut results = Vec::with_capacity(total_count);
        let mut errors = Vec::new();
        let mut success_count: usize = 0;
        let mut failed_count: usize = 0;
        let mut total_chars = 0;
        let mut total_latency_ms = 0u64;

        // Process texts in batches
        let chunks: Vec<&[String]> = texts.chunks(self.batch_size).collect();
        let total_batches = chunks.len();

        info!(
            total_texts = total_count,
            batch_size = self.batch_size,
            total_batches = total_batches,
            "Starting batch translation"
        );

        for (batch_idx, batch) in chunks.iter().enumerate() {
            let batch_start = Instant::now();
            let batch_chars: usize = batch.iter().map(|t| t.len()).sum();
            total_chars += batch_chars;

            // Acquire semaphore permit for concurrency control
            let permit = self.semaphore.clone().acquire_owned().await.map_err(|e| {
                error!(error = %e, "Failed to acquire semaphore");
                TranslateError::Translation(format!("Failed to acquire semaphore: {}", e))
            })?;

            // Apply rate limiting
            {
                let limiter = self.rate_limiter.read().await;
                if let Some(ref limiter) = *limiter {
                    limiter.until_ready().await;
                }
            }

            // Translate the batch
            let batch_result = self
                .translate_batch_chunk(batch, source_lang, target_lang)
                .await;

            let batch_latency = batch_start.elapsed().as_millis() as u64;
            total_latency_ms += batch_latency;

            match batch_result {
                Ok(batch_responses) => {
                    for response in batch_responses {
                        if response.translated_text != response.original_text {
                            success_count += 1;
                        } else {
                            // If translation returned same text, count as success but log warning
                            debug!("Translation returned same text");
                            success_count += 1;
                        }
                        results.push(response);
                    }
                    debug!(
                        batch_idx = batch_idx + 1,
                        total_batches = total_batches,
                        batch_size = batch.len(),
                        "Batch completed successfully"
                    );
                }
                Err(e) => {
                    error!(
                        batch_idx = batch_idx + 1,
                        total_batches = total_batches,
                        error = %e,
                        "Batch translation failed"
                    );
                    errors.push(format!("Batch {}: {}", batch_idx + 1, e));

                    // For failed batch, add fallback responses for each text
                    for text in batch.iter() {
                        results.push(TranslateResponse {
                            original_text: text.clone(),
                            translated_text: text.clone(),
                            source_lang: source_lang.to_string(),
                            target_lang: target_lang.to_string(),
                            alternatives: Vec::new(),
                        });
                        failed_count += 1;
                    }
                }
            }

            drop(permit);
        }

        // Adjust success_count to not double count (it was counting per text, but we need to subtract failed)
        success_count = success_count.saturating_sub(failed_count);

        let processing_time = start_time.elapsed().as_millis() as u64;
        let average_latency_ms = if total_batches > 0 {
            total_latency_ms as f64 / total_batches as f64
        } else {
            0.0
        };

        info!(
            total_count = total_count,
            success_count = success_count,
            failed_count = failed_count,
            total_batches = total_batches,
            processing_time_ms = processing_time,
            average_latency_ms = average_latency_ms,
            total_chars = total_chars,
            "Batch translation completed"
        );

        Ok(BatchResult {
            total_count,
            success_count,
            failed_count,
            results,
            errors,
            processing_time,
            total_chars,
            total_tokens: 0,
            average_latency_ms,
            total_batches,
        })
    }

    /// Translate a batch chunk (subset of texts)
    async fn translate_batch_chunk(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<TranslateResponse>> {
        let mut responses = Vec::with_capacity(texts.len());

        // Try to translate the entire batch at once
        match self
            .translate_batch_request(texts, source_lang, target_lang)
            .await
        {
            Ok(batch_responses) => {
                return Ok(batch_responses);
            }
            Err(e) => {
                warn!(
                    error = %e,
                    batch_size = texts.len(),
                    "Batch translation failed, falling back to individual translations"
                );

                // Fallback: translate each text individually
                for text in texts {
                    match self
                        .translate_with_retry(text, source_lang, target_lang)
                        .await
                    {
                        Ok(response) => {
                            responses.push(response);
                        }
                        Err(e) => {
                            error!(
                                error = %e,
                                text_length = text.len(),
                                "Individual translation failed"
                            );
                            responses.push(TranslateResponse {
                                original_text: text.clone(),
                                translated_text: text.clone(),
                                source_lang: source_lang.to_string(),
                                target_lang: target_lang.to_string(),
                                alternatives: Vec::new(),
                            });
                        }
                    }
                }
            }
        }

        Ok(responses)
    }

    /// Send a batch translation request to the translator
    async fn translate_batch_request(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<TranslateResponse>> {
        let entry = self
            .select_translator()
            .ok_or_else(|| TranslateError::Translation("No translator available".to_string()))?;

        debug!(
            batch_size = texts.len(),
            translator = %entry.name,
            "Sending batch translation request"
        );

        let start_time = Instant::now();
        let batch_chars: usize = texts.iter().map(|t| t.len()).sum();

        match entry
            .translator
            .translate(texts, source_lang, target_lang)
            .await
        {
            Ok(translated_texts) => {
                entry.mark_healthy();

                // Record successful batch translation statistics
                if let Some(ref shared_stats) = self.shared_stats {
                    let latency_ms = start_time.elapsed().as_millis() as u64;
                    shared_stats.record_translator_call(&entry.name, latency_ms, true, batch_chars);
                }

                let responses: Vec<TranslateResponse> = texts
                    .iter()
                    .zip(translated_texts.iter())
                    .map(|(original, translated)| TranslateResponse {
                        original_text: original.clone(),
                        translated_text: translated.clone(),
                        source_lang: source_lang.to_string(),
                        target_lang: target_lang.to_string(),
                        alternatives: Vec::new(),
                    })
                    .collect();

                Ok(responses)
            }
            Err(e) => {
                entry.increment_failure();

                // Record failed batch translation statistics
                if let Some(ref shared_stats) = self.shared_stats {
                    let latency_ms = start_time.elapsed().as_millis() as u64;
                    shared_stats.record_translator_call(
                        &entry.name,
                        latency_ms,
                        false,
                        batch_chars,
                    );
                }

                Err(e)
            }
        }
    }

    /// Translate with exponential backoff retry and failover
    async fn translate_with_retry(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let mut last_error = None;
        let mut attempted_translators = std::collections::HashSet::new();
        let start_time = Instant::now();
        let mut final_translator_type: Option<String> = None;

        for attempt in 0..self.max_retries {
            // Check character limit and split if needed
            if self.limit_policy.max_char_count > 0 && text.len() > self.limit_policy.max_char_count
            {
                return Box::pin(self.translate_with_split(text, source_lang, target_lang)).await;
            }

            // Select a translator that hasn't been attempted yet
            let entry = loop {
                let candidate = self.select_translator();
                match candidate {
                    Some(e) if !attempted_translators.contains(&e.name) => break e,
                    Some(_) => {
                        // This translator was already attempted, try next
                        if attempted_translators.len() >= self.translators.len() {
                            break candidate.unwrap();
                        }
                        continue;
                    }
                    None => {
                        return Err(TranslateError::Translation(
                            "No translator available".to_string(),
                        ));
                    }
                }
            };

            attempted_translators.insert(entry.name.clone());
            final_translator_type = Some(entry.name.clone());

            debug!(
                attempt = attempt + 1,
                max_retries = self.max_retries,
                translator = %entry.name,
                "Attempting translation"
            );

            match entry
                .translator
                .translate(&[text.to_string()], source_lang, target_lang)
                .await
            {
                Ok(translated) => {
                    if let Some(translated_text) = translated.first() {
                        entry.mark_healthy();

                        // Record statistics if shared stats is available
                        if let Some(ref shared_stats) = self.shared_stats {
                            let latency_ms = start_time.elapsed().as_millis() as u64;
                            if let Some(ref translator_type) = final_translator_type {
                                shared_stats.record_translator_call(
                                    translator_type,
                                    latency_ms,
                                    true,
                                    text.len(),
                                );
                            }
                        }

                        return Ok(TranslateResponse {
                            original_text: text.to_string(),
                            translated_text: translated_text.clone(),
                            source_lang: source_lang.to_string(),
                            target_lang: target_lang.to_string(),
                            alternatives: Vec::new(),
                        });
                    }
                }
                Err(e) => {
                    warn!(
                        attempt = attempt + 1,
                        translator = %entry.name,
                        error = %e,
                        "Translation attempt failed"
                    );
                    entry.increment_failure();
                    last_error = Some(e);

                    // Exponential backoff (starting from 1 second)
                    if attempt < self.max_retries - 1 {
                        let delay = Duration::from_millis(1000 * 2_u64.pow(attempt as u32));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // Record failed statistics if shared stats is available
        if let Some(ref shared_stats) = self.shared_stats {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            if let Some(ref translator_type) = final_translator_type {
                shared_stats.record_translator_call(translator_type, latency_ms, false, text.len());
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TranslateError::Translation("All retry attempts failed".to_string())
        }))
    }

    /// Translate with text splitting using hierarchical strategy
    async fn translate_with_split(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let chunks = self.split_text_hierarchical(text);
        let mut translated_chunks = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let result = self
                .translate_with_retry(&chunk, source_lang, target_lang)
                .await?;
            translated_chunks.push(result.translated_text);
        }

        Ok(TranslateResponse {
            original_text: text.to_string(),
            translated_text: translated_chunks.join(""),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            alternatives: Vec::new(),
        })
    }

    /// Split text using hierarchical strategy: paragraph -> sentence -> character
    fn split_text_hierarchical(&self, text: &str) -> Vec<String> {
        let max_chars = self.limit_policy.split_max_chars;

        if text.len() <= max_chars {
            return vec![text.to_string()];
        }

        // Level 1: Split by paragraphs
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut chunks: Vec<String> = Vec::new();

        for paragraph in paragraphs {
            let trimmed = paragraph.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.len() <= max_chars {
                // Paragraph fits, add it directly
                if !chunks.is_empty() && !chunks.last().unwrap().ends_with("\n\n") {
                    chunks.push("\n\n".to_string());
                }
                chunks.push(trimmed.to_string());
            } else {
                // Paragraph too large, split by sentences
                let sentence_chunks = self.split_paragraph_by_sentences(trimmed, max_chars);
                chunks.extend(sentence_chunks);
            }
        }

        chunks
    }

    /// Split a paragraph by sentences, then by characters if needed
    fn split_paragraph_by_sentences(&self, paragraph: &str, max_chars: usize) -> Vec<String> {
        let sentences: Vec<&str> = paragraph.split(['.', '!', '?', '。', '！', '？']).collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for sentence in sentences {
            let trimmed = sentence.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.len() <= max_chars {
                // Sentence fits
                if current_chunk.len() + trimmed.len() + 1 > max_chars {
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.clone());
                    }
                    current_chunk = trimmed.to_string();
                } else {
                    if !current_chunk.is_empty() {
                        current_chunk.push('.');
                    }
                    current_chunk.push_str(trimmed);
                }
            } else {
                // Sentence too large, split by characters (final guarantee)
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                    current_chunk = String::new();
                }
                let char_chunks = self.split_by_chars(trimmed, max_chars);
                chunks.extend(char_chunks);
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        if chunks.is_empty() {
            chunks.push(paragraph.to_string());
        }

        chunks
    }

    /// Split by character count (final guarantee)
    fn split_by_chars(&self, text: &str, max_chars: usize) -> Vec<String> {
        text.chars()
            .collect::<Vec<_>>()
            .chunks(max_chars)
            .map(|chunk| chunk.iter().collect())
            .collect()
    }

    /// Get translator names
    pub fn name(&self) -> String {
        self.translators
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Create a batch translator from multiple translator instances
/// Uses static dispatch via TranslatorImpl for better performance.
pub fn create_batch_translator(
    translators: Vec<Arc<TranslatorImpl>>,
    options: BatchOptions,
) -> BatchTranslator {
    BatchTranslator::new(translators, options)
}
