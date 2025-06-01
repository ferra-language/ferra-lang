//! Top-level program parsing for complete Ferra programs
//!
//! This module provides parsers for complete compilation units,
//! integrating all the component parsers (expressions, statements, blocks)
//! to parse full Ferra programs.

pub mod parser;

pub use parser::ProgramParser;
