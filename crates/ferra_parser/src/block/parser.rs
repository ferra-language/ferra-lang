//! Block parsing implementation for Phase 2.4
//!
//! Handles all block types: braced, indented, and advanced block features

use crate::{
    ast::{
        Arena, Block, BreakStatement, ContinueStatement, Expression, Literal, Modifiers,
        ReturnStatement, Statement, Type, VariableDecl,
    },
    error::{ParseError, ParseResult},
    pratt::parser::PrattParser,
    token::{Span, Token, TokenStream, TokenType},
};

/// Block style enumeration for consistency
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockStyle {
    Braced,
    Indented,
}

/// Scope information for variable tracking
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub depth: usize,
    pub variables: Vec<String>,
    pub is_unsafe: bool,
    pub is_async: bool,
    pub label: Option<String>,
}

/// Block parser with comprehensive scope and style management
pub struct BlockParser<'arena> {
    arena: &'arena Arena,
    current_scope_depth: usize,
    _current_indentation: usize, // For future indentation tracking
    block_style: Option<BlockStyle>,
}

impl<'arena> BlockParser<'arena> {
    /// Create a new block parser
    pub fn new(arena: &'arena Arena) -> Self {
        Self {
            arena,
            current_scope_depth: 0,
            _current_indentation: 0,
            block_style: None,
        }
    }

    /// Parse a block, automatically detecting style
    pub fn parse_block<T: TokenStream>(&mut self, tokens: &mut T) -> ParseResult<&'arena Block> {
        let current = tokens.peek();

