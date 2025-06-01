//! Tests for Phase 2.8.1: Attribute Parsing
//!
//! Tests comprehensive attribute parsing including:
//! - Simple attributes (#[inline])
//! - Attributes with arguments (#[derive(Debug, Clone)])
//! - Multiple attributes
//! - Attributes on various declarations
//! - Error handling for malformed attributes

use ferra_parser::{
    ast::{Arena, Item, Statement},
    attribute::parser::{parse_attribute, parse_attributes},
    statement::parser::StatementParser,
    token::{stream::VecTokenStream, TokenType},
};

fn create_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
    VecTokenStream::from_token_types(token_types)
}

/// Test simple attribute parsing
#[test]
fn test_simple_attribute_parsing() {
    // #[inline]
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("inline".to_string()),
        TokenType::RightBracket,
    ]);

    let result = parse_attribute(&mut tokens).unwrap();
    assert_eq!(result.name, "inline");
    assert_eq!(result.arguments.len(), 0);
}

/// Test attribute with single argument
#[test]
fn test_attribute_with_single_argument() {
    // #[cfg(test)]
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("cfg".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("test".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
    ]);

    let result = parse_attribute(&mut tokens).unwrap();
    assert_eq!(result.name, "cfg");
    assert_eq!(result.arguments.len(), 1);
    assert_eq!(result.arguments[0], "test");
}

/// Test derive attribute with multiple arguments
#[test]
fn test_derive_attribute_multiple_arguments() {
    // #[derive(Debug, Clone, PartialEq)]
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("derive".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("Debug".to_string()),
        TokenType::Comma,
        TokenType::Identifier("Clone".to_string()),
        TokenType::Comma,
        TokenType::Identifier("PartialEq".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
    ]);

    let result = parse_attribute(&mut tokens).unwrap();
    assert_eq!(result.name, "derive");
    assert_eq!(result.arguments.len(), 3);
    assert_eq!(result.arguments[0], "Debug");
    assert_eq!(result.arguments[1], "Clone");
    assert_eq!(result.arguments[2], "PartialEq");
}

/// Test multiple consecutive attributes
#[test]
fn test_multiple_attributes() {
    // #[inline] #[derive(Debug)]
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("inline".to_string()),
        TokenType::RightBracket,
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("derive".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("Debug".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
    ]);

    let result = parse_attributes(&mut tokens).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "inline");
    assert_eq!(result[0].arguments.len(), 0);
    assert_eq!(result[1].name, "derive");
    assert_eq!(result[1].arguments.len(), 1);
    assert_eq!(result[1].arguments[0], "Debug");
}

/// Test attribute with string literal argument
#[test]
fn test_attribute_with_string_argument() {
    // #[doc("This is documentation")]
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("doc".to_string()),
        TokenType::LeftParen,
        TokenType::StringLiteral("This is documentation".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
    ]);

    let result = parse_attribute(&mut tokens).unwrap();
    assert_eq!(result.name, "doc");
    assert_eq!(result.arguments.len(), 1);
    assert_eq!(result.arguments[0], "\"This is documentation\"");
}

/// Test attribute with mixed argument types
#[test]
fn test_attribute_with_mixed_arguments() {
    // #[test_attr("string", 42, true)]
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("test_attr".to_string()),
        TokenType::LeftParen,
        TokenType::StringLiteral("string".to_string()),
        TokenType::Comma,
        TokenType::IntegerLiteral(42),
        TokenType::Comma,
        TokenType::BooleanLiteral(true),
        TokenType::RightParen,
        TokenType::RightBracket,
    ]);

    let result = parse_attribute(&mut tokens).unwrap();
    assert_eq!(result.name, "test_attr");
    assert_eq!(result.arguments.len(), 3);
    assert_eq!(result.arguments[0], "\"string\"");
    assert_eq!(result.arguments[1], "42");
    assert_eq!(result.arguments[2], "true");
}

