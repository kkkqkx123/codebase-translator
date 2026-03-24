use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct ProgressTracker {
    total: Arc<RwLock<usize>>,
    current: Arc<RwLock<usize>>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            total: Arc::new(RwLock::new(0)),
            current: Arc::new(RwLock::new(0)),
        }
    }

    pub fn set_total(&self, total: usize) {
        if let Ok(mut t) = self.total.write() {
            *t = total;
        }
    }

    pub fn update(&self, current: usize) {
        if let Ok(mut c) = self.current.write() {
            *c = current;
        }
    }

    pub fn get_total(&self) -> usize {
        self.total.read().map(|t| *t).unwrap_or(0)
    }

    pub fn get_current(&self) -> usize {
        self.current.read().map(|c| *c).unwrap_or(0)
    }

    pub fn get_percentage(&self) -> f64 {
        let total = self.get_total();
        let current = self.get_current();
        if total == 0 {
            0.0
        } else {
            (current as f64 / total as f64) * 100.0
        }
    }

    pub fn reset(&self) {
        if let Ok(mut total) = self.total.write() {
            *total = 0;
        }
        if let Ok(mut current) = self.current.write() {
            *current = 0;
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_new() {
        let tracker = ProgressTracker::new();
        assert_eq!(tracker.get_total(), 0);
        assert_eq!(tracker.get_current(), 0);
        assert_eq!(tracker.get_percentage(), 0.0);
    }

    #[test]
    fn test_progress_tracker_set_total() {
        let tracker = ProgressTracker::new();
        tracker.set_total(100);
        assert_eq!(tracker.get_total(), 100);
    }

    #[test]
    fn test_progress_tracker_update() {
        let tracker = ProgressTracker::new();
        tracker.set_total(100);
        tracker.update(50);
        assert_eq!(tracker.get_current(), 50);
        assert_eq!(tracker.get_percentage(), 50.0);
    }

    #[test]
    fn test_progress_tracker_percentage() {
        let tracker = ProgressTracker::new();
        tracker.set_total(100);
        tracker.update(75);
        assert_eq!(tracker.get_percentage(), 75.0);
    }

    #[test]
    fn test_progress_tracker_reset() {
        let tracker = ProgressTracker::new();
        tracker.set_total(100);
        tracker.update(50);
        tracker.reset();
        assert_eq!(tracker.get_total(), 0);
        assert_eq!(tracker.get_current(), 0);
        assert_eq!(tracker.get_percentage(), 0.0);
    }
}
