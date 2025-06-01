use crate::{
    ast::{
        Arena, Attribute, Block, BreakStatement, CompilationUnit, ContinueStatement, DataClassDecl,
        ExternBlock, ExternFunction, ExternItem, ExternVariable, Field, ForStatement, FunctionDecl,
        IfStatement, Item, Modifiers, Parameter, ReturnStatement, Statement, Type, VariableDecl,
        WhileStatement,
    },
    error::ParseError,
    token::{Span, Token, TokenStream, TokenType},
};

/// Statement parser for handling all statement types
pub struct StatementParser<'arena, T: TokenStream> {
    arena: &'arena Arena,
    tokens: T,
}

impl<'arena, T: TokenStream> StatementParser<'arena, T> {
    pub fn new(arena: &'arena Arena, tokens: T) -> Self {
        Self { arena, tokens }
    }

    /// Parse a complete compilation unit
    pub fn parse_compilation_unit(&mut self) -> Result<&'arena CompilationUnit, ParseError> {
        let mut items = Vec::new();
        let start_span = self.current_span();

        // Skip any leading whitespace/newlines
        self.skip_newlines();

        while !self.is_at_end() {
            let item = self.parse_item()?;
            items.push(item.clone());
            self.skip_newlines();
        }

        let end_span = self.previous_span();
        let span = Span::new(
            start_span.start,
            end_span.end,
            start_span.line,
            start_span.column,
        );

