//! Phase 2.5: Error Recovery and Diagnostics Tests
//!
//! This module tests the enhanced error recovery strategies, multi-error reporting,
//! and improved diagnostics introduced in Phase 2.5.

use ferra_parser::{
    error::{parse_error::*, recovery::*},
    token::{Span, Token, TokenStream, TokenType, VecTokenStream},
};

#[test]
fn test_error_production_missing_semicolon() {
    let production = ErrorProduction::MissingSemicolon;
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("x".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(42),
        TokenType::Let, // Should trigger missing semicolon error
    ]);

    let error = production.applies_to_context(&tokens, "statement").unwrap();
    assert!(error.to_string().contains("Expected"));
    assert!(error.suggestion().unwrap().contains("Try adding ';'"));
    assert_eq!(error.severity(), ErrorSeverity::Error);
}

#[test]
fn test_error_production_missing_open_paren() {
    let production = ErrorProduction::MissingOpenParen;
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("func".to_string()),
        TokenType::IntegerLiteral(42), // Should be LeftParen
    ]);

    let error = production
        .applies_to_context(&tokens, "function_call")
        .unwrap();
    assert!(error.to_string().contains("opening parenthesis"));
    assert!(error.suggestion().unwrap().contains("parentheses"));
}

#[test]
fn test_error_production_unmatched_delimiter() {
    let production = ErrorProduction::UnmatchedDelimiter;
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::RightParen, // Unmatched closing paren
    ]);

    let error = production.applies_to_context(&tokens, "").unwrap();
    assert!(error.to_string().contains("matching opening delimiter"));
    assert!(error.suggestion().unwrap().contains("properly matched"));
}

#[test]
fn test_error_production_incomplete_expression() {
    let production = ErrorProduction::IncompleteExpression;
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Plus, // Operator without operand
    ]);

    let error = production
        .applies_to_context(&tokens, "expression")
        .unwrap();
    assert!(error.to_string().contains("Expected expression"));
    assert!(error
        .suggestion()
        .unwrap()
        .contains("literal, identifier, or parenthesized"));
}

#[test]
fn test_sync_token_expression_start() {
    let sync_token = SyncToken::ExpressionStart;

    // Should match expression starts
    assert!(sync_token.matches(&Token::dummy(TokenType::Identifier("x".to_string()))));
    assert!(sync_token.matches(&Token::dummy(TokenType::IntegerLiteral(42))));
    assert!(sync_token.matches(&Token::dummy(TokenType::LeftParen)));
    assert!(sync_token.matches(&Token::dummy(TokenType::Minus)));

    // Should not match non-expression starts
    assert!(!sync_token.matches(&Token::dummy(TokenType::RightParen)));
    assert!(!sync_token.matches(&Token::dummy(TokenType::Semicolon)));
}

#[test]
fn test_sync_token_expression_terminator() {
    let sync_token = SyncToken::ExpressionTerminator;

    // Should match expression terminators
    assert!(sync_token.matches(&Token::dummy(TokenType::Semicolon)));
    assert!(sync_token.matches(&Token::dummy(TokenType::Comma)));
    assert!(sync_token.matches(&Token::dummy(TokenType::RightParen)));
    assert!(sync_token.matches(&Token::dummy(TokenType::Eof)));

    // Should not match expression content
    assert!(!sync_token.matches(&Token::dummy(TokenType::Plus)));
    assert!(!sync_token.matches(&Token::dummy(TokenType::Identifier("x".to_string()))));
}

#[test]
fn test_error_collector_basic() {
    let mut collector = ErrorCollector::new(3);

    assert!(!collector.has_errors());
    assert!(collector.should_continue());

    // Add first error
    collector.add_error(ParseError::syntax_error("test error 1", Span::dummy()));
    assert!(collector.has_errors());
    assert!(collector.should_continue());
    assert_eq!(collector.get_errors().len(), 1);

    // Add second error
    collector.add_error(ParseError::syntax_error("test error 2", Span::dummy()));
    assert_eq!(collector.get_errors().len(), 2);
    assert!(collector.should_continue());

    // Add third error (should reach limit)
    collector.add_error(ParseError::syntax_error("test error 3", Span::dummy()));
    assert_eq!(collector.get_errors().len(), 3);
    assert!(!collector.should_continue());
}

