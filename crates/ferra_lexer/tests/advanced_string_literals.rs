use ferra_lexer::*;

fn lex_all(input: &str) -> Vec<Token> {
    Lexer::new(input).lex()
}

// Hash-delimited raw string tests
#[test]
fn test_raw_string_basic_no_hash() {
    let input = r#"r"hello world""#;
    let tokens = lex_all(input);

    assert_eq!(tokens.len(), 2); // RawStringLiteral + EOF
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("hello world".to_string()))
    );
}

#[test]
fn test_raw_string_single_hash() {
    let input = r##"r#"He said "hello" world"#"##;
    let tokens = lex_all(input);

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(r#"He said "hello" world"#.to_string()))
    );
}

#[test]
fn test_raw_string_double_hash() {
    let input = r###"r##"String with "quotes" and #hashes#"##"###;
    let tokens = lex_all(input);

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            r#"String with "quotes" and #hashes#"#.to_string()
        ))
    );
}

#[test]
fn test_raw_string_multiline_with_hash() {
    let input = r##"r#"line 1
line 2 with "quotes"
line 3"#"##;
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "line 1\nline 2 with \"quotes\"\nline 3".to_string()
        ))
    );
    assert_eq!(tokens[0].span.start.line, 1);
    assert_eq!(tokens[0].span.end.line, 3);
}

#[test]
fn test_raw_string_unterminated_with_hash() {
    let input = r##"r#"unterminated"##; // Missing closing #
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert!(matches!(
        tokens[0].literal.as_ref().unwrap(),
        LiteralValue::String(msg) if msg.contains("Unterminated raw string literal")
    ));
}

#[test]
fn test_raw_string_hash_mismatch() {
    let input = r###"r##"content"#"###; // Wrong number of closing hashes
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert!(matches!(
        tokens[0].literal.as_ref().unwrap(),
        LiteralValue::String(msg) if msg.contains("Unterminated raw string literal")
    ));
}

#[test]
fn test_raw_string_invalid_no_quote() {
    let input = "r#hello"; // Missing quote after hash
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert!(matches!(
        tokens[0].literal.as_ref().unwrap(),
        LiteralValue::String(msg) if msg.contains("Expected '\"' after 'r' and hash characters")
    ));
}

#[test]
fn test_raw_string_no_escapes() {
    // Verify that r"\n" contains literal \ and n, not a newline character
    let input = r#"r"\n\t\r""#;  // Changed to avoid quote termination issues
    let tokens = lex_all(input);
    
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    // Should contain literal backslashes, not escape sequences
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("\\n\\t\\r".to_string()))
    );
    
    // Test with hash delimiters to include quotes
    let input2 = r##"r#"\n "quotes" \t\""#"##;
    let tokens2 = lex_all(input2);
    
    assert_eq!(tokens2[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(
        tokens2[0].literal,
        Some(LiteralValue::String("\\n \"quotes\" \\t\\\"".to_string()))
    );
}

// Multiline string tests
#[test]
fn test_multiline_string_basic() {
    let input = "\"\"\"\nHello\nWorld\n\"\"\"";
    let tokens = lex_all(input);

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::MultiLineStringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("Hello\nWorld".to_string()))
    );
}

#[test]
fn test_multiline_string_with_indentation() {
    let input = "\"\"\"\n    Line 1\n    Line 2\n        Indented line\n    Line 3\n\"\"\"";
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::MultiLineStringLiteral);
    // Common indentation (4 spaces) should be stripped
    let expected = "Line 1\nLine 2\n    Indented line\nLine 3";
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(expected.to_string()))
    );
}

#[test]
fn test_multiline_string_first_line_content() {
    let input = "\"\"\"Hello\n    World\n    !\"\"\"";
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::MultiLineStringLiteral);
    // First line has content, so indentation is preserved
    let expected = "Hello\n    World\n    !";
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(expected.to_string()))
    );
}

#[test]
fn test_multiline_string_empty_lines() {
    let input = "\"\"\"\n    Line 1\n\n    Line 3\n\"\"\"";
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::MultiLineStringLiteral);
    let expected = "Line 1\n\nLine 3";
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(expected.to_string()))
    );
}

