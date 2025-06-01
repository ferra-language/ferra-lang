//! Generic type parameter parser implementation
//!
//! Parses generic syntax including:
//! - Type parameters: `<T, U>`
//! - Lifetime parameters: `<'a, 'b>`
//! - Type constraints: `<T: Clone + Debug>`
//! - Where clauses: `where T: Clone + Debug, U: Default`

use crate::ast::{
    GenericParam, GenericParams, GenericType, Type, TypeBound, WhereClause, WhereConstraint,
};
use crate::error::{ParseError, ParseResult};
use crate::token::{Span, Token, TokenStream, TokenType};

/// Parse generic parameters `<T, U>` or `<T: Clone + Debug, U: Default>`
pub fn parse_generic_params<T: TokenStream>(tokens: &mut T) -> ParseResult<Option<GenericParams>> {
    let mut parser = GenericParser::new(tokens);
    parser.parse_generic_params()
}

/// Parse a generic type instantiation like `Vec<T>` or `HashMap<K, V>`
pub fn parse_generic_type<T: TokenStream>(
    tokens: &mut T,
    base_name: String,
) -> ParseResult<GenericType> {
    let mut parser = GenericParser::new(tokens);
    parser.parse_generic_type(base_name)
}

struct GenericParser<'a, T: TokenStream> {
    tokens: &'a mut T,
}

impl<'a, T: TokenStream> GenericParser<'a, T> {
    fn new(tokens: &'a mut T) -> Self {
        Self { tokens }
    }

    fn parse_generic_params(&mut self) -> ParseResult<Option<GenericParams>> {
        if !matches!(self.peek().token_type, TokenType::Less) {
            return Ok(None);
        }

        let start_span = self.consume().span; // consume '<'
        let mut params = Vec::new();

        // Handle empty generic parameters: <>
        if matches!(self.peek().token_type, TokenType::Greater) {
            let end_span = self.consume().span; // consume '>'
            return Ok(Some(GenericParams {
                params,
                where_clause: None,
                span: start_span.combine(end_span),
            }));
        }

        // Parse first parameter
        params.push(self.parse_generic_param()?);

        // Parse additional parameters
        while matches!(self.peek().token_type, TokenType::Comma) {
            self.consume(); // consume ','

            // Allow trailing comma
            if matches!(self.peek().token_type, TokenType::Greater) {
                break;
            }

            params.push(self.parse_generic_param()?);
        }

        if !matches!(self.peek().token_type, TokenType::Greater) {
            return Err(ParseError::unexpected_token(">", &self.peek()));
        }

        let end_span = self.consume().span; // consume '>'

        // Check for where clause
        let where_clause = if matches!(self.peek().token_type, TokenType::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        let final_span = if let Some(ref wc) = where_clause {
            start_span.combine(wc.span.clone())
        } else {
            start_span.combine(end_span)
        };

        Ok(Some(GenericParams {
            params,
            where_clause,
            span: final_span,
        }))
    }

    fn parse_generic_param(&mut self) -> ParseResult<GenericParam> {
        let token = self.peek();
        let start_span = token.span.clone();

        match token.token_type {
            TokenType::Apostrophe => {
                // Lifetime parameter: 'a, 'static
                self.consume(); // consume "'"

                if let TokenType::Identifier(name) = &self.peek().token_type {
                    let name = name.clone();
                    let end_span = self.consume().span;

                    Ok(GenericParam {
                        name: format!("'{}", name),
                        bounds: Vec::new(),
                        default: None,
                        is_lifetime: true,
                        span: start_span.combine(end_span),
                    })
                } else {
                    Err(ParseError::unexpected_token("lifetime name", &self.peek()))
                }
            }
            TokenType::Identifier(name) => {
                // Type parameter: T, U, T: Clone + Debug
                let name = name.clone();
                let mut end_span = self.consume().span;
                let mut bounds = Vec::new();
                let mut default = None;

                // Check for bounds: T: Clone + Debug
                if matches!(self.peek().token_type, TokenType::Colon) {
                    self.consume(); // consume ':'
                    bounds = self.parse_type_bounds()?;

                    if let Some(last_bound) = bounds.last() {
                        end_span = last_bound.span.clone();
                    }
                }

                // Check for default type: T = DefaultType
                if matches!(self.peek().token_type, TokenType::Equal) {
                    self.consume(); // consume '='
                    let default_type = self.parse_type()?;
                    end_span = default_type.span();
                    default = Some(default_type);
                }

                Ok(GenericParam {
                    name,
                    bounds,
                    default,
                    is_lifetime: false,
                    span: start_span.combine(end_span),
                })
            }
            _ => Err(ParseError::unexpected_token("type", &self.peek())),
        }
    }

    fn parse_type_bounds(&mut self) -> ParseResult<Vec<TypeBound>> {
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

    fn parse_type_bound(&mut self) -> ParseResult<TypeBound> {
        if let TokenType::Identifier(trait_name) = &self.peek().token_type {
            let trait_name = trait_name.clone();
            let _span = self.consume().span;

            Ok(TypeBound {
                trait_name,
                span: _span,
            })
        } else {
            Err(ParseError::unexpected_token("trait name", &self.peek()))
        }
    }

    fn parse_where_clause(&mut self) -> ParseResult<WhereClause> {
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

        Ok(WhereClause {
            constraints,
            span: start_span.combine(end_span),
        })
    }

    fn parse_where_constraint(&mut self) -> ParseResult<WhereConstraint> {
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

            Ok(WhereConstraint {
                type_name,
                bounds,
                span: start_span.combine(end_span),
            })
        } else {
            Err(ParseError::unexpected_token("type name", &self.peek()))
        }
    }

    fn parse_generic_type(&mut self, base_name: String) -> ParseResult<GenericType> {
        let start_span = self.current_span();

        if !matches!(self.peek().token_type, TokenType::Less) {
            return Err(ParseError::unexpected_token("<", &self.peek()));
        }

        self.consume(); // consume '<'
        let mut args = Vec::new();

        // Handle empty type arguments: HashMap<>
        if matches!(self.peek().token_type, TokenType::Greater) {
            let end_span = self.consume().span;
            return Ok(GenericType {
                base: base_name,
                args,
                span: start_span.combine(end_span),
            });
        }

        // Parse first type argument
        args.push(self.parse_type()?);

        // Parse additional type arguments
        while matches!(self.peek().token_type, TokenType::Comma) {
            self.consume(); // consume ','

            // Allow trailing comma
            if matches!(self.peek().token_type, TokenType::Greater) {
                break;
            }

            args.push(self.parse_type()?);
        }

        if !matches!(self.peek().token_type, TokenType::Greater) {
            return Err(ParseError::unexpected_token(">", &self.peek()));
        }

        let end_span = self.consume().span;

        Ok(GenericType {
            base: base_name,
            args,
            span: start_span.combine(end_span),
        })
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        // This is a simplified type parser for generic contexts
        // It will delegate to the main type parser
        match &self.peek().token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                let _span = self.consume().span;

                // Check if this is a generic type instantiation
                if matches!(self.peek().token_type, TokenType::Less) {
                    let generic_type = self.parse_generic_type(name)?;
                    Ok(Type::Generic(generic_type))
                } else {
                    Ok(Type::Identifier(name))
                }
            }
            _ => Err(ParseError::unexpected_token("type", &self.peek())),
        }
    }

    fn is_where_clause_end(&self) -> bool {
        matches!(
            self.peek().token_type,
            TokenType::LeftBrace | TokenType::Semicolon | TokenType::Eof | TokenType::Newline
        )
    }

    fn peek(&self) -> Token {
        self.tokens.peek().clone()
    }

    fn consume(&mut self) -> Token {
        self.tokens.consume()
    }

    fn current_span(&self) -> Span {
        self.peek().span
    }
}

