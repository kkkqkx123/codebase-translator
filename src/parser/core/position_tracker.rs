//! Position tracking utilities

use crate::core::models::Position;

/// Position tracker for managing source positions
#[derive(Debug, Clone)]
pub struct PositionTracker {
    line: usize,
    column: usize,
    offset: usize,
}

impl PositionTracker {
    /// Create a new position tracker at the start
    pub fn new() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Create from tree-sitter position (0-indexed)
    pub fn from_ts_position(row: usize, column: usize, byte: usize) -> Position {
        Position::new(row + 1, column + 1, byte)
    }

    /// Advance by a character
    pub fn advance(&mut self, ch: char) {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    /// Advance by a string
    pub fn advance_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.advance(ch);
        }
    }

    /// Get current position
    pub fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.offset)
    }

    /// Reset to start
    pub fn reset(&mut self) {
        self.line = 1;
        self.column = 1;
        self.offset = 0;
    }
}

impl Default for PositionTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for position manipulation
pub mod utils {
    use super::*;

    /// Convert tree-sitter position to our Position
    pub fn ts_to_position(row: usize, column: usize, byte: usize) -> Position {
        Position::new(row + 1, column + 1, byte)
    }

    /// Check if a position is within a range
    pub fn is_within(pos: &Position, start: &Position, end: &Position) -> bool {
        pos.offset >= start.offset && pos.offset <= end.offset
    }

    /// Calculate the distance between two positions
    pub fn distance(start: &Position, end: &Position) -> usize {
        end.offset.saturating_sub(start.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_tracker() {
        let mut tracker = PositionTracker::new();
        assert_eq!(tracker.current_position().line, 1);
        assert_eq!(tracker.current_position().column, 1);

        tracker.advance('a');
        assert_eq!(tracker.current_position().line, 1);
        assert_eq!(tracker.current_position().column, 2);

        tracker.advance('\n');
        assert_eq!(tracker.current_position().line, 2);
        assert_eq!(tracker.current_position().column, 1);
    }

    #[test]
    fn test_ts_to_position() {
        let pos = PositionTracker::from_ts_position(0, 0, 0);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);
    }
}
