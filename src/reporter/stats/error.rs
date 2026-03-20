use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub file_path: String,
    pub error: String,
    pub time: DateTime<Utc>,
}
