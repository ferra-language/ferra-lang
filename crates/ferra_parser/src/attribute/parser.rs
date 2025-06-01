//! Attribute parser implementation for Phase 2.8.1
//!
//! Supports parsing of Ferra attribute syntax:
//! - #[derive(Debug, Clone)] - standard attributes with arguments
//! - #[inline] - simple attributes without arguments
//! - #[cfg(feature = "std")] - configuration attributes
//! - @inline - alternative syntax (future support)

use crate::{
    ast::Attribute,
    error::{ParseError, ParseResult},
    token::{Span, TokenStream, TokenType},
};

/// Parse a list of attributes from token stream
pub fn parse_attributes<T: TokenStream>(tokens: &mut T) -> ParseResult<Vec<Attribute>> {
    let mut parser = AttributeParser::new(tokens);
    parser.parse_attribute_list()
}

/// Parse a single attribute from token stream
pub fn parse_attribute<T: TokenStream>(tokens: &mut T) -> ParseResult<Attribute> {
    let mut parser = AttributeParser::new(tokens);
    parser.parse_attribute()
}

/// Comprehensive attribute parser for Phase 2.8.1
struct AttributeParser<'a, T: TokenStream> {
    tokens: &'a mut T,
}

impl<'a, T: TokenStream> AttributeParser<'a, T> {
    fn new(tokens: &'a mut T) -> Self {
        Self { tokens }
    }

    /// Parse a list of consecutive attributes
    fn parse_attribute_list(&mut self) -> ParseResult<Vec<Attribute>> {
        let mut attributes = Vec::new();

        // Parse consecutive attributes
        while matches!(self.tokens.peek().token_type, TokenType::Hash) {
            let attribute = self.parse_attribute()?;
            attributes.push(attribute);
        }

        Ok(attributes)
    }

