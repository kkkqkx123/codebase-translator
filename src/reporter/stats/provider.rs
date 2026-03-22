use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatorStats {
    pub translator_type: String,
    pub total_calls: usize,
    pub successful_calls: usize,
    pub failed_calls: usize,
    pub total_chars: usize,
    pub average_latency_ms: f64,
    pub last_call_time: Option<DateTime<Utc>>,
    pub min_latency_ms: Option<f64>,
    pub max_latency_ms: Option<f64>,
}

impl TranslatorStats {
    pub fn new(translator_type: String) -> Self {
        Self {
            translator_type,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_chars: 0,
            average_latency_ms: 0.0,
            last_call_time: None,
            min_latency_ms: None,
            max_latency_ms: None,
        }
    }

    pub fn record_call(&mut self, latency_ms: u64, success: bool, chars: usize) {
        self.total_calls += 1;
        self.total_chars += chars;
        self.last_call_time = Some(Utc::now());

        let latency = latency_ms as f64;

        if success {
            self.successful_calls += 1;

            // Use incremental averaging to reduce floating-point precision issues
            // NewAvg = OldAvg + (NewValue - OldAvg) / Count
            let delta = latency - self.average_latency_ms;
            self.average_latency_ms += delta / self.successful_calls as f64;

            if let Some(min) = self.min_latency_ms {
                self.min_latency_ms = Some(min.min(latency));
            } else {
                self.min_latency_ms = Some(latency);
            }

            if let Some(max) = self.max_latency_ms {
                self.max_latency_ms = Some(max.max(latency));
            } else {
                self.max_latency_ms = Some(latency);
            }
        } else {
            self.failed_calls += 1;
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        (self.failed_calls as f64 / self.total_calls as f64) * 100.0
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        (self.successful_calls as f64 / self.total_calls as f64) * 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMProviderStats {
    pub provider_id: String,
    pub provider_name: String,
    pub model: String,
    pub total_calls: usize,
    pub successful_calls: usize,
    pub failed_calls: usize,
    pub total_chars: usize,
    pub average_latency_ms: f64,
    pub last_call_time: Option<DateTime<Utc>>,
    pub min_latency_ms: Option<f64>,
    pub max_latency_ms: Option<f64>,
}

impl LLMProviderStats {
    pub fn new(provider_id: String, provider_name: String, model: String) -> Self {
        Self {
            provider_id,
            provider_name,
            model,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_chars: 0,
            average_latency_ms: 0.0,
            last_call_time: None,
            min_latency_ms: None,
            max_latency_ms: None,
        }
    }

    pub fn record_call(&mut self, latency_ms: u64, success: bool, chars: usize) {
        self.total_calls += 1;
        self.total_chars += chars;
        self.last_call_time = Some(Utc::now());

        let latency = latency_ms as f64;

        if success {
            self.successful_calls += 1;

            let total_latency = self.average_latency_ms * (self.successful_calls - 1) as f64;
            self.average_latency_ms = (total_latency + latency) / self.successful_calls as f64;

            if let Some(min) = self.min_latency_ms {
                self.min_latency_ms = Some(min.min(latency));
            } else {
                self.min_latency_ms = Some(latency);
            }

            if let Some(max) = self.max_latency_ms {
                self.max_latency_ms = Some(max.max(latency));
            } else {
                self.max_latency_ms = Some(latency);
            }
        } else {
            self.failed_calls += 1;
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        (self.failed_calls as f64 / self.total_calls as f64) * 100.0
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        (self.successful_calls as f64 / self.total_calls as f64) * 100.0
    }
}
