//! Report generation

pub mod default;
pub mod progress;
pub mod stats;
pub mod r#trait;

pub use default::{create_reporter, DefaultReporter};
pub use progress::ProgressReporter;
pub use r#trait::{ReportFormat, Reporter};
pub use stats::{ErrorRecord, SharedStats, TranslationStats};

#[cfg(feature = "progress")]
pub use default::create_progress_reporter;