// Add span method to Type enum
impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Identifier(_) => Span::dummy(), // Would need actual span tracking
            Type::Generic(gt) => gt.span.clone(),
            Type::Tuple(_) => Span::dummy(),
            Type::Array(_) => Span::dummy(),
            Type::Function(_) => Span::dummy(),
            Type::Pointer(_) => Span::dummy(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::stream::VecTokenStream;

    fn create_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
        let tokens = token_types.into_iter().map(|t| Token::dummy(t)).collect();
        VecTokenStream::new(tokens)
    }

    #[test]
    fn test_simple_generic_params() {
        let mut tokens = create_token_stream(vec![
            TokenType::Less,
            TokenType::Identifier("T".to_string()),
            TokenType::Greater,
        ]);

        let result = parse_generic_params(&mut tokens).unwrap();
        assert!(result.is_some());

        let generics = result.unwrap();
        assert_eq!(generics.params.len(), 1);
        assert_eq!(generics.params[0].name, "T");
        assert!(!generics.params[0].is_lifetime);
        assert!(generics.params[0].bounds.is_empty());
    }

    #[test]
    fn test_multiple_generic_params() {
        let mut tokens = create_token_stream(vec![
            TokenType::Less,
            TokenType::Identifier("T".to_string()),
            TokenType::Comma,
            TokenType::Identifier("U".to_string()),
            TokenType::Greater,
        ]);

        let result = parse_generic_params(&mut tokens).unwrap();
        assert!(result.is_some());

        let generics = result.unwrap();
        assert_eq!(generics.params.len(), 2);
        assert_eq!(generics.params[0].name, "T");
        assert_eq!(generics.params[1].name, "U");
    }

    #[test]
    fn test_lifetime_params() {
        let mut tokens = create_token_stream(vec![
            TokenType::Less,
            TokenType::Apostrophe,
            TokenType::Identifier("a".to_string()),
            TokenType::Comma,
            TokenType::Apostrophe,
            TokenType::Identifier("b".to_string()),
            TokenType::Greater,
        ]);

        let result = parse_generic_params(&mut tokens).unwrap();
        assert!(result.is_some());

        let generics = result.unwrap();
        assert_eq!(generics.params.len(), 2);
        assert_eq!(generics.params[0].name, "'a");
        assert_eq!(generics.params[1].name, "'b");
        assert!(generics.params[0].is_lifetime);
        assert!(generics.params[1].is_lifetime);
    }

    #[test]
    fn test_type_bounds() {
        let mut tokens = create_token_stream(vec![
            TokenType::Less,
            TokenType::Identifier("T".to_string()),
            TokenType::Colon,
            TokenType::Identifier("Clone".to_string()),
            TokenType::Plus,
            TokenType::Identifier("Debug".to_string()),
            TokenType::Greater,
        ]);

        let result = parse_generic_params(&mut tokens).unwrap();
        assert!(result.is_some());

        let generics = result.unwrap();
        assert_eq!(generics.params.len(), 1);
        assert_eq!(generics.params[0].bounds.len(), 2);
        assert_eq!(generics.params[0].bounds[0].trait_name, "Clone");
        assert_eq!(generics.params[0].bounds[1].trait_name, "Debug");
    }

    #[test]
    fn test_where_clause() {
        let mut tokens = create_token_stream(vec![
            TokenType::Less,
            TokenType::Identifier("T".to_string()),
            TokenType::Greater,
            TokenType::Where,
            TokenType::Identifier("T".to_string()),
            TokenType::Colon,
            TokenType::Identifier("Clone".to_string()),
            TokenType::Plus,
            TokenType::Identifier("Debug".to_string()),
        ]);

        let result = parse_generic_params(&mut tokens).unwrap();
        assert!(result.is_some());

        let generics = result.unwrap();
        assert!(generics.where_clause.is_some());

        let where_clause = generics.where_clause.unwrap();
        assert_eq!(where_clause.constraints.len(), 1);
        assert_eq!(where_clause.constraints[0].type_name, "T");
        assert_eq!(where_clause.constraints[0].bounds.len(), 2);
    }

    #[test]
    fn test_generic_type_instantiation() {
        let mut tokens = create_token_stream(vec![
            TokenType::Less,
            TokenType::Identifier("i32".to_string()),
            TokenType::Greater,
        ]);

        let result = parse_generic_type(&mut tokens, "Vec".to_string()).unwrap();
        assert_eq!(result.base, "Vec");
        assert_eq!(result.args.len(), 1);

        if let Type::Identifier(name) = &result.args[0] {
            assert_eq!(name, "i32");
        } else {
            panic!("Expected identifier type");
        }
    }

    #[test]
    fn test_nested_generic_types() {
        let mut tokens = create_token_stream(vec![
            TokenType::Less,
            TokenType::Identifier("Vec".to_string()),
            TokenType::Less,
            TokenType::Identifier("i32".to_string()),
            TokenType::Greater,
            TokenType::Greater,
        ]);

        let result = parse_generic_type(&mut tokens, "Option".to_string()).unwrap();
        assert_eq!(result.base, "Option");
        assert_eq!(result.args.len(), 1);

        if let Type::Generic(inner) = &result.args[0] {
            assert_eq!(inner.base, "Vec");
            assert_eq!(inner.args.len(), 1);
        } else {
            panic!("Expected generic type");
        }
    }

    #[test]
    fn test_empty_generic_params() {
        let mut tokens = create_token_stream(vec![TokenType::Less, TokenType::Greater]);

        let result = parse_generic_params(&mut tokens).unwrap();
        assert!(result.is_some());

        let generics = result.unwrap();
        assert_eq!(generics.params.len(), 0);
    }

    #[test]
    fn test_trailing_comma_in_generics() {
        let mut tokens = create_token_stream(vec![
            TokenType::Less,
            TokenType::Identifier("T".to_string()),
            TokenType::Comma,
            TokenType::Identifier("U".to_string()),
            TokenType::Comma,
            TokenType::Greater,
        ]);

        let result = parse_generic_params(&mut tokens).unwrap();
        assert!(result.is_some());

        let generics = result.unwrap();
        assert_eq!(generics.params.len(), 2);
    }

    #[test]
    fn test_complex_where_clause() {
        let mut tokens = create_token_stream(vec![
            TokenType::Less,
            TokenType::Identifier("T".to_string()),
            TokenType::Comma,
            TokenType::Identifier("U".to_string()),
            TokenType::Greater,
            TokenType::Where,
            TokenType::Identifier("T".to_string()),
            TokenType::Colon,
            TokenType::Identifier("Clone".to_string()),
            TokenType::Comma,
            TokenType::Identifier("U".to_string()),
            TokenType::Colon,
            TokenType::Identifier("Default".to_string()),
        ]);

        let result = parse_generic_params(&mut tokens).unwrap();
        assert!(result.is_some());

        let generics = result.unwrap();
        assert!(generics.where_clause.is_some());

        let where_clause = generics.where_clause.unwrap();
        assert_eq!(where_clause.constraints.len(), 2);
        assert_eq!(where_clause.constraints[0].type_name, "T");
        assert_eq!(where_clause.constraints[1].type_name, "U");
    }
}