/// Test attribute with trailing comma
#[test]
fn test_attribute_with_trailing_comma() {
    // #[derive(Debug, Clone,)]
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("derive".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("Debug".to_string()),
        TokenType::Comma,
        TokenType::Identifier("Clone".to_string()),
        TokenType::Comma,
        TokenType::RightParen,
        TokenType::RightBracket,
    ]);

    let result = parse_attribute(&mut tokens).unwrap();
    assert_eq!(result.name, "derive");
    assert_eq!(result.arguments.len(), 2);
    assert_eq!(result.arguments[0], "Debug");
    assert_eq!(result.arguments[1], "Clone");
}

/// Test function declaration with attributes
#[test]
fn test_function_with_attributes() {
    // #[inline] #[test] fn test_function() {}
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("inline".to_string()),
        TokenType::RightBracket,
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("test".to_string()),
        TokenType::RightBracket,
        TokenType::Fn,
        TokenType::Identifier("test_function".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
    ]);

    let mut parser = StatementParser::new(&arena, tokens);
    let item = parser.parse_item().unwrap();

    match item {
        Item::FunctionDecl(func) => {
            assert_eq!(func.name, "test_function");
            assert_eq!(func.attributes.len(), 2);
            assert_eq!(func.attributes[0].name, "inline");
            assert_eq!(func.attributes[1].name, "test");
        }
        _ => panic!("Expected function declaration"),
    }
}

/// Test variable declaration with attributes
#[test]
fn test_variable_with_attributes() {
    // #[allow(unused)] let x: int = 42
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("allow".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("unused".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Let,
        TokenType::Identifier("x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(42),
    ]);

    let mut parser = StatementParser::new(&arena, tokens);
    let statement = parser.parse_statement().unwrap();

    match statement {
        Statement::VariableDecl(var) => {
            assert_eq!(var.name, "x");
            assert_eq!(var.attributes.len(), 1);
            assert_eq!(var.attributes[0].name, "allow");
            assert_eq!(var.attributes[0].arguments[0], "unused");
        }
        _ => panic!("Expected variable declaration"),
    }
}

/// Test data class with attributes
#[test]
fn test_data_class_with_attributes() {
    // #[derive(Debug)] data Person { name: string }
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("derive".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("Debug".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Data,
        TokenType::Identifier("Person".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("name".to_string()),
        TokenType::Colon,
        TokenType::Identifier("string".to_string()),
        TokenType::RightBrace,
    ]);

    let mut parser = StatementParser::new(&arena, tokens);
    let item = parser.parse_item().unwrap();

    match item {
        Item::DataClassDecl(data_class) => {
            assert_eq!(data_class.name, "Person");
            assert_eq!(data_class.attributes.len(), 1);
            assert_eq!(data_class.attributes[0].name, "derive");
            assert_eq!(data_class.attributes[0].arguments[0], "Debug");
        }
        _ => panic!("Expected data class declaration"),
    }
}

/// Test field with attributes
#[test]
fn test_field_with_attributes() {
    // data Person { #[serde(rename = "full_name")] name: string }
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Data,
        TokenType::Identifier("Person".to_string()),
        TokenType::LeftBrace,
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("serde".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("rename".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Identifier("name".to_string()),
        TokenType::Colon,
        TokenType::Identifier("string".to_string()),
        TokenType::RightBrace,
    ]);

    let mut parser = StatementParser::new(&arena, tokens);
    let item = parser.parse_item().unwrap();

    match item {
        Item::DataClassDecl(data_class) => {
            assert_eq!(data_class.name, "Person");
            assert_eq!(data_class.fields.len(), 1);
            assert_eq!(data_class.fields[0].name, "name");
            assert_eq!(data_class.fields[0].attributes.len(), 1);
            assert_eq!(data_class.fields[0].attributes[0].name, "serde");
            assert_eq!(data_class.fields[0].attributes[0].arguments[0], "rename");
        }
        _ => panic!("Expected data class declaration"),
    }
}

/// Test parameter with attributes
#[test]
fn test_parameter_with_attributes() {
    // fn test(#[unused] param: int) {}
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::Identifier("test".to_string()),
        TokenType::LeftParen,
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("unused".to_string()),
        TokenType::RightBracket,
        TokenType::Identifier("param".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
    ]);

    let mut parser = StatementParser::new(&arena, tokens);
    let item = parser.parse_item().unwrap();

    match item {
        Item::FunctionDecl(func) => {
            assert_eq!(func.name, "test");
            assert_eq!(func.parameters.len(), 1);
            assert_eq!(func.parameters[0].name, "param");
            assert_eq!(func.parameters[0].attributes.len(), 1);
            assert_eq!(func.parameters[0].attributes[0].name, "unused");
        }
        _ => panic!("Expected function declaration"),
    }
}

/// Test attribute parsing error cases
#[test]
fn test_attribute_parsing_errors() {
    // Missing opening bracket: #identifier
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::Identifier("inline".to_string()),
    ]);

    let result = parse_attribute(&mut tokens);
    assert!(result.is_err());

    // Missing closing bracket: #[inline
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("inline".to_string()),
    ]);

    let result = parse_attribute(&mut tokens);
    assert!(result.is_err());

    // Invalid attribute name: #[123]
    let mut tokens = create_token_stream(vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(123),
        TokenType::RightBracket,
    ]);

    let result = parse_attribute(&mut tokens);
    assert!(result.is_err());
}

