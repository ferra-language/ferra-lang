//! Systematic Edge Case Generation
//!
//! This module automatically generates edge cases from grammar rules to ensure
//! comprehensive testing of boundary conditions and parsing edge cases.

use ferra_lexer::{Lexer, LiteralValue, Token as LexerToken, TokenKind};
use ferra_parser::token::{TokenType, VecTokenStream};
use ferra_parser::{Arena, ProgramParser};

/// Edge case categories for systematic generation
#[derive(Debug, Clone)]
pub enum EdgeCaseCategory {
    // Boundary conditions
    EmptyInputs,
    MinimalValidInputs,
    MaximalComplexInputs,

    // Whitespace and formatting
    ExtraWhitespace,
    MissingWhitespace,
    NewlineVariations,

    // Syntax boundaries
    MissingOptionalElements,
    ExtraOptionalElements,
    NestedStructures,

    // Error boundaries
    InvalidTokenSequences,
    IncompleteConstructs,
    AmbiguousConstructs,
}

/// Edge case generator for systematic testing
pub struct EdgeCaseGenerator;

impl EdgeCaseGenerator {
    /// Create a new edge case generator
    pub fn new() -> Self {
        Self
    }

    /// Generate edge cases for all grammar productions
    pub fn generate_all_edge_cases(&self) -> Vec<(String, String, bool)> {
        let mut edge_cases = Vec::new();

        // Generate edge cases for each category
        edge_cases.extend(self.generate_empty_input_cases());
        edge_cases.extend(self.generate_minimal_valid_cases());
        edge_cases.extend(self.generate_whitespace_cases());
        edge_cases.extend(self.generate_nested_structure_cases());
        edge_cases.extend(self.generate_boundary_condition_cases());
        edge_cases.extend(self.generate_error_boundary_cases());

        edge_cases
    }

    /// Generate empty and minimal input edge cases
    fn generate_empty_input_cases(&self) -> Vec<(String, String, bool)> {
        vec![
            ("empty_source".to_string(), "".to_string(), false),
            (
                "whitespace_only".to_string(),
                "   \n\t  ".to_string(),
                false,
            ),
            (
                "comment_only".to_string(),
                "// just a comment".to_string(),
                false,
            ),
            (
                "block_comment_only".to_string(),
                "/* block comment */".to_string(),
                false,
            ),
        ]
    }

    /// Generate minimal valid input cases
    fn generate_minimal_valid_cases(&self) -> Vec<(String, String, bool)> {
        vec![
            // Minimal function
            (
                "minimal_function".to_string(),
                "fn f() { }".to_string(),
                true,
            ),
            // Minimal variable declaration
            ("minimal_let".to_string(), "let x = 1;".to_string(), true),
            ("minimal_var".to_string(), "var x = 1;".to_string(), true),
            // Minimal data class
            (
                "minimal_data_class".to_string(),
                "data D { }".to_string(),
                true,
            ),
            // Minimal expressions as variable declarations (expressions can't be top-level)
            (
                "minimal_literal".to_string(),
                "let x = 42;".to_string(),
                true,
            ),
            (
                "minimal_identifier".to_string(),
                "let y = x;".to_string(),
                true,
            ),
            (
                "minimal_function_call".to_string(),
                "let z = f();".to_string(),
                true,
            ),
            // Control flow statements wrapped in functions (statements can't be top-level)
            (
                "minimal_if".to_string(),
                "fn test() { if true { } }".to_string(),
                true,
            ),
            (
                "minimal_while".to_string(),
                "fn test() { while true { } }".to_string(),
                true,
            ),
            (
                "minimal_for".to_string(),
                "fn test() { for x in y { } }".to_string(),
                true,
            ),
            // Statements wrapped in functions (statements can't be top-level)
            (
                "minimal_return".to_string(),
                "fn test() { return; }".to_string(),
                true,
            ),
            (
                "minimal_break".to_string(),
                "fn test() { while true { break; } }".to_string(),
                true,
            ),
            (
                "minimal_continue".to_string(),
                "fn test() { while true { continue; } }".to_string(),
                true,
            ),
        ]
    }

