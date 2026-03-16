//! Parser coordinator module
//!
//! Provides high-level coordination for parsing operations, managing multiple
//! parsers and routing files to the appropriate parser based on file extension.

mod coordinator;
mod tests;
mod types;

pub use coordinator::ParserCoordinator;
pub use types::ParserType;