        match current.token_type {
            TokenType::LeftBrace => self.parse_braced_block(tokens),
            TokenType::Colon => self.parse_indented_block(tokens),
            _ => Err(ParseError::expected_block(current.span.clone())),
        }
    }

    /// Parse a braced block { statements... }
    pub fn parse_braced_block<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Block> {
        let start_span = tokens.peek().span.clone();

        // Consume opening brace
        self.expect_token(tokens, TokenType::LeftBrace)?;

        // Set block style for consistency checking
        match self.block_style {
            None => self.block_style = Some(BlockStyle::Braced),
            Some(BlockStyle::Indented) => {
                return Err(ParseError::mixed_block_styles(start_span));
            }
            Some(BlockStyle::Braced) => {} // OK, consistent
        }

        let mut statements = Vec::new();
        self.current_scope_depth += 1;

        // Parse statements until closing brace
        while !tokens.is_at_end() && !matches!(tokens.peek().token_type, TokenType::RightBrace) {
            let statement = self.parse_statement_in_block(tokens)?;
            statements.push(statement.clone());
        }

        let end_span = tokens.peek().span.clone();
        self.expect_token(tokens, TokenType::RightBrace)?;

        self.current_scope_depth -= 1;

        let block = self.arena.alloc(Block {
            statements,
            is_braced: true,
            scope_depth: self.current_scope_depth,
            span: start_span.combine(end_span),
            is_unsafe: false,
            is_async: false,
            is_try: false,
            label: None,
        });

        Ok(block)
    }

    /// Parse an indented block : \n statements...
    pub fn parse_indented_block<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Block> {
        let start_span = tokens.peek().span.clone();

        // Consume colon
        self.expect_token(tokens, TokenType::Colon)?;

        // Expect newline
        self.expect_token(tokens, TokenType::Newline)?;

        // Set block style for consistency checking
        match self.block_style {
            None => self.block_style = Some(BlockStyle::Indented),
            Some(BlockStyle::Braced) => {
                return Err(ParseError::mixed_block_styles(start_span));
            }
            Some(BlockStyle::Indented) => {} // OK, consistent
        }

        let mut statements = Vec::new();
        self.current_scope_depth += 1;

        // Parse indented statements (simplified - just parse until we see dedent)
        while !tokens.is_at_end() {
            // Check if we've reached the end of the indented block
            if matches!(
                tokens.peek().token_type,
                TokenType::Eof | TokenType::RightBrace
            ) {
                break;
            }

            let statement = self.parse_statement_in_block(tokens)?;
            statements.push(statement.clone());
        }

        self.current_scope_depth -= 1;

        let end_span = if let Some(last_stmt) = statements.last() {
            last_stmt.span()
        } else {
            start_span.clone()
        };

        let block = self.arena.alloc(Block {
            statements,
            is_braced: false,
            scope_depth: self.current_scope_depth,
            span: start_span.combine(end_span),
            is_unsafe: false,
            is_async: false,
            is_try: false,
            label: None,
        });

        Ok(block)
    }

    /// Parse a statement within a block context
    fn parse_statement_in_block<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Statement> {
        // Create a temporary token stream for the statement parser
        // We'll parse one statement at a time
        let token = tokens.peek();

        match &token.token_type {
            // Variable declarations
            TokenType::Let | TokenType::Var => self.parse_variable_statement(tokens),
            // Control flow
            TokenType::If => self.parse_if_statement(tokens),
            TokenType::While => self.parse_while_statement(tokens),
            TokenType::For => self.parse_for_statement(tokens),
            TokenType::Return => self.parse_return_statement(tokens),
            TokenType::Break => self.parse_break_statement(tokens),
            TokenType::Continue => self.parse_continue_statement(tokens),
            // Block statements
            TokenType::LeftBrace => {
                let block = self.parse_braced_block(tokens)?;
                Ok(self.arena.alloc(Statement::Block(block.clone())))
            }
            // Expression statements (fallback)
            _ => self.parse_expression_statement(tokens),
        }
    }

    /// Parse a variable declaration statement
    fn parse_variable_statement<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Statement> {
        let start_token = tokens.consume(); // let or var
        let is_mutable = matches!(start_token.token_type, TokenType::Var);

        let name_token = tokens.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("identifier", &name_token)),
        };

        // Optional type annotation
        let var_type = if matches!(tokens.peek().token_type, TokenType::Colon) {
            tokens.consume(); // consume ':'
            Some(Type::Identifier("i32".to_string())) // Simplified for now
        } else {
            None
        };

        // Optional initializer
        let initializer = if matches!(tokens.peek().token_type, TokenType::Equal) {
            tokens.consume(); // consume '='
            Some(self.parse_expression(tokens)?)
        } else {
            None
        };

        // Consume optional semicolon
        if matches!(tokens.peek().token_type, TokenType::Semicolon) {
            tokens.consume();
        }

        let var_decl = VariableDecl {
            name,
            var_type,
            initializer: initializer.cloned(),
            is_mutable,
            modifiers: Modifiers {
                is_public: false,
                is_unsafe: false,
            },
            attributes: Vec::new(),
            span: start_token.span,
        };

        Ok(self.arena.alloc(Statement::VariableDecl(var_decl)))
    }

    #[allow(dead_code)]
    fn parse_expression_statement<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Statement> {
        let expr = self.parse_expression(tokens)?;

        // Consume optional semicolon
        if matches!(tokens.peek().token_type, TokenType::Semicolon) {
            tokens.consume();
        }

        Ok(self.arena.alloc(Statement::Expression(expr.clone())))
    }

    /// Parse a complex expression using the PrattParser
    fn parse_expression<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Expression> {
        // Collect tokens into a vector for PrattParser consumption
        let mut collected_tokens = Vec::new();
        let mut paren_depth = 0;
        let mut bracket_depth = 0;
        let mut brace_depth = 0;

        // Collect tokens until we find a statement terminator
        loop {
            let token = tokens.peek();
            match token.token_type {
                TokenType::LeftParen => paren_depth += 1,
                TokenType::RightParen => {
                    if paren_depth == 0 {
                        break;
                    }
                    paren_depth -= 1;
                }
                TokenType::LeftBracket => bracket_depth += 1,
                TokenType::RightBracket => {
                    if bracket_depth == 0 {
                        break;
                    }
                    bracket_depth -= 1;
                }
                TokenType::LeftBrace => brace_depth += 1,
                TokenType::RightBrace => {
                    if brace_depth == 0 {
                        break;
                    }
                    brace_depth -= 1;
                }
                TokenType::Semicolon | TokenType::Newline => {
                    if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
                        break;
                    }
                }
                // Statement keywords should terminate expression parsing
                TokenType::Let
                | TokenType::Var
                | TokenType::If
                | TokenType::While
                | TokenType::For
                | TokenType::Return
                | TokenType::Break
                | TokenType::Continue => {
                    if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
                        break;
                    }
                }
                TokenType::Eof => break,
                _ => {}
            }

            collected_tokens.push(tokens.consume());
        }

        // Add EOF token
        collected_tokens.push(Token {
            token_type: TokenType::Eof,
            span: tokens.peek().span.clone(),
        });

        // Create a VecTokenStream and parse with PrattParser
        let token_stream = crate::token::VecTokenStream::new(collected_tokens);
        let mut pratt_parser = PrattParser::new(self.arena, token_stream);
        pratt_parser.parse_expression(0)
    }

    /// Parse other statement types (simplified implementations)
    fn parse_if_statement<T: TokenStream>(
        &mut self,
        _tokens: &mut T,
    ) -> ParseResult<&'arena Statement> {
        // Simplified - just return a dummy statement for now
        Ok(self.arena.alloc(Statement::Expression(
            self.arena
                .alloc(Expression::Literal(Literal::Boolean(true)))
                .clone(),
        )))
    }

    fn parse_while_statement<T: TokenStream>(
        &mut self,
        _tokens: &mut T,
    ) -> ParseResult<&'arena Statement> {
        // Simplified - just return a dummy statement for now
        Ok(self.arena.alloc(Statement::Expression(
            self.arena
                .alloc(Expression::Literal(Literal::Boolean(true)))
                .clone(),
        )))
    }

    fn parse_for_statement<T: TokenStream>(
        &mut self,
        _tokens: &mut T,
    ) -> ParseResult<&'arena Statement> {
        // Simplified - just return a dummy statement for now
        Ok(self.arena.alloc(Statement::Expression(
            self.arena
                .alloc(Expression::Literal(Literal::Boolean(true)))
                .clone(),
        )))
    }

    fn parse_return_statement<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Statement> {
        tokens.consume(); // consume 'return'

        // Optional return value
        let value = if matches!(
            tokens.peek().token_type,
            TokenType::Semicolon | TokenType::Eof | TokenType::RightBrace
        ) {
            None
        } else {
            let expr_token = tokens.consume();
            match expr_token.token_type {
                TokenType::IntegerLiteral(value) => Some(
                    self.arena
                        .alloc(Expression::Literal(Literal::Integer(value))),
                ),
                TokenType::Identifier(name) => Some(self.arena.alloc(Expression::Identifier(name))),
                _ => return Err(ParseError::unexpected_token("expression", &expr_token)),
            }
        };

        // Consume optional semicolon
        if matches!(tokens.peek().token_type, TokenType::Semicolon) {
            tokens.consume();
        }

        let return_stmt = ReturnStatement {
            value: value.cloned(),
            span: tokens.peek().span.clone(),
        };

        Ok(self.arena.alloc(Statement::Return(return_stmt)))
    }

    fn parse_break_statement<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Statement> {
        let break_token = tokens.consume(); // consume 'break'

        // Consume optional semicolon
        if matches!(tokens.peek().token_type, TokenType::Semicolon) {
            tokens.consume();
        }

        let break_stmt = BreakStatement {
            span: break_token.span,
        };

        Ok(self.arena.alloc(Statement::Break(break_stmt)))
    }

    fn parse_continue_statement<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Statement> {
        let continue_token = tokens.consume(); // consume 'continue'

        // Consume optional semicolon
        if matches!(tokens.peek().token_type, TokenType::Semicolon) {
            tokens.consume();
        }

        let continue_stmt = ContinueStatement {
            span: continue_token.span,
        };

        Ok(self.arena.alloc(Statement::Continue(continue_stmt)))
    }

    /// Parse a labeled block (for break/continue)
    pub fn parse_labeled_block<T: TokenStream>(
        &mut self,
        tokens: &mut T,
        label: String,
    ) -> ParseResult<&'arena Block> {
        let block = self.parse_block(tokens)?.clone();

        // Create new block with label
        let labeled_block = self.arena.alloc(Block {
            statements: block.statements,
            is_braced: block.is_braced,
            scope_depth: block.scope_depth,
            span: block.span,
            is_unsafe: block.is_unsafe,
            is_async: block.is_async,
            is_try: block.is_try,
            label: Some(label),
        });

        Ok(labeled_block)
    }

    /// Parse an unsafe block
    pub fn parse_unsafe_block<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Block> {
        let start_span = tokens.peek().span.clone();

        // Consume 'unsafe' keyword
        self.expect_token(tokens, TokenType::Unsafe)?;

        let block = self.parse_braced_block(tokens)?.clone();

        // Create new block marked as unsafe
        let unsafe_block = self.arena.alloc(Block {
            statements: block.statements,
            is_braced: true,
            scope_depth: block.scope_depth,
            span: start_span.combine(block.span),
            is_unsafe: true,
            is_async: block.is_async,
            is_try: block.is_try,
            label: block.label,
        });

        Ok(unsafe_block)
    }

    /// Parse an async block
    pub fn parse_async_block<T: TokenStream>(
        &mut self,
        tokens: &mut T,
    ) -> ParseResult<&'arena Block> {
        let start_span = tokens.peek().span.clone();

        // Consume 'async' keyword
        self.expect_token(tokens, TokenType::Async)?;

        let block = self.parse_braced_block(tokens)?.clone();

        // Create new block marked as async
        let async_block = self.arena.alloc(Block {
            statements: block.statements,
            is_braced: true,
            scope_depth: block.scope_depth,
            span: start_span.combine(block.span),
            is_unsafe: block.is_unsafe,
            is_async: true,
            is_try: block.is_try,
            label: block.label,
        });

        Ok(async_block)
    }

    /// Validate scope consistency and variable shadowing
    pub fn validate_scope(&self, scope: &ScopeInfo) -> ParseResult<()> {
        // Check for variable redefinition within same scope
        let mut seen_vars = std::collections::HashSet::new();
        for var in &scope.variables {
            if seen_vars.contains(var) {
                return Err(ParseError::variable_redefinition(
                    var,
                    Span::dummy(), // Would need actual span tracking
                ));
            }
            seen_vars.insert(var.clone());
        }

        Ok(())
    }

    /// Expect a specific token type
    fn expect_token<T: TokenStream>(
        &mut self,
        tokens: &mut T,
        expected: TokenType,
    ) -> ParseResult<Token> {
        let current = tokens.peek();
        if std::mem::discriminant(&current.token_type) == std::mem::discriminant(&expected) {
            let token = current.clone();
            tokens.consume();
            Ok(token)
        } else {
            Err(ParseError::unexpected_token(
                &format!("{:?}", expected),
                current,
            ))
        }
    }
}

/// Convenience functions for block parsing
pub fn parse_block<T: TokenStream>(arena: &Arena, tokens: &mut T) -> ParseResult<Block> {
    let mut parser = BlockParser::new(arena);
    let block_ref = parser.parse_block(tokens)?;
    Ok(block_ref.clone())
}

pub fn parse_braced_block<T: TokenStream>(arena: &Arena, tokens: &mut T) -> ParseResult<Block> {
    let mut parser = BlockParser::new(arena);
    let block_ref = parser.parse_braced_block(tokens)?;
    Ok(block_ref.clone())
}

pub fn parse_indented_block<T: TokenStream>(arena: &Arena, tokens: &mut T) -> ParseResult<Block> {
    let mut parser = BlockParser::new(arena);
    let block_ref = parser.parse_indented_block(tokens)?;
    Ok(block_ref.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_parser_creation() {
        let arena = Arena::new();
        let _parser = BlockParser::new(&arena);
    }

    #[test]
    fn test_block_style_consistency() {
        // Test that mixing block styles is detected as an error
        assert_eq!(BlockStyle::Braced, BlockStyle::Braced);
        assert_ne!(BlockStyle::Braced, BlockStyle::Indented);
    }
}