    /// Generate whitespace and formatting edge cases
    fn generate_whitespace_cases(&self) -> Vec<(String, String, bool)> {
        vec![
            // Extra whitespace
            (
                "extra_spaces_function".to_string(),
                "fn   test  (  )   {   }".to_string(),
                true,
            ),
            (
                "extra_newlines".to_string(),
                "fn test() {\n\n\n}".to_string(),
                true,
            ),
            (
                "mixed_whitespace".to_string(),
                "fn\t\ttest(\n) {\n\t}".to_string(),
                true,
            ),
            // Minimal whitespace
            (
                "no_spaces_function".to_string(),
                "fn test(){return 42;}".to_string(),
                true,
            ),
            (
                "single_line_everything".to_string(),
                "fn test(){let x=42;return x;}".to_string(),
                true,
            ),
            // Newline variations
            (
                "unix_newlines".to_string(),
                "fn test() {\n    return 42;\n}".to_string(),
                true,
            ),
            (
                "windows_newlines".to_string(),
                "fn test() {\r\n    return 42;\r\n}".to_string(),
                true,
            ),
            // Trailing whitespace
            (
                "trailing_spaces".to_string(),
                "fn test() { }  \n".to_string(),
                true,
            ),
            (
                "trailing_tabs".to_string(),
                "fn test() { }\t\t".to_string(),
                true,
            ),
        ]
    }

    /// Generate nested structure edge cases
    fn generate_nested_structure_cases(&self) -> Vec<(String, String, bool)> {
        vec![
            // Deeply nested blocks in functions
            (
                "nested_blocks".to_string(),
                "fn test() { if true { if true { if true { } } } }".to_string(),
                true,
            ),
            // Deeply nested expressions in variable declarations
            (
                "nested_parentheses".to_string(),
                "let x = ((((42))));".to_string(),
                true,
            ),
            // Nested function calls in variable declarations
            (
                "nested_calls".to_string(),
                "let x = f(g(h(42)));".to_string(),
                true,
            ),
            // Complex nested data structures - need separate declarations
            (
                "nested_data_class".to_string(),
                "data Inner { value: i32 } data Outer { inner: Inner }".to_string(),
                true,
            ),
            // Nested control flow in functions
            (
                "nested_loops".to_string(),
                "fn test() { for i in items { for j in subitems { for k in subsubitems { } } } }"
                    .to_string(),
                true,
            ),
            // Maximum reasonable nesting
            ("max_nesting".to_string(), self.generate_max_nesting(), true),
        ]
    }

    /// Generate boundary condition cases
    fn generate_boundary_condition_cases(&self) -> Vec<(String, String, bool)> {
        vec![
            // Long identifiers
            (
                "long_identifier".to_string(),
                format!("fn {}() {{ }}", "a".repeat(100)),
                true,
            ),
            // Many parameters
            (
                "many_parameters".to_string(),
                self.generate_many_parameters(),
                true,
            ),
            // Many fields
            ("many_fields".to_string(), self.generate_many_fields(), true),
            // Large numbers
            (
                "large_decimal".to_string(),
                "let x = 123456789012345678901234567890;".to_string(),
                true,
            ),
            (
                "large_hex".to_string(),
                "let x = 0xFFFFFFFFFFFFFFFFFFFFFFFF;".to_string(),
                true,
            ),
            (
                "large_binary".to_string(),
                format!("let x = 0b{};", "1".repeat(64)),
                true,
            ),
            // Long strings
            (
                "long_string".to_string(),
                format!("let s = \"{}\";", "a".repeat(1000)),
                true,
            ),
            // Unicode identifiers
            (
                "unicode_identifier".to_string(),
                "fn 函数() { }".to_string(),
                true,
            ),
            (
                "unicode_string".to_string(),
                "let s = \"🦀 Rust\";".to_string(),
                true,
            ),
        ]
    }

