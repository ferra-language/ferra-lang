//! # Ferra Parser v0.1
//!
//! The Ferra Parser is the second stage of the Ferra compiler front-end.
//! It takes a stream of tokens from the lexer and produces an Abstract Syntax Tree (AST)
//! representing the syntactic structure of the source code according to the Ferra grammar.
//!
//! ## Architecture
//!
//! - **Recursive Descent**: Used for top-level constructs, declarations, and statements
//! - **Pratt Parser**: Used for expression parsing with proper precedence handling
//! - **Arena Allocation**: AST nodes are allocated in an arena for performance
//! - **Error Recovery**: Panic mode recovery for continuing parsing after errors
//!
//! ## Usage
//!
//! ```rust
//! use ferra_parser::{Arena, ProgramParser};
//! use ferra_parser::token::{VecTokenStream, TokenType};
//!
//! // Create arena for AST allocation
//! let arena = Arena::new();
//!
//! // Create token stream (normally from lexer)
//! let tokens = VecTokenStream::from_token_types(vec![
//!     TokenType::Fn,
//!     TokenType::Identifier("test".to_string()),
//!     TokenType::LeftParen,
//!     TokenType::RightParen,
//!     TokenType::LeftBrace,
//!     TokenType::RightBrace,
//!     TokenType::Eof,
//! ]);
//!
//! // Parse the program
//! let mut parser = ProgramParser::new(&arena, tokens);
//! let ast = parser.parse_compilation_unit();
//! ```

pub mod ast;
pub mod attribute; // Phase 2.8.1: Attribute parsing
pub mod block;
pub mod error;
pub mod generic; // Phase 2.8.2: Generic type parameters
pub mod macro_parser; // Phase 2.8.4: Macro system foundation
pub mod pattern;
pub mod pratt;
pub mod program;
pub mod statement;
pub mod token;
pub mod types;

// Re-export commonly used types
pub use ast::{Arena, CompilationUnit, Expression, Item, Statement};
pub use error::{ParseError, ParseResult};
pub use pratt::PrattParser;
pub use program::ProgramParser;
pub use statement::StatementParser;
pub use token::{TokenStream, TokenType};

/// Main parser interface
pub struct Parser<'arena, T: TokenStream> {
    arena: &'arena Arena,
    tokens: T,
}

impl<'arena, T: TokenStream> Parser<'arena, T> {
    /// Create a new parser with the given token stream
    pub fn new(arena: &'arena Arena, tokens: T) -> Self {
        Self { arena, tokens }
    }

    /// Parse a complete compilation unit
    pub fn parse_compilation_unit(&mut self) -> Result<&'arena ast::CompilationUnit, ParseError> {
        statement::parser::StatementParser::new(self.arena, &mut self.tokens)
            .parse_compilation_unit()
    }

    /// Parse a single expression
    pub fn parse_expression(&mut self) -> Result<&'arena Expression, ParseError> {
        let mut pratt_parser = pratt::parser::PrattParser::new(self.arena, &mut self.tokens);
        pratt_parser.parse_expression(0)
    }

    /// Parse a single statement
    pub fn parse_statement(&mut self) -> Result<&'arena Statement, ParseError> {
        statement::parser::StatementParser::new(self.arena, &mut self.tokens).parse_statement()
    }
}

/// Convenience function to parse a file from path
pub fn parse_file<P: AsRef<std::path::Path>>(_path: P) -> ParseResult<CompilationUnit> {
    todo!("Implementation will be done after lexer integration")
}

/// Convenience function to parse source code from string
pub fn parse_source(_source: &str) -> ParseResult<CompilationUnit> {
    todo!("Implementation will be done after lexer integration")
}

// Legacy Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::VecTokenStream;

    #[test]
    fn test_parser_creation() {
        let arena = Arena::new();
        let tokens = VecTokenStream::new(vec![]);
        let _parser = Parser::new(&arena, tokens);
    }
}