    /// Parse a single attribute: #[name] or #[name(args)]
    fn parse_attribute(&mut self) -> ParseResult<Attribute> {
        let start_span = self.current_span();

        // Consume '#'
        let hash_token = self.tokens.consume();
        if !matches!(hash_token.token_type, TokenType::Hash) {
            return Err(ParseError::unexpected_token("'#'", &hash_token));
        }

        // Consume '['
        let open_bracket = self.tokens.consume();
        if !matches!(open_bracket.token_type, TokenType::LeftBracket) {
            return Err(ParseError::unexpected_token("'['", &open_bracket));
        }

        // Parse attribute name
        let name_token = self.tokens.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("attribute name", &name_token)),
        };

        // Parse optional arguments
        let arguments = if matches!(self.tokens.peek().token_type, TokenType::LeftParen) {
            self.parse_attribute_arguments()?
        } else {
            Vec::new()
        };

        // Consume ']'
        let close_bracket = self.tokens.consume();
        if !matches!(close_bracket.token_type, TokenType::RightBracket) {
            return Err(ParseError::unexpected_token("']'", &close_bracket));
        }

        Ok(Attribute {
            name,
            arguments,
            span: start_span,
        })
    }

    /// Parse attribute arguments: (arg1, arg2, ...)
    fn parse_attribute_arguments(&mut self) -> ParseResult<Vec<String>> {
        // Consume '('
        let open_paren = self.tokens.consume();
        if !matches!(open_paren.token_type, TokenType::LeftParen) {
            return Err(ParseError::unexpected_token("'('", &open_paren));
        }

        let mut arguments = Vec::new();

        // Handle empty argument list
        if matches!(self.tokens.peek().token_type, TokenType::RightParen) {
            self.tokens.consume(); // consume ')'
            return Ok(arguments);
        }

        // Parse argument list
        loop {
            let arg = self.parse_attribute_argument()?;
            arguments.push(arg);

            match self.tokens.peek().token_type {
                TokenType::Comma => {
                    self.tokens.consume(); // consume ','
                                           // Allow trailing comma
                    if matches!(self.tokens.peek().token_type, TokenType::RightParen) {
                        break;
                    }
                }
                TokenType::RightParen => break,
                _ => {
                    return Err(ParseError::unexpected_token(
                        "',' or ')'",
                        self.tokens.peek(),
                    ))
                }
            }
        }

        // Consume ')'
        let close_paren = self.tokens.consume();
        if !matches!(close_paren.token_type, TokenType::RightParen) {
            return Err(ParseError::unexpected_token("')'", &close_paren));
        }

        Ok(arguments)
    }

    /// Parse a single attribute argument (identifier, string, or simple expression)
    fn parse_attribute_argument(&mut self) -> ParseResult<String> {
        let mut result = String::new();
        let mut paren_depth = 0;
        let mut bracket_depth = 0;

        // Parse complex expressions by collecting tokens until we hit a delimiter
        loop {
            let token = self.tokens.peek();

            match &token.token_type {
                TokenType::Comma if paren_depth == 0 && bracket_depth == 0 => break,
                TokenType::RightParen if paren_depth == 0 => break,
                TokenType::RightBracket if bracket_depth == 0 => break,
                TokenType::Eof => break,

                TokenType::LeftParen => {
                    paren_depth += 1;
                    result.push('(');
                    self.tokens.consume();
                }
                TokenType::RightParen => {
                    paren_depth -= 1;
                    result.push(')');
                    self.tokens.consume();
                }
                TokenType::LeftBracket => {
                    bracket_depth += 1;
                    result.push('[');
                    self.tokens.consume();
                }
                TokenType::RightBracket => {
                    bracket_depth -= 1;
                    result.push(']');
                    self.tokens.consume();
                }

                TokenType::Identifier(name) => {
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(name);
                    self.tokens.consume();
                }
                TokenType::StringLiteral(s) => {
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&format!("\"{}\"", s));
                    self.tokens.consume();
                }
                TokenType::IntegerLiteral(i) => {
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&i.to_string());
                    self.tokens.consume();
                }
                TokenType::BooleanLiteral(b) => {
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&b.to_string());
                    self.tokens.consume();
                }
                TokenType::Equal => {
                    result.push_str(" = ");
                    self.tokens.consume();
                }
                TokenType::Comma => {
                    result.push_str(", ");
                    self.tokens.consume();
                }
                _ => {
                    // For any other token, just add it as a string representation
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&format!("{:?}", token.token_type));
                    self.tokens.consume();
                }
            }
        }

        if result.is_empty() {
            Err(ParseError::unexpected_token(
                "attribute argument",
                self.tokens.peek(),
            ))
        } else {
            Ok(result.trim().to_string())
        }
    }

    /// Get current token span
    fn current_span(&self) -> Span {
        self.tokens.peek().span.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::VecTokenStream;

    fn create_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
        VecTokenStream::from_token_types(token_types)
    }

    #[test]
    fn test_simple_attribute() {
        // #[inline]
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("inline".to_string()),
            TokenType::RightBracket,
        ]);

        let result = parse_attribute(&mut tokens).unwrap();
        assert_eq!(result.name, "inline");
        assert_eq!(result.arguments.len(), 0);
    }

    #[test]
    fn test_attribute_with_single_argument() {
        // #[cfg(test)]
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("cfg".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("test".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
        ]);

        let result = parse_attribute(&mut tokens).unwrap();
        assert_eq!(result.name, "cfg");
        assert_eq!(result.arguments.len(), 1);
        assert_eq!(result.arguments[0], "test");
    }

    #[test]
    fn test_attribute_with_multiple_arguments() {
        // #[derive(Debug, Clone)]
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("derive".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("Debug".to_string()),
            TokenType::Comma,
            TokenType::Identifier("Clone".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
        ]);

        let result = parse_attribute(&mut tokens).unwrap();
        assert_eq!(result.name, "derive");
        assert_eq!(result.arguments.len(), 2);
        assert_eq!(result.arguments[0], "Debug");
        assert_eq!(result.arguments[1], "Clone");
    }

    #[test]
    fn test_attribute_with_trailing_comma() {
        // #[derive(Debug, Clone,)]
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("derive".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("Debug".to_string()),
            TokenType::Comma,
            TokenType::Identifier("Clone".to_string()),
            TokenType::Comma,
            TokenType::RightParen,
            TokenType::RightBracket,
        ]);

        let result = parse_attribute(&mut tokens).unwrap();
        assert_eq!(result.name, "derive");
        assert_eq!(result.arguments.len(), 2);
        assert_eq!(result.arguments[0], "Debug");
        assert_eq!(result.arguments[1], "Clone");
    }

    #[test]
    fn test_attribute_with_string_argument() {
        // #[doc("This is documentation")]
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("doc".to_string()),
            TokenType::LeftParen,
            TokenType::StringLiteral("This is documentation".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
        ]);

        let result = parse_attribute(&mut tokens).unwrap();
        assert_eq!(result.name, "doc");
        assert_eq!(result.arguments.len(), 1);
        assert_eq!(result.arguments[0], "\"This is documentation\"");
    }

    #[test]
    fn test_attribute_with_mixed_arguments() {
        // #[test_attr("string", 42, true)]
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("test_attr".to_string()),
            TokenType::LeftParen,
            TokenType::StringLiteral("string".to_string()),
            TokenType::Comma,
            TokenType::IntegerLiteral(42),
            TokenType::Comma,
            TokenType::BooleanLiteral(true),
            TokenType::RightParen,
            TokenType::RightBracket,
        ]);

        let result = parse_attribute(&mut tokens).unwrap();
        assert_eq!(result.name, "test_attr");
        assert_eq!(result.arguments.len(), 3);
        assert_eq!(result.arguments[0], "\"string\"");
        assert_eq!(result.arguments[1], "42");
        assert_eq!(result.arguments[2], "true");
    }

    #[test]
    fn test_multiple_attributes() {
        // #[inline] #[derive(Debug)]
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("inline".to_string()),
            TokenType::RightBracket,
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("derive".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("Debug".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
        ]);

        let result = parse_attributes(&mut tokens).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "inline");
        assert_eq!(result[0].arguments.len(), 0);
        assert_eq!(result[1].name, "derive");
        assert_eq!(result[1].arguments.len(), 1);
        assert_eq!(result[1].arguments[0], "Debug");
    }

    #[test]
    fn test_empty_attribute_list() {
        // No attributes, just some other token
        let mut tokens = create_token_stream(vec![TokenType::Identifier("something".to_string())]);

        let result = parse_attributes(&mut tokens).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_attribute_parsing_errors() {
        // Missing opening bracket: #identifier
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::Identifier("inline".to_string()),
        ]);

        let result = parse_attribute(&mut tokens);
        assert!(result.is_err());

        // Missing closing bracket: #[inline
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("inline".to_string()),
        ]);

        let result = parse_attribute(&mut tokens);
        assert!(result.is_err());

        // Invalid attribute name: #[123]
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(123),
            TokenType::RightBracket,
        ]);

        let result = parse_attribute(&mut tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_derive_attribute() {
        // #[derive(Debug, Clone, PartialEq, Eq)]
        let mut tokens = create_token_stream(vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("derive".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("Debug".to_string()),
            TokenType::Comma,
            TokenType::Identifier("Clone".to_string()),
            TokenType::Comma,
            TokenType::Identifier("PartialEq".to_string()),
            TokenType::Comma,
            TokenType::Identifier("Eq".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
        ]);

        let result = parse_attribute(&mut tokens).unwrap();
        assert_eq!(result.name, "derive");
        assert_eq!(result.arguments.len(), 4);
        assert_eq!(result.arguments[0], "Debug");
        assert_eq!(result.arguments[1], "Clone");
        assert_eq!(result.arguments[2], "PartialEq");
        assert_eq!(result.arguments[3], "Eq");
    }
}
