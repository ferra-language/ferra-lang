//! Token stream interface and implementations
//!
//! Provides traits and implementations for consuming tokens during parsing,
//! including mock implementations for testing and development.

use super::{Token, TokenType};

/// Trait for token streams that can be consumed by the parser
pub trait TokenStream {
    /// Peek at the current token without consuming it
    fn peek(&self) -> &Token;

    /// Peek at the token at the given offset from current position
    fn peek_ahead(&self, offset: usize) -> Option<&Token>;

    /// Consume and return the current token
    fn consume(&mut self) -> Token;

    /// Check if we're at the end of the stream
    fn is_at_end(&self) -> bool;

    /// Get the current position in the stream
    fn position(&self) -> usize;
}

/// A simple vector-based token stream for testing and development
#[derive(Debug, Clone)]
pub struct VecTokenStream {
    tokens: Vec<Token>,
    current: usize,
}

impl VecTokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut stream = Self { tokens, current: 0 };

        // Ensure there's always an EOF token at the end
        if stream.tokens.is_empty() || !stream.tokens.last().unwrap().is_eof() {
            stream.tokens.push(Token::dummy(TokenType::Eof));
        }

        stream
    }

    pub fn from_token_types(token_types: Vec<TokenType>) -> Self {
        let tokens = token_types.into_iter().map(Token::dummy).collect();
        Self::new(tokens)
    }
}

impl TokenStream for VecTokenStream {
    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.current + offset)
    }

    fn consume(&mut self) -> Token {
        let token = self.peek().clone();
        if !self.is_at_end() {
            self.current += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() - 1 || self.peek().is_eof()
    }

    fn position(&self) -> usize {
        self.current
    }
}

impl Default for VecTokenStream {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl<T: TokenStream> TokenStream for &mut T {
    fn peek(&self) -> &Token {
        (**self).peek()
    }

    fn consume(&mut self) -> Token {
        (**self).consume()
    }

    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        (**self).peek_ahead(offset)
    }

    fn is_at_end(&self) -> bool {
        (**self).is_at_end()
    }

    fn position(&self) -> usize {
        (**self).position()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_token_stream_basic() {
        let tokens = vec![
            TokenType::Let,
            TokenType::Identifier("x".to_string()),
            TokenType::Equal,
            TokenType::IntegerLiteral(42),
        ];
        let mut stream = VecTokenStream::from_token_types(tokens);

        assert_eq!(stream.peek().token_type, TokenType::Let);
        assert_eq!(stream.position(), 0);

        let token = stream.consume();
        assert_eq!(token.token_type, TokenType::Let);
        assert_eq!(stream.position(), 1);

        assert_eq!(
            stream.peek().token_type,
            TokenType::Identifier("x".to_string())
        );
    }

    #[test]
    fn test_vec_token_stream_eof() {
        let mut stream = VecTokenStream::from_token_types(vec![TokenType::Let]);

        stream.consume(); // consume Let
        assert_eq!(stream.peek().token_type, TokenType::Eof);
        assert!(stream.is_at_end());

        // Should stay at EOF
        let eof_token = stream.consume();
        assert_eq!(eof_token.token_type, TokenType::Eof);
        assert!(stream.is_at_end());
    }

    #[test]
    fn test_peek_ahead() {
        let tokens = vec![
            TokenType::Let,
            TokenType::Identifier("x".to_string()),
            TokenType::Equal,
        ];
        let stream = VecTokenStream::from_token_types(tokens);

        assert_eq!(stream.peek_ahead(0).unwrap().token_type, TokenType::Let);
        assert_eq!(
            stream.peek_ahead(1).unwrap().token_type,
            TokenType::Identifier("x".to_string())
        );
        assert_eq!(stream.peek_ahead(2).unwrap().token_type, TokenType::Equal);
        assert_eq!(stream.peek_ahead(3).unwrap().token_type, TokenType::Eof);
        assert!(stream.peek_ahead(4).is_none());
    }
}
