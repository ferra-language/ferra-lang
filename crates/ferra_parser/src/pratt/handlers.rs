//! NUD and LED handlers for the Pratt parser
//!
//! NUD (Null Denotation) handlers parse expressions that don't require a left operand
//! LED (Left Denotation) handlers parse expressions that operate on a left operand

use crate::{
    ast::{Arena, BinaryOperator, Expression, Literal, UnaryOperator},
    error::ParseError,
    token::{TokenStream, TokenType},
};

/// NUD handler for expressions that don't require a left operand
pub struct NudHandler<'arena, T: TokenStream> {
    arena: &'arena Arena,
    tokens: &'arena mut T,
}

impl<'arena, T: TokenStream> NudHandler<'arena, T> {
    pub fn new(arena: &'arena Arena, tokens: &'arena mut T) -> Self {
        Self { arena, tokens }
    }

    pub fn handle(&mut self, token_type: &TokenType) -> Result<&'arena Expression, ParseError> {
        match token_type {
            // Literals
            TokenType::IntegerLiteral(value) => Ok(self
                .arena
                .alloc(Expression::Literal(Literal::Integer(*value)))),
            TokenType::StringLiteral(value) => Ok(self
                .arena
                .alloc(Expression::Literal(Literal::String(value.clone())))),
            TokenType::FloatLiteral(value) => Ok(self
                .arena
                .alloc(Expression::Literal(Literal::Float(*value)))),
            TokenType::BooleanLiteral(value) => Ok(self
                .arena
                .alloc(Expression::Literal(Literal::Boolean(*value)))),

            // Identifiers
            TokenType::Identifier(name) => {
                Ok(self.arena.alloc(Expression::Identifier(name.clone())))
            }

            // Grouped expressions
            TokenType::LeftParen => self.parse_grouped_or_tuple_expression(),

            // Array literals
            TokenType::LeftBracket => self.parse_array_literal(),

            // Unary operators
            TokenType::Minus => self.parse_unary_expression(UnaryOperator::Minus),
            TokenType::Bang => self.parse_unary_expression(UnaryOperator::Not),
            TokenType::Plus => self.parse_unary_expression(UnaryOperator::Plus),

            _ => {
                let token = self.tokens.peek();
                Err(ParseError::unexpected_token("expression", token))
            }
        }
    }

    fn parse_grouped_or_tuple_expression(&mut self) -> Result<&'arena Expression, ParseError> {
        // For now, just handle simple cases. TODO: Implement proper recursive parsing
        // Expect the inner expression to be consumed already
        let token = self.tokens.consume();
        if !matches!(token.token_type, TokenType::RightParen) {
            return Err(ParseError::unexpected_token("')'", &token));
        }

        // Return a dummy expression for now - this needs proper implementation
        Ok(self.arena.alloc(Expression::Literal(Literal::Integer(0))))
    }

    fn parse_array_literal(&mut self) -> Result<&'arena Expression, ParseError> {
        // For now, just consume until closing bracket
        loop {
            let token = self.tokens.consume();
            if matches!(token.token_type, TokenType::RightBracket) {
                break;
            }
            if matches!(token.token_type, TokenType::Eof) {
                return Err(ParseError::unexpected_token("']'", &token));
            }
        }

        use crate::ast::ArrayLiteral;
        let token = self.tokens.peek();
        Ok(self.arena.alloc(Expression::Array(ArrayLiteral {
            elements: vec![],
            span: token.span.clone(),
        })))
    }

    fn parse_unary_expression(
        &mut self,
        operator: UnaryOperator,
    ) -> Result<&'arena Expression, ParseError> {
        // For now, just parse a simple operand (no recursive parsing)
        let token = self.tokens.consume();
        let operand = match &token.token_type {
            TokenType::IntegerLiteral(value) => Expression::Literal(Literal::Integer(*value)),
            TokenType::StringLiteral(value) => Expression::Literal(Literal::String(value.clone())),
            TokenType::FloatLiteral(value) => Expression::Literal(Literal::Float(*value)),
            TokenType::BooleanLiteral(value) => Expression::Literal(Literal::Boolean(*value)),
            TokenType::Identifier(name) => Expression::Identifier(name.clone()),
            _ => return Err(ParseError::unexpected_token("expression", &token)),
        };

        use crate::ast::UnaryExpression;
        Ok(self.arena.alloc(Expression::Unary(UnaryExpression {
            operator,
            operand: Box::new(operand),
            span: token.span.clone(),
        })))
    }
}

