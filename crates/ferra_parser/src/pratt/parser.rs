//! Core Pratt parser implementation
//!
//! This module implements the main expression parsing algorithm using
//! Top-Down Operator Precedence (Pratt parsing) with NUD/LED handlers.

use crate::{
    ast::{
        Arena, BinaryExpression, BinaryOperator, Expression, Literal, UnaryExpression,
        UnaryOperator,
    },
    error::ParseError,
    pratt::precedence::{
        can_continue_expression, infix_binding_power, Associativity, BindingPower,
    },
    token::{Span, Token, TokenStream, TokenType},
};

/// The main Pratt parser for expressions
pub struct PrattParser<'arena, T: TokenStream> {
    arena: &'arena Arena,
    tokens: T,
}

impl<'arena, T: TokenStream> PrattParser<'arena, T> {
    /// Create a new Pratt parser
    pub fn new(arena: &'arena Arena, tokens: T) -> Self {
        Self { arena, tokens }
    }

    /// Parse an expression with the given minimum binding power
    pub fn parse_expression(
        &mut self,
        min_bp: BindingPower,
    ) -> Result<&'arena Expression, ParseError> {
        // Parse the primary expression (NUD)
        let mut left = self.parse_primary()?;

        // Parse any operators with sufficient binding power (LED)
        loop {
            let token = self.tokens.peek();

            // Check if we should stop parsing
            if token.token_type == TokenType::Eof {
                break;
            }

            // Check if this token can continue the expression
            if !can_continue_expression(&token.token_type) {
                break;
            }

            // Get the binding power for this operator
            let op_info = match infix_binding_power(&token.token_type) {
                Some(info) => info,
                None => break, // Not an infix operator, stop parsing
            };

            // If the binding power is too low, stop parsing
            if op_info.binding_power < min_bp {
                break;
            }

            // Consume the operator token
            let operator_token = self.tokens.consume();

            // Parse the right operand using LED handler
            left = self.handle_led(left, &operator_token)?;
        }

