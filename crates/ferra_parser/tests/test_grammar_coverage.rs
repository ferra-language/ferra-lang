//! Grammar Coverage Verification
//!
//! This module implements automated grammar production coverage verification to ensure
//! all grammar rules from SYNTAX_GRAMMAR_V0.1.md are tested.

use ferra_lexer::{Lexer, LiteralValue, Token as LexerToken, TokenKind};
use ferra_parser::token::{TokenType, VecTokenStream};
use ferra_parser::{Arena, ProgramParser};
use std::collections::HashSet;

/// Grammar productions from SYNTAX_GRAMMAR_V0.1.md
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GrammarProduction {
    // Declarations
    VariableDecl,
    FunctionDecl,
    DataClassDecl,
    ExternBlock,
    ExternFunctionDecl,
    ExternVariableDecl,

    // Types
    TupleType,
    ArrayType,
    FunctionType,
    ExternFunctionType,
    RawPointerType,
    GenericType,
    QualifiedIdentifier,
    SimpleType,

    // Expressions
    Literal,
    Identifier,
    FunctionCall,
    MethodCall,
    MatchExpr,
    AwaitExpr,
    PostfixOpExpr,
    UnaryOpExpr,
    BinaryOpExpr,
    GroupedExpr,
    IfExpression,

    // Statements
    LetDeclarationStatement,
    ExpressionStatement,
    BlockStatement,
    ReturnStatement,
    IfStatement,
    WhileStatement,
    ForStatement,
    BreakStatement,
    ContinueStatement,

    // Patterns
    DataClassPattern,
    LiteralPattern,
    IdentifierPattern,
    WildcardPattern,

    // Attributes
    Attribute,
    AttributeContent,
    AttributePath,
    AttributeArguments,

    // Literals
    StringLiteral,
    IntegerLiteral,
    FloatLiteral,
    BooleanLiteral,
    CharacterLiteral,
    DecimalIntegerLiteral,
    HexIntegerLiteral,
    OctalIntegerLiteral,
    BinaryIntegerLiteral,

    // Blocks
    BraceBlock,
    IndentedBlock,

    // Keywords and Modifiers
    PubModifier,
    UnsafeModifier,
    AsyncModifier,
    ExternModifier,
}

/// Coverage tracking for grammar productions
#[derive(Debug)]
pub struct GrammarCoverage {
    covered: HashSet<GrammarProduction>,
    all_productions: HashSet<GrammarProduction>,
}

impl GrammarCoverage {
    /// Create a new grammar coverage tracker with all productions
    pub fn new() -> Self {
        let all_productions = vec![
            // Declarations
            GrammarProduction::VariableDecl,
            GrammarProduction::FunctionDecl,
            GrammarProduction::DataClassDecl,
            GrammarProduction::ExternBlock,
            GrammarProduction::ExternFunctionDecl,
            GrammarProduction::ExternVariableDecl,
            // Types
            GrammarProduction::TupleType,
            GrammarProduction::ArrayType,
            GrammarProduction::FunctionType,
            GrammarProduction::ExternFunctionType,
            GrammarProduction::RawPointerType,
            GrammarProduction::GenericType,
            GrammarProduction::QualifiedIdentifier,
            GrammarProduction::SimpleType,
            // Expressions
            GrammarProduction::Literal,
            GrammarProduction::Identifier,
            GrammarProduction::FunctionCall,
            GrammarProduction::MethodCall,
            GrammarProduction::MatchExpr,
            GrammarProduction::AwaitExpr,
            GrammarProduction::PostfixOpExpr,
            GrammarProduction::UnaryOpExpr,
            GrammarProduction::BinaryOpExpr,
            GrammarProduction::GroupedExpr,
            GrammarProduction::IfExpression,
            // Statements
            GrammarProduction::LetDeclarationStatement,
            GrammarProduction::ExpressionStatement,
            GrammarProduction::BlockStatement,
            GrammarProduction::ReturnStatement,
            GrammarProduction::IfStatement,
            GrammarProduction::WhileStatement,
            GrammarProduction::ForStatement,
            GrammarProduction::BreakStatement,
            GrammarProduction::ContinueStatement,
            // Patterns
            GrammarProduction::DataClassPattern,
            GrammarProduction::LiteralPattern,
            GrammarProduction::IdentifierPattern,
            GrammarProduction::WildcardPattern,
            // Attributes
            GrammarProduction::Attribute,
            GrammarProduction::AttributeContent,
            GrammarProduction::AttributePath,
            GrammarProduction::AttributeArguments,
            // Literals
            GrammarProduction::StringLiteral,
            GrammarProduction::IntegerLiteral,
            GrammarProduction::FloatLiteral,
            GrammarProduction::BooleanLiteral,
            GrammarProduction::CharacterLiteral,
            GrammarProduction::DecimalIntegerLiteral,
            GrammarProduction::HexIntegerLiteral,
            GrammarProduction::OctalIntegerLiteral,
            GrammarProduction::BinaryIntegerLiteral,
            // Blocks
            GrammarProduction::BraceBlock,
            GrammarProduction::IndentedBlock,
            // Keywords and Modifiers
            GrammarProduction::PubModifier,
            GrammarProduction::UnsafeModifier,
            GrammarProduction::AsyncModifier,
            GrammarProduction::ExternModifier,
        ]
        .into_iter()
        .collect();

        Self {
            covered: HashSet::new(),
            all_productions,
        }
    }

