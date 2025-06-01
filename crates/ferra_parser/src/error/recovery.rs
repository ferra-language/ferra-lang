//! Error recovery strategies for continuing parsing after errors

use crate::error::ParseError;
use crate::token::{Token, TokenStream, TokenType};

/// Tokens that can be used for synchronization during error recovery
#[derive(Debug, Clone, PartialEq)]
pub enum SyncToken {
    StatementStart,
    DeclarationStart,
    BlockEnd,
    StatementTerminator,
    ExpressionStart,
    ExpressionTerminator,
}

impl SyncToken {
    /// Check if a token can be used for synchronization
    pub fn matches(&self, token: &Token) -> bool {
        match self {
            SyncToken::StatementStart => matches!(
                token.token_type,
                TokenType::Let
                    | TokenType::Var
                    | TokenType::If
                    | TokenType::While
                    | TokenType::For
                    | TokenType::Return
                    | TokenType::Break
                    | TokenType::Continue
                    | TokenType::Match
            ),
            SyncToken::DeclarationStart => matches!(
                token.token_type,
                TokenType::Let
                    | TokenType::Var
                    | TokenType::Fn
                    | TokenType::Data
                    | TokenType::Extern
            ),
            SyncToken::BlockEnd => {
                matches!(token.token_type, TokenType::RightBrace | TokenType::Dedent)
            }
            SyncToken::StatementTerminator => matches!(
                token.token_type,
                TokenType::Semicolon
                    | TokenType::Newline
                    | TokenType::RightBrace
                    | TokenType::Dedent
                    | TokenType::Eof
            ),
            SyncToken::ExpressionStart => matches!(
                token.token_type,
                TokenType::Identifier(_)
                    | TokenType::IntegerLiteral(_)
                    | TokenType::FloatLiteral(_)
                    | TokenType::StringLiteral(_)
                    | TokenType::BooleanLiteral(_)
                    | TokenType::LeftParen
                    | TokenType::LeftBracket
                    | TokenType::Minus
                    | TokenType::Bang
            ),
            SyncToken::ExpressionTerminator => matches!(
                token.token_type,
                TokenType::Semicolon
                    | TokenType::Comma
                    | TokenType::RightParen
                    | TokenType::RightBracket
                    | TokenType::RightBrace
                    | TokenType::Newline
                    | TokenType::Eof
            ),
        }
    }
}

/// Error production rules for handling common syntax errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorProduction {
    /// Missing semicolon after statement
    MissingSemicolon,
    /// Missing opening parenthesis in function call
    MissingOpenParen,
    /// Missing closing parenthesis in function call
    MissingCloseParen,
    /// Missing opening brace in block
    MissingOpenBrace,
    /// Missing closing brace in block
    MissingCloseBrace,
    /// Unmatched delimiter
    UnmatchedDelimiter,
    /// Incomplete expression
    IncompleteExpression,
    /// Invalid operator usage
    InvalidOperator,
}

