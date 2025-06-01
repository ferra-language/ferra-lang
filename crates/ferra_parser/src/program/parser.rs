//! Program parser for complete Ferra programs

use crate::{
    ast::{
        Arena, Attribute, Block, CompilationUnit, DataClassDecl, ExternBlock, Field, FunctionDecl,
        Item, Modifiers, Parameter, Type,
    },
    error::{DiagnosticReport, ErrorCollector, ParseError},
    statement::StatementParser,
    token::{Span, Token, TokenStream, TokenType},
};

/// Top-level program parser that integrates all component parsers
pub struct ProgramParser<'arena, T: TokenStream + Clone> {
    arena: &'arena Arena,
    tokens: T,
    error_collector: Option<ErrorCollector>, // Lazy initialization
}

#[allow(dead_code)] // Legacy methods kept for compatibility
impl<'arena, T: TokenStream + Clone> ProgramParser<'arena, T> {
    /// Create a new program parser
    pub fn new(arena: &'arena Arena, tokens: T) -> Self {
        Self {
            arena,
            tokens,
            error_collector: None, // Lazy initialization for better creation performance
        }
    }

    /// Get or initialize the error collector (lazy initialization)
    #[inline]
    fn error_collector(&mut self) -> &mut ErrorCollector {
        if self.error_collector.is_none() {
            self.error_collector = Some(ErrorCollector::new(50));
        }
        self.error_collector.as_mut().unwrap()
    }

    /// Check if any errors have been collected
    pub fn has_errors(&mut self) -> bool {
        self.error_collector
            .as_ref()
            .is_some_and(|ec| ec.has_errors())
    }

    /// Get all collected errors
    pub fn get_errors(&mut self) -> Vec<ParseError> {
        self.error_collector().get_errors().to_vec()
    }

    /// Parse a complete compilation unit (top-level program)
    pub fn parse_compilation_unit(&mut self) -> Result<&'arena CompilationUnit, Vec<ParseError>> {
        let start_span = self.current_span();

        // Fast path for empty programs
        if self.tokens.is_at_end() {
            let compilation_unit = self.arena.alloc(CompilationUnit {
                items: Vec::new(),
                span: start_span,
            });
            return Ok(compilation_unit);
        }

        // Pre-allocate items vector with reasonable capacity
        let mut items = Vec::with_capacity(8);

        // Parse top-level items until EOF
        while !self.tokens.is_at_end() {
            match self.parse_top_level_item() {
                Ok(item) => items.push(item.clone()),
                Err(error) => {
                    // Initialize error collector if not already done
                    if self.error_collector.is_none() {
                        self.error_collector = Some(ErrorCollector::new(50));
                    }

                    // Add error and try recovery
                    self.error_collector.as_mut().unwrap().add_error(error);

                    // Try to recover to next top-level item using improved error recovery
                    use crate::error::recovery::ErrorRecovery;
                    let recovery_result = ErrorRecovery::smart_recovery(
                        &mut self.tokens,
                        "declaration",
                        self.error_collector.as_mut().unwrap(),
                    );

                    if recovery_result.is_some() {
                        continue;
                    } else {
                        // Can't recover, stop parsing
                        break;
                    }
                }
            }
        }

        let end_span = self.current_span();
        let span = start_span.combine(end_span);

