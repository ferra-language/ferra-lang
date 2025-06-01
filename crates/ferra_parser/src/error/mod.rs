//! Error handling and diagnostics for the parser
//!
//! Implements "positive-first" error messaging and recovery strategies

pub mod parse_error;
pub mod recovery;

pub use parse_error::*;
pub use recovery::*;

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;
