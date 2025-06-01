//! Phase 2.8.2: Generic Type Parameter Tests
//!
//! Tests for generic type parameter parsing including:
//! - Type parameters: `<T, U>`
//! - Lifetime parameters: `<'a, 'b>`
//! - Type constraints: `<T: Clone + Debug>`
//! - Where clauses: `where T: Clone + Debug, U: Default`
//! - Generic type instantiations: `Vec<T>`, `HashMap<K, V>`
//! - Integration with function and data class declarations

use ferra_parser::{
    ast::{Arena, Item, Type},
    generic::parser::{parse_generic_params, parse_generic_type},
    statement::StatementParser,
    token::stream::VecTokenStream,
    token::{Token, TokenType},
};

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
    assert!(generics.where_clause.is_none());
}

#[test]
fn test_multiple_generic_params() {
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Comma,
        TokenType::Identifier("U".to_string()),
        TokenType::Comma,
        TokenType::Identifier("V".to_string()),
        TokenType::Greater,
    ]);

    let result = parse_generic_params(&mut tokens).unwrap();
    assert!(result.is_some());

    let generics = result.unwrap();
    assert_eq!(generics.params.len(), 3);
    assert_eq!(generics.params[0].name, "T");
    assert_eq!(generics.params[1].name, "U");
    assert_eq!(generics.params[2].name, "V");
}

#[test]
fn test_lifetime_parameters() {
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Apostrophe,
        TokenType::Identifier("a".to_string()),
        TokenType::Comma,
        TokenType::Apostrophe,
        TokenType::Identifier("static".to_string()),
        TokenType::Greater,
    ]);

    let result = parse_generic_params(&mut tokens).unwrap();
    assert!(result.is_some());

    let generics = result.unwrap();
    assert_eq!(generics.params.len(), 2);
    assert_eq!(generics.params[0].name, "'a");
    assert_eq!(generics.params[1].name, "'static");
    assert!(generics.params[0].is_lifetime);
    assert!(generics.params[1].is_lifetime);
}

#[test]
fn test_mixed_type_and_lifetime_params() {
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Apostrophe,
        TokenType::Identifier("a".to_string()),
        TokenType::Comma,
        TokenType::Identifier("T".to_string()),
        TokenType::Comma,
        TokenType::Apostrophe,
        TokenType::Identifier("b".to_string()),
        TokenType::Greater,
    ]);

    let result = parse_generic_params(&mut tokens).unwrap();
    assert!(result.is_some());

    let generics = result.unwrap();
    assert_eq!(generics.params.len(), 3);
    assert_eq!(generics.params[0].name, "'a");
    assert!(generics.params[0].is_lifetime);
    assert_eq!(generics.params[1].name, "T");
    assert!(!generics.params[1].is_lifetime);
    assert_eq!(generics.params[2].name, "'b");
    assert!(generics.params[2].is_lifetime);
}

#[test]
fn test_type_bounds_single_trait() {
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Colon,
        TokenType::Identifier("Clone".to_string()),
        TokenType::Greater,
    ]);

    let result = parse_generic_params(&mut tokens).unwrap();
    assert!(result.is_some());

    let generics = result.unwrap();
    assert_eq!(generics.params.len(), 1);
    assert_eq!(generics.params[0].bounds.len(), 1);
    assert_eq!(generics.params[0].bounds[0].trait_name, "Clone");
}

#[test]
fn test_type_bounds_multiple_traits() {
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Colon,
        TokenType::Identifier("Clone".to_string()),
        TokenType::Plus,
        TokenType::Identifier("Debug".to_string()),
        TokenType::Plus,
        TokenType::Identifier("Send".to_string()),
        TokenType::Greater,
    ]);

    let result = parse_generic_params(&mut tokens).unwrap();
    assert!(result.is_some());

    let generics = result.unwrap();
    assert_eq!(generics.params.len(), 1);
    assert_eq!(generics.params[0].bounds.len(), 3);
    assert_eq!(generics.params[0].bounds[0].trait_name, "Clone");
    assert_eq!(generics.params[0].bounds[1].trait_name, "Debug");
    assert_eq!(generics.params[0].bounds[2].trait_name, "Send");
}