        Ok(left)
    }

    /// Parse primary expressions (literals, identifiers, etc.)
    fn parse_primary(&mut self) -> Result<&'arena Expression, ParseError> {
        let token = self.tokens.consume();

        match &token.token_type {
            // Literal expressions
            TokenType::StringLiteral(s) => Ok(self
                .arena
                .alloc(Expression::Literal(Literal::String(s.clone())))),
            TokenType::IntegerLiteral(i) => {
                Ok(self.arena.alloc(Expression::Literal(Literal::Integer(*i))))
            }
            TokenType::FloatLiteral(f) => {
                Ok(self.arena.alloc(Expression::Literal(Literal::Float(*f))))
            }
            TokenType::BooleanLiteral(b) => {
                Ok(self.arena.alloc(Expression::Literal(Literal::Boolean(*b))))
            }

            // Identifier expressions
            TokenType::Identifier(name) => {
                // Check for macro invocation: identifier!
                if let TokenType::Bang = self.tokens.peek().token_type {
                    // Parse as macro invocation
                    let mut macro_parser =
                        crate::macro_parser::MacroParser::new(self.arena, &mut self.tokens);
                    let macro_invocation = macro_parser.parse_macro_invocation(name.clone())?;
                    Ok(self
                        .arena
                        .alloc(Expression::Macro(macro_invocation.clone())))
                } else {
                    // Simple identifier (qualified identifiers handled as postfix dot operations)
                    Ok(self.arena.alloc(Expression::Identifier(name.clone())))
                }
            }

            // Unary expressions
            TokenType::Bang | TokenType::Minus => {
                let operator = match &token.token_type {
                    TokenType::Bang => UnaryOperator::Not,
                    TokenType::Minus => UnaryOperator::Minus,
                    _ => return Err(ParseError::unexpected_token("unary operator", &token)),
                };

                // Use a fixed precedence for unary operators
                let operand = self.parse_expression(100)?; // High precedence for unary

                Ok(self.arena.alloc(Expression::Unary(UnaryExpression {
                    operator,
                    operand: Box::new(operand.clone()),
                    span: token.span.combine(operand.span()),
                })))
            }

            // Grouped expressions
            TokenType::LeftParen => {
                let expr = self.parse_expression(0)?;
                let close_token = self.tokens.consume();
                if !matches!(close_token.token_type, TokenType::RightParen) {
                    return Err(ParseError::unexpected_token(")", &close_token));
                }
                Ok(self
                    .arena
                    .alloc(Expression::Grouped(Box::new(expr.clone()))))
            }

            // Array literals
            TokenType::LeftBracket => self.parse_array_literal(),

            _ => Err(ParseError::unexpected_token("expression", &token)),
        }
    }

    fn handle_led(
        &mut self,
        left: &'arena Expression,
        token: &Token,
    ) -> Result<&'arena Expression, ParseError> {
        match &token.token_type {
            // Binary operators
            TokenType::Plus => self.parse_binary_expression(left, BinaryOperator::Add, token),
            TokenType::Minus => self.parse_binary_expression(left, BinaryOperator::Sub, token),
            TokenType::Star => self.parse_binary_expression(left, BinaryOperator::Mul, token),
            TokenType::Slash => self.parse_binary_expression(left, BinaryOperator::Div, token),
            TokenType::Percent => self.parse_binary_expression(left, BinaryOperator::Mod, token),

            // Comparison operators
            TokenType::EqualEqual => {
                self.parse_binary_expression(left, BinaryOperator::Equal, token)
            }
            TokenType::BangEqual => {
                self.parse_binary_expression(left, BinaryOperator::NotEqual, token)
            }
            TokenType::Less => self.parse_binary_expression(left, BinaryOperator::Less, token),
            TokenType::LessEqual => {
                self.parse_binary_expression(left, BinaryOperator::LessEqual, token)
            }
            TokenType::Greater => {
                self.parse_binary_expression(left, BinaryOperator::Greater, token)
            }
            TokenType::GreaterEqual => {
                self.parse_binary_expression(left, BinaryOperator::GreaterEqual, token)
            }

            // Logical operators
            TokenType::AmpAmp => self.parse_binary_expression(left, BinaryOperator::And, token),
            TokenType::PipePipe => self.parse_binary_expression(left, BinaryOperator::Or, token),

            // Assignment operators
            TokenType::Equal => self.parse_binary_expression(left, BinaryOperator::Assign, token),

            // Postfix operators
            TokenType::Dot => self.parse_member_access(left, token),
            TokenType::LeftParen => self.parse_function_call(left, token),
            TokenType::LeftBracket => self.parse_index_expression(left, token),
            TokenType::Question => self.parse_try_expression(left, token),

            _ => Err(ParseError::unexpected_token("binary operator", token)),
        }
    }

    fn parse_binary_expression(
        &mut self,
        left: &'arena Expression,
        operator: BinaryOperator,
        token: &Token,
    ) -> Result<&'arena Expression, ParseError> {
        // Get the precedence for this operator
        let op_info = infix_binding_power(&token.token_type)
            .ok_or_else(|| ParseError::unexpected_token("binary operator", token))?;

        // Parse the right operand with appropriate precedence
        let right_bp = match op_info.associativity {
            Associativity::Left => op_info.binding_power + 1,
            Associativity::Right => op_info.binding_power,
            Associativity::None => op_info.binding_power + 1,
        };

        let right = self.parse_expression(right_bp)?;

        Ok(self.arena.alloc(Expression::Binary(BinaryExpression {
            left: Box::new(left.clone()),
            operator,
            right: Box::new(right.clone()),
            span: token.span.clone(),
        })))
    }

    /// Parse qualified identifiers like module.function or simple identifiers
    #[allow(dead_code)]
    fn parse_qualified_identifier(
        &mut self,
        first_part: String,
        _start_span: Span,
    ) -> Result<&'arena Expression, ParseError> {
        // Just return a simple identifier - dots are handled as postfix operators
        Ok(self.arena.alloc(Expression::Identifier(first_part)))
    }

    /// Parse array literals like [1, 2, 3]
    fn parse_array_literal(&mut self) -> Result<&'arena Expression, ParseError> {
        let start_span = self.tokens.peek().span.clone();
        let mut elements = Vec::new();

        // Check for empty array
        if let TokenType::RightBracket = self.tokens.peek().token_type {
            let end_token = self.tokens.consume();
            use crate::ast::ArrayLiteral;
            return Ok(self.arena.alloc(Expression::Array(ArrayLiteral {
                elements,
                span: Span::new(
                    start_span.start,
                    end_token.span.end,
                    start_span.line,
                    start_span.column,
                ),
            })));
        }

        // Parse comma-separated expressions
        loop {
            let expr = self.parse_expression(0)?;
            elements.push(expr.clone());

            let token = self.tokens.consume();
            match token.token_type {
                TokenType::Comma => {
                    // Check if there's another element or if this is a trailing comma
                    if let TokenType::RightBracket = self.tokens.peek().token_type {
                        let end_token = self.tokens.consume();
                        use crate::ast::ArrayLiteral;
                        return Ok(self.arena.alloc(Expression::Array(ArrayLiteral {
                            elements,
                            span: Span::new(
                                start_span.start,
                                end_token.span.end,
                                start_span.line,
                                start_span.column,
                            ),
                        })));
                    }
                    // Continue parsing next element
                }
                TokenType::RightBracket => {
                    use crate::ast::ArrayLiteral;
                    return Ok(self.arena.alloc(Expression::Array(ArrayLiteral {
                        elements,
                        span: Span::new(
                            start_span.start,
                            token.span.end,
                            start_span.line,
                            start_span.column,
                        ),
                    })));
                }
                _ => return Err(ParseError::unexpected_token("',' or ']'", &token)),
            }
        }
    }

    /// Parse member access like obj.member
    fn parse_member_access(
        &mut self,
        left: &'arena Expression,
        _token: &Token,
    ) -> Result<&'arena Expression, ParseError> {
        let member_token = self.tokens.consume();
        match member_token.token_type {
            TokenType::Identifier(member_name) => {
                use crate::ast::MemberAccessExpression;
                Ok(self
                    .arena
                    .alloc(Expression::MemberAccess(MemberAccessExpression {
                        object: Box::new(left.clone()),
                        member: member_name,
                        span: member_token.span.clone(),
                    })))
            }
            _ => Err(ParseError::unexpected_token("member name", &member_token)),
        }
    }

    /// Parse function calls like func(arg1, arg2)
    fn parse_function_call(
        &mut self,
        left: &'arena Expression,
        _token: &Token,
    ) -> Result<&'arena Expression, ParseError> {
        let mut arguments = Vec::new();

        // Check for empty argument list
        if let TokenType::RightParen = self.tokens.peek().token_type {
            self.tokens.consume();
            use crate::ast::CallExpression;
            return Ok(self.arena.alloc(Expression::Call(CallExpression {
                callee: Box::new(left.clone()),
                arguments,
                span: _token.span.clone(),
            })));
        }

        // Parse comma-separated arguments
        loop {
            let arg = self.parse_expression(0)?;
            arguments.push(arg.clone());

            let token = self.tokens.consume();
            match token.token_type {
                TokenType::Comma => {
                    // Check for trailing comma
                    if let TokenType::RightParen = self.tokens.peek().token_type {
                        self.tokens.consume();
                        break;
                    }
                    // Continue parsing next argument
                }
                TokenType::RightParen => break,
                _ => return Err(ParseError::unexpected_token("',' or ')'", &token)),
            }
        }

        use crate::ast::CallExpression;
        Ok(self.arena.alloc(Expression::Call(CallExpression {
            callee: Box::new(left.clone()),
            arguments,
            span: _token.span.clone(),
        })))
    }

    /// Parse index expressions like arr[index]
    fn parse_index_expression(
        &mut self,
        left: &'arena Expression,
        _token: &Token,
    ) -> Result<&'arena Expression, ParseError> {
        let index = self.parse_expression(0)?;

        let close_token = self.tokens.consume();
        if !matches!(close_token.token_type, TokenType::RightBracket) {
            return Err(ParseError::unexpected_token("']'", &close_token));
        }

        use crate::ast::IndexExpression;
        Ok(self.arena.alloc(Expression::Index(IndexExpression {
            object: Box::new(left.clone()),
            index: Box::new(index.clone()),
            span: _token.span.clone(),
        })))
    }

    /// Parse try expressions like expr?
    fn parse_try_expression(
        &mut self,
        left: &'arena Expression,
        token: &Token,
    ) -> Result<&'arena Expression, ParseError> {
        use crate::ast::UnaryExpression;
        Ok(self.arena.alloc(Expression::Unary(UnaryExpression {
            operator: crate::ast::UnaryOperator::Try,
            operand: Box::new(left.clone()),
            span: token.span.clone(),
        })))
    }

    /// Parse patterns for match expressions
    pub fn parse_pattern(&mut self) -> Result<&'arena crate::ast::Pattern, ParseError> {
        self.parse_pattern_with_precedence(0)
    }

    /// Parse pattern with precedence support for or patterns
    fn parse_pattern_with_precedence(
        &mut self,
        min_precedence: u8,
    ) -> Result<&'arena crate::ast::Pattern, ParseError> {
        let mut left = self.parse_primary_pattern()?;

        // Handle Or patterns with precedence
        while let TokenType::Pipe = self.tokens.peek().token_type {
            if min_precedence > 10 {
                // Or patterns have low precedence
                break;
            }

            self.tokens.consume(); // consume '|'
            let right = self.parse_pattern_with_precedence(11)?;

            // Convert to or pattern
            let patterns = match left {
                crate::ast::Pattern::Or(or_pattern) => {
                    let mut patterns = or_pattern.patterns.clone();
                    patterns.push(right.clone());
                    patterns
                }
                _ => vec![left.clone(), right.clone()],
            };

            left = self
                .arena
                .alloc(crate::ast::Pattern::Or(crate::ast::OrPattern {
                    patterns,
                    span: left.span().combine(right.span()),
                }));
        }

        Ok(left)
    }

    /// Parse primary patterns (not including or patterns)
    fn parse_primary_pattern(&mut self) -> Result<&'arena crate::ast::Pattern, ParseError> {
        let token = self.tokens.consume();

        match &token.token_type {
            // Literal patterns
            TokenType::StringLiteral(s) => {
                let pattern = self
                    .arena
                    .alloc(crate::ast::Pattern::Literal(Literal::String(s.clone())));
                self.check_for_guard_or_binding(pattern)
            }
            TokenType::IntegerLiteral(i) => {
                let pattern = self
                    .arena
                    .alloc(crate::ast::Pattern::Literal(Literal::Integer(*i)));
                self.check_for_range_or_guard_or_binding(*i, pattern)
            }
            TokenType::FloatLiteral(f) => {
                let pattern = self
                    .arena
                    .alloc(crate::ast::Pattern::Literal(Literal::Float(*f)));
                self.check_for_guard_or_binding(pattern)
            }
            TokenType::BooleanLiteral(b) => {
                let pattern = self
                    .arena
                    .alloc(crate::ast::Pattern::Literal(Literal::Boolean(*b)));
                self.check_for_guard_or_binding(pattern)
            }

            // Identifier patterns
            TokenType::Identifier(name) => {
                // Check for wildcard pattern first
                if name == "_" {
                    let pattern = self.arena.alloc(crate::ast::Pattern::Wildcard);
                    self.check_for_guard_or_binding(pattern)
                } else if let TokenType::LeftBrace = self.tokens.peek().token_type {
                    // Data class pattern
                    let pattern = self.parse_data_class_pattern(name.clone())?;
                    self.check_for_guard_or_binding(pattern)
                } else if let TokenType::At = self.tokens.peek().token_type {
                    // Binding pattern: name @ pattern
                    self.parse_binding_pattern(name.clone())
                } else {
                    // Simple identifier pattern
                    let pattern = self
                        .arena
                        .alloc(crate::ast::Pattern::Identifier(name.clone()));
                    self.check_for_guard_or_binding(pattern)
                }
            }

            // Slice patterns: [head, tail @ ..]
            TokenType::LeftBracket => self.parse_slice_pattern(),

            // Range patterns starting with .. : ..=10
            TokenType::DotDot | TokenType::DotDotEqual => {
                self.parse_range_pattern_from_operator(&token)
            }

            _ => Err(ParseError::unexpected_token("pattern", &token)),
        }
    }

    /// Check for range patterns when we have an integer literal
    fn check_for_range_or_guard_or_binding(
        &mut self,
        value: i64,
        pattern: &'arena crate::ast::Pattern,
    ) -> Result<&'arena crate::ast::Pattern, ParseError> {
        match self.tokens.peek().token_type {
            TokenType::DotDot | TokenType::DotDotEqual => {
                self.parse_range_pattern_from_start(value)
            }
            _ => self.check_for_guard_or_binding(pattern),
        }
    }

    /// Check for guard expressions: pattern if condition
    fn check_for_guard_or_binding(
        &mut self,
        pattern: &'arena crate::ast::Pattern,
    ) -> Result<&'arena crate::ast::Pattern, ParseError> {
        match self.tokens.peek().token_type {
            TokenType::If => self.parse_guard_pattern(pattern),
            _ => Ok(pattern),
        }
    }

    /// Parse range patterns like 1..10 or 1..=10
    fn parse_range_pattern_from_start(
        &mut self,
        start_value: i64,
    ) -> Result<&'arena crate::ast::Pattern, ParseError> {
        let range_token = self.tokens.consume(); // consume .. or ..=
        let inclusive = matches!(range_token.token_type, TokenType::DotDotEqual);

        let start_pattern = self
            .arena
            .alloc(crate::ast::Pattern::Literal(Literal::Integer(start_value)));

        // Parse end pattern if present
        let end_pattern = if matches!(self.tokens.peek().token_type, TokenType::IntegerLiteral(_)) {
            Some(Box::new(self.parse_primary_pattern()?.clone()))
        } else {
            None
        };

        let pattern = self
            .arena
            .alloc(crate::ast::Pattern::Range(crate::ast::RangePattern {
                start: Some(Box::new(start_pattern.clone())),
                end: end_pattern,
                inclusive,
                span: range_token.span.clone(),
            }));

        self.check_for_guard_or_binding(pattern)
    }

    /// Parse range patterns starting with .. : ..10 or ..=10
    fn parse_range_pattern_from_operator(
        &mut self,
        range_token: &Token,
    ) -> Result<&'arena crate::ast::Pattern, ParseError> {
        let inclusive = matches!(range_token.token_type, TokenType::DotDotEqual);

        // Parse end pattern if present
        let end_pattern = if matches!(self.tokens.peek().token_type, TokenType::IntegerLiteral(_)) {
            Some(Box::new(self.parse_primary_pattern()?.clone()))
        } else {
            None
        };

        let pattern = self
            .arena
            .alloc(crate::ast::Pattern::Range(crate::ast::RangePattern {
                start: None,
                end: end_pattern,
                inclusive,
                span: range_token.span.clone(),
            }));

        self.check_for_guard_or_binding(pattern)
    }

    /// Parse slice patterns like [head, tail @ ..]
    fn parse_slice_pattern(&mut self) -> Result<&'arena crate::ast::Pattern, ParseError> {
        use crate::ast::SlicePattern;

        let mut prefix = Vec::new();
        let mut rest = None;
        let mut suffix = Vec::new();
        let mut found_rest = false;

        // Handle empty slice pattern
        if let TokenType::RightBracket = self.tokens.peek().token_type {
            let close_token = self.tokens.consume();
            return Ok(self.arena.alloc(crate::ast::Pattern::Slice(SlicePattern {
                prefix,
                rest,
                suffix,
                span: close_token.span.clone(),
            })));
        }

        // Parse slice elements
        loop {
            // Check for rest pattern: .. or name @ ..
            if let TokenType::DotDot = self.tokens.peek().token_type {
                self.tokens.consume(); // consume '..'
                found_rest = true;
            } else if let TokenType::Identifier(name) = &self.tokens.peek().token_type {
                let name = name.clone();
                if let Some(at_token) = self.tokens.peek_ahead(1) {
                    if matches!(at_token.token_type, TokenType::At) {
                        if let Some(dot_token) = self.tokens.peek_ahead(2) {
                            if matches!(dot_token.token_type, TokenType::DotDot) {
                                // Found name @ .. pattern
                                self.tokens.consume(); // consume name
                                self.tokens.consume(); // consume '@'
                                self.tokens.consume(); // consume '..'
                                rest = Some(name);
                                found_rest = true;
                            }
                        }
                    }
                }

                if !found_rest {
                    // Regular pattern
                    let pattern = self.parse_primary_pattern()?;
                    if found_rest {
                        suffix.push(pattern.clone());
                    } else {
                        prefix.push(pattern.clone());
                    }
                }
            } else {
                // Regular pattern
                let pattern = self.parse_primary_pattern()?;
                if found_rest {
                    suffix.push(pattern.clone());
                } else {
                    prefix.push(pattern.clone());
                }
            }

            // Check for continuation
            let next_token = self.tokens.consume();
            match next_token.token_type {
                TokenType::Comma => {
                    // Check for trailing comma
                    if let TokenType::RightBracket = self.tokens.peek().token_type {
                        let close_token = self.tokens.consume();
                        return Ok(self.arena.alloc(crate::ast::Pattern::Slice(SlicePattern {
                            prefix,
                            rest,
                            suffix,
                            span: close_token.span.clone(),
                        })));
                    }
                    // Continue parsing
                }
                TokenType::RightBracket => {
                    return Ok(self.arena.alloc(crate::ast::Pattern::Slice(SlicePattern {
                        prefix,
                        rest,
                        suffix,
                        span: next_token.span.clone(),
                    })));
                }
                _ => return Err(ParseError::unexpected_token("',' or ']'", &next_token)),
            }
        }
    }

    /// Parse guard patterns like x if x > 0
    fn parse_guard_pattern(
        &mut self,
        pattern: &'arena crate::ast::Pattern,
    ) -> Result<&'arena crate::ast::Pattern, ParseError> {
        self.tokens.consume(); // consume 'if'

        let guard_expr = self.parse_expression(0)?;

        Ok(self
            .arena
            .alloc(crate::ast::Pattern::Guard(crate::ast::GuardPattern {
                pattern: Box::new(pattern.clone()),
                guard: guard_expr.clone(),
                span: pattern.span().combine(guard_expr.span()),
            })))
    }

    /// Parse binding patterns like name @ pattern
    fn parse_binding_pattern(
        &mut self,
        name: String,
    ) -> Result<&'arena crate::ast::Pattern, ParseError> {
        self.tokens.consume(); // consume '@'

        let pattern = self.parse_primary_pattern()?;

        Ok(self
            .arena
            .alloc(crate::ast::Pattern::Binding(crate::ast::BindingPattern {
                name,
                pattern: Box::new(pattern.clone()),
                span: pattern.span(), // TODO: Better span calculation
            })))
    }

    /// Parse data class patterns like Person { name, age }
    fn parse_data_class_pattern(
        &mut self,
        name: String,
    ) -> Result<&'arena crate::ast::Pattern, ParseError> {
        use crate::ast::{DataClassPattern, FieldPattern};

        // Consume the opening brace
        let open_brace = self.tokens.consume();
        if !matches!(open_brace.token_type, TokenType::LeftBrace) {
            return Err(ParseError::unexpected_token("'{'", &open_brace));
        }

        let mut fields = Vec::new();
        let has_rest = false;

        // Handle empty pattern
        if let TokenType::RightBrace = self.tokens.peek().token_type {
            let close_token = self.tokens.consume();
            return Ok(self
                .arena
                .alloc(crate::ast::Pattern::DataClass(DataClassPattern {
                    name,
                    fields,
                    has_rest,
                    span: close_token.span.clone(),
                })));
        }

        // Parse field patterns
        loop {
            let token = self.tokens.consume();
            match token.token_type {
                TokenType::Identifier(field_name) => {
                    // Check for field binding: field: pattern
                    if let TokenType::Colon = self.tokens.peek().token_type {
                        self.tokens.consume(); // consume ':'
                        let pattern = self.parse_pattern()?;
                        fields.push(FieldPattern {
                            name: field_name,
                            pattern: Some(pattern.clone()),
                            span: token.span.clone(),
                        });
                    } else {
                        // Simple field binding
                        fields.push(FieldPattern {
                            name: field_name,
                            pattern: None,
                            span: token.span.clone(),
                        });
                    }
                }
                _ => return Err(ParseError::unexpected_token("field name", &token)),
            }

            // Check for continuation
            let next_token = self.tokens.consume();
            match next_token.token_type {
                TokenType::Comma => {
                    // Check for trailing comma or rest pattern
                    if let TokenType::RightBrace = self.tokens.peek().token_type {
                        let close_token = self.tokens.consume();
                        return Ok(self.arena.alloc(crate::ast::Pattern::DataClass(
                            DataClassPattern {
                                name,
                                fields,
                                has_rest,
                                span: close_token.span.clone(),
                            },
                        )));
                    }
                    // Continue parsing
                }
                TokenType::RightBrace => {
                    return Ok(self.arena.alloc(crate::ast::Pattern::DataClass(
                        DataClassPattern {
                            name,
                            fields,
                            has_rest,
                            span: next_token.span.clone(),
                        },
                    )));
                }
                _ => return Err(ParseError::unexpected_token("',' or '}'", &next_token)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::Arena,
        token::{TokenType, VecTokenStream},
    };

    fn create_test_arena() -> Arena {
        Arena::new()
    }

    fn create_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
        VecTokenStream::from_token_types(token_types)
    }

    #[test]
    fn test_pratt_parser_creation() {
        let arena = create_test_arena();
        let tokens = create_token_stream(vec![TokenType::IntegerLiteral(42)]);
        let parser = PrattParser::new(&arena, tokens);

        assert!(!parser.tokens.is_at_end());
    }

    #[test]
    fn test_primary_expression_parsing() {
        let arena = create_test_arena();
        let tokens = create_token_stream(vec![TokenType::IntegerLiteral(42)]);
        let mut parser = PrattParser::new(&arena, tokens);

        // This should now work for simple literals
        let result = parser.parse_primary();
        assert!(result.is_ok());

        if let Ok(expr) = result {
            if let Expression::Literal(Literal::Integer(value)) = expr {
                assert_eq!(*value, 42);
            } else {
                panic!("Expected integer literal");
            }
        }
    }

    #[test]
    fn test_binary_expression_parsing() {
        let arena = create_test_arena();
        let tokens = create_token_stream(vec![
            TokenType::IntegerLiteral(1),
            TokenType::Plus,
            TokenType::IntegerLiteral(2),
        ]);
        let mut parser = PrattParser::new(&arena, tokens);

        // Use parse_expression for full expression parsing
        let result = parser.parse_expression(0);
        assert!(result.is_ok());

        if let Ok(expr) = result {
            if let Expression::Binary(binary_expr) = expr {
                assert!(matches!(binary_expr.operator, BinaryOperator::Add));
            } else {
                panic!("Expected binary expression");
            }
        }
    }

    #[test]
    fn test_precedence_binding() {
        let arena = create_test_arena();
        let tokens = create_token_stream(vec![
            TokenType::IntegerLiteral(1),
            TokenType::Plus,
            TokenType::IntegerLiteral(2),
            TokenType::Star,
            TokenType::IntegerLiteral(3),
        ]);
        let mut parser = PrattParser::new(&arena, tokens);

        // Use parse_expression for full expression parsing
        let result = parser.parse_expression(0);
        assert!(result.is_ok());

        if let Ok(expr) = result {
            if let Expression::Binary(binary_expr) = expr {
                assert!(matches!(binary_expr.operator, BinaryOperator::Add));
                // Right side should be another binary expression (2 * 3)
                if let Expression::Binary(right_expr) = binary_expr.right.as_ref() {
                    assert!(matches!(right_expr.operator, BinaryOperator::Mul));
                } else {
                    panic!("Expected right side to be binary expression");
                }
            } else {
                panic!("Expected binary expression");
            }
        }
    }

    #[test]
    fn test_parser_state_management() {
        let arena = create_test_arena();
        let tokens = create_token_stream(vec![
            TokenType::Identifier("a".to_string()),
            TokenType::Identifier("b".to_string()),
            TokenType::Identifier("c".to_string()),
        ]);
        let mut parser = PrattParser::new(&arena, tokens);

        // Test token navigation
        assert!(!parser.tokens.is_at_end());

        parser.tokens.consume();
        assert!(!parser.tokens.is_at_end());

        parser.tokens.consume();
        assert!(!parser.tokens.is_at_end());

        parser.tokens.consume();
        assert!(parser.tokens.is_at_end());
    }
}
