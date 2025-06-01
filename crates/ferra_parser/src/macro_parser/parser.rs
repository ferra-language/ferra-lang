//! Macro parsing implementation
//!
//! Provides basic macro parsing functionality for Ferra language

use crate::{
    ast::{
        Arena, GroupDelimiter, MacroDefinition, MacroInvocation, MacroRule, TokenGroup, TokenTree,
    },
    error::ParseError,
    token::{TokenStream, TokenType},
};

/// Macro parser for handling macro invocations and definitions
pub struct MacroParser<'arena, T: TokenStream> {
    arena: &'arena Arena,
    tokens: T,
}

impl<'arena, T: TokenStream> MacroParser<'arena, T> {
    /// Create a new macro parser
    pub fn new(arena: &'arena Arena, tokens: T) -> Self {
        Self { arena, tokens }
    }

    /// Parse a macro invocation like `println!("Hello, world!")`
    pub fn parse_macro_invocation(
        &mut self,
        name: String,
    ) -> Result<&'arena MacroInvocation, ParseError> {
        // Expect '!' after macro name
        let bang_token = self.tokens.consume();
        if !matches!(bang_token.token_type, TokenType::Bang) {
            return Err(ParseError::unexpected_token("!", &bang_token));
        }

        // Parse arguments as token tree
        let arguments = self.parse_token_tree_group()?;