        Ok(self.arena.alloc(CompilationUnit { items, span }))
    }

    /// Parse a single statement
    pub fn parse_statement(&mut self) -> Result<&'arena Statement, ParseError> {
        // Check for attributes first
        let attributes = self.parse_attributes()?;

        let token = self.peek();

        match &token.token_type {
            // Variable declarations
            TokenType::Let | TokenType::Var => {
                let var_decl = self.parse_variable_declaration_with_modifiers_and_attributes(
                    Modifiers {
                        is_public: false,
                        is_unsafe: false,
                    },
                    attributes,
                )?;
                Ok(self.arena.alloc(Statement::VariableDecl(var_decl.clone())))
            }

            // Control flow
            TokenType::If => {
                let if_stmt = self.parse_if_statement()?;
                Ok(self.arena.alloc(Statement::If(if_stmt.clone())))
            }
            TokenType::While => {
                let while_stmt = self.parse_while_statement()?;
                Ok(self.arena.alloc(Statement::While(while_stmt.clone())))
            }
            TokenType::For => {
                let for_stmt = self.parse_for_statement()?;
                Ok(self.arena.alloc(Statement::For(for_stmt.clone())))
            }
            TokenType::Return => {
                let return_stmt = self.parse_return_statement()?;
                Ok(self.arena.alloc(Statement::Return(return_stmt.clone())))
            }
            TokenType::Break => {
                let break_stmt = self.parse_break_statement()?;
                Ok(self.arena.alloc(Statement::Break(break_stmt.clone())))
            }
            TokenType::Continue => {
                let continue_stmt = self.parse_continue_statement()?;
                Ok(self.arena.alloc(Statement::Continue(continue_stmt.clone())))
            }

            // Block statements
            TokenType::LeftBrace => {
                let block = self.parse_block()?;
                Ok(self.arena.alloc(Statement::Block(block.clone())))
            }

            // Expression statements (fallback)
            _ => {
                // If we have attributes but no statement that can use them, that's an error
                if !attributes.is_empty() {
                    return Err(ParseError::unexpected_token(
                        "statement that supports attributes",
                        &token,
                    ));
                }

                let expr = self.parse_expression()?;
                // Consume optional semicolon
                if matches!(self.peek().token_type, TokenType::Semicolon) {
                    self.consume();
                }
                Ok(self.arena.alloc(Statement::Expression(expr.clone())))
            }
        }
    }

    /// Parse a top-level item (function, data class, extern block, etc.)
    pub fn parse_item(&mut self) -> Result<&'arena Item, ParseError> {
        // Parse optional attributes first
        let attributes = self.parse_attributes()?;

        // Parse optional modifiers
        let modifiers = self.parse_modifiers()?;

        let token = self.peek();
        match &token.token_type {
            TokenType::Fn | TokenType::Async => {
                let func_decl =
                    self.parse_function_declaration_with_attributes(modifiers, attributes)?;
                Ok(self.arena.alloc(Item::FunctionDecl(func_decl.clone())))
            }
            TokenType::Let | TokenType::Var => {
                let var_decl = self.parse_variable_declaration_with_modifiers_and_attributes(
                    modifiers, attributes,
                )?;
                Ok(self.arena.alloc(Item::VariableDecl(var_decl.clone())))
            }
            TokenType::Data => {
                let data_decl = self.parse_data_class_declaration_with_attributes(attributes)?;
                Ok(self.arena.alloc(Item::DataClassDecl(data_decl.clone())))
            }
            TokenType::Extern => {
                let extern_block = self.parse_extern_block()?;
                Ok(self.arena.alloc(Item::ExternBlock(extern_block.clone())))
            }
            _ => Err(ParseError::unexpected_token("item declaration", &token)),
        }
    }

    // Utility methods
    fn peek(&self) -> Token {
        self.tokens.peek().clone()
    }

    fn consume(&mut self) -> Token {
        self.tokens.consume()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn current_span(&self) -> Span {
        self.peek().span.clone()
    }

    fn previous_span(&self) -> Span {
        // For now, return current span - in real implementation we'd track previous
        self.peek().span.clone()
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek().token_type, TokenType::Newline) {
            self.consume();
        }
    }

    fn parse_expression(&mut self) -> Result<&'arena crate::ast::Expression, ParseError> {
        let token = self.consume();

        match token.token_type {
            TokenType::BooleanLiteral(b) => Ok(self.arena.alloc(crate::ast::Expression::Literal(
                crate::ast::Literal::Boolean(b),
            ))),
            TokenType::IntegerLiteral(i) => Ok(self.arena.alloc(crate::ast::Expression::Literal(
                crate::ast::Literal::Integer(i),
            ))),
            TokenType::Identifier(name) => {
                Ok(self.arena.alloc(crate::ast::Expression::Identifier(name)))
            }
            _ => Err(ParseError::unexpected_token("expression", &token)),
        }
    }

    // Placeholder implementations - we'll implement these in the next steps
    fn parse_modifiers(&mut self) -> Result<Modifiers, ParseError> {
        let mut is_public = false;
        let mut is_unsafe = false;

        while matches!(self.peek().token_type, TokenType::Pub | TokenType::Unsafe) {
            match self.consume().token_type {
                TokenType::Pub => is_public = true,
                TokenType::Unsafe => is_unsafe = true,
                _ => unreachable!(),
            }
        }

        Ok(Modifiers {
            is_public,
            is_unsafe,
        })
    }

    #[allow(dead_code)]
    fn parse_variable_declaration(&mut self) -> Result<VariableDecl, ParseError> {
        self.parse_variable_declaration_with_modifiers(Modifiers {
            is_public: false,
            is_unsafe: false,
        })
    }

    #[allow(dead_code)]
    fn parse_variable_declaration_with_modifiers(
        &mut self,
        modifiers: Modifiers,
    ) -> Result<VariableDecl, ParseError> {
        self.parse_variable_declaration_with_modifiers_and_attributes(modifiers, Vec::new())
    }

    fn parse_variable_declaration_with_modifiers_and_attributes(
        &mut self,
        modifiers: Modifiers,
        attributes: Vec<Attribute>,
    ) -> Result<VariableDecl, ParseError> {
        let start_token = self.consume(); // let or var
        let is_mutable = matches!(start_token.token_type, TokenType::Var);

        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("variable name", &name_token)),
        };

        // Optional type annotation
        let var_type = if matches!(self.peek().token_type, TokenType::Colon) {
            self.consume(); // consume ':'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Optional initializer
        let initializer = if matches!(self.peek().token_type, TokenType::Equal) {
            self.consume(); // consume '='
            Some(self.parse_expression()?.clone())
        } else {
            None
        };

        Ok(VariableDecl {
            name,
            var_type,
            initializer,
            is_mutable,
            modifiers,
            attributes,
            span: start_token.span,
        })
    }

    #[allow(dead_code)]
    fn parse_function_declaration(
        &mut self,
        modifiers: Modifiers,
    ) -> Result<FunctionDecl, ParseError> {
        self.parse_function_declaration_with_attributes(modifiers, Vec::new())
    }

    fn parse_function_declaration_with_attributes(
        &mut self,
        modifiers: Modifiers,
        attributes: Vec<Attribute>,
    ) -> Result<FunctionDecl, ParseError> {
        // Check for async keyword first
        let is_async = if matches!(self.peek().token_type, TokenType::Async) {
            self.consume(); // consume 'async'
            true
        } else {
            false
        };

        // Now consume 'fn'
        if !matches!(self.peek().token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("fn", &self.peek()));
        }
        self.consume(); // consume 'fn'

        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err(ParseError::unexpected_token("function name", &self.peek()));
        };
        let name_span = self.consume().span;

        // Parse generic parameters if present
        let generics = crate::generic::parser::parse_generic_params(&mut self.tokens)?;

        if !matches!(self.peek().token_type, TokenType::LeftParen) {
            return Err(ParseError::unexpected_token("(", &self.peek()));
        }

        let parameters = self.parse_parameter_list()?;

        let return_type = if matches!(self.peek().token_type, TokenType::Arrow) {
            self.consume(); // consume '->'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Handle where clause that might come after return type
        let final_generics = if matches!(self.peek().token_type, TokenType::Where) {
            // Parse where clause
            let where_clause = self.parse_where_clause()?;
            let where_clause_span = where_clause.span.clone();

            if let Some(mut gen) = generics {
                gen.where_clause = Some(where_clause);
                Some(gen)
            } else {
                // Create new generics with just the where clause
                Some(crate::ast::GenericParams {
                    params: Vec::new(),
                    where_clause: Some(where_clause),
                    span: where_clause_span,
                })
            }
        } else {
            generics
        };

        let body = if matches!(self.peek().token_type, TokenType::LeftBrace) {
            Some(self.parse_block()?)
        } else if matches!(self.peek().token_type, TokenType::Semicolon) {
            self.consume(); // consume ';'
            None
        } else {
            return Err(ParseError::unexpected_token("{ or ;", &self.peek()));
        };

        Ok(FunctionDecl {
            name,
            generics: final_generics,
            parameters,
            return_type,
            body,
            is_async,
            is_extern: false,
            abi: None,
            modifiers,
            attributes,
            span: name_span,
        })
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let open_paren = self.consume();
        if !matches!(open_paren.token_type, TokenType::LeftParen) {
            return Err(ParseError::unexpected_token("'('", &open_paren));
        }

        let mut parameters = Vec::new();

        if matches!(self.peek().token_type, TokenType::RightParen) {
            self.consume();
            return Ok(parameters);
        }

        loop {
            let param = self.parse_parameter()?;
            parameters.push(param);

            match self.consume().token_type {
                TokenType::Comma => {
                    if matches!(self.peek().token_type, TokenType::RightParen) {
                        self.consume();
                        break;
                    }
                }
                TokenType::RightParen => break,
                _ => return Err(ParseError::unexpected_token("',' or ')'", &self.peek())),
            }
        }

        Ok(parameters)
    }

    fn parse_parameter(&mut self) -> Result<Parameter, ParseError> {
        // Parse optional attributes
        let attributes = self.parse_attributes()?;

        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("parameter name", &name_token)),
        };

        let colon_token = self.consume();
        if !matches!(colon_token.token_type, TokenType::Colon) {
            return Err(ParseError::unexpected_token("':'", &colon_token));
        }

        let param_type = self.parse_type()?;

        Ok(Parameter {
            name,
            param_type,
            attributes,
            span: name_token.span,
        })
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, ParseError> {
        crate::attribute::parse_attributes(&mut self.tokens)
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        crate::types::parse_type(&mut self.tokens)
    }

    #[allow(dead_code)]
    fn parse_data_class_declaration(&mut self) -> Result<DataClassDecl, ParseError> {
        self.parse_data_class_declaration_with_attributes(Vec::new())
    }

    fn parse_data_class_declaration_with_attributes(
        &mut self,
        attributes: Vec<Attribute>,
    ) -> Result<DataClassDecl, ParseError> {
        self.consume(); // consume 'data'

        let name = if let TokenType::Identifier(name) = &self.peek().token_type {
            name.clone()
        } else {
            return Err(ParseError::unexpected_token(
                "data class name",
                &self.peek(),
            ));
        };
        let name_span = self.consume().span;

        // Parse generic parameters if present
        let generics = crate::generic::parser::parse_generic_params(&mut self.tokens)?;

        if !matches!(self.peek().token_type, TokenType::LeftBrace) {
            return Err(ParseError::unexpected_token("'{'", &self.peek()));
        }

        self.consume(); // consume '{'
        let mut fields = Vec::new();

        while !matches!(self.peek().token_type, TokenType::RightBrace) && !self.is_at_end() {
            fields.push(self.parse_field()?);

            if matches!(self.peek().token_type, TokenType::Comma) {
                self.consume(); // consume ','
            }
        }

        if !matches!(self.peek().token_type, TokenType::RightBrace) {
            return Err(ParseError::unexpected_token("'}'", &self.peek()));
        }
        self.consume(); // consume '}'

        Ok(DataClassDecl {
            name,
            generics,
            fields,
            attributes,
            span: name_span,
        })
    }

    fn parse_field(&mut self) -> Result<Field, ParseError> {
        // Parse attributes before field declaration
        let attributes = self.parse_attributes()?;

        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("field name", &name_token)),
        };

        let colon_token = self.consume();
        if !matches!(colon_token.token_type, TokenType::Colon) {
            return Err(ParseError::unexpected_token("':'", &colon_token));
        }

        let field_type = self.parse_type()?;

        Ok(Field {
            name,
            field_type,
            attributes,
            span: name_token.span,
        })
    }

    fn parse_extern_block(&mut self) -> Result<ExternBlock, ParseError> {
        let extern_token = self.consume();
        if !matches!(extern_token.token_type, TokenType::Extern) {
            return Err(ParseError::unexpected_token("'extern'", &extern_token));
        }

        // ABI string
        let abi_token = self.consume();
        let abi = match abi_token.token_type {
            TokenType::StringLiteral(abi) => abi,
            _ => return Err(ParseError::unexpected_token("ABI string", &abi_token)),
        };

        let open_brace = self.consume();
        if !matches!(open_brace.token_type, TokenType::LeftBrace) {
            return Err(ParseError::unexpected_token("'{'", &open_brace));
        }

        let mut items = Vec::new();
        while !matches!(
            self.peek().token_type,
            TokenType::RightBrace | TokenType::Eof
        ) {
            let item = self.parse_extern_item()?;
            items.push(item);
        }

        let close_brace = self.consume();
        if !matches!(close_brace.token_type, TokenType::RightBrace) {
            return Err(ParseError::unexpected_token("'}'", &close_brace));
        }

        Ok(ExternBlock {
            abi,
            items,
            span: extern_token.span,
        })
    }

    fn parse_extern_item(&mut self) -> Result<ExternItem, ParseError> {
        let token = self.peek();
        match &token.token_type {
            TokenType::Fn => {
                let func = self.parse_extern_function()?;
                Ok(ExternItem::Function(func))
            }
            TokenType::Static => {
                let var = self.parse_extern_variable()?;
                Ok(ExternItem::Variable(var))
            }
            _ => Err(ParseError::unexpected_token("'fn' or 'static'", &token)),
        }
    }

    fn parse_extern_function(&mut self) -> Result<ExternFunction, ParseError> {
        let fn_token = self.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("function name", &name_token)),
        };

        let parameters = self.parse_parameter_list()?;

        let return_type = if matches!(self.peek().token_type, TokenType::Arrow) {
            self.consume(); // consume '->'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Expect semicolon
        let semicolon = self.consume();
        if !matches!(semicolon.token_type, TokenType::Semicolon) {
            return Err(ParseError::unexpected_token("';'", &semicolon));
        }

        Ok(ExternFunction {
            name,
            parameters,
            return_type,
            span: fn_token.span,
        })
    }

    fn parse_extern_variable(&mut self) -> Result<ExternVariable, ParseError> {
        let static_token = self.consume();
        if !matches!(static_token.token_type, TokenType::Static) {
            return Err(ParseError::unexpected_token("'static'", &static_token));
        }

        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("variable name", &name_token)),
        };

        let colon_token = self.consume();
        if !matches!(colon_token.token_type, TokenType::Colon) {
            return Err(ParseError::unexpected_token("':'", &colon_token));
        }

        let var_type = self.parse_type()?;

        let semicolon = self.consume();
        if !matches!(semicolon.token_type, TokenType::Semicolon) {
            return Err(ParseError::unexpected_token("';'", &semicolon));
        }

        Ok(ExternVariable {
            name,
            var_type,
            span: static_token.span,
        })
    }

    // Control flow statement parsers
    fn parse_if_statement(&mut self) -> Result<IfStatement, ParseError> {
        let if_token = self.consume();
        if !matches!(if_token.token_type, TokenType::If) {
            return Err(ParseError::unexpected_token("'if'", &if_token));
        }

        let condition = self.parse_expression()?.clone();
        let then_block = self.parse_block()?;

        let else_block = if matches!(self.peek().token_type, TokenType::Else) {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(IfStatement {
            condition,
            then_block,
            else_block,
            span: if_token.span,
        })
    }

    fn parse_while_statement(&mut self) -> Result<WhileStatement, ParseError> {
        let while_token = self.consume();
        if !matches!(while_token.token_type, TokenType::While) {
            return Err(ParseError::unexpected_token("'while'", &while_token));
        }

        let condition = self.parse_expression()?.clone();
        let body = self.parse_block()?;

        Ok(WhileStatement {
            condition,
            body,
            span: while_token.span,
        })
    }

    fn parse_for_statement(&mut self) -> Result<ForStatement, ParseError> {
        let for_token = self.consume();
        if !matches!(for_token.token_type, TokenType::For) {
            return Err(ParseError::unexpected_token("'for'", &for_token));
        }

        let var_token = self.consume();
        let variable = match var_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("variable name", &var_token)),
        };

        let in_token = self.consume();
        if !matches!(in_token.token_type, TokenType::In) {
            return Err(ParseError::unexpected_token("'in'", &in_token));
        }

        let iterable = self.parse_expression()?.clone();
        let body = self.parse_block()?;

        Ok(ForStatement {
            variable,
            iterable,
            body,
            span: for_token.span,
        })
    }

    fn parse_return_statement(&mut self) -> Result<ReturnStatement, ParseError> {
        let return_token = self.consume();
        if !matches!(return_token.token_type, TokenType::Return) {
            return Err(ParseError::unexpected_token("'return'", &return_token));
        }

        let value = if matches!(
            self.peek().token_type,
            TokenType::Semicolon | TokenType::Newline | TokenType::Eof
        ) {
            None
        } else {
            Some(self.parse_expression()?.clone())
        };

        Ok(ReturnStatement {
            value,
            span: return_token.span,
        })
    }

    fn parse_break_statement(&mut self) -> Result<BreakStatement, ParseError> {
        let break_token = self.consume();
        if !matches!(break_token.token_type, TokenType::Break) {
            return Err(ParseError::unexpected_token("'break'", &break_token));
        }

        Ok(BreakStatement {
            span: break_token.span,
        })
    }

    fn parse_continue_statement(&mut self) -> Result<ContinueStatement, ParseError> {
        let continue_token = self.consume();
        if !matches!(continue_token.token_type, TokenType::Continue) {
            return Err(ParseError::unexpected_token("'continue'", &continue_token));
        }

        Ok(ContinueStatement {
            span: continue_token.span,
        })
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let open_brace = self.consume();
        if !matches!(open_brace.token_type, TokenType::LeftBrace) {
            return Err(ParseError::unexpected_token("'{'", &open_brace));
        }

        let mut statements = Vec::new();
        self.skip_newlines();

        while !matches!(
            self.peek().token_type,
            TokenType::RightBrace | TokenType::Eof
        ) {
            let stmt = self.parse_statement()?;
            statements.push(stmt.clone());
            self.skip_newlines();
        }

        let close_brace = self.consume();
        if !matches!(close_brace.token_type, TokenType::RightBrace) {
            return Err(ParseError::unexpected_token("'}'", &close_brace));
        }

        let start_span = open_brace.span;
        let end_span = close_brace.span;

        Ok(Block {
            statements,
            is_braced: true,
            span: start_span.combine(end_span),
            scope_depth: 0,
            is_unsafe: false,
            is_async: false,
            is_try: false,
            label: None,
        })
    }

    fn parse_where_clause(&mut self) -> Result<crate::ast::WhereClause, ParseError> {
        let start_span = self.consume().span; // consume 'where'
        let mut constraints = Vec::new();

        // Parse first constraint
        constraints.push(self.parse_where_constraint()?);

        // Parse additional constraints separated by ','
        while matches!(self.peek().token_type, TokenType::Comma) {
            self.consume(); // consume ','

            // Allow trailing comma
            if self.is_where_clause_end() {
                break;
            }

            constraints.push(self.parse_where_constraint()?);
        }

        let end_span = if let Some(last_constraint) = constraints.last() {
            last_constraint.span.clone()
        } else {
            start_span.clone()
        };

        Ok(crate::ast::WhereClause {
            constraints,
            span: start_span.combine(end_span),
        })
    }

    fn parse_where_constraint(&mut self) -> Result<crate::ast::WhereConstraint, ParseError> {
        if let TokenType::Identifier(type_name) = &self.peek().token_type {
            let type_name = type_name.clone();
            let start_span = self.consume().span;

            if !matches!(self.peek().token_type, TokenType::Colon) {
                return Err(ParseError::unexpected_token(":", &self.peek()));
            }

            self.consume(); // consume ':'
            let bounds = self.parse_type_bounds()?;

            let end_span = if let Some(last_bound) = bounds.last() {
                last_bound.span.clone()
            } else {
                start_span.clone()
            };

            Ok(crate::ast::WhereConstraint {
                type_name,
                bounds,
                span: start_span.combine(end_span),
            })
        } else {
            Err(ParseError::unexpected_token("type name", &self.peek()))
        }
    }

    fn parse_type_bounds(&mut self) -> Result<Vec<crate::ast::TypeBound>, ParseError> {
        let mut bounds = Vec::new();

        // Parse first bound
        bounds.push(self.parse_type_bound()?);

        // Parse additional bounds separated by '+'
        while matches!(self.peek().token_type, TokenType::Plus) {
            self.consume(); // consume '+'
            bounds.push(self.parse_type_bound()?);
        }

        Ok(bounds)
    }

    fn parse_type_bound(&mut self) -> Result<crate::ast::TypeBound, ParseError> {
        if let TokenType::Identifier(trait_name) = &self.peek().token_type {
            let trait_name = trait_name.clone();
            let span = self.consume().span;

            Ok(crate::ast::TypeBound { trait_name, span })
        } else {
            Err(ParseError::unexpected_token("trait name", &self.peek()))
        }
    }

    fn is_where_clause_end(&self) -> bool {
        matches!(
            self.peek().token_type,
            TokenType::LeftBrace | TokenType::Semicolon | TokenType::Eof | TokenType::Newline
        )
    }
}
