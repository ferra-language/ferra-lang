//! Type expression parsing implementation
//!
//! Comprehensive type parsing for Ferra language Phase 2.7

use crate::{
    ast::{FunctionType, PointerType, Type},
    error::{ParseError, ParseResult},
    token::{TokenStream, TokenType},
};

/// Parse a type expression with full Phase 2.7 support
pub fn parse_type<T: TokenStream>(tokens: &mut T) -> ParseResult<Type> {
    let mut parser = TypeParser::new(tokens);
    parser.parse_type()
}

/// Parse a simple type (identifier)
pub fn parse_simple_type<T: TokenStream>(tokens: &mut T) -> ParseResult<Type> {
    let mut parser = TypeParser::new(tokens);
    parser.parse_simple_type()
}

/// Parse a tuple type
pub fn parse_tuple_type<T: TokenStream>(tokens: &mut T) -> ParseResult<Type> {
    let mut parser = TypeParser::new(tokens);
    parser.parse_tuple_type()
}

/// Parse an array type
pub fn parse_array_type<T: TokenStream>(tokens: &mut T) -> ParseResult<Type> {
    let mut parser = TypeParser::new(tokens);
    parser.parse_array_type()
}

/// Parse a function type
pub fn parse_function_type<T: TokenStream>(tokens: &mut T) -> ParseResult<Type> {
    let mut parser = TypeParser::new(tokens);
    parser.parse_function_type()
}

/// Comprehensive type parser for Phase 2.7
struct TypeParser<'a, T: TokenStream> {
    tokens: &'a mut T,
}

impl<'a, T: TokenStream> TypeParser<'a, T> {
    fn new(tokens: &'a mut T) -> Self {
        Self { tokens }
    }

    /// Parse any type expression
    fn parse_type(&mut self) -> ParseResult<Type> {
        let current = self.tokens.peek();

        match &current.token_type {
            // Function types: fn(T) -> T or extern "C" fn(T) -> T
            TokenType::Fn => self.parse_function_type(),
            TokenType::Extern => self.parse_extern_function_type(),

            // Pointer types: *const T or *mut T
            TokenType::Star => self.parse_pointer_type(),

            // Tuple types: (T, T, ...)
            TokenType::LeftParen => self.parse_tuple_type(),

            // Array types: [T]
            TokenType::LeftBracket => self.parse_array_type(),

            // Simple identifier types
            TokenType::Identifier(_) => self.parse_identifier_type(),

            _ => Err(ParseError::unexpected_token("type", current)),
        }
    }

    /// Parse simple identifier type or qualified identifier
    fn parse_identifier_type(&mut self) -> ParseResult<Type> {
        let token = self.tokens.consume();
        match token.token_type {
            TokenType::Identifier(name) => {
                // Check for generic type parameters: Name<T>
                if matches!(self.tokens.peek().token_type, TokenType::Less) {
                    self.parse_generic_type(name)
                } else {
                    Ok(Type::Identifier(name))
                }
            }
            _ => Err(ParseError::unexpected_token("identifier", &token)),
        }
    }

    /// Parse generic type: Name<T, U, ...>
    fn parse_generic_type(&mut self, base_name: String) -> ParseResult<Type> {
        // For now, we'll represent generic types as qualified identifiers
        // In future phases, we'll have a proper Generic variant
        let _open_bracket = self.tokens.consume(); // consume '<'

        let mut type_args = Vec::new();

        loop {
            if matches!(self.tokens.peek().token_type, TokenType::Greater) {
                break;
            }

            let type_arg = self.parse_type()?;
            type_args.push(type_arg);

            match self.tokens.peek().token_type {
                TokenType::Comma => {
                    self.tokens.consume(); // consume ','
                }
                TokenType::Greater => break,
                _ => {
                    return Err(ParseError::unexpected_token(
                        "',' or '>'",
                        self.tokens.peek(),
                    ))
                }
            }
        }

        let _close_bracket = self.tokens.consume(); // consume '>'

        // For now, represent as a structured identifier
        // Future phases will have proper generic type support
        Ok(Type::Identifier(format!(
            "{}<{}>",
            base_name,
            type_args.len()
        )))
    }

