//! Attribute parsing for Phase 2.8.1
//!
//! Implements parsing for Ferra attribute syntax including:
//! - Standard attributes: #[derive(Debug, Clone)]
//! - Simple attributes: #[inline]
//! - Alternative syntax: @inline (future)

pub mod parser;

pub use parser::*;