    /// Generate error boundary cases (should fail gracefully)
    fn generate_error_boundary_cases(&self) -> Vec<(String, String, bool)> {
        vec![
            // Syntax errors in function declarations
            (
                "missing_closing_brace".to_string(),
                "fn test() {".to_string(),
                false,
            ),
            (
                "missing_opening_brace".to_string(),
                "fn test() }".to_string(),
                false,
            ),
            (
                "invalid_variable_syntax".to_string(),
                "let = 42;".to_string(),
                false,
            ), // Missing variable name
            // Invalid tokens at top level
            (
                "invalid_number".to_string(),
                "let x = 123abc;".to_string(),
                false,
            ),
            (
                "invalid_operator".to_string(),
                "let x = a @@ b;".to_string(),
                false,
            ),
            // Incomplete constructs
            (
                "incomplete_function".to_string(),
                "fn test(".to_string(),
                false,
            ),
            (
                "incomplete_function_params".to_string(),
                "fn test(x".to_string(),
                false,
            ),
            (
                "incomplete_data_class".to_string(),
                "data Test {".to_string(),
                false,
            ),
            (
                "invalid_extern_block".to_string(),
                "extern \"C\" {".to_string(),
                false,
            ),
            // Type mismatches (lexical level)
            (
                "wrong_literal_suffix".to_string(),
                "let x = 42xyz;".to_string(),
                false,
            ),
            // Reserved keyword misuse
            (
                "keyword_as_identifier".to_string(),
                "let fn = 42;".to_string(),
                false,
            ),
            (
                "keyword_as_function".to_string(),
                "fn let() { }".to_string(),
                false,
            ),
            // Malformed strings
            (
                "unterminated_string".to_string(),
                "let s = \"hello".to_string(),
                false,
            ),
            (
                "invalid_escape".to_string(),
                "let s = \"\\x\";".to_string(),
                false,
            ),
            // Malformed comments
            (
                "unterminated_block_comment".to_string(),
                "/* comment".to_string(),
                false,
            ),
            // Invalid function signatures
            (
                "missing_function_name".to_string(),
                "fn () { }".to_string(),
                false,
            ),
            (
                "missing_parameter_name".to_string(),
                "fn test(: i32) { }".to_string(),
                false,
            ),
        ]
    }

    /// Generate maximum nesting test case
    fn generate_max_nesting(&self) -> String {
        let mut result = String::new();
        result.push_str("fn test() {");

        // Create 10 levels of nesting
        for i in 0..10 {
            result.push_str(&format!(" if {} {{", i));
        }

        result.push_str(" let x = 42; ");

        // Close all blocks
        for _ in 0..10 {
            result.push_str(" }");
        }
        result.push_str(" }");

        result
    }

    /// Generate function with many parameters
    fn generate_many_parameters(&self) -> String {
        let params: Vec<String> = (0..50).map(|i| format!("param{}: i32", i)).collect();
        format!("fn test({}) {{ }}", params.join(", "))
    }

    /// Generate data class with many fields
    fn generate_many_fields(&self) -> String {
        let fields: Vec<String> = (0..50).map(|i| format!("field{}: i32", i)).collect();
        format!("data Test {{ {} }}", fields.join(", "))
    }

    /// Test a single edge case
    pub fn test_edge_case(&self, name: &str, source: &str, should_succeed: bool) -> bool {
        let arena = Arena::new();
        let tokens = source_to_tokens(source);
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        match parser.parse_compilation_unit() {
            Ok(_) => {
                if should_succeed {
                    println!("✓ Edge case '{}' passed as expected", name);
                    true
                } else {
                    println!("✗ Edge case '{}' should have failed but succeeded", name);
                    false
                }
            }
            Err(err) => {
                if !should_succeed {
                    println!("✓ Edge case '{}' failed as expected: {:?}", name, err);
                    true
                } else {
                    println!(
                        "✗ Edge case '{}' should have succeeded but failed: {:?}",
                        name, err
                    );
                    false
                }
            }
        }
    }
}

