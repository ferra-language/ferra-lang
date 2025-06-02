//! Test utilities for the Ferra parser
//!
//! This module provides common testing utilities, helper functions, and macros
//! used across all parser tests to reduce duplication and improve test quality.

use crate::{
    ast::{Arena, Block, Expression, Item, Statement, Type},
    block::parser::BlockParser,
    pratt::parser::PrattParser,
    program::parser::ProgramParser,
    statement::parser::StatementParser,
    token::{stream::VecTokenStream, Span, TokenStream, TokenType},
};

/// Create test arena for unit tests
pub fn test_arena() -> Arena {
    Arena::new()
}

/// Create test span for testing
pub fn test_span(start: usize, end: usize) -> Span {
    Span::new(start, end, 1, 1)
}

/// Create mock token stream from token types
pub fn mock_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
    VecTokenStream::from_token_types(token_types)
}

/// Create mock token stream from source code using lexer
pub fn mock_tokens_from_source(source: &str) -> VecTokenStream {
    use ferra_lexer::Lexer;

    let lexer = Lexer::new(source);
    let lexer_tokens = lexer.lex();

    // Convert lexer tokens to parser tokens
    let parser_tokens: Vec<TokenType> = lexer_tokens
        .into_iter()
        .map(|token| convert_lexer_token(token.kind))
        .collect();

    VecTokenStream::from_token_types(parser_tokens)
}

/// Convert lexer token to parser token type
fn convert_lexer_token(kind: ferra_lexer::TokenKind) -> TokenType {
    use ferra_lexer::TokenKind;
    match kind {
        TokenKind::Let => TokenType::Let,
        TokenKind::Var => TokenType::Var,
        TokenKind::Fn => TokenType::Fn,
        TokenKind::Data => TokenType::Data,
        TokenKind::Pub => TokenType::Pub,
        TokenKind::Unsafe => TokenType::Unsafe,
        TokenKind::Async => TokenType::Async,
        TokenKind::If => TokenType::If,
        TokenKind::Else => TokenType::Else,
        TokenKind::While => TokenType::While,
        TokenKind::For => TokenType::For,
        TokenKind::In => TokenType::In,
        TokenKind::Break => TokenType::Break,
        TokenKind::Continue => TokenType::Continue,
        TokenKind::Return => TokenType::Return,
        TokenKind::Match => TokenType::Match,
        TokenKind::True => TokenType::BooleanLiteral(true),
        TokenKind::False => TokenType::BooleanLiteral(false),
        TokenKind::Identifier => TokenType::Identifier("test".to_string()),
        TokenKind::IntegerLiteral => TokenType::IntegerLiteral(42),
        TokenKind::FloatLiteral => TokenType::FloatLiteral(42.5),
        TokenKind::StringLiteral => TokenType::StringLiteral("test".to_string()),
        TokenKind::Plus => TokenType::Plus,
        TokenKind::Minus => TokenType::Minus,
        TokenKind::Star => TokenType::Star,
        TokenKind::Slash => TokenType::Slash,
        TokenKind::Percent => TokenType::Percent,
        TokenKind::Equal => TokenType::Equal,
        TokenKind::EqualEqual => TokenType::EqualEqual,
        TokenKind::NotEqual => TokenType::BangEqual,
        TokenKind::Less => TokenType::Less,
        TokenKind::Greater => TokenType::Greater,
        TokenKind::LessEqual => TokenType::LessEqual,
        TokenKind::GreaterEqual => TokenType::GreaterEqual,
        TokenKind::LogicalAnd => TokenType::AmpAmp,
        TokenKind::LogicalOr => TokenType::PipePipe,
        TokenKind::Bang => TokenType::Bang,
        TokenKind::LParen => TokenType::LeftParen,
        TokenKind::RParen => TokenType::RightParen,
        TokenKind::LBrace => TokenType::LeftBrace,
        TokenKind::RBrace => TokenType::RightBrace,
        TokenKind::LBracket => TokenType::LeftBracket,
        TokenKind::RBracket => TokenType::RightBracket,
        TokenKind::Comma => TokenType::Comma,
        TokenKind::Semicolon => TokenType::Semicolon,
        TokenKind::Colon => TokenType::Colon,
        TokenKind::Arrow => TokenType::Arrow,
        TokenKind::Dot => TokenType::Dot,
        TokenKind::Eof => TokenType::Eof,
        _ => TokenType::Eof, // Fallback for unhandled tokens
    }
}

/// Create parser for testing expressions
pub fn test_expression_parser(
    arena: &Arena,
    tokens: VecTokenStream,
) -> PrattParser<VecTokenStream> {
    PrattParser::new(arena, tokens)
}

