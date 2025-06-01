//! Generic type parameter parsing
//!
//! This module handles parsing of generic type parameters, constraints, and where clauses.
//! Supports syntax like:
//! - Simple generics: `<T, U>`
//! - Type constraints: `<T: Clone + Debug>`
//! - Lifetime parameters: `<'a, 'b>`
//! - Where clauses: `where T: Clone + Debug, U: Default`

pub mod parser;
