//! Capacity-based translator routing
//!
//! This module provides intelligent routing based on translator capacity.
//! Routes texts to the most appropriate translator based on text length.

use std::sync::Arc;

use crate::core::error::{Result, TranslateError};
use crate::translator::{Translator, TranslatorImpl};

/// Translator with capacity information
#[derive(Debug, Clone)]
pub struct CapacityTranslator {
    translator: Arc<TranslatorImpl>,
    max_chars: usize,
    priority: u32,
}

impl CapacityTranslator {
    /// Create a new capacity-aware translator wrapper
    pub fn new(translator: Arc<TranslatorImpl>, priority: u32) -> Self {
        let max_chars = translator.max_input_chars();
        Self {
            translator,
            max_chars,
            priority,
        }
    }

    /// Check if this translator can handle the given text length
    pub fn can_handle(&self, text_len: usize) -> bool {
        self.max_chars == 0 || text_len <= self.max_chars
    }

    /// Get the translator
    pub fn translator(&self) -> &Arc<TranslatorImpl> {
        &self.translator
    }

    /// Get maximum characters
    pub fn max_chars(&self) -> usize {
        self.max_chars
    }

    /// Get priority (lower = higher priority)
    pub fn priority(&self) -> u32 {
        self.priority
    }
}

/// Capacity-based router for translators
///
/// Routes texts to the most appropriate translator based on:
/// 1. Text length (must fit within translator's capacity)
/// 2. Priority (lower number = higher priority)
///
/// For LLM providers with multiple models, each model is treated as a separate
/// translator with its own capacity.
pub struct CapacityRouter {
    translators: Vec<CapacityTranslator>,
}

impl CapacityRouter {
    /// Create a new capacity router
    pub fn new(translators: Vec<(Arc<TranslatorImpl>, u32)>) -> Self {
        let capacity_translators: Vec<_> = translators
            .into_iter()
            .map(|(t, p)| CapacityTranslator::new(t, p))
            .collect();

        Self {
            translators: capacity_translators,
        }
    }

    /// Select the best translator for the given text length
    ///
    /// Selection strategy:
    /// 1. Filter translators that can handle the text length
    /// 2. Select the one with highest priority (lowest priority number)
    /// 3. If multiple have same priority, select the one with smallest capacity
    ///    (to reserve larger capacity translators for longer texts)
    pub fn select_translator(&self, text_len: usize) -> Option<&Arc<TranslatorImpl>> {
        let candidates: Vec<&CapacityTranslator> = self
            .translators
            .iter()
            .filter(|t| t.can_handle(text_len))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Find the highest priority (lowest number)
        let min_priority = candidates.iter().map(|t| t.priority()).min().unwrap_or(0);

        // Among same priority, select the one with smallest sufficient capacity
        candidates
            .into_iter()
            .filter(|t| t.priority() == min_priority)
            .min_by_key(|t| t.max_chars())
            .map(|t| t.translator())
    }

    /// Get all translators sorted by capacity (ascending)
    pub fn get_translators_by_capacity(&self) -> Vec<&Arc<TranslatorImpl>> {
        let mut sorted: Vec<_> = self.translators.iter().collect();
        sorted.sort_by_key(|t| t.max_chars());
        sorted.into_iter().map(|t| t.translator()).collect()
    }

    /// Get the maximum capacity among all translators
    pub fn max_capacity(&self) -> usize {
        self.translators
            .iter()
            .map(|t| t.max_chars())
            .max()
            .unwrap_or(0)
    }

    /// Check if any translator can handle the given text length
    pub fn can_handle(&self, text_len: usize) -> bool {
        self.translators.iter().any(|t| t.can_handle(text_len))
    }

    /// Route and translate a single text
    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let text_len = text.len();

        let translator = self.select_translator(text_len).ok_or_else(|| {
            TranslateError::Translation(format!(
                "No translator can handle text of length {}. Maximum capacity: {}",
                text_len,
                self.max_capacity()
            ))
        })?;

        translator
            .translate_single(text, source_lang, target_lang)
            .await
    }

    /// Route and translate multiple texts
    pub async fn translate_batch(
        &self,
        texts: &[String],
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let translated = self.translate(text, "AUTO", target_lang).await?;
            results.push(translated);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require mock translators
    // For now, we just verify the structure compiles

    #[test]
    fn test_capacity_translator_can_handle() {
        // This is a placeholder test
        // Real tests would use mock translators
    }
}