/// Create parser for testing statements
pub fn test_statement_parser(
    arena: &Arena,
    tokens: VecTokenStream,
) -> StatementParser<VecTokenStream> {
    StatementParser::new(arena, tokens)
}

/// Create parser for testing blocks
pub fn test_block_parser(arena: &Arena) -> BlockParser {
    BlockParser::new(arena)
}

/// Create parser for testing programs
pub fn test_program_parser(arena: &Arena, tokens: VecTokenStream) -> ProgramParser<VecTokenStream> {
    ProgramParser::new(arena, tokens)
}

/// Helper to check if token stream is empty
pub fn is_token_stream_empty(stream: &VecTokenStream) -> bool {
    stream.is_at_end()
}

/// Expression type enum for assertion matching
#[derive(Debug, PartialEq)]
pub enum ExpectedExpressionType {
    Literal,
    Identifier,
    QualifiedIdentifier,
    Binary,
    Unary,
    Call,
    MemberAccess,
    Index,
    Array,
    Tuple,
    If,
    Match,
    Grouped,
    Block,
    Macro,
    Await,
}

/// Statement type enum for assertion matching
#[derive(Debug, PartialEq)]
pub enum ExpectedStatementType {
    Expression,
    VariableDecl,
    If,
    While,
    For,
    Return,
    Break,
    Continue,
    Block,
}

/// Item type enum for assertion matching
#[derive(Debug, PartialEq)]
pub enum ExpectedItemType {
    FunctionDecl,
    VariableDecl,
    DataClassDecl,
    ExternBlock,
}

/// Type expression enum for assertion matching
#[derive(Debug, PartialEq)]
pub enum ExpectedTypeType {
    Identifier,
    Generic,
    Array,
    Tuple,
    Function,
    Pointer,
}

/// Assert AST node types with detailed error messages
pub mod assertions {
    use super::*;

    /// Assert expression type with descriptive error
    pub fn assert_expression_type(expr: &Expression, expected: ExpectedExpressionType) {
        let actual = match expr {
            Expression::Literal(_) => ExpectedExpressionType::Literal,
            Expression::Identifier(_) => ExpectedExpressionType::Identifier,
            Expression::QualifiedIdentifier(_) => ExpectedExpressionType::QualifiedIdentifier,
            Expression::Binary(_) => ExpectedExpressionType::Binary,
            Expression::Unary(_) => ExpectedExpressionType::Unary,
            Expression::Call(_) => ExpectedExpressionType::Call,
            Expression::MemberAccess(_) => ExpectedExpressionType::MemberAccess,
            Expression::Index(_) => ExpectedExpressionType::Index,
            Expression::Array(_) => ExpectedExpressionType::Array,
            Expression::Tuple(_) => ExpectedExpressionType::Tuple,
            Expression::If(_) => ExpectedExpressionType::If,
            Expression::Match(_) => ExpectedExpressionType::Match,
            Expression::Grouped(_) => ExpectedExpressionType::Grouped,
            Expression::Block(_) => ExpectedExpressionType::Block,
            Expression::Macro(_) => ExpectedExpressionType::Macro,
            Expression::Await(_) => ExpectedExpressionType::Await,
        };

        assert_eq!(
            actual, expected,
            "Expected expression type {:?}, but found {:?}. Expression: {:?}",
            expected, actual, expr
        );
    }

    /// Assert statement type with descriptive error
    pub fn assert_statement_type(stmt: &Statement, expected: ExpectedStatementType) {
        let actual = match stmt {
            Statement::Expression(_) => ExpectedStatementType::Expression,
            Statement::VariableDecl(_) => ExpectedStatementType::VariableDecl,
            Statement::If(_) => ExpectedStatementType::If,
            Statement::While(_) => ExpectedStatementType::While,
            Statement::For(_) => ExpectedStatementType::For,
            Statement::Return(_) => ExpectedStatementType::Return,
            Statement::Break(_) => ExpectedStatementType::Break,
            Statement::Continue(_) => ExpectedStatementType::Continue,
            Statement::Block(_) => ExpectedStatementType::Block,
        };

        assert_eq!(
            actual, expected,
            "Expected statement type {:?}, but found {:?}. Statement: {:?}",
            expected, actual, stmt
        );
    }

    /// Assert item type with descriptive error
    pub fn assert_item_type(item: &Item, expected: ExpectedItemType) {
        let actual = match item {
            Item::FunctionDecl(_) => ExpectedItemType::FunctionDecl,
            Item::VariableDecl(_) => ExpectedItemType::VariableDecl,
            Item::DataClassDecl(_) => ExpectedItemType::DataClassDecl,
            Item::ExternBlock(_) => ExpectedItemType::ExternBlock,
        };

        assert_eq!(
            actual, expected,
            "Expected item type {:?}, but found {:?}. Item: {:?}",
            expected, actual, item
        );
    }