impl ErrorProduction {
    /// Check if this error production applies to the current parsing context
    pub fn applies_to_context<T: TokenStream>(
        &self,
        tokens: &T,
        context: &str,
    ) -> Option<ParseError> {
        let current = tokens.peek();

        match self {
            ErrorProduction::MissingSemicolon => {
                if context.contains("statement")
                    && !matches!(
                        current.token_type,
                        TokenType::Semicolon
                            | TokenType::RightBrace
                            | TokenType::Dedent
                            | TokenType::Eof
                    )
                {
                    Some(ParseError::unexpected_token_with_suggestion(
                        "semicolon or newline",
                        current,
                        "Try adding ';' at the end of the statement",
                    ))
                } else {
                    None
                }
            }
            ErrorProduction::MissingOpenParen => {
                if context.contains("function_call")
                    && !matches!(current.token_type, TokenType::LeftParen)
                {
                    Some(ParseError::unexpected_token_with_suggestion(
                        "opening parenthesis",
                        current,
                        "Function calls require parentheses: func(args)",
                    ))
                } else {
                    None
                }
            }
            ErrorProduction::MissingCloseParen => {
                if context.contains("parenthesized")
                    && !matches!(current.token_type, TokenType::RightParen)
                {
                    Some(ParseError::unexpected_token_with_suggestion(
                        "closing parenthesis",
                        current,
                        "Check for matching parentheses",
                    ))
                } else {
                    None
                }
            }
            ErrorProduction::MissingOpenBrace => {
                if context.contains("block")
                    && !matches!(current.token_type, TokenType::LeftBrace | TokenType::Colon)
                {
                    Some(ParseError::expected_block(current.span.clone()))
                } else {
                    None
                }
            }
            ErrorProduction::MissingCloseBrace => {
                if context.contains("braced_block")
                    && !matches!(current.token_type, TokenType::RightBrace)
                {
                    Some(ParseError::unexpected_token_with_suggestion(
                        "closing brace",
                        current,
                        "Check for matching braces in blocks",
                    ))
                } else {
                    None
                }
            }
            ErrorProduction::UnmatchedDelimiter => match current.token_type {
                TokenType::RightParen | TokenType::RightBracket | TokenType::RightBrace => {
                    Some(ParseError::unexpected_token_with_suggestion(
                        "matching opening delimiter",
                        current,
                        "Check that all delimiters are properly matched",
                    ))
                }
                _ => None,
            },
            ErrorProduction::IncompleteExpression => {
                if context.contains("expression")
                    && matches!(
                        current.token_type,
                        TokenType::Plus
                            | TokenType::Minus
                            | TokenType::Star
                            | TokenType::Slash
                            | TokenType::Equal
                            | TokenType::EqualEqual
                            | TokenType::BangEqual
                            | TokenType::Less
                            | TokenType::Greater
                            | TokenType::LessEqual
                            | TokenType::GreaterEqual
                    )
                {
                    Some(ParseError::expected_expression(current.span.clone()))
                } else {
                    None
                }
            }
            ErrorProduction::InvalidOperator => {
                if context.contains("operator") {
                    Some(ParseError::unexpected_token_with_suggestion(
                        "valid operator",
                        current,
                        "Check operator precedence and usage",
                    ))
                } else {
                    None
                }
            }
        }
    }

    /// Get a suggested fix for this error production
    pub fn get_suggestion(&self) -> &'static str {
        match self {
            ErrorProduction::MissingSemicolon => "Add ';' at the end of the statement",
            ErrorProduction::MissingOpenParen => "Add '(' before function arguments",
            ErrorProduction::MissingCloseParen => "Add ')' to close the parentheses",
            ErrorProduction::MissingOpenBrace => {
                "Add '{' to start a block or ':' for indented block"
            }
            ErrorProduction::MissingCloseBrace => "Add '}' to close the block",
            ErrorProduction::UnmatchedDelimiter => "Check that all delimiters are properly matched",
            ErrorProduction::IncompleteExpression => {
                "Complete the expression with a value or identifier"
            }
            ErrorProduction::InvalidOperator => "Use a valid operator for this context",
        }
    }
}

/// Recovery strategy for building partial AST nodes
#[derive(Debug, Clone)]
pub struct PartialRecovery {
    /// Whether to create placeholder nodes for missing elements
    pub create_placeholders: bool,
    /// Maximum number of errors to collect before giving up
    pub max_errors: usize,
    /// Whether to attempt expression completion
    pub attempt_expression_completion: bool,
}

impl Default for PartialRecovery {
    fn default() -> Self {
        Self {
            create_placeholders: true,
            max_errors: 50,
            attempt_expression_completion: true,
        }
    }
}

/// Multi-error collector for gathering multiple parse errors
#[derive(Debug, Clone)]
pub struct ErrorCollector {
    /// Collected errors
    pub errors: Vec<ParseError>,
    /// Maximum errors to collect
    pub max_errors: usize,
    /// Whether to continue parsing after errors
    pub continue_on_error: bool,
}