/// LED handler for expressions that operate on a left operand
pub struct LedHandler<'arena, T: TokenStream> {
    arena: &'arena Arena,
    tokens: &'arena mut T,
}

impl<'arena, T: TokenStream> LedHandler<'arena, T> {
    pub fn new(arena: &'arena Arena, tokens: &'arena mut T) -> Self {
        Self { arena, tokens }
    }

    pub fn handle(
        &mut self,
        left: &'arena Expression,
        token_type: &TokenType,
    ) -> Result<&'arena Expression, ParseError> {
        match token_type {
            // Binary operators
            TokenType::Plus => self.parse_binary_expression(left, BinaryOperator::Add, token_type),
            TokenType::Minus => self.parse_binary_expression(left, BinaryOperator::Sub, token_type),
            TokenType::Star => self.parse_binary_expression(left, BinaryOperator::Mul, token_type),
            TokenType::Slash => self.parse_binary_expression(left, BinaryOperator::Div, token_type),
            TokenType::Percent => {
                self.parse_binary_expression(left, BinaryOperator::Mod, token_type)
            }

            // Comparison operators
            TokenType::EqualEqual => {
                self.parse_binary_expression(left, BinaryOperator::Equal, token_type)
            }
            TokenType::BangEqual => {
                self.parse_binary_expression(left, BinaryOperator::NotEqual, token_type)
            }
            TokenType::Less => self.parse_binary_expression(left, BinaryOperator::Less, token_type),
            TokenType::LessEqual => {
                self.parse_binary_expression(left, BinaryOperator::LessEqual, token_type)
            }
            TokenType::Greater => {
                self.parse_binary_expression(left, BinaryOperator::Greater, token_type)
            }
            TokenType::GreaterEqual => {
                self.parse_binary_expression(left, BinaryOperator::GreaterEqual, token_type)
            }

            // Logical operators
            TokenType::AmpAmp => {
                self.parse_binary_expression(left, BinaryOperator::And, token_type)
            }
            TokenType::PipePipe => {
                self.parse_binary_expression(left, BinaryOperator::Or, token_type)
            }

            // Assignment operators
            TokenType::Equal => {
                self.parse_binary_expression(left, BinaryOperator::Assign, token_type)
            }

            _ => {
                let token = self.tokens.peek();
                Err(ParseError::unexpected_token("binary operator", token))
            }
        }
    }

    fn parse_binary_expression(
        &mut self,
        left: &'arena Expression,
        operator: BinaryOperator,
        _token_type: &TokenType,
    ) -> Result<&'arena Expression, ParseError> {
        // For now, just parse a simple right operand (no recursive parsing)
        let token = self.tokens.consume();
        let right = match &token.token_type {
            TokenType::IntegerLiteral(value) => Expression::Literal(Literal::Integer(*value)),
            TokenType::StringLiteral(value) => Expression::Literal(Literal::String(value.clone())),
            TokenType::FloatLiteral(value) => Expression::Literal(Literal::Float(*value)),
            TokenType::BooleanLiteral(value) => Expression::Literal(Literal::Boolean(*value)),
            TokenType::Identifier(name) => Expression::Identifier(name.clone()),
            _ => return Err(ParseError::unexpected_token("expression", &token)),
        };

        use crate::ast::BinaryExpression;
        Ok(self.arena.alloc(Expression::Binary(BinaryExpression {
            left: Box::new(left.clone()),
            operator,
            right: Box::new(right),
            span: token.span.clone(),
        })))
    }
}