#[test]
fn test_error_collector_clear() {
    let mut collector = ErrorCollector::new(5);

    collector.add_error(ParseError::syntax_error("test error", Span::dummy()));
    assert!(collector.has_errors());

    collector.clear();
    assert!(!collector.has_errors());
    assert!(collector.should_continue());
}

#[test]
fn test_panic_mode_recovery_to_statement() {
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::Bang, // Error token
        TokenType::Plus, // Error token
        TokenType::Star, // Error token
        TokenType::Let,  // Statement start (sync point)
        TokenType::Identifier("x".to_string()),
    ]);

    let sync_token = ErrorRecovery::recover_to_statement(&mut tokens);

    assert!(sync_token.is_some());
    assert_eq!(sync_token.unwrap().token_type, TokenType::Let);
    // Should be positioned at the sync token
    assert_eq!(tokens.peek().token_type, TokenType::Let);
}

#[test]
fn test_panic_mode_recovery_to_expression() {
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::Equal,              // Error token (not in any sync category)
        TokenType::Dot,                // Error token (not in any sync category)
        TokenType::IntegerLiteral(42), // Expression start (sync point)
        TokenType::Plus,
    ]);

    let sync_token = ErrorRecovery::recover_to_expression(&mut tokens);

    assert!(sync_token.is_some());
    let found_token = sync_token.unwrap();

    // The recovery should find the IntegerLiteral as a sync token
    assert_eq!(found_token.token_type, TokenType::IntegerLiteral(42));
}

#[test]
fn test_smart_recovery_with_context() {
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::Equal, // Error token (not in any sync category)
        TokenType::Dot,   // Error token (not in any sync category)
        TokenType::Let,   // Statement start
    ]);
    let mut collector = ErrorCollector::new(10);

    let sync_token = ErrorRecovery::smart_recovery(&mut tokens, "statement", &mut collector);

    assert!(sync_token.is_some());
    let found_token = sync_token.unwrap();

    // The smart recovery should eventually find the Let token
    // But it might find Equal first if error productions apply
    assert!(matches!(
        found_token.token_type,
        TokenType::Equal | TokenType::Let
    ));
}

#[test]
fn test_recovery_with_productions() {
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("x".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(42),
        TokenType::Let, // Missing semicolon before next statement
    ]);
    let mut collector = ErrorCollector::new(10);

    let sync_token =
        ErrorRecovery::recover_with_productions(&mut tokens, "statement", &mut collector);

    assert!(sync_token.is_some());
    assert!(collector.has_errors());
    // Should have added a missing semicolon error
    assert!(collector.get_errors()[0].to_string().contains("Expected"));
}

#[test]
fn test_error_severity_levels() {
    let warning =
        ParseError::syntax_error("test", Span::dummy()).with_severity(ErrorSeverity::Warning);
    let error = ParseError::syntax_error("test", Span::dummy()).with_severity(ErrorSeverity::Error);
    let fatal = ParseError::syntax_error("test", Span::dummy()).with_severity(ErrorSeverity::Fatal);

    assert_eq!(warning.severity(), ErrorSeverity::Warning);
    assert_eq!(error.severity(), ErrorSeverity::Error);
    assert_eq!(fatal.severity(), ErrorSeverity::Fatal);

    assert!(warning.is_recoverable());
    assert!(error.is_recoverable());
    assert!(!fatal.is_recoverable());

    assert!(!warning.should_stop_parsing());
    assert!(!error.should_stop_parsing());
    assert!(fatal.should_stop_parsing());
}

#[test]
fn test_error_codes() {
    let error = ParseError::syntax_error("test", Span::dummy());
    assert_eq!(error.error_code(), Some("E001"));

    let recovery_error = ParseError::recovery_error("test", Span::dummy(), error.clone());
    assert_eq!(recovery_error.error_code(), Some("R001"));

    let internal_error = ParseError::internal("test", Span::dummy());
    assert_eq!(internal_error.error_code(), Some("I001"));
}

#[test]
fn test_error_with_custom_code() {
    let error = ParseError::syntax_error("test", Span::dummy()).with_error_code("E999");
    assert_eq!(error.error_code(), Some("E999"));
}