    /// Parse simple type (just identifier)
    fn parse_simple_type(&mut self) -> ParseResult<Type> {
        let token = self.tokens.consume();
        match token.token_type {
            TokenType::Identifier(name) => Ok(Type::Identifier(name)),
            _ => Err(ParseError::unexpected_token("identifier", &token)),
        }
    }

    /// Parse tuple type: (T, T, ...)
    fn parse_tuple_type(&mut self) -> ParseResult<Type> {
        let open_paren = self.tokens.consume();
        if !matches!(open_paren.token_type, TokenType::LeftParen) {
            return Err(ParseError::unexpected_token("'('", &open_paren));
        }

        let mut types = Vec::new();

        // Handle empty tuple: ()
        if matches!(self.tokens.peek().token_type, TokenType::RightParen) {
            self.tokens.consume(); // consume ')'
            return Ok(Type::Tuple(types));
        }

        // Parse type list
        loop {
            let type_expr = self.parse_type()?;
            types.push(type_expr);

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

        let close_paren = self.tokens.consume();
        if !matches!(close_paren.token_type, TokenType::RightParen) {
            return Err(ParseError::unexpected_token("')'", &close_paren));
        }

        Ok(Type::Tuple(types))
    }

    /// Parse array type: [T]
    fn parse_array_type(&mut self) -> ParseResult<Type> {
        let open_bracket = self.tokens.consume();
        if !matches!(open_bracket.token_type, TokenType::LeftBracket) {
            return Err(ParseError::unexpected_token("'['", &open_bracket));
        }

        let element_type = self.parse_type()?;

        let close_bracket = self.tokens.consume();
        if !matches!(close_bracket.token_type, TokenType::RightBracket) {
            return Err(ParseError::unexpected_token("']'", &close_bracket));
        }

        Ok(Type::Array(Box::new(element_type)))
    }

    /// Parse function type: fn(T, U) -> V
    fn parse_function_type(&mut self) -> ParseResult<Type> {
        let fn_token = self.tokens.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        // Parse parameter types
        let open_paren = self.tokens.consume();
        if !matches!(open_paren.token_type, TokenType::LeftParen) {
            return Err(ParseError::unexpected_token("'('", &open_paren));
        }

        let mut parameters = Vec::new();

        // Handle empty parameter list
        if !matches!(self.tokens.peek().token_type, TokenType::RightParen) {
            loop {
                let param_type = self.parse_type()?;
                parameters.push(param_type);

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
        }

        let close_paren = self.tokens.consume();
        if !matches!(close_paren.token_type, TokenType::RightParen) {
            return Err(ParseError::unexpected_token("')'", &close_paren));
        }

        // Parse return type (if present)
        let return_type = if matches!(self.tokens.peek().token_type, TokenType::Arrow) {
            self.tokens.consume(); // consume '->'
            Box::new(self.parse_type()?)
        } else {
            // Default to unit type if no return type specified
            Box::new(Type::Tuple(Vec::new()))
        };

        Ok(Type::Function(FunctionType {
            parameters,
            return_type,
            is_extern: false,
            abi: None,
        }))
    }

    /// Parse extern function type: extern "C" fn(T) -> U
    fn parse_extern_function_type(&mut self) -> ParseResult<Type> {
        let extern_token = self.tokens.consume();
        if !matches!(extern_token.token_type, TokenType::Extern) {
            return Err(ParseError::unexpected_token("'extern'", &extern_token));
        }

        // Parse ABI string
        let abi = if let TokenType::StringLiteral(abi_string) = &self.tokens.peek().token_type {
            let abi_string = abi_string.clone();
            self.tokens.consume(); // consume ABI string
            Some(abi_string)
        } else {
            None
        };

        // Parse 'fn'
        let fn_token = self.tokens.consume();
        if !matches!(fn_token.token_type, TokenType::Fn) {
            return Err(ParseError::unexpected_token("'fn'", &fn_token));
        }

        // Parse parameter types
        let open_paren = self.tokens.consume();
        if !matches!(open_paren.token_type, TokenType::LeftParen) {
            return Err(ParseError::unexpected_token("'('", &open_paren));
        }

        let mut parameters = Vec::new();

        // Handle empty parameter list
        if !matches!(self.tokens.peek().token_type, TokenType::RightParen) {
            loop {
                let param_type = self.parse_type()?;
                parameters.push(param_type);

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
        }

        let close_paren = self.tokens.consume();
        if !matches!(close_paren.token_type, TokenType::RightParen) {
            return Err(ParseError::unexpected_token("')'", &close_paren));
        }

        // Parse return type (if present)
        let return_type = if matches!(self.tokens.peek().token_type, TokenType::Arrow) {
            self.tokens.consume(); // consume '->'
            Box::new(self.parse_type()?)
        } else {
            // Default to unit type if no return type specified
            Box::new(Type::Tuple(Vec::new()))
        };

        Ok(Type::Function(FunctionType {
            parameters,
            return_type,
            is_extern: true,
            abi,
        }))
    }

    /// Parse pointer type: *Type (simplified without const/mut for now)
    fn parse_pointer_type(&mut self) -> ParseResult<Type> {
        let star_token = self.tokens.consume();
        if !matches!(star_token.token_type, TokenType::Star) {
            return Err(ParseError::unexpected_token("'*'", &star_token));
        }

        let target_type = self.parse_type()?;

        // For now, default to mutable pointers since we don't have const/mut tokens
        Ok(Type::Pointer(PointerType {
            target: Box::new(target_type),
            is_mutable: true, // Default to mutable for now
        }))
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
    fn test_simple_identifier_type() {
        let mut tokens = create_token_stream(vec![TokenType::Identifier("int".to_string())]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Identifier(name) => assert_eq!(name, "int"),
            _ => panic!("Expected identifier type"),
        }
    }

    #[test]
    fn test_tuple_type() {
        let mut tokens = create_token_stream(vec![
            TokenType::LeftParen,
            TokenType::Identifier("int".to_string()),
            TokenType::Comma,
            TokenType::Identifier("string".to_string()),
            TokenType::RightParen,
        ]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Tuple(types) => {
                assert_eq!(types.len(), 2);
                match (&types[0], &types[1]) {
                    (Type::Identifier(t1), Type::Identifier(t2)) => {
                        assert_eq!(t1, "int");
                        assert_eq!(t2, "string");
                    }
                    _ => panic!("Expected identifier types in tuple"),
                }
            }
            _ => panic!("Expected tuple type"),
        }
    }

    #[test]
    fn test_empty_tuple_type() {
        let mut tokens = create_token_stream(vec![TokenType::LeftParen, TokenType::RightParen]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Tuple(types) => assert_eq!(types.len(), 0),
            _ => panic!("Expected empty tuple type"),
        }
    }

    #[test]
    fn test_array_type() {
        let mut tokens = create_token_stream(vec![
            TokenType::LeftBracket,
            TokenType::Identifier("int".to_string()),
            TokenType::RightBracket,
        ]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Array(element_type) => match element_type.as_ref() {
                Type::Identifier(name) => assert_eq!(name, "int"),
                _ => panic!("Expected identifier type in array"),
            },
            _ => panic!("Expected array type"),
        }
    }

    #[test]
    fn test_function_type() {
        let mut tokens = create_token_stream(vec![
            TokenType::Fn,
            TokenType::LeftParen,
            TokenType::Identifier("int".to_string()),
            TokenType::Comma,
            TokenType::Identifier("string".to_string()),
            TokenType::RightParen,
            TokenType::Arrow,
            TokenType::Identifier("bool".to_string()),
        ]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Function(func_type) => {
                assert_eq!(func_type.parameters.len(), 2);
                assert!(!func_type.is_extern);
                assert!(func_type.abi.is_none());

                match func_type.return_type.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "bool"),
                    _ => panic!("Expected bool return type"),
                }
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_extern_function_type() {
        let mut tokens = create_token_stream(vec![
            TokenType::Extern,
            TokenType::StringLiteral("C".to_string()),
            TokenType::Fn,
            TokenType::LeftParen,
            TokenType::Identifier("int".to_string()),
            TokenType::RightParen,
            TokenType::Arrow,
            TokenType::Identifier("void".to_string()),
        ]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Function(func_type) => {
                assert_eq!(func_type.parameters.len(), 1);
                assert!(func_type.is_extern);
                assert_eq!(func_type.abi, Some("C".to_string()));
            }
            _ => panic!("Expected extern function type"),
        }
    }

    #[test]
    fn test_pointer_type() {
        let mut tokens = create_token_stream(vec![
            TokenType::Star,
            TokenType::Identifier("int".to_string()),
        ]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Pointer(ptr_type) => {
                assert!(ptr_type.is_mutable); // Default to mutable for now
                match ptr_type.target.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "int"),
                    _ => panic!("Expected int pointer target"),
                }
            }
            _ => panic!("Expected pointer type"),
        }
    }

    #[test]
    fn test_mutable_pointer_type() {
        let mut tokens = create_token_stream(vec![
            TokenType::Star,
            TokenType::Identifier("int".to_string()),
        ]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Pointer(ptr_type) => {
                assert!(ptr_type.is_mutable);
                match ptr_type.target.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "int"),
                    _ => panic!("Expected int pointer target"),
                }
            }
            _ => panic!("Expected mutable pointer type"),
        }
    }

    #[test]
    fn test_nested_array_type() {
        let mut tokens = create_token_stream(vec![
            TokenType::LeftBracket,
            TokenType::LeftBracket,
            TokenType::Identifier("int".to_string()),
            TokenType::RightBracket,
            TokenType::RightBracket,
        ]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Array(outer_element) => match outer_element.as_ref() {
                Type::Array(inner_element) => match inner_element.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "int"),
                    _ => panic!("Expected int in nested array"),
                },
                _ => panic!("Expected nested array"),
            },
            _ => panic!("Expected array type"),
        }
    }

    #[test]
    fn test_complex_function_type() {
        let mut tokens = create_token_stream(vec![
            TokenType::Fn,
            TokenType::LeftParen,
            TokenType::LeftBracket,
            TokenType::Identifier("int".to_string()),
            TokenType::RightBracket,
            TokenType::Comma,
            TokenType::LeftParen,
            TokenType::Identifier("string".to_string()),
            TokenType::Comma,
            TokenType::Identifier("bool".to_string()),
            TokenType::RightParen,
            TokenType::RightParen,
            TokenType::Arrow,
            TokenType::Star,
            TokenType::Identifier("char".to_string()),
        ]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Function(func_type) => {
                assert_eq!(func_type.parameters.len(), 2);

                // First parameter: [int]
                match &func_type.parameters[0] {
                    Type::Array(elem) => match elem.as_ref() {
                        Type::Identifier(name) => assert_eq!(name, "int"),
                        _ => panic!("Expected int array"),
                    },
                    _ => panic!("Expected array parameter"),
                }

                // Second parameter: (string, bool)
                match &func_type.parameters[1] {
                    Type::Tuple(types) => {
                        assert_eq!(types.len(), 2);
                    }
                    _ => panic!("Expected tuple parameter"),
                }

                // Return type: *char (simplified)
                match func_type.return_type.as_ref() {
                    Type::Pointer(ptr) => {
                        assert!(ptr.is_mutable); // Default to mutable
                        match ptr.target.as_ref() {
                            Type::Identifier(name) => assert_eq!(name, "char"),
                            _ => panic!("Expected char pointer"),
                        }
                    }
                    _ => panic!("Expected pointer return type"),
                }
            }
            _ => panic!("Expected function type"),
        }
    }
}
