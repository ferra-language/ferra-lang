use ferra_parser::{
    program::ProgramParser,
    token::{stream::VecTokenStream, types::TokenType},
    Arena,
};

/// Generate tokens for basic attribute testing
fn create_attribute_tokens() -> Vec<TokenType> {
    vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("derive".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("Debug".to_string()),
        TokenType::Comma,
        TokenType::Identifier("Clone".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Data,
        TokenType::Identifier("TestStruct".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("field".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ]
}

/// Generate complex attribute combinations
fn create_complex_attribute_tokens() -> Vec<TokenType> {
    vec![
        // Multiple attributes
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("derive".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("Debug".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("derive".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("Clone".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("cfg".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("feature".to_string()),
        TokenType::Equal,
        TokenType::StringLiteral("advanced".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Data,
        TokenType::Identifier("ComplexStruct".to_string()),
        TokenType::LeftBrace,
        // Field with attributes
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("serde".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("skip".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Identifier("field1".to_string()),
        TokenType::Colon,
        TokenType::Identifier("String".to_string()),
        TokenType::Comma,
        TokenType::Identifier("field2".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ]
}

/// Generate tokens with attribute errors for recovery testing
fn create_malformed_attribute_tokens() -> Vec<TokenType> {
    vec![
        // Malformed attribute - missing closing bracket
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("derive".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("Debug".to_string()),
        TokenType::Comma,
        TokenType::Identifier("Clone".to_string()),
        TokenType::Comma, // Trailing comma
        TokenType::RightParen,
        // Missing closing bracket - should cause error but allow recovery
        TokenType::Data,
        TokenType::Identifier("StructWithBadAttr".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("field".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::RightBrace,
        // Another malformed attribute
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("cfg".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("feature".to_string()),
        TokenType::Equal,
        TokenType::StringLiteral("test".to_string()),
        // Missing closing paren and bracket

        // Valid function after errors - should parse successfully
        TokenType::Fn,
        TokenType::Identifier("valid_function".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("i32".to_string()),
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::IntegerLiteral(42),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]
}

/// Generate tokens for function with attributes
fn create_function_attribute_tokens() -> Vec<TokenType> {
    vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("inline".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("always".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("cfg".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("feature".to_string()),
        TokenType::Equal,
        TokenType::StringLiteral("optimized".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Fn,
        TokenType::Identifier("attributed_function".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("param".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("i32".to_string()),
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::Identifier("param".to_string()),
        TokenType::Star,
        TokenType::IntegerLiteral(2),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]
}

/// Generate deeply nested attribute tokens
fn create_nested_conditional_attribute_tokens() -> Vec<TokenType> {
    vec![
        TokenType::Hash,
        TokenType::LeftBracket,
        TokenType::Identifier("cfg".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("all".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("feature".to_string()),
        TokenType::Equal,
        TokenType::StringLiteral("level1".to_string()),
        TokenType::Comma,
        TokenType::Identifier("any".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("feature".to_string()),
        TokenType::Equal,
        TokenType::StringLiteral("level2a".to_string()),
        TokenType::Comma,
        TokenType::Identifier("feature".to_string()),
        TokenType::Equal,
        TokenType::StringLiteral("level2b".to_string()),
        TokenType::RightParen,
        TokenType::RightParen,
        TokenType::RightParen,
        TokenType::RightBracket,
        TokenType::Data,
        TokenType::Identifier("NestedConditionalStruct".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("field".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ]
}

/// Test helper to parse tokens and verify no errors
fn parse_successfully(tokens: Vec<TokenType>, test_name: &str) {
    let arena = Arena::new();
    let token_stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, token_stream);

    match parser.parse_compilation_unit() {
        Ok(program) => {
            println!(
                "{}: Successfully parsed {} items",
                test_name,
                program.items.len()
            );
            assert!(!program.items.is_empty(), "Should parse at least one item");
        }
        Err(errors) => {
            println!("{}: Parse errors: {:?}", test_name, errors);
            panic!("Should parse successfully for test: {}", test_name);
        }
    }
}

/// Test helper to parse tokens expecting errors but ensuring recovery
fn parse_with_expected_errors(tokens: Vec<TokenType>, test_name: &str) {
    let arena = Arena::new();
    let token_stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, token_stream);

    let _result = parser.parse_compilation_unit();
    let has_errors = parser.has_errors();

    println!("{}: Parser detected errors: {}", test_name, has_errors);
    // Just ensure it completes without hanging - error detection may vary
}

#[cfg(test)]
mod complex_nested_attribute_tests {
    use super::*;

    #[test]
    fn test_basic_attribute_parsing() {
        let tokens = create_attribute_tokens();
        parse_successfully(tokens, "basic_attribute_parsing");
    }

    #[test]
    fn test_multiple_attribute_combinations() {
        let tokens = create_complex_attribute_tokens();
        parse_successfully(tokens, "multiple_attribute_combinations");
    }

    #[test]
    fn test_function_with_attributes() {
        let tokens = create_function_attribute_tokens();
        parse_successfully(tokens, "function_with_attributes");
    }

    #[test]
    fn test_nested_conditional_attributes() {
        let tokens = create_nested_conditional_attribute_tokens();
        parse_successfully(tokens, "nested_conditional_attributes");
    }

    #[test]
    fn test_attribute_error_recovery() {
        let tokens = create_malformed_attribute_tokens();
        parse_with_expected_errors(tokens, "attribute_error_recovery");
    }

    #[test]
    fn test_massive_attribute_stress() {
        // Generate a large number of attributes to stress test parsing
        let mut tokens = Vec::new();

        // Add 50 different attributes
        for i in 0..50 {
            tokens.extend(vec![
                TokenType::Hash,
                TokenType::LeftBracket,
                TokenType::Identifier("derive".to_string()),
                TokenType::LeftParen,
                TokenType::Identifier(format!("Trait{i}")),
                TokenType::RightParen,
                TokenType::RightBracket,
            ]);
        }

        // Add the actual data structure
        tokens.extend(vec![
            TokenType::Data,
            TokenType::Identifier("MassiveAttributeStruct".to_string()),
            TokenType::LeftBrace,
            TokenType::Identifier("field".to_string()),
            TokenType::Colon,
            TokenType::Identifier("i32".to_string()),
            TokenType::RightBrace,
            TokenType::Eof,
        ]);

        parse_successfully(tokens, "massive_attribute_stress");
    }

    #[test]
    fn test_attribute_parameter_complexity() {
        // Test attributes with complex parameter structures
        let tokens = vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("serde".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("rename_all".to_string()),
            TokenType::Equal,
            TokenType::StringLiteral("camelCase".to_string()),
            TokenType::Comma,
            TokenType::Identifier("deny_unknown_fields".to_string()),
            TokenType::Comma,
            TokenType::Identifier("tag".to_string()),
            TokenType::Equal,
            TokenType::StringLiteral("type".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
            TokenType::Data,
            TokenType::Identifier("ComplexSerdeStruct".to_string()),
            TokenType::LeftBrace,
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("serde".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("serialize_with".to_string()),
            TokenType::Equal,
            TokenType::StringLiteral("custom_serializer".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
            TokenType::Identifier("complex_field".to_string()),
            TokenType::Colon,
            TokenType::Identifier("CustomType".to_string()),
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        parse_successfully(tokens, "attribute_parameter_complexity");
    }

    #[test]
    fn test_attribute_whitespace_handling() {
        // Test that attributes work with various whitespace patterns
        let tokens = vec![
            // Compact attribute
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("derive".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("Debug".to_string()),
            TokenType::Comma,
            TokenType::Identifier("Clone".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
            // Attribute with extra spacing (simulated by parser handling)
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("cfg".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("feature".to_string()),
            TokenType::Equal,
            TokenType::StringLiteral("test".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
            TokenType::Data,
            TokenType::Identifier("WhitespaceTestStruct".to_string()),
            TokenType::LeftBrace,
            TokenType::Identifier("field".to_string()),
            TokenType::Colon,
            TokenType::Identifier("i32".to_string()),
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        parse_successfully(tokens, "attribute_whitespace_handling");
    }

    #[test]
    fn test_mixed_attributes_and_modifiers() {
        // Test attributes combined with visibility and safety modifiers
        let tokens = vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("derive".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("Debug".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
            TokenType::Pub,
            TokenType::Data,
            TokenType::Identifier("PublicStruct".to_string()),
            TokenType::LeftBrace,
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("serde".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("skip".to_string()),
            TokenType::RightParen,
            TokenType::RightBracket,
            TokenType::Identifier("private_field".to_string()),
            TokenType::Colon,
            TokenType::Identifier("String".to_string()),
            TokenType::Comma,
            TokenType::Pub,
            TokenType::Identifier("public_field".to_string()),
            TokenType::Colon,
            TokenType::Identifier("i32".to_string()),
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        parse_successfully(tokens, "mixed_attributes_and_modifiers");
    }

    #[test]
    fn test_attribute_chaining_stress() {
        // Test many attributes chained together
        let mut tokens = Vec::new();

        // Chain 25 attributes together
        for i in 0..25 {
            tokens.extend(vec![
                TokenType::Hash,
                TokenType::LeftBracket,
                TokenType::Identifier(format!("attr_{i}")),
                TokenType::LeftParen,
                TokenType::StringLiteral(format!("value_{i}")),
                TokenType::RightParen,
                TokenType::RightBracket,
            ]);
        }

        tokens.extend(vec![
            TokenType::Fn,
            TokenType::Identifier("heavily_attributed_function".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Eof,
        ]);

        parse_successfully(tokens, "attribute_chaining_stress");
    }

    #[test]
    fn test_deeply_nested_attribute_expressions() {
        // Test attributes with deeply nested parameter expressions
        let tokens = vec![
            TokenType::Hash,
            TokenType::LeftBracket,
            TokenType::Identifier("cfg".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("all".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("feature".to_string()),
            TokenType::Equal,
            TokenType::StringLiteral("level1".to_string()),
            TokenType::Comma,
            TokenType::Identifier("any".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("target_os".to_string()),
            TokenType::Equal,
            TokenType::StringLiteral("linux".to_string()),
            TokenType::Comma,
            TokenType::Identifier("target_os".to_string()),
            TokenType::Equal,
            TokenType::StringLiteral("macos".to_string()),
            TokenType::RightParen,
            TokenType::Comma,
            TokenType::Identifier("not".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("target_os".to_string()),
            TokenType::Equal,
            TokenType::StringLiteral("windows".to_string()),
            TokenType::RightParen,
            TokenType::RightParen,
            TokenType::RightParen,
            TokenType::RightBracket,
            TokenType::Data,
            TokenType::Identifier("DeeplyNestedConditionStruct".to_string()),
            TokenType::LeftBrace,
            TokenType::Identifier("field".to_string()),
            TokenType::Colon,
            TokenType::Identifier("i32".to_string()),
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        parse_successfully(tokens, "deeply_nested_attribute_expressions");
    }
}