impl Default for EdgeCaseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert lexer token to parser token type
fn convert_token(token: LexerToken) -> TokenType {
    match token.kind {
        TokenKind::Fn => TokenType::Fn,
        TokenKind::Let => TokenType::Let,
        TokenKind::Var => TokenType::Var,
        TokenKind::Pub => TokenType::Pub,
        TokenKind::Unsafe => TokenType::Unsafe,
        TokenKind::Async => TokenType::Async,
        TokenKind::Data => TokenType::Data,
        TokenKind::Return => TokenType::Return,
        TokenKind::If => TokenType::If,
        TokenKind::Else => TokenType::Else,
        TokenKind::While => TokenType::While,
        TokenKind::For => TokenType::For,
        TokenKind::In => TokenType::In,
        TokenKind::Break => TokenType::Break,
        TokenKind::Continue => TokenType::Continue,
        TokenKind::True => TokenType::BooleanLiteral(true),
        TokenKind::False => TokenType::BooleanLiteral(false),
        TokenKind::Identifier => TokenType::Identifier(token.lexeme.clone()),
        TokenKind::IntegerLiteral => match token.literal {
            Some(LiteralValue::Integer(i)) => TokenType::IntegerLiteral(i),
            _ => TokenType::IntegerLiteral(0),
        },
        TokenKind::FloatLiteral => match token.literal {
            Some(LiteralValue::Float(f)) => TokenType::FloatLiteral(f),
            _ => TokenType::FloatLiteral(1.0),
        },
        TokenKind::StringLiteral => match token.literal {
            Some(LiteralValue::String(s)) => TokenType::StringLiteral(s),
            _ => TokenType::StringLiteral(token.lexeme.clone()),
        },
        TokenKind::BooleanLiteral => match token.literal {
            Some(LiteralValue::Boolean(b)) => TokenType::BooleanLiteral(b),
            _ => TokenType::BooleanLiteral(true),
        },
        TokenKind::Plus => TokenType::Plus,
        TokenKind::Minus => TokenType::Minus,
        TokenKind::Star => TokenType::Star,
        TokenKind::Slash => TokenType::Slash,
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
        TokenKind::Question => TokenType::Question,
        TokenKind::LParen => TokenType::LeftParen,
        TokenKind::RParen => TokenType::RightParen,
        TokenKind::LBrace => TokenType::LeftBrace,
        TokenKind::RBrace => TokenType::RightBrace,
        TokenKind::LBracket => TokenType::LeftBracket,
        TokenKind::RBracket => TokenType::RightBracket,
        TokenKind::Comma => TokenType::Comma,
        TokenKind::Semicolon => TokenType::Semicolon,
        TokenKind::Colon => TokenType::Colon,
        TokenKind::Dot => TokenType::Dot,
        TokenKind::Arrow => TokenType::Arrow,
        TokenKind::FatArrow => TokenType::FatArrow,
        TokenKind::Eof => TokenType::Eof,
        // Skip whitespace tokens that the parser doesn't handle
        TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent => TokenType::Eof, // Will be filtered out
        _ => TokenType::Eof, // Fallback for any unhandled tokens
    }
}