#[test]
fn test_complex_mixed_constraints() {
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Colon,
        TokenType::Identifier("Clone".to_string()),
        TokenType::Plus,
        TokenType::Identifier("Debug".to_string()),
        TokenType::Comma,
        TokenType::Identifier("U".to_string()),
        TokenType::Colon,
        TokenType::Identifier("Default".to_string()),
        TokenType::Greater,
    ]);

    let result = parse_generic_params(&mut tokens).unwrap();
    assert!(result.is_some());

    let generics = result.unwrap();
    assert_eq!(generics.params.len(), 2);

    // First parameter: T: Clone + Debug
    assert_eq!(generics.params[0].name, "T");
    assert_eq!(generics.params[0].bounds.len(), 2);
    assert_eq!(generics.params[0].bounds[0].trait_name, "Clone");
    assert_eq!(generics.params[0].bounds[1].trait_name, "Debug");

    // Second parameter: U: Default
    assert_eq!(generics.params[1].name, "U");
    assert_eq!(generics.params[1].bounds.len(), 1);
    assert_eq!(generics.params[1].bounds[0].trait_name, "Default");
}

#[test]
fn test_where_clause_simple() {
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Greater,
        TokenType::Where,
        TokenType::Identifier("T".to_string()),
        TokenType::Colon,
        TokenType::Identifier("Clone".to_string()),
    ]);

    let result = parse_generic_params(&mut tokens).unwrap();
    assert!(result.is_some());

    let generics = result.unwrap();
    assert!(generics.where_clause.is_some());

    let where_clause = generics.where_clause.unwrap();
    assert_eq!(where_clause.constraints.len(), 1);
    assert_eq!(where_clause.constraints[0].type_name, "T");
    assert_eq!(where_clause.constraints[0].bounds.len(), 1);
    assert_eq!(where_clause.constraints[0].bounds[0].trait_name, "Clone");
}

#[test]
fn test_where_clause_complex() {
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
        TokenType::Plus,
        TokenType::Identifier("Debug".to_string()),
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

    // First constraint: T: Clone + Debug
    assert_eq!(where_clause.constraints[0].type_name, "T");
    assert_eq!(where_clause.constraints[0].bounds.len(), 2);
    assert_eq!(where_clause.constraints[0].bounds[0].trait_name, "Clone");
    assert_eq!(where_clause.constraints[0].bounds[1].trait_name, "Debug");

    // Second constraint: U: Default
    assert_eq!(where_clause.constraints[1].type_name, "U");
    assert_eq!(where_clause.constraints[1].bounds.len(), 1);
    assert_eq!(where_clause.constraints[1].bounds[0].trait_name, "Default");
}

#[test]
fn test_generic_type_instantiation_simple() {
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
fn test_generic_type_instantiation_multiple_args() {
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Identifier("String".to_string()),
        TokenType::Comma,
        TokenType::Identifier("i32".to_string()),
        TokenType::Greater,
    ]);

    let result = parse_generic_type(&mut tokens, "HashMap".to_string()).unwrap();
    assert_eq!(result.base, "HashMap");
    assert_eq!(result.args.len(), 2);

    if let Type::Identifier(name) = &result.args[0] {
        assert_eq!(name, "String");
    } else {
        panic!("Expected identifier type");
    }

    if let Type::Identifier(name) = &result.args[1] {
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

        if let Type::Identifier(name) = &inner.args[0] {
            assert_eq!(name, "i32");
        } else {
            panic!("Expected identifier type");
        }
    } else {
        panic!("Expected generic type");
    }
}

#[test]
fn test_generic_function_declaration() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::Identifier("compare".to_string()),
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Colon,
        TokenType::Identifier("PartialOrd".to_string()),
        TokenType::Greater,
        TokenType::LeftParen,
        TokenType::Identifier("a".to_string()),
        TokenType::Colon,
        TokenType::Identifier("T".to_string()),
        TokenType::Comma,
        TokenType::Identifier("b".to_string()),
        TokenType::Colon,
        TokenType::Identifier("T".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("bool".to_string()),
        TokenType::Semicolon,
    ]);

    let mut parser = StatementParser::new(&arena, tokens);
    let result = parser.parse_item().unwrap();

    if let Item::FunctionDecl(func) = result {
        assert_eq!(func.name, "compare");
        assert!(func.generics.is_some());

        let generics = func.generics.as_ref().unwrap();
        assert_eq!(generics.params.len(), 1);
        assert_eq!(generics.params[0].name, "T");
        assert_eq!(generics.params[0].bounds.len(), 1);
        assert_eq!(generics.params[0].bounds[0].trait_name, "PartialOrd");

        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.parameters[0].name, "a");
        assert_eq!(func.parameters[1].name, "b");
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_generic_data_class_declaration() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Data,
        TokenType::Identifier("Container".to_string()),
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Greater,
        TokenType::LeftBrace,
        TokenType::Identifier("value".to_string()),
        TokenType::Colon,
        TokenType::Identifier("T".to_string()),
        TokenType::RightBrace,
    ]);

    let mut parser = StatementParser::new(&arena, tokens);
    let result = parser.parse_item().unwrap();

    if let Item::DataClassDecl(data_class) = result {
        assert_eq!(data_class.name, "Container");
        assert!(data_class.generics.is_some());

        let generics = data_class.generics.as_ref().unwrap();
        assert_eq!(generics.params.len(), 1);
        assert_eq!(generics.params[0].name, "T");

        assert_eq!(data_class.fields.len(), 1);
        assert_eq!(data_class.fields[0].name, "value");
    } else {
        panic!("Expected data class declaration");
    }
}

