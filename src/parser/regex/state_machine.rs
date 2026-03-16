//! State machine pattern matcher
//!
//! Provides a finite state machine-based pattern matcher for complex extraction scenarios.
//! Supports multi-step matching with state transitions and capture groups.

use regex::Regex;
use std::collections::HashMap;

use crate::core::error::{Result, TranslateError};
use crate::core::models::Position;

/// A match result from the state machine
#[derive(Debug, Clone)]
pub struct StateMachineMatch {
    /// The matched content
    pub content: String,
    /// Start position in the source
    pub start_pos: Position,
    /// End position in the source
    pub end_pos: Position,
    /// Name of the accepting state
    pub state_name: String,
    /// Capture groups (if any)
    pub captures: Vec<String>,
}

/// A state in the pattern state machine
#[derive(Debug, Clone)]
pub struct State {
    /// State name
    pub name: String,
    /// Regex to match at this state
    pub regex: Regex,
    /// Capture group to extract (None = use full match)
    pub capture_group: Option<usize>,
    /// Transitions to other states
    pub transitions: Vec<Transition>,
    /// Whether this state can be the final state
    pub is_final: bool,
}

/// Transition between states
#[derive(Debug, Clone)]
pub struct Transition {
    /// Target state name
    pub target: String,
    /// Condition regex (if None, always matches)
    pub condition: Option<Regex>,
}

/// State machine pattern matcher
pub struct StateMachineMatcher {
    /// States indexed by name
    states: HashMap<String, State>,
    /// Initial state name
    initial_state: String,
    /// Accepting state names
    accepting_states: Vec<String>,
    /// Pattern name for identification
    pub name: String,
}

impl StateMachineMatcher {
    /// Create a new state machine matcher from configuration
    pub fn from_config(
        name: String,
        initial_state: String,
        accepting_states: Vec<String>,
        state_configs: &[crate::config::project::PatternState],
    ) -> Result<Self> {
        let mut states = HashMap::new();

        for config in state_configs {
            let regex = Regex::new(&config.regex).map_err(|e| {
                TranslateError::Config(format!(
                    "Invalid regex in state '{}': {}",
                    config.name, e
                ))
            })?;

            let transitions = config
                .transitions
                .iter()
                .map(|t| {
                    let condition = t.condition.as_ref().and_then(|c| Regex::new(c).ok());
                    Transition {
                        target: t.target.clone(),
                        condition,
                    }
                })
                .collect();

            let state = State {
                name: config.name.clone(),
                regex,
                capture_group: config.capture_group,
                transitions,
                is_final: config.is_final,
            };

            states.insert(config.name.clone(), state);
        }

        // Validate initial state exists
        if !states.contains_key(&initial_state) {
            return Err(TranslateError::Config(format!(
                "Initial state '{}' not found",
                initial_state
            )));
        }

        // Validate accepting states exist
        for state_name in &accepting_states {
            if !states.contains_key(state_name) {
                return Err(TranslateError::Config(format!(
                    "Accepting state '{}' not found",
                    state_name
                )));
            }
        }

        Ok(Self {
            states,
            initial_state,
            accepting_states,
            name,
        })
    }

    /// Find all matches in the content
    pub fn find_matches(&self, content: &str) -> Result<Vec<StateMachineMatch>> {
        let mut matches = Vec::new();
        let mut current_pos = 0;

        while current_pos < content.len() {
            if let Some(m) = self.try_match(content, current_pos)? {
                current_pos = m.end_pos.offset;
                matches.push(m);
            } else {
                current_pos += 1;
            }
        }

        Ok(matches)
    }

    /// Try to match starting at the given position
    fn try_match(&self, content: &str, start_pos: usize) -> Result<Option<StateMachineMatch>> {
        let mut current_state_name = self.initial_state.clone();
        let mut current_offset = start_pos;
        let mut last_accepting_match: Option<(String, usize, Vec<String>)> = None;

        loop {
            let state = self
                .states
                .get(&current_state_name)
                .ok_or_else(|| TranslateError::Parse(format!("State '{}' not found", current_state_name)))?;

            // Try to match the current state's regex at current position
            let remaining = &content[current_offset.min(content.len())..];

            if let Some(mat) = state.regex.find(remaining) {
                let match_start = current_offset + mat.start();
                let match_end = current_offset + mat.end();

                // Extract captures if any
                let captures: Vec<String> = if let Some(caps) = state.regex.captures(remaining) {
                    caps.iter()
                        .skip(1) // Skip full match
                        .flatten()
                        .map(|m| m.as_str().to_string())
                        .collect()
                } else {
                    Vec::new()
                };

                // Get the content to extract
                let extracted_content = if let Some(group) = state.capture_group {
                    captures.get(group - 1).cloned().unwrap_or_default()
                } else {
                    mat.as_str().to_string()
                };

                // If this is a final/accepting state, record the match
                if state.is_final || self.accepting_states.contains(&current_state_name) {
                    last_accepting_match = Some((
                        extracted_content,
                        match_end,
                        captures.clone(),
                    ));
                }

                // Try to transition to next state
                let mut transitioned = false;
                for transition in &state.transitions {
                    // Check transition condition if any
                    let can_transition = if let Some(ref condition) = transition.condition {
                        condition.is_match(&content[match_end.min(content.len())..])
                    } else {
                        true
                    };

                    if can_transition {
                        current_state_name = transition.target.clone();
                        current_offset = match_end;
                        transitioned = true;
                        break;
                    }
                }

                if !transitioned {
                    // No transition possible, check if we have an accepting match
                    break;
                }
            } else {
                // Current state's regex doesn't match
                break;
            }
        }

        // Return the last accepting match if any
        if let Some((extracted_content, end_offset, captures)) = last_accepting_match {
            let start_pos_obj = self.byte_to_position(content, start_pos);
            let end_pos_obj = self.byte_to_position(content, end_offset);

            Ok(Some(StateMachineMatch {
                content: extracted_content,
                start_pos: start_pos_obj,
                end_pos: end_pos_obj,
                state_name: current_state_name,
                captures,
            }))
        } else {
            Ok(None)
        }
    }