impl ErrorCollector {
    /// Create a new error collector
    pub fn new(max_errors: usize) -> Self {
        Self {
            errors: Vec::new(),
            max_errors,
            continue_on_error: true,
        }
    }

    /// Add an error to the collection
    pub fn add_error(&mut self, error: ParseError) {
        if self.errors.len() < self.max_errors {
            self.errors.push(error);
        }

        if self.errors.len() >= self.max_errors {
            self.continue_on_error = false;
        }
    }

    /// Check if we should continue parsing
    pub fn should_continue(&self) -> bool {
        self.continue_on_error && self.errors.len() < self.max_errors
    }

    /// Get all collected errors
    pub fn get_errors(&self) -> &[ParseError] {
        &self.errors
    }

    /// Check if any errors were collected
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the most recent error
    pub fn last_error(&self) -> Option<&ParseError> {
        self.errors.last()
    }

    /// Clear all errors
    pub fn clear(&mut self) {
        self.errors.clear();
        self.continue_on_error = true;
    }
}

/// Enhanced error recovery strategies
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Panic mode recovery: skip tokens until a synchronizing token is found
    pub fn panic_mode_recovery<T: TokenStream>(
        tokens: &mut T,
        sync_tokens: &[SyncToken],
    ) -> Option<Token> {
        while !tokens.is_at_end() {
            let current = tokens.peek();

            // Check if current token is a synchronizing token
            for sync_token in sync_tokens {
                if sync_token.matches(current) {
                    return Some(current.clone());
                }
            }

            // Skip the current token
            tokens.consume();
        }

        None
    }

    /// Recover to the next statement boundary
    pub fn recover_to_statement<T: TokenStream>(tokens: &mut T) -> Option<Token> {
        Self::panic_mode_recovery(
            tokens,
            &[SyncToken::StatementStart, SyncToken::StatementTerminator],
        )
    }

    /// Recover to the next declaration boundary
    pub fn recover_to_declaration<T: TokenStream>(tokens: &mut T) -> Option<Token> {
        Self::panic_mode_recovery(tokens, &[SyncToken::DeclarationStart])
    }

    /// Recover to the end of a block
    pub fn recover_to_block_end<T: TokenStream>(tokens: &mut T) -> Option<Token> {
        Self::panic_mode_recovery(tokens, &[SyncToken::BlockEnd])
    }

    /// Recover to the next expression boundary
    pub fn recover_to_expression<T: TokenStream>(tokens: &mut T) -> Option<Token> {
        Self::panic_mode_recovery(
            tokens,
            &[SyncToken::ExpressionStart, SyncToken::ExpressionTerminator],
        )
    }

    /// Advanced recovery with error production rules
    pub fn recover_with_productions<T: TokenStream>(
        tokens: &mut T,
        context: &str,
        collector: &mut ErrorCollector,
    ) -> Option<Token> {
        let productions = [
            ErrorProduction::MissingSemicolon,
            ErrorProduction::MissingOpenParen,
            ErrorProduction::MissingCloseParen,
            ErrorProduction::MissingOpenBrace,
            ErrorProduction::MissingCloseBrace,
            ErrorProduction::UnmatchedDelimiter,
            ErrorProduction::IncompleteExpression,
            ErrorProduction::InvalidOperator,
        ];

        for production in &productions {
            if let Some(error) = production.applies_to_context(tokens, context) {
                collector.add_error(error);

                // Try to insert missing token or skip problematic token
                match production {
                    ErrorProduction::MissingSemicolon => {
                        // Continue parsing, assuming semicolon is optional
                        return Some(tokens.peek().clone());
                    }
                    ErrorProduction::MissingOpenParen
                    | ErrorProduction::MissingCloseParen
                    | ErrorProduction::MissingOpenBrace
                    | ErrorProduction::MissingCloseBrace => {
                        // Skip to next synchronization point
                        return Self::panic_mode_recovery(
                            tokens,
                            &[
                                SyncToken::StatementTerminator,
                                SyncToken::ExpressionTerminator,
                            ],
                        );
                    }
                    ErrorProduction::UnmatchedDelimiter => {
                        // Skip the unmatched delimiter
                        tokens.consume();
                        if !tokens.is_at_end() {
                            return Some(tokens.peek().clone());
                        }
                    }
                    ErrorProduction::IncompleteExpression | ErrorProduction::InvalidOperator => {
                        // Skip to next expression start
                        return Self::recover_to_expression(tokens);
                    }
                }
            }
        }

        // Fallback to panic mode recovery
        Self::panic_mode_recovery(
            tokens,
            &[SyncToken::StatementStart, SyncToken::StatementTerminator],
        )
    }

    /// Check if we should attempt recovery or give up
    pub fn should_continue_recovery<T: TokenStream>(tokens: &T, error_count: usize) -> bool {
        // Continue if we haven't hit too many errors and aren't at EOF
        error_count < 100 && !tokens.is_at_end()
    }

    /// Smart recovery that preserves context
    pub fn smart_recovery<T: TokenStream>(
        tokens: &mut T,
        expected_context: &str,
        collector: &mut ErrorCollector,
    ) -> Option<Token> {
        // First try production-based recovery
        if let Some(token) = Self::recover_with_productions(tokens, expected_context, collector) {
            return Some(token);
        }

        // Then try context-specific recovery
        if expected_context.contains("expression") {
            Self::recover_to_expression(tokens)
        } else if expected_context.contains("statement") {
            Self::recover_to_statement(tokens)
        } else if expected_context.contains("declaration") {
            Self::recover_to_declaration(tokens)
        } else if expected_context.contains("block") {
            Self::recover_to_block_end(tokens)
        } else {
            // Generic recovery
            Self::panic_mode_recovery(
                tokens,
                &[SyncToken::StatementStart, SyncToken::StatementTerminator],
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{TokenType, VecTokenStream};

    #[test]
    fn test_sync_token_matches() {
        let token = Token::dummy(TokenType::Let);
        assert!(SyncToken::StatementStart.matches(&token));
        assert!(SyncToken::DeclarationStart.matches(&token));

        let token = Token::dummy(TokenType::RightBrace);
        assert!(SyncToken::BlockEnd.matches(&token));
        assert!(SyncToken::StatementTerminator.matches(&token));
    }

    #[test]
    fn test_panic_mode_recovery() {
        let tokens = vec![
            TokenType::Plus, // Error token
            TokenType::Star, // Error token
            TokenType::Let,  // Sync token
            TokenType::Identifier("x".to_string()),
        ];
        let mut stream = VecTokenStream::from_token_types(tokens);

        let sync_token =
            ErrorRecovery::panic_mode_recovery(&mut stream, &[SyncToken::StatementStart]);

        assert!(sync_token.is_some());
        assert_eq!(sync_token.unwrap().token_type, TokenType::Let);
        assert_eq!(stream.peek().token_type, TokenType::Let);
    }

    #[test]
    fn test_recover_to_statement() {
        let tokens = vec![
            TokenType::Bang,      // Error token
            TokenType::Plus,      // Error token
            TokenType::Semicolon, // Statement terminator
            TokenType::Let,
        ];
        let mut stream = VecTokenStream::from_token_types(tokens);

        let sync_token = ErrorRecovery::recover_to_statement(&mut stream);

        assert!(sync_token.is_some());
        assert_eq!(sync_token.unwrap().token_type, TokenType::Semicolon);
    }

    #[test]
    fn test_should_continue_recovery() {
        let tokens = vec![TokenType::Let];
        let stream = VecTokenStream::from_token_types(tokens);

        assert!(ErrorRecovery::should_continue_recovery(&stream, 5));
        assert!(!ErrorRecovery::should_continue_recovery(&stream, 101));
    }
}