        if self.has_errors() {
            Err(self.get_errors())
        } else {
            let compilation_unit = self.arena.alloc(CompilationUnit { items, span });
            Ok(compilation_unit)
        }
    }

    /// Parse a complete program and return both result and diagnostics
    pub fn parse_program_with_diagnostics(
        &mut self,
    ) -> (Option<&'arena CompilationUnit>, DiagnosticReport) {
        let mut report = DiagnosticReport::new(None);

        match self.parse_compilation_unit() {
            Ok(program) => (Some(program), report),
            Err(errors) => {
                report.add_errors(errors);
                (None, report)
            }
        }
    }

    /// Parse a top-level item (function, data class, extern block, etc.)
    fn parse_top_level_item(&mut self) -> Result<&'arena Item, ParseError> {
        // Parse optional attributes first
        let attributes = self.parse_attributes()?;

        // Parse optional modifiers
        let modifiers = self.parse_modifiers()?;

        let current = self.tokens.peek();

        match current.token_type {
            TokenType::Fn => self.parse_function_declaration_with_attributes(modifiers, attributes),
            TokenType::Async => self.parse_async_item_with_attributes(modifiers, attributes),
            TokenType::Data => self.parse_data_class_declaration_with_attributes(attributes),
            TokenType::Extern => self.parse_extern_block(),
            TokenType::Static => self.parse_static_variable_with_attributes(modifiers, attributes),
            TokenType::Let | TokenType::Var => self.parse_variable_declaration_with_attributes(modifiers, attributes),
            _ => Err(ParseError::unexpected_token(
                "function, data class, extern block, variable declaration, or other top-level declaration",
                current,
            )),
        }
    }

    /// Parse a function declaration
    fn parse_function_declaration(&mut self) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume 'fn'
        let fn_token = self.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        // Function name
        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("function name", &name_token)),
        };

        // Parameters
        let parameters = self.parse_parameter_list()?;

        // Return type
        let return_type = if matches!(self.tokens.peek().token_type, TokenType::Arrow) {
            self.consume(); // consume '->'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body
        let body = if matches!(self.tokens.peek().token_type, TokenType::LeftBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let func_decl = FunctionDecl {
            name,
            generics: None,
            parameters,
            return_type,
            body,
            is_async: false,
            is_extern: false,
            abi: None,
            modifiers: Modifiers {
                is_public: false,
                is_unsafe: false,
            },
            attributes: Vec::new(),
            span: start_span,
        };

        Ok(self.arena.alloc(Item::FunctionDecl(func_decl)))
    }

    /// Parse a data class declaration
    fn parse_data_class_declaration(&mut self) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume 'data'
        let data_token = self.consume();
        if !matches!(data_token.token_type, TokenType::Data) {
            return Err(ParseError::unexpected_token("'data'", &data_token));
        }

        // Class name
        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("class name", &name_token)),
        };

        // Fields
        let fields = self.parse_field_list()?;

        let data_decl = DataClassDecl {
            name,
            generics: None,
            fields,
            attributes: Vec::new(),
            span: start_span,
        };

        Ok(self.arena.alloc(Item::DataClassDecl(data_decl)))
    }

    /// Parse an extern block
    fn parse_extern_block(&mut self) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume 'extern'
        let extern_token = self.consume();
        if !matches!(extern_token.token_type, TokenType::Extern) {
            return Err(ParseError::unexpected_token("'extern'", &extern_token));
        }

        // Optional ABI specification
        let abi = if matches!(self.tokens.peek().token_type, TokenType::StringLiteral(_)) {
            let abi_token = self.consume();
            match abi_token.token_type {
                TokenType::StringLiteral(abi) => Some(abi),
                _ => unreachable!(),
            }
        } else {
            None
        };

        // Parse extern items
        let items = self.parse_extern_item_list()?;

        let extern_block = ExternBlock {
            abi: abi.unwrap_or_else(|| "C".to_string()),
            items,
            span: start_span,
        };

        Ok(self.arena.alloc(Item::ExternBlock(extern_block)))
    }

    /// Parse a public item (pub fn, pub data, etc.)
    fn parse_public_item(&mut self) -> Result<&'arena Item, ParseError> {
        // Consume 'pub' keyword
        self.consume();

        // Check what kind of item follows
        let current = self.tokens.peek();
        match current.token_type {
            TokenType::Fn => self.parse_function_declaration_with_modifiers(Modifiers {
                is_public: true,
                is_unsafe: false,
            }),
            TokenType::Async => {
                // Handle pub async combination
                self.consume(); // consume 'async'
                let inner_token = self.tokens.peek();
                match inner_token.token_type {
                    TokenType::Fn => {
                        self.parse_async_function_declaration_with_modifiers(Modifiers {
                            is_public: true,
                            is_unsafe: false,
                        })
                    }
                    _ => Err(ParseError::unexpected_token(
                        "fn after pub async",
                        inner_token,
                    )),
                }
            }
            TokenType::Let | TokenType::Var => {
                self.parse_variable_declaration_with_modifiers(Modifiers {
                    is_public: true,
                    is_unsafe: false,
                })
            }
            TokenType::Data => self.parse_data_class_declaration_with_modifiers(Modifiers {
                is_public: true,
                is_unsafe: false,
            }),
            TokenType::Unsafe => {
                // Handle pub unsafe combination
                self.consume(); // consume 'unsafe'
                let inner_token = self.tokens.peek();
                match inner_token.token_type {
                    TokenType::Fn => self.parse_function_declaration_with_modifiers(Modifiers {
                        is_public: true,
                        is_unsafe: true,
                    }),
                    TokenType::Async => {
                        // Handle pub unsafe async combination
                        self.consume(); // consume 'async'
                        let async_token = self.tokens.peek();
                        match async_token.token_type {
                            TokenType::Fn => {
                                self.parse_async_function_declaration_with_modifiers(Modifiers {
                                    is_public: true,
                                    is_unsafe: true,
                                })
                            }
                            _ => Err(ParseError::unexpected_token(
                                "fn after pub unsafe async",
                                async_token,
                            )),
                        }
                    }
                    _ => Err(ParseError::unexpected_token(
                        "fn or async after pub unsafe",
                        inner_token,
                    )),
                }
            }
            _ => Err(ParseError::unexpected_token(
                "fn, async, let, var, data, or unsafe after pub",
                current,
            )),
        }
    }

    /// Parse an unsafe item
    fn parse_unsafe_item(&mut self) -> Result<&'arena Item, ParseError> {
        // Consume 'unsafe' keyword
        self.consume();

        // Check what kind of item follows
        let current = self.tokens.peek();
        match current.token_type {
            TokenType::Fn => self.parse_function_declaration_with_modifiers(Modifiers {
                is_public: false,
                is_unsafe: true,
            }),
            TokenType::Async => {
                // Handle unsafe async combination
                self.consume(); // consume 'async'
                let inner_token = self.tokens.peek();
                match inner_token.token_type {
                    TokenType::Fn => {
                        self.parse_async_function_declaration_with_modifiers(Modifiers {
                            is_public: false,
                            is_unsafe: true,
                        })
                    }
                    _ => Err(ParseError::unexpected_token(
                        "fn after unsafe async",
                        inner_token,
                    )),
                }
            }
            _ => Err(ParseError::unexpected_token(
                "fn or async after unsafe",
                current,
            )),
        }
    }

    /// Parse an async item (async fn)
    fn parse_async_item(&mut self) -> Result<&'arena Item, ParseError> {
        // Consume 'async' keyword
        self.consume();

        // Check what kind of item follows
        let current = self.tokens.peek();
        match current.token_type {
            TokenType::Fn => self.parse_async_function_declaration(),
            _ => Err(ParseError::unexpected_token("fn after async", current)),
        }
    }

    /// Parse an async function declaration (async fn)
    fn parse_async_function_declaration(&mut self) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume 'fn'
        let fn_token = self.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        // Function name
        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("function name", &name_token)),
        };

        // Parameters
        let parameters = self.parse_parameter_list()?;

        // Return type
        let return_type = if matches!(self.tokens.peek().token_type, TokenType::Arrow) {
            self.consume(); // consume '->'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body
        let body = if matches!(self.tokens.peek().token_type, TokenType::LeftBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let func_decl = FunctionDecl {
            name,
            generics: None,
            parameters,
            return_type,
            body,
            is_async: true, // This is the key difference
            is_extern: false,
            abi: None,
            modifiers: Modifiers {
                is_public: false,
                is_unsafe: false,
            },
            attributes: Vec::new(),
            span: start_span,
        };

        Ok(self.arena.alloc(Item::FunctionDecl(func_decl)))
    }

    /// Parse an async function declaration with modifiers
    fn parse_async_function_declaration_with_modifiers(
        &mut self,
        modifiers: Modifiers,
    ) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume 'fn'
        let fn_token = self.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        // Function name
        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("function name", &name_token)),
        };

        // Parameters
        let parameters = self.parse_parameter_list()?;

        // Return type
        let return_type = if matches!(self.tokens.peek().token_type, TokenType::Arrow) {
            self.consume(); // consume '->'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body
        let body = if matches!(self.tokens.peek().token_type, TokenType::LeftBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let func_decl = FunctionDecl {
            name,
            generics: None,
            parameters,
            return_type,
            body,
            is_async: true,
            is_extern: false,
            abi: None,
            modifiers,
            attributes: Vec::new(),
            span: start_span,
        };

        Ok(self.arena.alloc(Item::FunctionDecl(func_decl)))
    }

    /// Parse a static variable declaration
    fn parse_static_variable(&mut self) -> Result<&'arena Item, ParseError> {
        // For now, delegate to StatementParser for static variables
        let mut statement_parser = StatementParser::new(self.arena, self.tokens.clone());
        statement_parser.parse_item()
    }

    /// Parse a variable declaration (let/var)
    fn parse_variable_declaration(&mut self) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume let/var
        let var_token = self.consume();
        let is_mutable = matches!(var_token.token_type, TokenType::Var);

        // Variable name
        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("variable name", &name_token)),
        };

        // Optional type annotation
        let var_type = if matches!(self.tokens.peek().token_type, TokenType::Colon) {
            self.consume(); // consume ':'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Optional initializer
        let initializer = if matches!(self.tokens.peek().token_type, TokenType::Equal) {
            self.consume(); // consume '='
                            // Use proper expression parsing instead of single-token parsing
            Some(self.parse_expression()?.clone())
        } else {
            None
        };

        // Consume optional semicolon
        if matches!(self.tokens.peek().token_type, TokenType::Semicolon) {
            self.consume();
        }

        let var_decl = crate::ast::VariableDecl {
            name,
            var_type,
            initializer,
            is_mutable,
            modifiers: crate::ast::Modifiers {
                is_public: false,
                is_unsafe: false,
            },
            attributes: Vec::new(),
            span: start_span,
        };

        Ok(self.arena.alloc(crate::ast::Item::VariableDecl(var_decl)))
    }

    /// Parse a variable declaration with modifiers (let/var)
    fn parse_variable_declaration_with_attributes(
        &mut self,
        modifiers: Modifiers,
        attributes: Vec<Attribute>,
    ) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume let/var
        let var_token = self.consume();
        let is_mutable = matches!(var_token.token_type, TokenType::Var);

        // Variable name
        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("variable name", &name_token)),
        };

        // Optional type annotation
        let var_type = if matches!(self.tokens.peek().token_type, TokenType::Colon) {
            self.consume(); // consume ':'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Optional initializer
        let initializer = if matches!(self.tokens.peek().token_type, TokenType::Equal) {
            self.consume(); // consume '='
                            // Use proper expression parsing instead of single-token parsing
            Some(self.parse_expression()?.clone())
        } else {
            None
        };

        // Consume optional semicolon
        if matches!(self.tokens.peek().token_type, TokenType::Semicolon) {
            self.consume();
        }

        let var_decl = crate::ast::VariableDecl {
            name,
            var_type,
            initializer,
            is_mutable,
            modifiers,
            attributes,
            span: start_span,
        };

        Ok(self.arena.alloc(crate::ast::Item::VariableDecl(var_decl)))
    }

    /// Parse a variable declaration with modifiers (legacy compatibility)
    fn parse_variable_declaration_with_modifiers(
        &mut self,
        modifiers: Modifiers,
    ) -> Result<&'arena Item, ParseError> {
        self.parse_variable_declaration_with_attributes(modifiers, Vec::new())
    }

    /// Parse a function declaration with modifiers
    fn parse_function_declaration_with_modifiers(
        &mut self,
        modifiers: Modifiers,
    ) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume 'fn'
        let fn_token = self.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        // Function name
        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("function name", &name_token)),
        };

        // Parameters
        let parameters = self.parse_parameter_list()?;

        // Return type
        let return_type = if matches!(self.tokens.peek().token_type, TokenType::Arrow) {
            self.consume(); // consume '->'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body
        let body = if matches!(self.tokens.peek().token_type, TokenType::LeftBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let func_decl = FunctionDecl {
            name,
            generics: None,
            parameters,
            return_type,
            body,
            is_async: false,
            is_extern: false,
            abi: None,
            modifiers,
            attributes: Vec::new(),
            span: start_span,
        };

        Ok(self.arena.alloc(Item::FunctionDecl(func_decl)))
    }

    /// Parse a data class declaration with attributes
    fn parse_data_class_declaration_with_attributes(
        &mut self,
        attributes: Vec<Attribute>,
    ) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume 'data'
        let data_token = self.tokens.consume();
        if !matches!(data_token.token_type, TokenType::Data) {
            return Err(ParseError::unexpected_token("'data'", &data_token));
        }

        // Class name
        let name_token = self.tokens.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("class name", &name_token)),
        };

        // Fields
        let fields = self.parse_field_list()?;

        let data_decl = DataClassDecl {
            name,
            generics: None,
            fields,
            attributes,
            span: start_span,
        };

        Ok(self.arena.alloc(Item::DataClassDecl(data_decl)))
    }

    /// Parse parameter list for functions
    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let open_paren = self.consume();
        if !matches!(open_paren.token_type, TokenType::LeftParen) {
            return Err(ParseError::unexpected_token("'('", &open_paren));
        }

        let mut parameters = Vec::new();

        if matches!(self.tokens.peek().token_type, TokenType::RightParen) {
            self.consume();
            return Ok(parameters);
        }

        loop {
            let param = self.parse_parameter()?;
            parameters.push(param);

            if matches!(self.tokens.peek().token_type, TokenType::Comma) {
                self.consume();
            } else {
                break;
            }
        }

        let close_paren = self.consume();
        if !matches!(close_paren.token_type, TokenType::RightParen) {
            return Err(ParseError::unexpected_token("')'", &close_paren));
        }

        Ok(parameters)
    }

    /// Parse a single parameter
    fn parse_parameter(&mut self) -> Result<Parameter, ParseError> {
        let start_span = self.current_span();

        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("parameter name", &name_token)),
        };

        // Optional type annotation (for type inference support)
        let param_type = if matches!(self.tokens.peek().token_type, TokenType::Colon) {
            self.consume(); // consume ':'
            self.parse_type()?
        } else {
            // If no type annotation, use a placeholder type that inference can fill in
            Type::Identifier("_".to_string()) // Inferred type placeholder
        };

        Ok(Parameter {
            name,
            param_type,
            attributes: vec![],
            span: start_span,
        })
    }

    /// Parse a type
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        crate::types::parse_type(&mut self.tokens)
    }

    /// Parse field list for data classes
    fn parse_field_list(&mut self) -> Result<Vec<Field>, ParseError> {
        let open_brace = self.consume();
        if !matches!(open_brace.token_type, TokenType::LeftBrace) {
            return Err(ParseError::unexpected_token("'{'", &open_brace));
        }

        let mut fields = Vec::new();

        while !matches!(self.tokens.peek().token_type, TokenType::RightBrace) {
            let field = self.parse_field()?;
            fields.push(field);

            if matches!(self.tokens.peek().token_type, TokenType::Comma) {
                self.consume();
            }
        }

        let close_brace = self.consume();
        if !matches!(close_brace.token_type, TokenType::RightBrace) {
            return Err(ParseError::unexpected_token("'}'", &close_brace));
        }

        Ok(fields)
    }

    /// Parse a single field
    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let start_span = self.current_span();

        // Parse optional attributes for the field
        let attributes = self.parse_attributes()?;

        // Parse optional modifiers (like pub)
        let _modifiers = self.parse_modifiers()?;

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
            span: start_span,
        })
    }

    /// Parse extern item list
    fn parse_extern_item_list(&mut self) -> Result<Vec<crate::ast::ExternItem>, ParseError> {
        let open_brace = self.consume();
        if !matches!(open_brace.token_type, TokenType::LeftBrace) {
            return Err(ParseError::unexpected_token("'{'", &open_brace));
        }

        let mut items = Vec::new();

        while !matches!(self.tokens.peek().token_type, TokenType::RightBrace) {
            let item = self.parse_extern_item()?;
            items.push(item);
        }

        let close_brace = self.consume();
        if !matches!(close_brace.token_type, TokenType::RightBrace) {
            return Err(ParseError::unexpected_token("'}'", &close_brace));
        }

        Ok(items)
    }

    /// Parse a single extern item
    fn parse_extern_item(&mut self) -> Result<crate::ast::ExternItem, ParseError> {
        let current = self.tokens.peek();

        match current.token_type {
            TokenType::Fn => {
                let extern_func = self.parse_extern_function()?;
                Ok(crate::ast::ExternItem::Function(extern_func))
            }
            TokenType::Static => {
                let extern_var = self.parse_extern_variable()?;
                Ok(crate::ast::ExternItem::Variable(extern_var))
            }
            _ => Err(ParseError::unexpected_token(
                "extern function or variable",
                current,
            )),
        }
    }

    /// Parse extern function
    fn parse_extern_function(&mut self) -> Result<crate::ast::ExternFunction, ParseError> {
        let start_span = self.current_span();

        // Consume 'fn'
        let fn_token = self.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        // Function name
        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("function name", &name_token)),
        };

        // Parameters
        let parameters = self.parse_parameter_list()?;

        // Return type
        let return_type = if matches!(self.tokens.peek().token_type, TokenType::Arrow) {
            self.consume(); // consume '->'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Consume semicolon
        let semicolon = self.consume();
        if !matches!(semicolon.token_type, TokenType::Semicolon) {
            return Err(ParseError::unexpected_token("';'", &semicolon));
        }

        Ok(crate::ast::ExternFunction {
            name,
            parameters,
            return_type,
            span: start_span,
        })
    }

    /// Parse extern variable
    fn parse_extern_variable(&mut self) -> Result<crate::ast::ExternVariable, ParseError> {
        let start_span = self.current_span();

        // Consume 'static'
        let static_token = self.consume();
        if !matches!(static_token.token_type, TokenType::Static) {
            return Err(ParseError::unexpected_token("'static'", &static_token));
        }

        // Variable name
        let name_token = self.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("variable name", &name_token)),
        };

        // Type
        let colon_token = self.consume();
        if !matches!(colon_token.token_type, TokenType::Colon) {
            return Err(ParseError::unexpected_token("':'", &colon_token));
        }

        let var_type = self.parse_type()?;

        // Consume semicolon
        let semicolon = self.consume();
        if !matches!(semicolon.token_type, TokenType::Semicolon) {
            return Err(ParseError::unexpected_token("';'", &semicolon));
        }

        Ok(crate::ast::ExternVariable {
            name,
            var_type,
            span: start_span,
        })
    }

    /// Parse a block (simplified version)
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let mut block_parser = crate::block::parser::BlockParser::new(self.arena);
        let block_ref = block_parser.parse_block(&mut self.tokens)?;
        Ok(block_ref.clone())
    }

    /// Parse an expression (simplified)
    fn parse_expression(&mut self) -> Result<&'arena crate::ast::Expression, ParseError> {
        let mut pratt_parser = crate::pratt::parser::PrattParser::new(self.arena, &mut self.tokens);
        pratt_parser.parse_expression(0)
    }

    /// Get current span from token stream
    #[inline]
    fn current_span(&self) -> Span {
        self.tokens.peek().span.clone()
    }

    /// Consume next token
    #[inline]
    fn consume(&mut self) -> Token {
        self.tokens.consume()
    }

    /// Parse optional attributes
    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, ParseError> {
        crate::attribute::parser::parse_attributes(&mut self.tokens)
    }

    /// Parse optional modifiers
    fn parse_modifiers(&mut self) -> Result<Modifiers, ParseError> {
        let mut is_public = false;
        let mut is_unsafe = false;

        while matches!(
            self.tokens.peek().token_type,
            TokenType::Pub | TokenType::Unsafe
        ) {
            match self.tokens.consume().token_type {
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

    /// Parse function declaration with attributes
    fn parse_function_declaration_with_attributes(
        &mut self,
        modifiers: Modifiers,
        attributes: Vec<Attribute>,
    ) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume 'fn'
        let fn_token = self.tokens.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        // Function name
        let name_token = self.tokens.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("function name", &name_token)),
        };

        // Parameters
        let parameters = self.parse_parameter_list()?;

        // Return type
        let return_type = if matches!(self.tokens.peek().token_type, TokenType::Arrow) {
            self.tokens.consume(); // consume '->'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body
        let body = if matches!(self.tokens.peek().token_type, TokenType::LeftBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let func_decl = FunctionDecl {
            name,
            generics: None,
            parameters,
            return_type,
            body,
            is_async: false,
            is_extern: false,
            abi: None,
            modifiers,
            attributes,
            span: start_span,
        };

        Ok(self.arena.alloc(Item::FunctionDecl(func_decl)))
    }

    /// Parse async item with attributes
    fn parse_async_item_with_attributes(
        &mut self,
        modifiers: Modifiers,
        attributes: Vec<Attribute>,
    ) -> Result<&'arena Item, ParseError> {
        // Consume 'async'
        let async_token = self.tokens.consume();
        if !matches!(async_token.token_type, TokenType::Async) {
            return Err(ParseError::unexpected_token("'async'", &async_token));
        }

        // Must be followed by 'fn'
        if !matches!(self.tokens.peek().token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token(
                "'fn' after 'async'",
                self.tokens.peek(),
            ));
        }

        self.parse_async_function_declaration_with_attributes(modifiers, attributes)
    }

    /// Parse async function declaration with attributes
    fn parse_async_function_declaration_with_attributes(
        &mut self,
        modifiers: Modifiers,
        attributes: Vec<Attribute>,
    ) -> Result<&'arena Item, ParseError> {
        let start_span = self.current_span();

        // Consume 'fn' (we already consumed 'async')
        let fn_token = self.tokens.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        // Function name
        let name_token = self.tokens.consume();
        let name = match name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => return Err(ParseError::unexpected_token("function name", &name_token)),
        };

        // Parameters
        let parameters = self.parse_parameter_list()?;

        // Return type
        let return_type = if matches!(self.tokens.peek().token_type, TokenType::Arrow) {
            self.tokens.consume(); // consume '->'
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body
        let body = if matches!(self.tokens.peek().token_type, TokenType::LeftBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let func_decl = FunctionDecl {
            name,
            generics: None,
            parameters,
            return_type,
            body,
            is_async: true,
            is_extern: false,
            abi: None,
            modifiers,
            attributes,
            span: start_span,
        };

        Ok(self.arena.alloc(Item::FunctionDecl(func_decl)))
    }

    /// Parse static variable with attributes
    fn parse_static_variable_with_attributes(
        &mut self,
        _modifiers: Modifiers,
        _attributes: Vec<Attribute>,
    ) -> Result<&'arena Item, ParseError> {
        // For now, delegate to the existing static variable parser
        self.parse_static_variable()
    }

    /// Parse data class declaration with modifiers (legacy compatibility)
    fn parse_data_class_declaration_with_modifiers(
        &mut self,
        _modifiers: Modifiers,
    ) -> Result<&'arena Item, ParseError> {
        // For now, just call the existing method without attributes
        self.parse_data_class_declaration()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{TokenType, VecTokenStream};

    #[test]
    fn test_program_parser_creation() {
        let arena = Arena::new();
        let tokens = VecTokenStream::from_token_types(vec![TokenType::Eof]);
        let _parser = ProgramParser::new(&arena, tokens);
    }

    #[test]
    fn test_empty_compilation_unit() {
        let arena = Arena::new();
        let tokens = VecTokenStream::from_token_types(vec![TokenType::Eof]);
        let mut parser = ProgramParser::new(&arena, tokens);

        let result = parser.parse_compilation_unit();

        assert!(result.is_ok());
        let unit = result.unwrap();
        assert_eq!(unit.items.len(), 0);
    }

    #[test]
    fn test_program_with_diagnostics() {
        let arena = Arena::new();
        let tokens = VecTokenStream::from_token_types(vec![TokenType::Eof]);
        let mut parser = ProgramParser::new(&arena, tokens);

        let (program, report) = parser.parse_program_with_diagnostics();

        assert!(program.is_some());
        assert!(!report.has_errors());
    }

    #[test]
    fn test_invalid_top_level_item() {
        let arena = Arena::new();
        let tokens = VecTokenStream::from_token_types(vec![
            TokenType::Plus, // Invalid top-level token
            TokenType::Eof,
        ]);
        let mut parser = ProgramParser::new(&arena, tokens);

        let result = parser.parse_compilation_unit();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_collection() {
        let arena = Arena::new();
        let tokens = VecTokenStream::from_token_types(vec![
            TokenType::Plus, // Invalid
            TokenType::Star, // Invalid
            TokenType::Eof,
        ]);
        let mut parser = ProgramParser::new(&arena, tokens);

        let result = parser.parse_compilation_unit();
        assert!(result.is_err());
        assert!(parser.has_errors());
    }
}