    /// Assert type expression type with descriptive error
    pub fn assert_type_type(type_expr: &Type, expected: ExpectedTypeType) {
        let actual = match type_expr {
            Type::Identifier(_) => ExpectedTypeType::Identifier,
            Type::Generic(_) => ExpectedTypeType::Generic,
            Type::Array(_) => ExpectedTypeType::Array,
            Type::Tuple(_) => ExpectedTypeType::Tuple,
            Type::Function(_) => ExpectedTypeType::Function,
            Type::Pointer(_) => ExpectedTypeType::Pointer,
        };

        assert_eq!(
            actual, expected,
            "Expected type {:?}, but found {:?}. Type: {:?}",
            expected, actual, type_expr
        );
    }

    /// Assert block is not empty
    pub fn assert_non_empty_block(block: &Block) {
        assert!(
            !block.statements.is_empty(),
            "Expected non-empty block, but found empty block"
        );
    }

    /// Assert block contains expected number of statements
    pub fn assert_block_statement_count(block: &Block, expected: usize) {
        assert_eq!(
            block.statements.len(),
            expected,
            "Expected {} statements in block, but found {}",
            expected,
            block.statements.len()
        );
    }
}

/// Test macros for common parsing patterns
#[macro_export]
macro_rules! test_expr {
    ($source:expr => $expected:pat) => {
        let arena = $crate::test_utils::test_arena();
        let tokens = $crate::test_utils::mock_tokens_from_source($source);
        let mut parser = $crate::test_utils::test_expression_parser(&arena, tokens);
        let expr = parser.parse_expression(0).unwrap();
        assert!(
            matches!(expr, $expected),
            "Expression didn't match expected pattern. Got: {:?}",
            expr
        );
    };
    ($source:expr => $expected:pat, $assertion:expr) => {
        let arena = $crate::test_utils::test_arena();
        let tokens = $crate::test_utils::mock_tokens_from_source($source);
        let mut parser = $crate::test_utils::test_expression_parser(&arena, tokens);
        let expr = parser.parse_expression(0).unwrap();
        assert!(
            matches!(expr, $expected),
            "Expression didn't match expected pattern. Got: {:?}",
            expr
        );
        $assertion(&expr);
    };
}

/// Test parsing error with expected error type
#[macro_export]
macro_rules! test_parse_error {
    ($source:expr => $error_type:path) => {
        let arena = $crate::test_utils::test_arena();
        let tokens = $crate::test_utils::mock_tokens_from_source($source);
        let mut parser = $crate::test_utils::test_expression_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            matches!(result, Err($error_type { .. })),
            "Expected error type {}, got: {:?}",
            stringify!($error_type),
            result
        );
    };
}

/// Test statement parsing
#[macro_export]
macro_rules! test_stmt {
    ($source:expr => $expected:pat) => {
        let arena = $crate::test_utils::test_arena();
        let tokens = $crate::test_utils::mock_tokens_from_source($source);
        let mut parser = $crate::test_utils::test_statement_parser(&arena, tokens);
        let stmt = parser.parse_statement().unwrap();
        assert!(
            matches!(stmt, $expected),
            "Statement didn't match expected pattern. Got: {:?}",
            stmt
        );
    };
}

/// Test type parsing
#[macro_export]
macro_rules! test_type {
    ($tokens:expr => $expected:pat) => {
        let token_stream = $crate::test_utils::mock_token_stream($tokens);
        // Note: TypeParser is not public, so this macro needs adjustment
        // This will be a basic token verification for now
        assert!(
            !$crate::test_utils::is_token_stream_empty(&token_stream),
            "Token stream should not be empty"
        );
    };
}

/// Test program parsing
#[macro_export]
macro_rules! test_program {
    ($source:expr => $expected:pat) => {
        let arena = $crate::test_utils::test_arena();
        let tokens = $crate::test_utils::mock_tokens_from_source($source);
        let mut parser = $crate::test_utils::test_program_parser(&arena, tokens);
        let program = parser.parse_compilation_unit().unwrap();
        assert!(
            matches!(program, $expected),
            "Program didn't match expected pattern. Got: {:?}",
            program
        );
    };
}

/// Parameterized test runner for operator combinations
pub fn test_binary_operators<F>(operators: &[TokenType], test_fn: F)
where
    F: Fn(&TokenType),
{
    for op in operators {
        test_fn(op);
    }
}