#[test]
fn test_multiline_string_quotes_inside() {
    let input = "\"\"\"\nHe said \"hello\" and 'goodbye'\nShe replied \"see you later\"\n\"\"\"";
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::MultiLineStringLiteral);
    let expected = "He said \"hello\" and 'goodbye'\nShe replied \"see you later\"";
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(expected.to_string()))
    );
}

#[test]
fn test_multiline_string_unterminated() {
    let input = "\"\"\"\nUnterminated string\nStill going...";
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert!(matches!(
        tokens[0].literal.as_ref().unwrap(),
        LiteralValue::String(msg) if msg.contains("Unterminated multiline string literal")
    ));
}

#[test]
fn test_multiline_string_single_quote_inside() {
    let input = "\"\"\"\nThis has one \" quote\nAnd another \" quote\n\"\"\"";
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::MultiLineStringLiteral);
    let expected = "This has one \" quote\nAnd another \" quote";
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(expected.to_string()))
    );
}

#[test]
fn test_multiline_string_precise_spans() {
    let input = "\"\"\"\nline 1\nline 2\n\"\"\"";
    let tokens = lex_all(input);

    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, input.len());
    assert_eq!(tokens[0].span.start.line, 1);
    assert_eq!(tokens[0].span.end.line, 4);
}

#[test]
fn test_multiline_string_complex_indentation() {
    let input = "\"\"\"\n        function example() {\n            if (condition) {\n                return value;\n            }\n        }\n\"\"\"";
    let tokens = lex_all(input);

    assert_eq!(tokens[0].kind, TokenKind::MultiLineStringLiteral);
    let expected = "function example() {\n    if (condition) {\n        return value;\n    }\n}";
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(expected.to_string()))
    );
}

// Mixed usage tests
#[test]
fn test_mixed_string_types_advanced() {
    let input = "\"\"\"\nmultiline\ncontent\n\"\"\" r#\"raw with \"quotes\"\"# \"regular\"";
    let tokens = lex_all(input);

    assert_eq!(tokens.len(), 4); // 3 strings + EOF

    assert_eq!(tokens[0].kind, TokenKind::MultiLineStringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("multiline\ncontent".to_string()))
    );

    assert_eq!(tokens[1].kind, TokenKind::RawStringLiteral);
    assert_eq!(
        tokens[1].literal,
        Some(LiteralValue::String("raw with \"quotes\"".to_string()))
    );

    assert_eq!(tokens[2].kind, TokenKind::StringLiteral);
    assert_eq!(
        tokens[2].literal,
        Some(LiteralValue::String("regular".to_string()))
    );
}

#[test]
fn test_multiline_string_indentation_interaction() {
    // Test that multiline strings don't interfere with INDENT/DEDENT logic
    let input = "let x = \"\"\"\n    inner content\n    more content\n\"\"\"\nlet y = 5";
    let tokens = lex_all(input);
    
    // Find the tokens we care about
    let mut found_multiline = false;
    let mut found_second_let = false;
    let mut unexpected_indents = 0;
    
    for token in tokens.iter() {
        match &token.kind {
            TokenKind::MultiLineStringLiteral => {
                found_multiline = true;
                // Verify the content is properly processed
                assert_eq!(
                    token.literal,
                    Some(LiteralValue::String("inner content\nmore content".to_string()))
                );
            },
            TokenKind::Let if found_multiline && !found_second_let => {
                found_second_let = true;
                // Should be the second 'let' statement
            },
            TokenKind::Indent | TokenKind::Dedent => {
                // There should be no INDENT/DEDENT tokens in this example
                // because the multiline string content doesn't affect the main indentation level
                unexpected_indents += 1;
            },
            _ => {}
        }
    }
    
    assert!(found_multiline, "Should find multiline string");
    assert!(found_second_let, "Should find second let statement");
    assert_eq!(unexpected_indents, 0, "Should not have any INDENT/DEDENT tokens in this flat structure");
}
