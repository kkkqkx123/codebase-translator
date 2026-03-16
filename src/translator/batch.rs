//! Batch translator with rate limiting
//!
//! This module provides batch translation with rate limiting and retry logic.
//! Uses static dispatch via TranslatorImpl enum for better performance.

use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{error, info, warn};

use crate::core::error::{Result, TranslateError};
use crate::translator::common::{BatchOptions, BatchResult, LimitPolicy, TranslateResponse};
use crate::translator::{Translator, TranslatorImpl};

/// Batch translator with rate limiting
/// Uses static dispatch via TranslatorImpl for better performance.
pub struct BatchTranslator {
    translator: Arc<TranslatorImpl>,
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
}

impl BatchTranslator {
    /// Create a new batch translator
    pub fn new(translator: Arc<TranslatorImpl>, options: BatchOptions) -> Self {
        let limit_policy = options.limit_policy.unwrap_or_default();

        let rate_limiter = if limit_policy.rate_limit > 0 {
            let quota = Quota::per_second(
                NonZeroU32::new(limit_policy.rate_limit.max(1)).expect("max(1) is always non-zero"),
            );
            Some(RateLimiter::direct(quota))
        } else {
            None
        };

        let semaphore = Arc::new(Semaphore::new(options.workers.max(1)));

        Self {
            translator,
            rate_limiter: Arc::new(RwLock::new(rate_limiter)),
            semaphore,
            max_retries: options.max_retries.max(1),
            limit_policy,
        }
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

    /// Translate a batch of texts
    pub async fn translate_batch(
        &self,
        texts: &[String],
        target_lang: &str,
    ) -> Result<BatchResult> {
        let start_time = Instant::now();
        let total_count = texts.len();
        let mut results = Vec::with_capacity(total_count);
        let mut errors = Vec::new();
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut total_chars = 0;
        let mut total_latency_ms = 0u64;

        for text in texts {
            let text_start = Instant::now();
            total_chars += text.len();

            let permit = self.semaphore.clone().acquire_owned().await.map_err(|e| {
                TranslateError::Translation(format!("Failed to acquire semaphore: {}", e))
            })?;

            {
                let limiter = self.rate_limiter.read().await;
                if let Some(ref limiter) = *limiter {
                    limiter.until_ready().await;
                }
            }

            let result = self.translate_with_retry(text, target_lang).await;

            let latency = text_start.elapsed().as_millis() as u64;
            total_latency_ms += latency;

            match result {
                Ok(response) => {
                    results.push(response);
                    success_count += 1;
                }
                Err(e) => {
                    error!("Translation failed: {}", e);
                    errors.push(e.to_string());
                    results.push(TranslateResponse {
                        original_text: text.clone(),
                        translated_text: text.clone(),
                        source_lang: String::new(),
                        target_lang: target_lang.to_string(),
                        alternatives: Vec::new(),
                    });
                    failed_count += 1;
                }
            }

            drop(permit);
        }

        let processing_time = start_time.elapsed().as_millis() as u64;
        let average_latency_ms = if success_count > 0 {
            total_latency_ms as f64 / success_count as f64
        } else {
            0.0
        };

        info!(
            "Batch translation completed: total={}, success={}, failed={}, time={}ms, avg_latency={}ms",
            total_count, success_count, failed_count, processing_time, average_latency_ms
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
        })
    }

    /// Translate with exponential backoff retry
    async fn translate_with_retry(
        &self,
        text: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            // Check character limit and split if needed
            if self.limit_policy.max_char_count > 0 && text.len() > self.limit_policy.max_char_count
            {
                return Box::pin(self.translate_with_split(text, target_lang)).await;
            }

            match self
                .translator
                .translate(&[text.to_string()], target_lang)
                .await
            {
                Ok(translated) => {
                    if let Some(translated_text) = translated.first() {
                        return Ok(TranslateResponse {
                            original_text: text.to_string(),
                            translated_text: translated_text.clone(),
                            source_lang: String::new(),
                            target_lang: target_lang.to_string(),
                            alternatives: Vec::new(),
                        });
                    }
                }
                Err(e) => {
                    warn!("Translation attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);

                    // Exponential backoff
                    if attempt < self.max_retries - 1 {
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                        tokio::time::sleep(delay).await;
                    }
                }
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
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let chunks = self.split_text_hierarchical(text);
        let mut translated_chunks = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let result = self.translate_with_retry(&chunk, target_lang).await?;
            translated_chunks.push(result.translated_text);
        }

        Ok(TranslateResponse {
            original_text: text.to_string(),
            translated_text: translated_chunks.join(""),
            source_lang: String::new(),
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

    /// Get translator name
    pub fn name(&self) -> &str {
        self.translator.name()
    }
}

/// Create a batch translator from a translator instance
/// Uses static dispatch via TranslatorImpl for better performance.
pub fn create_batch_translator(
    translator: Arc<TranslatorImpl>,
    options: BatchOptions,
) -> BatchTranslator {
    BatchTranslator::new(translator, options)
}