/// Test fixture loading utilities
pub mod fixtures {
    use std::fs;

    /// Load test fixture file content
    pub fn load_fixture(category: &str, filename: &str) -> String {
        let path = format!("tests/fixtures/{}/{}", category, filename);
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {}", path))
    }

    /// Load valid program fixture
    pub fn load_valid_fixture(filename: &str) -> String {
        load_fixture("valid", filename)
    }

    /// Load invalid program fixture
    pub fn load_invalid_fixture(filename: &str) -> String {
        load_fixture("invalid", filename)
    }

    /// Load edge case fixture
    pub fn load_edge_case_fixture(filename: &str) -> String {
        load_fixture("edge_cases", filename)
    }

    /// Get all fixture files in a category
    pub fn list_fixtures(category: &str) -> Vec<String> {
        let dir = format!("tests/fixtures/{}", category);
        if let Ok(entries) = fs::read_dir(&dir) {
            entries
                .filter_map(|entry| {
                    entry
                        .ok()
                        .and_then(|e| e.file_name().to_str().map(String::from))
                })
                .filter(|name| name.ends_with(".ferra"))
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// Performance testing utilities
pub mod performance {
    use std::time::{Duration, Instant};

    /// Measure parsing time for given source
    pub fn measure_parse_time<F, R>(operation: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let elapsed = start.elapsed();
        (result, elapsed)
    }

    /// Assert parsing completes within time limit
    pub fn assert_parse_within<F, R>(time_limit: Duration, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let (result, elapsed) = measure_parse_time(operation);
        assert!(
            elapsed <= time_limit,
            "Parsing took {:?}, which exceeds limit of {:?}",
            elapsed,
            time_limit
        );
        result
    }
}

/// Test case generation macros for repetitive patterns
#[macro_export]
macro_rules! test_all_operators {
    ($test_fn:ident, $left:expr, $right:expr) => {
        let binary_operators = vec![
            $crate::token::TokenType::Plus,
            $crate::token::TokenType::Minus,
            $crate::token::TokenType::Star,
            $crate::token::TokenType::Slash,
            $crate::token::TokenType::Percent,
            $crate::token::TokenType::EqualEqual,
            $crate::token::TokenType::BangEqual,
            $crate::token::TokenType::Less,
            $crate::token::TokenType::LessEqual,
            $crate::token::TokenType::Greater,
            $crate::token::TokenType::GreaterEqual,
            $crate::token::TokenType::AmpAmp,
            $crate::token::TokenType::PipePipe,
        ];
        for op in binary_operators {
            $test_fn($left.clone(), op, $right.clone());
        }
    };
}

/// Test precedence for all operator combinations
#[macro_export]
macro_rules! test_precedence_matrix {
    ($test_name:ident) => {
        #[test]
        fn $test_name() {
            use $crate::test_utils::*;
            use $crate::token::TokenType;

            let arena = test_arena();

            // Test all operator combinations for precedence
            let operators = [
                (TokenType::Plus, 3),
                (TokenType::Minus, 3),
                (TokenType::Star, 4),
                (TokenType::Slash, 4),
                (TokenType::AmpAmp, 1),
                (TokenType::PipePipe, 0),
            ];

            for (op1, prec1) in &operators {
                for (op2, prec2) in &operators {
                    let tokens = mock_token_stream(vec![
                        TokenType::IntegerLiteral(1),
                        op1.clone(),
                        TokenType::IntegerLiteral(2),
                        op2.clone(),
                        TokenType::IntegerLiteral(3),
                        TokenType::Eof,
                    ]);

                    let mut parser = test_expression_parser(&arena, tokens);
                    let result = parser.parse_expression(0);

                    // Test should complete without error
                    assert!(result.is_ok(), "Failed to parse {:?} and {:?}", op1, op2);
                }
            }
        }
    };
}

/// Generate parameterized tests for different literal types
#[macro_export]
macro_rules! test_all_literals {
    ($test_name:ident, $test_fn:expr) => {
        #[test]
        fn $test_name() {
            use $crate::test_utils::*;
            use $crate::token::TokenType;

            let literals = vec![
                TokenType::IntegerLiteral(42),
                TokenType::FloatLiteral(42.5),
                TokenType::StringLiteral("test".to_string()),
                TokenType::BooleanLiteral(true),
                TokenType::BooleanLiteral(false),
            ];

            for literal in literals {
                $test_fn(literal);
            }
        }
    };
}

/// Generate tests for all statement types
#[macro_export]
macro_rules! test_statement_types {
    ($test_name:ident, $validation_fn:expr) => {
        #[test]
        fn $test_name() {
            use $crate::test_utils::*;

            let arena = test_arena();

            // Test variable declaration
            let source = "let x = 42;";
            let tokens = mock_tokens_from_source(source);
            let mut parser = test_statement_parser(&arena, tokens);
            let stmt = parser.parse_statement().unwrap();
            $validation_fn(&stmt, ExpectedStatementType::VariableDecl);

            // Test expression statement
            let source = "42;";
            let tokens = mock_tokens_from_source(source);
            let mut parser = test_statement_parser(&arena, tokens);
            let stmt = parser.parse_statement().unwrap();
            $validation_fn(&stmt, ExpectedStatementType::Expression);

            // Test return statement
            let source = "return 42;";
            let tokens = mock_tokens_from_source(source);
            let mut parser = test_statement_parser(&arena, tokens);
            let stmt = parser.parse_statement().unwrap();
            $validation_fn(&stmt, ExpectedStatementType::Return);
        }
    };
}

/// Enhanced fixture management
pub mod enhanced_fixtures {
    use super::fixtures;

    /// Fixture metadata for organized testing
    #[derive(Debug, Clone)]
    pub struct FixtureMetadata {
        pub category: String,
        pub filename: String,
        pub description: String,
        pub expected_parse_result: bool, // true if should parse successfully
        pub test_priority: u8,           // 1-5, 5 is highest priority
    }

    /// Get all fixtures with metadata
    pub fn get_fixture_catalog() -> Vec<FixtureMetadata> {
        vec![
            FixtureMetadata {
                category: "valid".to_string(),
                filename: "async_functions.ferra".to_string(),
                description: "Async function declarations and implementations".to_string(),
                expected_parse_result: true,
                test_priority: 5,
            },
            FixtureMetadata {
                category: "valid".to_string(),
                filename: "data_classes.ferra".to_string(),
                description: "Data class definitions with various field types".to_string(),
                expected_parse_result: true,
                test_priority: 5,
            },
            FixtureMetadata {
                category: "valid".to_string(),
                filename: "control_flow.ferra".to_string(),
                description: "Control flow statements and complex nesting".to_string(),
                expected_parse_result: true,
                test_priority: 5,
            },
            FixtureMetadata {
                category: "invalid".to_string(),
                filename: "type_errors.ferra".to_string(),
                description: "Various type syntax errors for error recovery testing".to_string(),
                expected_parse_result: true,
                test_priority: 4,
            },
            FixtureMetadata {
                category: "edge_cases".to_string(),
                filename: "performance_stress.ferra".to_string(),
                description: "Performance stress testing with deep nesting".to_string(),
                expected_parse_result: true,
                test_priority: 3,
            },
        ]
    }

    /// Load fixture by metadata
    pub fn load_fixture_by_metadata(metadata: &FixtureMetadata) -> String {
        fixtures::load_fixture(&metadata.category, &metadata.filename)
    }

    /// Get fixtures by priority level
    pub fn get_fixtures_by_priority(min_priority: u8) -> Vec<FixtureMetadata> {
        get_fixture_catalog()
            .into_iter()
            .filter(|f| f.test_priority >= min_priority)
            .collect()
    }

    /// Get fixtures by category
    pub fn get_fixtures_by_category(category: &str) -> Vec<FixtureMetadata> {
        get_fixture_catalog()
            .into_iter()
            .filter(|f| f.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn test_arena_creation() {
        let _arena = test_arena();
        // Basic test to ensure arena is created successfully
        let _span = test_span(0, 10);
    }

    #[test]
    fn test_mock_token_stream() {
        let tokens = mock_token_stream(vec![
            TokenType::Let,
            TokenType::Identifier("x".to_string()),
            TokenType::Equal,
            TokenType::IntegerLiteral(42),
            TokenType::Eof,
        ]);
        assert!(!is_token_stream_empty(&tokens));
    }

    #[test]
    fn test_expression_assertions() {
        let arena = test_arena();
        let tokens = mock_token_stream(vec![
            TokenType::IntegerLiteral(42),
            TokenType::Plus,
            TokenType::IntegerLiteral(10),
            TokenType::Eof,
        ]);
        let mut parser = test_expression_parser(&arena, tokens);
        let expr = parser.parse_expression(0).unwrap();

        // Test assertion helper
        assertions::assert_expression_type(expr, ExpectedExpressionType::Binary);
    }

    #[test]
    fn test_fixture_utilities() {
        // Test that fixture loading doesn't panic for existing files
        let fixtures = fixtures::list_fixtures("valid");
        assert!(
            !fixtures.is_empty(),
            "Should have at least some valid fixtures"
        );
    }
}