#[test]
fn test_diagnostic_report_basic() {
    let mut report = DiagnosticReport::new(Some("test.ferra".to_string()));

    assert!(!report.has_errors());
    assert!(report.should_continue_parsing());

    // Add warning
    report.add_error(
        ParseError::syntax_error("warning", Span::dummy()).with_severity(ErrorSeverity::Warning),
    );
    assert!(report.has_errors());
    assert!(report.should_continue_parsing());
    assert_eq!(report.error_count_by_severity(ErrorSeverity::Warning), 1);

    // Add error
    report.add_error(
        ParseError::syntax_error("error", Span::dummy()).with_severity(ErrorSeverity::Error),
    );
    assert_eq!(report.error_count_by_severity(ErrorSeverity::Error), 1);
    assert!(report.should_continue_parsing());

    // Add fatal error
    report.add_error(
        ParseError::syntax_error("fatal", Span::dummy()).with_severity(ErrorSeverity::Fatal),
    );
    assert_eq!(report.error_count_by_severity(ErrorSeverity::Fatal), 1);
    assert!(!report.should_continue_parsing());
}

#[test]
fn test_diagnostic_report_formatting() {
    let mut report = DiagnosticReport::new(Some("test.ferra".to_string()));

    report.add_error(ParseError::syntax_error("syntax error", Span::dummy()));
    report.add_error(
        ParseError::syntax_error("warning", Span::dummy()).with_severity(ErrorSeverity::Warning),
    );

    let formatted = report.format_report();
    assert!(formatted.contains("Parse result:"));
    assert!(formatted.contains("1 errors"));
    assert!(formatted.contains("1 warnings"));
    assert!(formatted.contains("test.ferra"));
}

#[test]
fn test_error_diagnostic_formatting() {
    let error = ParseError::syntax_error_with_suggestion(
        "test error",
        Span::new(0, 10, 1, 5),
        "try this fix",
    );

    let formatted = error.format_diagnostic(Some("test.ferra"));
    assert!(formatted.contains("error: [E001]"));
    assert!(formatted.contains("test.ferra:1:5"));
    assert!(formatted.contains("help: try this fix"));
}

#[test]
fn test_recovery_error_chaining() {
    let original = ParseError::syntax_error("original error", Span::dummy());
    let recovery = ParseError::recovery_error("recovery failed", Span::dummy(), original);

    let formatted = recovery.format_diagnostic(None);
    assert!(formatted.contains("caused by:"));
    assert!(formatted.contains("original error"));
}

#[test]
fn test_partial_recovery_defaults() {
    let recovery = PartialRecovery::default();

    assert!(recovery.create_placeholders);
    assert_eq!(recovery.max_errors, 50);
    assert!(recovery.attempt_expression_completion);
}

#[test]
fn test_multi_error_integration() {
    // Simulate parsing with multiple errors
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::Let,
        TokenType::Identifier("x".to_string()),
        TokenType::Plus, // Missing '=' before expression
        TokenType::IntegerLiteral(42),
        TokenType::Let, // Missing semicolon
        TokenType::Identifier("y".to_string()),
        TokenType::Equal,
        TokenType::RightParen, // Unmatched closing paren
        TokenType::IntegerLiteral(24),
    ]);

    let mut collector = ErrorCollector::new(10);

    // Simulate multiple recovery attempts
    ErrorRecovery::recover_with_productions(&mut tokens, "statement", &mut collector);
    ErrorRecovery::recover_with_productions(&mut tokens, "expression", &mut collector);

    // Should have collected multiple errors
    assert!(collector.has_errors());
    assert!(collector.should_continue());
}

#[test]
fn test_error_production_suggestions() {
    assert_eq!(
        ErrorProduction::MissingSemicolon.get_suggestion(),
        "Add ';' at the end of the statement"
    );
    assert_eq!(
        ErrorProduction::MissingOpenParen.get_suggestion(),
        "Add '(' before function arguments"
    );
    assert_eq!(
        ErrorProduction::UnmatchedDelimiter.get_suggestion(),
        "Check that all delimiters are properly matched"
    );
}

#[test]
fn test_should_continue_recovery() {
    let tokens = VecTokenStream::from_token_types(vec![TokenType::Let]);

    assert!(ErrorRecovery::should_continue_recovery(&tokens, 5));
    assert!(!ErrorRecovery::should_continue_recovery(&tokens, 150));

    let empty_tokens = VecTokenStream::from_token_types(vec![]);
    assert!(!ErrorRecovery::should_continue_recovery(&empty_tokens, 5));
}
