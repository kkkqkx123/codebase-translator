//! Concurrent file writer
//!
//! This module provides concurrent file writing capabilities with
//! rate limiting and error handling for batch processing.
//!
//! Note: This module uses spawn_blocking for file I/O operations
//! to avoid blocking the async runtime threads.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinSet;
use tracing::error;

use crate::core::models::{File, TranslationUnit};

use super::file::{FileWriter, WriterConfig};

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
    /// Note: This method uses spawn_blocking to perform file I/O operations
    /// on a dedicated thread pool, preventing blocking of async runtime threads.
    pub async fn write_files(
        &self,
        files: Vec<(File, Vec<TranslationUnit>, HashMap<String, String>)>,
    ) -> Vec<WriteResult> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut results = Vec::with_capacity(files.len());
        let mut join_set = JoinSet::new();

        for (file, units, translations) in files {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("Semaphore should not be closed");
            let config = self.config.clone();

            // Use spawn_blocking for file I/O to avoid blocking async runtime
            join_set.spawn(tokio::task::spawn_blocking(move || {
                let _permit = permit;
                let writer = FileWriter::new(config);
                let path = file.path.clone();
                let unit_count = units.len();

                match writer.write(&file, &units, &translations) {
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
            }));
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(write_result)) => results.push(write_result),
                Ok(Err(e)) => {
                    error!(error = %e, "Blocking task panicked during concurrent write");
                    results.push(WriteResult {
                        path: PathBuf::from("<unknown>"),
                        success: false,
                        error: Some(format!("Blocking task panicked: {e}")),
                        units_written: 0,
                    });
                }
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
    /// Note: This method uses spawn_blocking for file I/O operations
    /// on a dedicated thread pool, preventing blocking of async runtime threads.
    pub async fn write_files_streaming(
        &self,
        mut file_receiver: mpsc::Receiver<(File, Vec<TranslationUnit>, HashMap<String, String>)>,
    ) -> mpsc::Receiver<WriteResult> {
        let (result_sender, result_receiver) = mpsc::channel(self.max_concurrent * 2);
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let config = self.config.clone();

        tokio::spawn(async move {
            while let Some((file, units, translations)) = file_receiver.recv().await {
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => break,
                };
                let sender = result_sender.clone();
                let writer_config = config.clone();

                // Use spawn_blocking for file I/O
                tokio::spawn(tokio::task::spawn_blocking(move || {
                    let _permit = permit;
                    let writer = FileWriter::new(writer_config);
                    let path = file.path.clone();
                    let unit_count = units.len();

                    let result = match writer.write(&file, &units, &translations) {
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

                    // Use blocking_send since we're in a blocking context
                    let _ = sender.blocking_send(result);
                }));
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

    /// Check if all writes were successful
    pub fn all_success(&self) -> bool {
        self.failure_count == 0 && self.total_files > 0
    }
}
