//! Report generation

pub mod default;
pub mod generator;
pub mod logger;
pub mod progress;
pub mod stats;
pub mod r#trait;

pub use default::{create_reporter, create_reporter_with_stats, DefaultReporter};
pub use generator::{DefaultReportGenerator, ReportGenerator};
pub use logger::EventLogger;
pub use progress::ProgressTracker;
pub use r#trait::{ReportFormat, Reporter};
pub use stats::{ErrorRecord, SharedStats, TranslationStats};