    /// Convert byte offset to line/column position
    fn byte_to_position(&self, content: &str, byte_offset: usize) -> Position {
        let content_up_to_offset = &content[..byte_offset.min(content.len())];
        let lines: Vec<&str> = content_up_to_offset.lines().collect();

        let line = lines.len();
        let column = lines.last().map(|l| l.len() + 1).unwrap_or(1);

        Position::new(line, column, byte_offset)
    }
}

/// Builder for creating state machine matchers programmatically
pub struct StateMachineBuilder {
    states: Vec<crate::config::project::PatternState>,
    initial_state: Option<String>,
    accepting_states: Vec<String>,
    name: Option<String>,
}

impl StateMachineBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            initial_state: None,
            accepting_states: Vec::new(),
            name: None,
        }
    }

    /// Set the pattern name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the initial state
    pub fn initial_state(mut self, state: impl Into<String>) -> Self {
        self.initial_state = Some(state.into());
        self
    }

    /// Add an accepting state
    pub fn accepting_state(mut self, state: impl Into<String>) -> Self {
        self.accepting_states.push(state.into());
        self
    }

    /// Add a state
    pub fn state(
        mut self,
        name: impl Into<String>,
        regex: impl Into<String>,
        is_final: bool,
    ) -> Self {
        self.states.push(crate::config::project::PatternState {
            name: name.into(),
            regex: regex.into(),
            capture_group: None,
            transitions: Vec::new(),
            is_final,
        });
        self
    }

    /// Add a state with capture group
    pub fn state_with_capture(
        mut self,
        name: impl Into<String>,
        regex: impl Into<String>,
        capture_group: usize,
        is_final: bool,
    ) -> Self {
        self.states.push(crate::config::project::PatternState {
            name: name.into(),
            regex: regex.into(),
            capture_group: Some(capture_group),
            transitions: Vec::new(),
            is_final,
        });
        self
    }

    /// Add a transition to the last added state
    pub fn transition(mut self, target: impl Into<String>) -> Self {
        if let Some(last_state) = self.states.last_mut() {
            last_state.transitions.push(crate::config::project::StateTransition {
                target: target.into(),
                condition: None,
            });
        }
        self
    }

    /// Add a conditional transition to the last added state
    pub fn transition_with_condition(
        mut self,
        target: impl Into<String>,
        condition: impl Into<String>,
    ) -> Self {
        if let Some(last_state) = self.states.last_mut() {
            last_state.transitions.push(crate::config::project::StateTransition {
                target: target.into(),
                condition: Some(condition.into()),
            });
        }
        self
    }

    /// Build the state machine matcher
    pub fn build(self) -> Result<StateMachineMatcher> {
        let name = self.name.ok_or_else(|| {
            TranslateError::Config("State machine name is required".to_string())
        })?;

        let initial_state = self.initial_state.ok_or_else(|| {
            TranslateError::Config("Initial state is required".to_string())
        })?;

        if self.accepting_states.is_empty() {
            return Err(TranslateError::Config(
                "At least one accepting state is required".to_string(),
            ));
        }

        StateMachineMatcher::from_config(
            name,
            initial_state,
            self.accepting_states,
            &self.states,
        )
    }
}

impl Default for StateMachineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_state_machine() {
        // Build a simple two-state machine for matching i18n calls with defaults
        // t("key", "default value") -> extracts "default value"
        let matcher = StateMachineBuilder::new()
            .name("i18n_with_default")
            .initial_state("start")
            .accepting_state("extract")
            .state_with_capture("start", r#"t\s*\(\s*["'][^"']+["']\s*,\s*["']"#, 0, false)
            .transition("extract")
            .state_with_capture("extract", r#"([^"']+)"#, 1, true)
            .build()
            .expect("Failed to build state machine");

        let content = r#"t("welcome", "Hello World")"#;
        let matches = matcher.find_matches(content).expect("Matching failed");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].content, "Hello World");
    }

    #[test]
    fn test_state_machine_no_match() {
        let matcher = StateMachineBuilder::new()
            .name("test")
            .initial_state("start")
            .accepting_state("end")
            .state("start", r"hello", false)
            .transition("end")
            .state("end", r"world", true)
            .build()
            .expect("Failed to build state machine");

        let content = "goodbye world";
        let matches = matcher.find_matches(content).expect("Matching failed");

        assert!(matches.is_empty());
    }
}