/// Test complex nested attributes
#[test]
fn test_complex_attributes() {
    // #[derive(Debug, Clone, Serialize, Deserialize)]
    // #[serde(rename_all = "camelCase")]
    // #[doc("A complex data structure")]
    let mut tokens = create_token_stream(vec![
        // First attribute
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("derive".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("Debug".to_string()),
        TokenType::Comma,
        TokenType::Identifier("Clone".to_string()),
        TokenType::Comma,
        TokenType::Identifier("Serialize".to_string()),
        TokenType::Comma,
        TokenType::Identifier("Deserialize".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        // Second attribute
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("serde".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("rename_all".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        // Third attribute
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("doc".to_string()),
        TokenType::LeftParen,
        TokenType::StringLiteral("A complex data structure".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
    ]);

    let result = parse_attributes(&mut tokens).unwrap();
    assert_eq!(result.len(), 3);

    // First attribute
    assert_eq!(result[0].name, "derive");
    assert_eq!(result[0].arguments.len(), 4);
    assert_eq!(result[0].arguments[0], "Debug");
    assert_eq!(result[0].arguments[1], "Clone");
    assert_eq!(result[0].arguments[2], "Serialize");
    assert_eq!(result[0].arguments[3], "Deserialize");

    // Second attribute
    assert_eq!(result[1].name, "serde");
    assert_eq!(result[1].arguments.len(), 1);
    assert_eq!(result[1].arguments[0], "rename_all");

    // Third attribute
    assert_eq!(result[2].name, "doc");
    assert_eq!(result[2].arguments.len(), 1);
    assert_eq!(result[2].arguments[0], "\"A complex data structure\"");
}

/// Test empty attribute list when no attributes present
#[test]
fn test_empty_attribute_list() {
    // fn test() {} (no attributes)
    let mut tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::Identifier("test".to_string()),
    ]);

    let result = parse_attributes(&mut tokens).unwrap();
    assert_eq!(result.len(), 0);
}

/// Test attribute integration with existing parsing
#[test]
fn test_attribute_integration_with_existing_parsing() {
    // Test that existing parsing still works when no attributes are present
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::Identifier("regular_function".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
    ]);

    let mut parser = StatementParser::new(&arena, tokens);
    let item = parser.parse_item().unwrap();

    match item {
        Item::FunctionDecl(func) => {
            assert_eq!(func.name, "regular_function");
            assert_eq!(func.attributes.len(), 0); // No attributes
        }
        _ => panic!("Expected function declaration"),
    }
}