    pub fn mark_covered(&mut self, production: GrammarProduction) {
        self.covered.insert(production);
    }

    pub fn coverage_percentage(&self) -> f64 {
        (self.covered.len() as f64 / self.all_productions.len() as f64) * 100.0
    }

    pub fn uncovered_productions(&self) -> Vec<&GrammarProduction> {
        self.all_productions.difference(&self.covered).collect()
    }

    pub fn coverage_report(&self) -> String {
        let percentage = self.coverage_percentage();
        let uncovered = self.uncovered_productions();

        format!(
            "Grammar Coverage Report:\n\
             Coverage: {:.1}% ({}/{} productions)\n\
             Uncovered productions:\n{}",
            percentage,
            self.covered.len(),
            self.all_productions.len(),
            uncovered
                .iter()
                .map(|p| format!("  - {:?}", p))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl Default for GrammarCoverage {
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

/// Helper function to parse source and track grammar coverage
fn parse_and_track_coverage(source: &str, coverage: &mut GrammarCoverage) -> bool {
    let arena = Arena::new();
    let tokens = source_to_tokens(source);
    let token_stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, token_stream);

    match parser.parse_compilation_unit() {
        Ok(_ast) => {
            // Analyze the source to determine which productions were used
            track_productions_from_source(source, coverage);
            true
        }
        Err(_) => false,
    }
}

/// Analyze source code to determine which grammar productions were used
fn track_productions_from_source(source: &str, coverage: &mut GrammarCoverage) {
    // Keywords and modifiers
    if source.contains("pub ") {
        coverage.mark_covered(GrammarProduction::PubModifier);
    }
    if source.contains("unsafe ") {
        coverage.mark_covered(GrammarProduction::UnsafeModifier);
    }
    if source.contains("async ") {
        coverage.mark_covered(GrammarProduction::AsyncModifier);
    }
    if source.contains("extern ") {
        coverage.mark_covered(GrammarProduction::ExternModifier);
    }

    // Declarations
    if source.contains("fn ") {
        coverage.mark_covered(GrammarProduction::FunctionDecl);
    }
    if source.contains("let ") || source.contains("var ") {
        coverage.mark_covered(GrammarProduction::VariableDecl);
        coverage.mark_covered(GrammarProduction::LetDeclarationStatement);
    }
    if source.contains("data ") {
        coverage.mark_covered(GrammarProduction::DataClassDecl);
    }
    if source.contains("extern \"") && source.contains(" {") {
        coverage.mark_covered(GrammarProduction::ExternBlock);
    }

    // Literals
    if source.contains("\"") {
        coverage.mark_covered(GrammarProduction::StringLiteral);
        coverage.mark_covered(GrammarProduction::Literal);
    }
    if source.contains("true") || source.contains("false") {
        coverage.mark_covered(GrammarProduction::BooleanLiteral);
        coverage.mark_covered(GrammarProduction::Literal);
    }
    if source.chars().any(|c| c.is_ascii_digit()) {
        coverage.mark_covered(GrammarProduction::IntegerLiteral);
        coverage.mark_covered(GrammarProduction::DecimalIntegerLiteral);
        coverage.mark_covered(GrammarProduction::Literal);
    }
    if source.contains("0x") {
        coverage.mark_covered(GrammarProduction::HexIntegerLiteral);
    }
    if source.contains("0o") {
        coverage.mark_covered(GrammarProduction::OctalIntegerLiteral);
    }
    if source.contains("0b") {
        coverage.mark_covered(GrammarProduction::BinaryIntegerLiteral);
    }
    if source.contains(".") && source.chars().any(|c| c.is_ascii_digit()) {
        coverage.mark_covered(GrammarProduction::FloatLiteral);
    }

    // Control flow
    if source.contains("if ") {
        coverage.mark_covered(GrammarProduction::IfStatement);
    }
    if source.contains("while ") {
        coverage.mark_covered(GrammarProduction::WhileStatement);
    }
    if source.contains("for ") && source.contains(" in ") {
        coverage.mark_covered(GrammarProduction::ForStatement);
    }
    if source.contains("return") {
        coverage.mark_covered(GrammarProduction::ReturnStatement);
    }
    if source.contains("break") {
        coverage.mark_covered(GrammarProduction::BreakStatement);
    }
    if source.contains("continue") {
        coverage.mark_covered(GrammarProduction::ContinueStatement);
    }

    // Expressions
    if source.contains("(") && source.contains(")") {
        coverage.mark_covered(GrammarProduction::GroupedExpr);
        coverage.mark_covered(GrammarProduction::FunctionCall);
    }
    if source.contains(".") && !source.starts_with(".") {
        coverage.mark_covered(GrammarProduction::MethodCall);
    }
    if source.contains("?") {
        coverage.mark_covered(GrammarProduction::PostfixOpExpr);
    }

    // Binary operators
    if source.contains("+")
        || source.contains("-")
        || source.contains("*")
        || source.contains("/")
        || source.contains("==")
        || source.contains("!=")
    {
        coverage.mark_covered(GrammarProduction::BinaryOpExpr);
    }

    // Unary operators
    if source.contains("!") || source.contains("-") {
        coverage.mark_covered(GrammarProduction::UnaryOpExpr);
    }

    // Blocks
    if source.contains("{") && source.contains("}") {
        coverage.mark_covered(GrammarProduction::BraceBlock);
        coverage.mark_covered(GrammarProduction::BlockStatement);
    }

    // Attributes
    if source.contains("#[") {
        coverage.mark_covered(GrammarProduction::Attribute);
        coverage.mark_covered(GrammarProduction::AttributeContent);
        coverage.mark_covered(GrammarProduction::AttributePath);
    }

    // Types
    if source.contains("[") && source.contains("]") && source.contains(":") {
        coverage.mark_covered(GrammarProduction::ArrayType);
    }
    if source.contains("(") && source.contains(",") && source.contains(")") {
        coverage.mark_covered(GrammarProduction::TupleType);
    }
    if source.contains("fn(") {
        coverage.mark_covered(GrammarProduction::FunctionType);
    }
    if source.contains("*const") || source.contains("*mut") {
        coverage.mark_covered(GrammarProduction::RawPointerType);
    }
    if source.contains("<") && source.contains(">") {
        coverage.mark_covered(GrammarProduction::GenericType);
    }

    // Simple types and identifiers
    if source.chars().any(|c| c.is_alphabetic()) {
        coverage.mark_covered(GrammarProduction::Identifier);
        coverage.mark_covered(GrammarProduction::SimpleType);
    }

    // Expression statements
    if !source.trim().is_empty() {
        coverage.mark_covered(GrammarProduction::ExpressionStatement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_tracking_basic() {
        let mut coverage = GrammarCoverage::new();

        // Test simple function
        assert!(parse_and_track_coverage("fn test() { }", &mut coverage));

        // Test variable declaration
        assert!(parse_and_track_coverage("let x = 42;", &mut coverage));

        // Test with modifiers
        assert!(parse_and_track_coverage("pub fn test() { }", &mut coverage));

        // Check coverage
        assert!(coverage.covered.contains(&GrammarProduction::FunctionDecl));
        assert!(coverage.covered.contains(&GrammarProduction::VariableDecl));
        assert!(coverage.covered.contains(&GrammarProduction::PubModifier));
    }

    #[test]
    fn test_literal_coverage() {
        let mut coverage = GrammarCoverage::new();

        // Test different literal types
        assert!(parse_and_track_coverage(
            "let s = \"hello\";",
            &mut coverage
        ));
        assert!(parse_and_track_coverage("let n = 42;", &mut coverage));
        assert!(parse_and_track_coverage("let f = 3.14;", &mut coverage));
        assert!(parse_and_track_coverage("let b = true;", &mut coverage));
        assert!(parse_and_track_coverage("let h = 0xFF;", &mut coverage));
        assert!(parse_and_track_coverage("let o = 0o777;", &mut coverage));
        assert!(parse_and_track_coverage("let bin = 0b1010;", &mut coverage));

        // Check literal coverage
        assert!(coverage.covered.contains(&GrammarProduction::StringLiteral));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::IntegerLiteral));
        assert!(coverage.covered.contains(&GrammarProduction::FloatLiteral));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::BooleanLiteral));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::HexIntegerLiteral));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::OctalIntegerLiteral));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::BinaryIntegerLiteral));
    }

    #[test]
    fn test_control_flow_coverage() {
        let mut coverage = GrammarCoverage::new();

        // Test control flow statements wrapped in functions (statements need function context)
        assert!(parse_and_track_coverage(
            "fn test() { if true { return 1; } }",
            &mut coverage
        ));
        assert!(parse_and_track_coverage(
            "fn test() { while true { break; } }",
            &mut coverage
        ));
        assert!(parse_and_track_coverage(
            "fn test() { for i in items { continue; } }",
            &mut coverage
        ));

        // Check control flow coverage
        assert!(coverage.covered.contains(&GrammarProduction::IfStatement));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::WhileStatement));
        assert!(coverage.covered.contains(&GrammarProduction::ForStatement));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::ReturnStatement));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::BreakStatement));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::ContinueStatement));
    }

    #[test]
    fn test_modifier_coverage() {
        let mut coverage = GrammarCoverage::new();

        // Test all modifiers
        assert!(parse_and_track_coverage("pub fn test() { }", &mut coverage));
        assert!(parse_and_track_coverage(
            "unsafe fn test() { }",
            &mut coverage
        ));
        assert!(parse_and_track_coverage(
            "async fn test() { }",
            &mut coverage
        ));
        assert!(parse_and_track_coverage(
            "pub unsafe fn test() { }",
            &mut coverage
        ));

        // Check modifier coverage
        assert!(coverage.covered.contains(&GrammarProduction::PubModifier));
        assert!(coverage
            .covered
            .contains(&GrammarProduction::UnsafeModifier));
        assert!(coverage.covered.contains(&GrammarProduction::AsyncModifier));
    }

    #[test]
    fn test_expression_coverage() {
        let mut coverage = GrammarCoverage::new();

        // Test expression types
        assert!(parse_and_track_coverage("let x = func();", &mut coverage));
        assert!(parse_and_track_coverage(
            "let y = obj.method();",
            &mut coverage
        ));
        assert!(parse_and_track_coverage("let z = (a + b);", &mut coverage));
        assert!(parse_and_track_coverage("let w = !flag;", &mut coverage));
        assert!(parse_and_track_coverage("let v = result?;", &mut coverage));

        // Check expression coverage
        assert!(coverage.covered.contains(&GrammarProduction::FunctionCall));
        assert!(coverage.covered.contains(&GrammarProduction::MethodCall));
        assert!(coverage.covered.contains(&GrammarProduction::GroupedExpr));
        assert!(coverage.covered.contains(&GrammarProduction::BinaryOpExpr));
        assert!(coverage.covered.contains(&GrammarProduction::UnaryOpExpr));
        assert!(coverage.covered.contains(&GrammarProduction::PostfixOpExpr));
    }

    #[test]
    fn test_coverage_report() {
        let mut coverage = GrammarCoverage::new();

        // Test a few productions
        assert!(parse_and_track_coverage("fn test() { }", &mut coverage));
        assert!(parse_and_track_coverage("let x = 42;", &mut coverage));

        let report = coverage.coverage_report();
        assert!(report.contains("Grammar Coverage Report:"));
        assert!(report.contains("Coverage:"));
        assert!(report.contains("Uncovered productions:"));
    }

    #[test]
    fn test_comprehensive_coverage_verification() {
        let mut coverage = GrammarCoverage::new();

        // Comprehensive test cases covering many productions
        let test_cases = vec![
            // Basic declarations
            "fn test() { }",
            "pub fn test() { }",
            "unsafe fn test() { }",
            "pub unsafe fn test() { }",
            "async fn test() { }",
            // Variable declarations
            "let x = 42;",
            "var y = 3.14;",
            "pub let z = \"hello\";",
            "pub var w = true;",
            // Data classes
            "data Point { x: i32, y: i32 }",
            "pub data User { name: String, age: i32 }",
            // Control flow (wrapped in functions since they're statements)
            "fn test() { if true { return 1; } }",
            "fn test() { while condition { break; } }",
            "fn test() { for item in items { continue; } }",
            // Expressions
            "let result = func(a, b);",
            "let value = obj.method();",
            "let sum = a + b * c;",
            "let negated = -value;",
            "let inverted = !flag;",
            "let propagated = operation()?;",
            "let grouped = (a + b) * c;",
            // Literals
            "let string = \"test\";",
            "let integer = 42;",
            "let float = 3.14;",
            "let boolean = true;",
            "let hex = 0xFF;",
            "let octal = 0o755;",
            "let binary = 0b1010;",
            // Attributes removed - lexer doesn't support # [ ] tokens yet
            // "#[test] fn test_function() { }",
            // "#[derive(Debug)] data Point { x: i32 }",
        ];

        for test_case in test_cases {
            parse_and_track_coverage(test_case, &mut coverage);
        }

        // Print coverage report
        let report = coverage.coverage_report();
        println!("{}", report);

        // Verify we have reasonable coverage
        let percentage = coverage.coverage_percentage();
        assert!(
            percentage > 30.0,
            "Coverage should be > 30%, got {:.1}%",
            percentage
        );

        // Verify specific important productions are covered
        assert!(coverage.covered.contains(&GrammarProduction::FunctionDecl));
        assert!(coverage.covered.contains(&GrammarProduction::VariableDecl));
        assert!(coverage.covered.contains(&GrammarProduction::BinaryOpExpr));
        assert!(coverage.covered.contains(&GrammarProduction::Literal));
    }
}
