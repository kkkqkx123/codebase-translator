//! Concurrent file writer
//!
//! This module provides concurrent file writing capabilities with
//! rate limiting and error handling for batch processing.
//!
//! Note: This module uses async file I/O operations via FileWriter
//! for better performance with concurrent writes.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinSet;
use tracing::error;

use crate::core::models::{File, TranslationUnit};

use super::{FileWriter, WriterConfig};

/// A writer that handles concurrent file writes with rate limiting
#[derive(Debug, Clone)]
pub struct ConcurrentWriter {
    config: WriterConfig,
    max_concurrent: usize,
}

/// Result of a single file write operation
#[derive(Debug, Clone)]
pub struct WriteResult {
    /// Path of the file
    pub path: PathBuf,
    /// Whether the write was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Number of translation units written
    pub units_written: usize,
}

impl ConcurrentWriter {
    /// Create a new concurrent writer
    pub fn new(config: WriterConfig, max_concurrent: usize) -> Self {
        Self {
            config,
            max_concurrent: max_concurrent.max(1),
        }
    }

    /// Write multiple files concurrently
    ///
    /// Note: This method uses async file I/O operations via FileWriter
    /// for better performance with concurrent writes.
    pub async fn write_files(&self, files: Vec<(File, Vec<TranslationUnit>)>) -> Vec<WriteResult> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut results = Vec::with_capacity(files.len());
        let mut join_set = JoinSet::new();

        for (file, units) in files {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("Semaphore should not be closed");
            let config = self.config.clone();

            // Use async file I/O via FileWriter
            join_set.spawn(async move {
                let _permit = permit;
                let writer = FileWriter::new(config);
                let path = file.path.clone();
                let unit_count = units.len();

                match writer.write(&file, &units).await {
                    Ok(()) => WriteResult {
                        path,
                        success: true,
                        error: None,
                        units_written: unit_count,
                    },
                    Err(e) => WriteResult {
                        path,
                        success: false,
                        error: Some(format!("{}", e)),
                        units_written: 0,
                    },
                }
            });
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(write_result) => results.push(write_result),
                Err(e) => {
                    error!(error = %e, "Task panicked during concurrent write");
                    results.push(WriteResult {
                        path: PathBuf::from("<unknown>"),
                        success: false,
                        error: Some(format!("Task panicked: {e}")),
                        units_written: 0,
                    });
                }
            }
        }

        results
    }

    /// Write files with a channel-based approach for backpressure
    ///
    /// Note: This method uses async file I/O operations via FileWriter
    /// for better performance with concurrent writes.
    pub async fn write_files_streaming(
        &self,
        mut file_receiver: mpsc::Receiver<(File, Vec<TranslationUnit>)>,
    ) -> mpsc::Receiver<WriteResult> {
        let (result_sender, result_receiver) = mpsc::channel(self.max_concurrent * 2);
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let config = self.config.clone();

        tokio::spawn(async move {
            while let Some((file, units)) = file_receiver.recv().await {
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => break,
                };
                let sender = result_sender.clone();
                let writer_config = config.clone();

                // Use async file I/O
                tokio::spawn(async move {
                    let _permit = permit;
                    let writer = FileWriter::new(writer_config);
                    let path = file.path.clone();
                    let unit_count = units.len();

                    let result = match writer.write(&file, &units).await {
                        Ok(()) => WriteResult {
                            path,
                            success: true,
                            error: None,
                            units_written: unit_count,
                        },
                        Err(e) => WriteResult {
                            path,
                            success: false,
                            error: Some(format!("{}", e)),
                            units_written: 0,
                        },
                    };

                    let _ = sender.send(result).await;
                });
            }
        });

        result_receiver
    }

    /// Get the maximum concurrent writes
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Update the maximum concurrent writes
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
    }
}

/// Statistics for concurrent write operations
#[derive(Debug, Clone, Default)]
pub struct ConcurrentWriteStats {
    /// Total files processed
    pub total_files: usize,
    /// Successfully written files
    pub success_count: usize,
    /// Failed writes
    pub failure_count: usize,
    /// Total translation units written
    pub total_units: usize,
}

impl ConcurrentWriteStats {
    /// Calculate statistics from write results
    pub fn from_results(results: &[WriteResult]) -> Self {
        let mut stats = Self {
            total_files: results.len(),
            ..Default::default()
        };

        for result in results {
            if result.success {
                stats.success_count += 1;
                stats.total_units += result.units_written;
            } else {
                stats.failure_count += 1;
            }
        }

        stats
    }

    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }
        (self.success_count as f64 / self.total_files as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests only use WriteResult and ConcurrentWriteStats from this module

    #[test]
    fn test_concurrent_write_stats() {
        let results = vec![
            WriteResult {
                path: PathBuf::from("file1.txt"),
                success: true,
                error: None,
                units_written: 5,
            },
            WriteResult {
                path: PathBuf::from("file2.txt"),
                success: true,
                error: None,
                units_written: 3,
            },
            WriteResult {
                path: PathBuf::from("file3.txt"),
                success: false,
                error: Some("error".to_string()),
                units_written: 0,
            },
        ];

        let stats = ConcurrentWriteStats::from_results(&results);
        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.total_units, 8);
        assert_eq!(stats.success_rate(), 66.66666666666667);
    }
}