        Ok(self.arena.alloc(MacroInvocation {
            name,
            arguments: vec![TokenTree::Group(arguments)],
            span: bang_token.span.clone(),
        }))
    }

    /// Parse a token tree group (parentheses, brackets, or braces)
    fn parse_token_tree_group(&mut self) -> Result<TokenGroup, ParseError> {
        let open_token = self.tokens.consume();

        let delimiter = match open_token.token_type {
            TokenType::LeftParen => GroupDelimiter::Parentheses,
            TokenType::LeftBracket => GroupDelimiter::Brackets,
            TokenType::LeftBrace => GroupDelimiter::Braces,
            _ => {
                return Err(ParseError::unexpected_token(
                    "opening delimiter",
                    &open_token,
                ))
            }
        };

        let mut tokens = Vec::new();

        // Parse tokens until closing delimiter
        loop {
            let token = self.tokens.peek();

            match (&delimiter, &token.token_type) {
                (GroupDelimiter::Parentheses, TokenType::RightParen)
                | (GroupDelimiter::Brackets, TokenType::RightBracket)
                | (GroupDelimiter::Braces, TokenType::RightBrace) => {
                    let close_token = self.tokens.consume();
                    return Ok(TokenGroup {
                        delimiter,
                        tokens,
                        span: open_token.span.combine(close_token.span),
                    });
                }
                (_, TokenType::Eof) => {
                    return Err(ParseError::unexpected_token("closing delimiter", token));
                }
                _ => {
                    // Check if this is a nested group
                    if matches!(
                        token.token_type,
                        TokenType::LeftParen | TokenType::LeftBracket | TokenType::LeftBrace
                    ) {
                        let nested_group = self.parse_token_tree_group()?;
                        tokens.push(TokenTree::Group(nested_group));
                    } else {
                        let consumed_token = self.tokens.consume();
                        tokens.push(TokenTree::Token(consumed_token));
                    }
                }
            }
        }
    }

    /// Parse a basic macro definition (framework only)
    pub fn parse_macro_definition(
        &mut self,
        name: String,
    ) -> Result<&'arena MacroDefinition, ParseError> {
        // This is a basic framework implementation
        // In a full implementation, this would parse macro rules with patterns and replacements

        // For now, just parse a simple rule structure
        let _open_brace = self.tokens.consume(); // consume '{'
        if !matches!(_open_brace.token_type, TokenType::LeftBrace) {
            return Err(ParseError::unexpected_token("{", &_open_brace));
        }

        let mut rules = Vec::new();

        // Parse basic rule (pattern => replacement)
        while !matches!(
            self.tokens.peek().token_type,
            TokenType::RightBrace | TokenType::Eof
        ) {
            let rule = self.parse_macro_rule()?;
            rules.push(rule);

            // Check for continuation
            if matches!(self.tokens.peek().token_type, TokenType::Semicolon) {
                self.tokens.consume(); // consume ';'
            }
        }

        let close_brace = self.tokens.consume(); // consume '}'
        if !matches!(close_brace.token_type, TokenType::RightBrace) {
            return Err(ParseError::unexpected_token("}", &close_brace));
        }

        Ok(self.arena.alloc(MacroDefinition {
            name,
            rules,
            span: close_brace.span.clone(),
        }))
    }

    /// Parse a single macro rule (basic framework)
    fn parse_macro_rule(&mut self) -> Result<MacroRule, ParseError> {
        let mut pattern = Vec::new();
        let mut replacement = Vec::new();

        // Parse pattern until '=>'
        while !matches!(self.tokens.peek().token_type, TokenType::FatArrow) {
            let token = self.tokens.consume();
            if matches!(token.token_type, TokenType::Eof) {
                return Err(ParseError::unexpected_token("=>", &token));
            }
            pattern.push(TokenTree::Token(token));
        }

        // Consume '=>'
        let arrow_token = self.tokens.consume();
        if !matches!(arrow_token.token_type, TokenType::FatArrow) {
            return Err(ParseError::unexpected_token("=>", &arrow_token));
        }

        // Parse replacement until ';' or '}'
        while !matches!(
            self.tokens.peek().token_type,
            TokenType::Semicolon | TokenType::RightBrace
        ) {
            let token = self.tokens.consume();
            if matches!(token.token_type, TokenType::Eof) {
                break;
            }
            replacement.push(TokenTree::Token(token));
        }

        Ok(MacroRule {
            pattern,
            replacement,
            span: arrow_token.span.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ast::Arena, token::VecTokenStream};

    fn create_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
        VecTokenStream::from_token_types(token_types)
    }

    #[test]
    fn test_macro_parser_creation() {
        let arena = Arena::new();
        let tokens = create_token_stream(vec![TokenType::Eof]);
        let _parser = MacroParser::new(&arena, tokens);
    }

    #[test]
    fn test_simple_macro_invocation() {
        let arena = Arena::new();
        let tokens = create_token_stream(vec![
            TokenType::Bang,
            TokenType::LeftParen,
            TokenType::StringLiteral("hello".to_string()),
            TokenType::RightParen,
            TokenType::Eof,
        ]);
        let mut parser = MacroParser::new(&arena, tokens);

        let result = parser.parse_macro_invocation("println".to_string());
        assert!(result.is_ok());

        if let Ok(macro_invocation) = result {
            assert_eq!(macro_invocation.name, "println");
            assert_eq!(macro_invocation.arguments.len(), 1);
        }
    }

    #[test]
    fn test_nested_token_groups() {
        let arena = Arena::new();
        let tokens = create_token_stream(vec![
            TokenType::Bang,
            TokenType::LeftBrace,
            TokenType::LeftParen,
            TokenType::IntegerLiteral(42),
            TokenType::RightParen,
            TokenType::RightBrace,
            TokenType::Eof,
        ]);
        let mut parser = MacroParser::new(&arena, tokens);

        let result = parser.parse_macro_invocation("test".to_string());
        assert!(result.is_ok());

        if let Ok(macro_invocation) = result {
            assert_eq!(macro_invocation.name, "test");
            assert_eq!(macro_invocation.arguments.len(), 1);

            if let TokenTree::Group(group) = &macro_invocation.arguments[0] {
                assert!(matches!(group.delimiter, GroupDelimiter::Braces));
                assert_eq!(group.tokens.len(), 1);
            }
        }
    }

    #[test]
    fn test_macro_definition_basic() {
        let arena = Arena::new();
        let tokens = create_token_stream(vec![
            TokenType::LeftBrace,
            TokenType::Identifier("$x".to_string()),
            TokenType::FatArrow,
            TokenType::Identifier("$x".to_string()),
            TokenType::Plus,
            TokenType::IntegerLiteral(1),
            TokenType::RightBrace,
            TokenType::Eof,
        ]);
        let mut parser = MacroParser::new(&arena, tokens);

        let result = parser.parse_macro_definition("increment".to_string());
        assert!(result.is_ok());

        if let Ok(macro_def) = result {
            assert_eq!(macro_def.name, "increment");
            assert_eq!(macro_def.rules.len(), 1);
        }
    }

    #[test]
    fn test_macro_parsing_errors() {
        let arena = Arena::new();

        // Test missing closing delimiter
        let tokens = create_token_stream(vec![
            TokenType::Bang,
            TokenType::LeftParen,
            TokenType::StringLiteral("hello".to_string()),
            TokenType::Eof,
        ]);
        let mut parser = MacroParser::new(&arena, tokens);
        let result = parser.parse_macro_invocation("println".to_string());
        assert!(result.is_err());

        // Test missing bang
        let tokens = create_token_stream(vec![
            TokenType::LeftParen,
            TokenType::StringLiteral("hello".to_string()),
            TokenType::RightParen,
            TokenType::Eof,
        ]);
        let mut parser = MacroParser::new(&arena, tokens);
        let result = parser.parse_macro_invocation("println".to_string());
        assert!(result.is_err());
    }
}
