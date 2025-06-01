//! Program parser for complete Ferra programs

use crate::{
    ast::{
        Arena, Block, CompilationUnit, DataClassDecl, ExternBlock, Field, FunctionDecl, Item,
        Modifiers, Parameter, Type,
    },
    error::{parse_error::*, recovery::*},
    statement::StatementParser,
    token::{Span, Token, TokenStream, TokenType},
};

/// Top-level program parser that integrates all component parsers
pub struct ProgramParser<'arena, T: TokenStream + Clone> {
    arena: &'arena Arena,
    tokens: T,
    error_collector: ErrorCollector,
}

impl<'arena, T: TokenStream + Clone> ProgramParser<'arena, T> {
    /// Create a new program parser
    pub fn new(arena: &'arena Arena, tokens: T) -> Self {
        Self {
            arena,
            tokens,
            error_collector: ErrorCollector::new(50),
        }
    }

    /// Parse a complete compilation unit (top-level program)
    pub fn parse_compilation_unit(&mut self) -> Result<&'arena CompilationUnit, Vec<ParseError>> {
        let start_span = self.current_span();
        let mut items = Vec::new();

        // Parse top-level items until EOF
        while !self.tokens.is_at_end() {
            match self.parse_top_level_item() {
                Ok(item) => items.push(item.clone()),
                Err(error) => {
                    self.error_collector.add_error(error);

                    // Try to recover to next top-level item
                    if ErrorRecovery::smart_recovery(
                        &mut self.tokens,
                        "declaration",
                        &mut self.error_collector,
                    ).is_some() {
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

        if self.error_collector.has_errors() {
            Err(self.error_collector.get_errors().to_vec())
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
        let current = self.tokens.peek();

        match current.token_type {
            TokenType::Fn => self.parse_function_declaration(),
            TokenType::Data => self.parse_data_class_declaration(),
            TokenType::Extern => self.parse_extern_block(),
            TokenType::Pub => self.parse_public_item(),
            TokenType::Unsafe => self.parse_unsafe_item(),
            TokenType::Static => self.parse_static_variable(),
            _ => Err(ParseError::unexpected_token(
                "function, data class, extern block, or other top-level declaration",
                current,
            )),
        }
    }

    /// Parse a function declaration at the top level
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

        // ABI string
        let abi_token = self.consume();
        let abi = match abi_token.token_type {
            TokenType::StringLiteral(abi) => abi,
            _ => return Err(ParseError::unexpected_token("ABI string", &abi_token)),
        };

        // Extern items
        let items = self.parse_extern_item_list()?;

        let extern_block = ExternBlock {
            abi,
            items,
            span: start_span,
        };

        Ok(self.arena.alloc(Item::ExternBlock(extern_block)))
    }

    /// Parse a public item (pub fn, pub data, etc.)
    fn parse_public_item(&mut self) -> Result<&'arena Item, ParseError> {
        // Consume 'pub' keyword
        self.consume();

        // Parse the next item with pub modifier
        self.parse_top_level_item()
    }

    /// Parse an unsafe item
    fn parse_unsafe_item(&mut self) -> Result<&'arena Item, ParseError> {
        // Consume 'unsafe' keyword
        self.consume();

        // Parse the next item with unsafe modifier
        self.parse_top_level_item()
    }

    /// Parse a static variable declaration
    fn parse_static_variable(&mut self) -> Result<&'arena Item, ParseError> {
        // For now, delegate to StatementParser for static variables
        let mut statement_parser = StatementParser::new(self.arena, self.tokens.clone());
        statement_parser.parse_item()
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

        let colon_token = self.consume();
        if !matches!(colon_token.token_type, TokenType::Colon) {
            return Err(ParseError::unexpected_token("':'", &colon_token));
        }

        let param_type = self.parse_type()?;

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
            attributes: Vec::new(),
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
        let start_span = self.current_span();

        let open_brace = self.consume();
        if !matches!(open_brace.token_type, TokenType::LeftBrace) {
            return Err(ParseError::unexpected_token("'{'", &open_brace));
        }

        // For now, just consume tokens until closing brace
        // In a real implementation, we'd parse statements
        let statements = Vec::new();

        while !matches!(self.tokens.peek().token_type, TokenType::RightBrace) {
            // Skip tokens for now - in real implementation we'd parse statements
            self.consume();
        }

        let close_brace = self.consume();
        if !matches!(close_brace.token_type, TokenType::RightBrace) {
            return Err(ParseError::unexpected_token("'}'", &close_brace));
        }

        Ok(Block {
            statements,
            is_braced: true,
            span: start_span,
            scope_depth: 0,
            is_unsafe: false,
            is_async: false,
            is_try: false,
            label: None,
        })
    }

    /// Get the current token span
    fn current_span(&self) -> Span {
        self.tokens.peek().span.clone()
    }

    /// Consume the current token
    fn consume(&mut self) -> Token {
        self.tokens.consume()
    }

    /// Get collected errors
    pub fn get_errors(&self) -> &[ParseError] {
        self.error_collector.get_errors()
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.error_collector.has_errors()
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