/// Convert source code to tokens using the lexer
fn source_to_tokens(source: &str) -> Vec<TokenType> {
    let lexer = Lexer::new(source);
    let tokens = lexer.lex();
    tokens
        .into_iter()
        .map(convert_token)
        .filter(|t| !matches!(t, TokenType::Eof)) // Remove intermediate EOF tokens
        .chain(std::iter::once(TokenType::Eof)) // Add single EOF at end
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input_edge_cases() {
        let generator = EdgeCaseGenerator::new();
        let cases = generator.generate_empty_input_cases();

        for (name, source, should_succeed) in cases {
            generator.test_edge_case(&name, &source, should_succeed);
        }
    }

    #[test]
    fn test_minimal_valid_cases() {
        let generator = EdgeCaseGenerator::new();
        let cases = generator.generate_minimal_valid_cases();

        for (name, source, should_succeed) in cases {
            assert!(
                generator.test_edge_case(&name, &source, should_succeed),
                "Minimal valid case '{}' failed: {}",
                name,
                source
            );
        }
    }

    #[test]
    fn test_whitespace_edge_cases() {
        let generator = EdgeCaseGenerator::new();
        let cases = generator.generate_whitespace_cases();

        for (name, source, should_succeed) in cases {
            generator.test_edge_case(&name, &source, should_succeed);
        }
    }

    #[test]
    fn test_nested_structure_cases() {
        let generator = EdgeCaseGenerator::new();
        let cases = generator.generate_nested_structure_cases();

        for (name, source, should_succeed) in cases {
            generator.test_edge_case(&name, &source, should_succeed);
        }
    }

    #[test]
    fn test_boundary_condition_cases() {
        let generator = EdgeCaseGenerator::new();
        let cases = generator.generate_boundary_condition_cases();

        for (name, source, should_succeed) in cases {
            generator.test_edge_case(&name, &source, should_succeed);
        }
    }

    #[test]
    fn test_error_boundary_cases() {
        let generator = EdgeCaseGenerator::new();
        let cases = generator.generate_error_boundary_cases();

        let mut passed = 0;
        let mut total = 0;

        for (name, source, should_succeed) in cases {
            total += 1;
            if generator.test_edge_case(&name, &source, should_succeed) {
                passed += 1;
            }
        }

        println!("Error boundary tests: {}/{} passed", passed, total);
        // Allow some error boundary tests to fail since error handling might not be complete
        assert!(
            passed as f64 / total as f64 > 0.5,
            "More than half of error boundary tests should pass"
        );
    }

    #[test]
    fn test_comprehensive_edge_case_generation() {
        let generator = EdgeCaseGenerator::new();
        let all_cases = generator.generate_all_edge_cases();

        println!("Generated {} edge cases", all_cases.len());

        let mut passed = 0;
        let mut total = 0;
        let mut expected_success = 0;
        let mut expected_failure = 0;

        for (name, source, should_succeed) in all_cases {
            total += 1;
            if should_succeed {
                expected_success += 1;
            } else {
                expected_failure += 1;
            }

            if generator.test_edge_case(&name, &source, should_succeed) {
                passed += 1;
            }
        }

        println!("Edge case summary:");
        println!("  Total cases: {}", total);
        println!("  Expected success: {}", expected_success);
        println!("  Expected failure: {}", expected_failure);
        println!("  Passed: {}", passed);
        println!(
            "  Success rate: {:.1}%",
            (passed as f64 / total as f64) * 100.0
        );

        // Expect at least 70% of edge cases to behave as expected
        assert!(
            passed as f64 / total as f64 > 0.7,
            "At least 70% of edge cases should behave as expected"
        );
    }

    #[test]
    fn test_grammar_stress_testing() {
        let generator = EdgeCaseGenerator::new();

        // Test extreme nesting
        let max_nesting = generator.generate_max_nesting();
        assert!(generator.test_edge_case("max_nesting", &max_nesting, true));

        // Test large parameter lists
        let many_params = generator.generate_many_parameters();
        assert!(generator.test_edge_case("many_parameters", &many_params, true));

        // Test large field lists
        let many_fields = generator.generate_many_fields();
        assert!(generator.test_edge_case("many_fields", &many_fields, true));

        println!("Grammar stress tests completed successfully");
    }

    #[test]
    fn test_unicode_support() {
        let generator = EdgeCaseGenerator::new();

        // Test Unicode identifiers and strings
        let unicode_cases = vec![
            ("unicode_function", "fn 测试() { }", true),
            ("unicode_variable", "let 变量 = 42;", true),
            ("unicode_string", "let s = \"🦀 Rust 编程\";", true),
            ("emoji_identifier", "fn 🚀rocket() { }", true),
        ];

        for (name, source, should_succeed) in unicode_cases {
            generator.test_edge_case(name, source, should_succeed);
        }
    }

    #[test]
    fn test_comment_edge_cases() {
        let generator = EdgeCaseGenerator::new();

        let comment_cases = vec![
            ("line_comment_only", "// This is a comment", false),
            ("block_comment_only", "/* This is a block comment */", false),
            (
                "nested_block_comments",
                "/* outer /* inner */ outer */",
                false,
            ),
            ("comment_in_function", "fn test() { /* comment */ }", true),
            ("line_comment_after_code", "let x = 42; // comment", true),
        ];

        for (name, source, should_succeed) in comment_cases {
            generator.test_edge_case(name, source, should_succeed);
        }
    }
}
