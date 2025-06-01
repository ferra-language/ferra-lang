//! Parse error types and positive-first error messaging

use crate::token::{Span, Token};
use thiserror::Error;

/// Error severity levels for better diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Warning: non-blocking issues that should be addressed
    Warning,
    /// Error: blocking issues that prevent successful parsing
    Error,
    /// Fatal: critical issues that stop all parsing
    Fatal,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Warning => write!(f, "warning"),
            ErrorSeverity::Error => write!(f, "error"),
            ErrorSeverity::Fatal => write!(f, "fatal"),
        }
    }
}

/// Parse errors with location information and positive-first messaging
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Expected {expected}, but found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Expected expression")]
    ExpectedExpression {
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Expected statement")]
    ExpectedStatement {
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Expected type expression")]
    ExpectedType {
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Invalid block structure: {message}")]
    InvalidBlock {
        message: String,
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Mixed block styles are not allowed in the same block")]
    MixedBlockStyles {
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Inconsistent indentation")]
    InconsistentIndentation {
        span: Span,
        expected_level: usize,
        found_level: usize,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Expected block (either braced or indented)")]
    ExpectedBlock {
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Invalid indentation level")]
    InvalidIndentation {
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Variable '{variable}' is already defined in this scope")]
    VariableRedefinition {
        variable: String,
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Unexpected end of file")]
    UnexpectedEof {
        expected: String,
        span: Span,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Internal parser error: {message}")]
    Internal {
        message: String,
        span: Span,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Syntax error: {message}")]
    SyntaxError {
        message: String,
        span: Span,
        suggestion: Option<String>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },

    #[error("Recovery error: {message}")]
    RecoveryError {
        message: String,
        span: Span,
        original_error: Box<ParseError>,
        severity: ErrorSeverity,
        error_code: Option<&'static str>,
    },
}

impl ParseError {
    /// Create an unexpected token error with positive-first messaging
    pub fn unexpected_token(expected: &str, found: &Token) -> Self {
        let found_str = format!("{:?}", found.token_type);
        Self::UnexpectedToken {
            expected: expected.to_string(),
            found: found_str,
            span: found.span.clone(),
            suggestion: None,
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create an unexpected token error with suggestion
    pub fn unexpected_token_with_suggestion(
        expected: &str,
        found: &Token,
        suggestion: &str,
    ) -> Self {
        let found_str = format!("{:?}", found.token_type);
        Self::UnexpectedToken {
            expected: expected.to_string(),
            found: found_str,
            span: found.span.clone(),
            suggestion: Some(suggestion.to_string()),
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create an expected expression error
    pub fn expected_expression(span: Span) -> Self {
        Self::ExpectedExpression {
            span,
            suggestion: Some(
                "Consider adding a literal, identifier, or parenthesized expression".to_string(),
            ),
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create an expected statement error
    pub fn expected_statement(span: Span) -> Self {
        Self::ExpectedStatement {
            span,
            suggestion: Some(
                "Statements can be declarations (let, var, fn, data) or expressions".to_string(),
            ),
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create an expected type error
    pub fn expected_type(span: Span) -> Self {
        Self::ExpectedType {
            span,
            suggestion: Some(
                "Type expressions include identifiers, tuples, arrays, and function types"
                    .to_string(),
            ),
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create a mixed block styles error
    pub fn mixed_block_styles(span: Span) -> Self {
        Self::MixedBlockStyles {
            span,
            suggestion: Some(
                "Use either braces {...} OR indentation consistently within a single block"
                    .to_string(),
            ),
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create an inconsistent indentation error
    pub fn inconsistent_indentation(span: Span, expected: usize, found: usize) -> Self {
        Self::InconsistentIndentation {
            span,
            expected_level: expected,
            found_level: found,
            suggestion: Some(format!("All statements in an indented block must be at the same level (expected {} spaces)", expected)),
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create an expected block error
    pub fn expected_block(span: Span) -> Self {
        Self::ExpectedBlock {
            span,
            suggestion: Some("Consider adding a block (either braced or indented)".to_string()),
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create an invalid indentation error
    pub fn invalid_indentation(span: Span) -> Self {
        Self::InvalidIndentation {
            span,
            suggestion: Some("Check the indentation level of the block".to_string()),
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create a variable redefinition error
    pub fn variable_redefinition(variable: &str, span: Span) -> Self {
        Self::VariableRedefinition {
            variable: variable.to_string(),
            span: span.clone(),
            suggestion: None,
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Create an unexpected EOF error
    pub fn unexpected_eof(expected: &str, span: Span) -> Self {
        Self::UnexpectedEof {
            expected: expected.to_string(),
            span,
            severity: ErrorSeverity::Error,
            error_code: None,
        }
    }

    /// Get the span of this error
    pub fn span(&self) -> &Span {
        match self {
            Self::UnexpectedToken { span, .. } => span,
            Self::ExpectedExpression { span, .. } => span,
            Self::ExpectedStatement { span, .. } => span,
            Self::ExpectedType { span, .. } => span,
            Self::InvalidBlock { span, .. } => span,
            Self::MixedBlockStyles { span, .. } => span,
            Self::InconsistentIndentation { span, .. } => span,
            Self::ExpectedBlock { span, .. } => span,
            Self::InvalidIndentation { span, .. } => span,
            Self::VariableRedefinition { span, .. } => span,
            Self::UnexpectedEof { span, .. } => span,
            Self::Internal { span, .. } => span,
            Self::SyntaxError { span, .. } => span,
            Self::RecoveryError { span, .. } => span,
        }
    }

    /// Get the suggestion for this error, if any
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Self::UnexpectedToken { suggestion, .. } => suggestion.as_deref(),
            Self::ExpectedExpression { suggestion, .. } => suggestion.as_deref(),
            Self::ExpectedStatement { suggestion, .. } => suggestion.as_deref(),
            Self::ExpectedType { suggestion, .. } => suggestion.as_deref(),
            Self::InvalidBlock { suggestion, .. } => suggestion.as_deref(),
            Self::MixedBlockStyles { suggestion, .. } => suggestion.as_deref(),
            Self::InconsistentIndentation { suggestion, .. } => suggestion.as_deref(),
            Self::ExpectedBlock { suggestion, .. } => suggestion.as_deref(),
            Self::InvalidIndentation { suggestion, .. } => suggestion.as_deref(),
            Self::VariableRedefinition { suggestion, .. } => suggestion.as_deref(),
            Self::UnexpectedEof { .. } => None,
            Self::Internal { .. } => None,
            Self::SyntaxError { suggestion, .. } => suggestion.as_deref(),
            Self::RecoveryError { .. } => None,
        }
    }

    /// Get the severity of this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::UnexpectedToken { severity, .. } => *severity,
            Self::ExpectedExpression { severity, .. } => *severity,
            Self::ExpectedStatement { severity, .. } => *severity,
            Self::ExpectedType { severity, .. } => *severity,
            Self::InvalidBlock { severity, .. } => *severity,
            Self::MixedBlockStyles { severity, .. } => *severity,
            Self::InconsistentIndentation { severity, .. } => *severity,
            Self::ExpectedBlock { severity, .. } => *severity,
            Self::InvalidIndentation { severity, .. } => *severity,
            Self::VariableRedefinition { severity, .. } => *severity,
            Self::UnexpectedEof { severity, .. } => *severity,
            Self::Internal { severity, .. } => *severity,
            Self::SyntaxError { severity, .. } => *severity,
            Self::RecoveryError { severity, .. } => *severity,
        }
    }

    /// Get the error code, if any
    pub fn error_code(&self) -> Option<&'static str> {
        match self {
            Self::UnexpectedToken { error_code, .. } => *error_code,
            Self::ExpectedExpression { error_code, .. } => *error_code,
            Self::ExpectedStatement { error_code, .. } => *error_code,
            Self::ExpectedType { error_code, .. } => *error_code,
            Self::InvalidBlock { error_code, .. } => *error_code,
            Self::MixedBlockStyles { error_code, .. } => *error_code,
            Self::InconsistentIndentation { error_code, .. } => *error_code,
            Self::ExpectedBlock { error_code, .. } => *error_code,
            Self::InvalidIndentation { error_code, .. } => *error_code,
            Self::VariableRedefinition { error_code, .. } => *error_code,
            Self::UnexpectedEof { error_code, .. } => *error_code,
            Self::Internal { error_code, .. } => *error_code,
            Self::SyntaxError { error_code, .. } => *error_code,
            Self::RecoveryError { error_code, .. } => *error_code,
        }
    }

    /// Create a syntax error with custom message
    pub fn syntax_error(message: &str, span: Span) -> Self {
        Self::SyntaxError {
            message: message.to_string(),
            span,
            suggestion: None,
            severity: ErrorSeverity::Error,
            error_code: Some("E001"),
        }
    }

    /// Create a syntax error with suggestion
    pub fn syntax_error_with_suggestion(message: &str, span: Span, suggestion: &str) -> Self {
        Self::SyntaxError {
            message: message.to_string(),
            span,
            suggestion: Some(suggestion.to_string()),
            severity: ErrorSeverity::Error,
            error_code: Some("E001"),
        }
    }

    /// Create a recovery error that wraps another error
    pub fn recovery_error(message: &str, span: Span, original: ParseError) -> Self {
        Self::RecoveryError {
            message: message.to_string(),
            span,
            original_error: Box::new(original),
            severity: ErrorSeverity::Warning,
            error_code: Some("R001"),
        }
    }

    /// Create an internal error (fatal)
    pub fn internal(message: &str, span: Span) -> Self {
        Self::Internal {
            message: message.to_string(),
            span,
            severity: ErrorSeverity::Fatal,
            error_code: Some("I001"),
        }
    }

    /// Set the severity of this error
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        match &mut self {
            Self::UnexpectedToken { severity: s, .. } => *s = severity,
            Self::ExpectedExpression { severity: s, .. } => *s = severity,
            Self::ExpectedStatement { severity: s, .. } => *s = severity,
            Self::ExpectedType { severity: s, .. } => *s = severity,
            Self::InvalidBlock { severity: s, .. } => *s = severity,
            Self::MixedBlockStyles { severity: s, .. } => *s = severity,
            Self::InconsistentIndentation { severity: s, .. } => *s = severity,
            Self::ExpectedBlock { severity: s, .. } => *s = severity,
            Self::InvalidIndentation { severity: s, .. } => *s = severity,
            Self::VariableRedefinition { severity: s, .. } => *s = severity,
            Self::UnexpectedEof { severity: s, .. } => *s = severity,
            Self::Internal { severity: s, .. } => *s = severity,
            Self::SyntaxError { severity: s, .. } => *s = severity,
            Self::RecoveryError { severity: s, .. } => *s = severity,
        }
        self
    }

    /// Set the error code of this error
    pub fn with_error_code(mut self, code: &'static str) -> Self {
        match &mut self {
            Self::UnexpectedToken { error_code, .. } => *error_code = Some(code),
            Self::ExpectedExpression { error_code, .. } => *error_code = Some(code),
            Self::ExpectedStatement { error_code, .. } => *error_code = Some(code),
            Self::ExpectedType { error_code, .. } => *error_code = Some(code),
            Self::InvalidBlock { error_code, .. } => *error_code = Some(code),
            Self::MixedBlockStyles { error_code, .. } => *error_code = Some(code),
            Self::InconsistentIndentation { error_code, .. } => *error_code = Some(code),
            Self::ExpectedBlock { error_code, .. } => *error_code = Some(code),
            Self::InvalidIndentation { error_code, .. } => *error_code = Some(code),
            Self::VariableRedefinition { error_code, .. } => *error_code = Some(code),
            Self::UnexpectedEof { error_code, .. } => *error_code = Some(code),
            Self::Internal { error_code, .. } => *error_code = Some(code),
            Self::SyntaxError { error_code, .. } => *error_code = Some(code),
            Self::RecoveryError { error_code, .. } => *error_code = Some(code),
        }
        self
    }

    /// Format the error with enhanced diagnostics
    pub fn format_diagnostic(&self, source_name: Option<&str>) -> String {
        let severity = self.severity();
        let span = self.span();
        let error_code = self.error_code().unwrap_or("E000");

        let mut output = format!("{}: [{}] {}", severity, error_code, self);

        if let Some(source) = source_name {
            output.push('\n');
            output.push_str(&format!("  --> {}:{}:{}", source, span.line, span.column));
        } else {
            output.push('\n');
            output.push_str(&format!("  --> line {}:{}", span.line, span.column));
        }

        if let Some(suggestion) = self.suggestion() {
            output.push('\n');
            output.push_str(&format!("  help: {}", suggestion));
        }

        if let Self::RecoveryError { original_error, .. } = self {
            output.push('\n');
            output.push_str(&format!("  caused by: {}", original_error));
        }

        output
    }

    /// Check if this error should stop parsing
    pub fn should_stop_parsing(&self) -> bool {
        self.severity() == ErrorSeverity::Fatal
    }

    /// Check if this error can be recovered from
    pub fn is_recoverable(&self) -> bool {
        self.severity() != ErrorSeverity::Fatal
    }
}

/// Multi-error diagnostic report
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    /// All collected errors
    pub errors: Vec<ParseError>,
    /// Source file name (optional)
    pub source_name: Option<String>,
    /// Whether parsing was successful despite errors
    pub success: bool,
}

impl DiagnosticReport {
    /// Create a new diagnostic report
    pub fn new(source_name: Option<String>) -> Self {
        Self {
            errors: Vec::new(),
            source_name,
            success: true,
        }
    }

    /// Add an error to the report
    pub fn add_error(&mut self, error: ParseError) {
        if error.should_stop_parsing() {
            self.success = false;
        }
        self.errors.push(error);
    }

    /// Add multiple errors to the report
    pub fn add_errors(&mut self, errors: Vec<ParseError>) {
        for error in errors {
            self.add_error(error);
        }
    }

    /// Check if the report has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get all errors of a specific severity
    pub fn errors_with_severity(&self, severity: ErrorSeverity) -> Vec<&ParseError> {
        self.errors
            .iter()
            .filter(|e| e.severity() == severity)
            .collect()
    }

    /// Get error count by severity
    pub fn error_count_by_severity(&self, severity: ErrorSeverity) -> usize {
        self.errors
            .iter()
            .filter(|e| e.severity() == severity)
            .count()
    }

    /// Format the entire diagnostic report
    pub fn format_report(&self) -> String {
        if self.errors.is_empty() {
            return "No errors found.".to_string();
        }

        let mut output = String::new();

        // Summary
        let fatal_count = self.error_count_by_severity(ErrorSeverity::Fatal);
        let error_count = self.error_count_by_severity(ErrorSeverity::Error);
        let warning_count = self.error_count_by_severity(ErrorSeverity::Warning);

        output.push_str(&format!(
            "Parse result: {} ({} errors, {} warnings, {} fatal)\n\n",
            if self.success { "success" } else { "failed" },
            error_count,
            warning_count,
            fatal_count
        ));

        // Individual errors
        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(&error.format_diagnostic(self.source_name.as_deref()));
        }

        output
    }

    /// Check if parsing should continue based on error severity
    pub fn should_continue_parsing(&self) -> bool {
        self.success && !self.errors.iter().any(|e| e.should_stop_parsing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn test_unexpected_token_error() {
        let token = Token::dummy(TokenType::Plus);
        let error = ParseError::unexpected_token("identifier", &token);

        assert!(error.to_string().contains("Expected identifier"));
        assert!(error.to_string().contains("found Plus"));
    }

    #[test]
    fn test_error_with_suggestion() {
        let token = Token::dummy(TokenType::LeftBrace);
        let error = ParseError::unexpected_token_with_suggestion(
            "semicolon",
            &token,
            "Try adding a ';' to end the statement",
        );

        assert_eq!(
            error.suggestion(),
            Some("Try adding a ';' to end the statement")
        );
    }
}
