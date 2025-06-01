//! Token abstraction layer for interfacing with the lexer
//!
//! This module provides the interface between the parser and lexer,
//! abstracting over token types and providing a stream-like interface
//! for consuming tokens during parsing.

pub mod stream;
pub mod types;

pub use stream::*;
pub use types::*;