#[test]
fn test_complex_generic_function_with_where_clause() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::Identifier("complex_fn".to_string()),
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Comma,
        TokenType::Identifier("U".to_string()),
        TokenType::Greater,
        TokenType::LeftParen,
        TokenType::Identifier("x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("T".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("U".to_string()),
        TokenType::Where,
        TokenType::Identifier("T".to_string()),
        TokenType::Colon,
        TokenType::Identifier("Clone".to_string()),
        TokenType::Plus,
        TokenType::Identifier("Debug".to_string()),
        TokenType::Comma,
        TokenType::Identifier("U".to_string()),
        TokenType::Colon,
        TokenType::Identifier("Default".to_string()),
        TokenType::Semicolon,
    ]);

    let mut parser = StatementParser::new(&arena, tokens);
    let result = parser.parse_item().unwrap();

    if let Item::FunctionDecl(func) = result {
        assert_eq!(func.name, "complex_fn");
        assert!(func.generics.is_some());

        let generics = func.generics.as_ref().unwrap();
        assert_eq!(generics.params.len(), 2);
        assert_eq!(generics.params[0].name, "T");
        assert_eq!(generics.params[1].name, "U");

        // Check where clause
        assert!(generics.where_clause.is_some());
        let where_clause = generics.where_clause.as_ref().unwrap();
        assert_eq!(where_clause.constraints.len(), 2);

        // T: Clone + Debug
        assert_eq!(where_clause.constraints[0].type_name, "T");
        assert_eq!(where_clause.constraints[0].bounds.len(), 2);
        assert_eq!(where_clause.constraints[0].bounds[0].trait_name, "Clone");
        assert_eq!(where_clause.constraints[0].bounds[1].trait_name, "Debug");

        // U: Default
        assert_eq!(where_clause.constraints[1].type_name, "U");
        assert_eq!(where_clause.constraints[1].bounds.len(), 1);
        assert_eq!(where_clause.constraints[1].bounds[0].trait_name, "Default");
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_empty_generic_params() {
    let mut tokens = create_token_stream(vec![TokenType::Less, TokenType::Greater]);

    let result = parse_generic_params(&mut tokens).unwrap();
    assert!(result.is_some());

    let generics = result.unwrap();
    assert_eq!(generics.params.len(), 0);
    assert!(generics.where_clause.is_none());
}

#[test]
fn test_trailing_comma_in_generic_params() {
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
    assert_eq!(generics.params[0].name, "T");
    assert_eq!(generics.params[1].name, "U");
}

#[test]
fn test_trailing_comma_in_where_clause() {
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Greater,
        TokenType::Where,
        TokenType::Identifier("T".to_string()),
        TokenType::Colon,
        TokenType::Identifier("Clone".to_string()),
        TokenType::Comma,
    ]);

    let result = parse_generic_params(&mut tokens).unwrap();
    assert!(result.is_some());

    let generics = result.unwrap();
    assert!(generics.where_clause.is_some());

    let where_clause = generics.where_clause.unwrap();
    assert_eq!(where_clause.constraints.len(), 1);
    assert_eq!(where_clause.constraints[0].type_name, "T");
}

#[test]
fn test_generic_parsing_errors() {
    // Missing closing bracket
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Eof,
    ]);

    let result = parse_generic_params(&mut tokens);
    assert!(result.is_err());

    // Invalid lifetime (missing name)
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Apostrophe,
        TokenType::Greater,
    ]);

    let result = parse_generic_params(&mut tokens);
    assert!(result.is_err());

    // Missing colon in where clause
    let mut tokens = create_token_stream(vec![
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        TokenType::Greater,
        TokenType::Where,
        TokenType::Identifier("T".to_string()),
        TokenType::Identifier("Clone".to_string()),
    ]);

    let result = parse_generic_params(&mut tokens);
    assert!(result.is_err());
}
