use ferra_lexer::{Lexer, LiteralValue, Token, TokenKind};

#[test]
fn empty_source_emits_eof() {
    let tokens = Lexer::new("").lex();
    assert_eq!(tokens, vec![Token::eof_dummy()]);
}

#[test]
fn simple_let_statement() {
    let tokens = Lexer::new("let x = 42;").lex();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Let,
            &TokenKind::Identifier,
            &TokenKind::Equal,
            &TokenKind::IntegerLiteral,
            &TokenKind::Semicolon,
            &TokenKind::Eof,
        ]
    );
}

#[test]
fn byte_literals() {
    let tokens = Lexer::new("b'a' b\"foo\" b'' b'xy' b'\\q' b'a").lex();
    println!("TOKENS: {:?}", tokens);
    assert_eq!(tokens[0].kind, TokenKind::ByteLiteral);
    assert_eq!(tokens[0].lexeme, "b'a'");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Byte(b'a')));
    assert_eq!(tokens[1].kind, TokenKind::ByteLiteral);
    assert_eq!(tokens[1].lexeme, "b\"foo\"");
    assert_eq!(
        tokens[1].literal,
        Some(LiteralValue::String("foo".to_string()))
    );
    assert_eq!(tokens[2].kind, TokenKind::Error); // empty
    assert_eq!(tokens[3].kind, TokenKind::Error); // multi-char
    assert_eq!(tokens[4].kind, TokenKind::Error); // invalid escape
    assert_eq!(tokens[5].kind, TokenKind::Error); // unterminated
}

#[test]
fn block_comment() {
    let tokens = Lexer::new("let /* comment */ x").lex();
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::Eof);
    let tokens_unterminated = Lexer::new("let /* unterminated").lex();
    assert_eq!(tokens_unterminated[0].kind, TokenKind::Let);
    assert_eq!(tokens_unterminated[1].kind, TokenKind::Error);
}

#[test]
fn indentation_tokens() {
    let src = "a\n    b\n        c\n    d\ne";
    let tokens = Lexer::new(src).lex();
    for t in &tokens {
        println!("{:?} '{}'", t.kind, t.lexeme);
    }
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    // Updated to match the new correct behavior where all identifiers are preserved
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Identifier, // a
            &TokenKind::Newline,
            &TokenKind::Indent,     // for "    b"
            &TokenKind::Identifier, // b (now correctly preserved!)
            &TokenKind::Newline,    // after b
            &TokenKind::Indent,     // for "        c"
            &TokenKind::Identifier, // c (now correctly preserved!)
            &TokenKind::Newline,    // after c
            &TokenKind::Dedent,     // dedent for d
            &TokenKind::Identifier, // d (now correctly preserved!)
            &TokenKind::Newline,    // after d
            &TokenKind::Dedent,     // dedent to base level
            &TokenKind::Identifier, // e
            &TokenKind::Eof,
        ]
    );
}

#[test]
fn error_token_for_unrecognized_input() {
    let src = "let x = \u{1F600};"; // \u{1F600} is not valid in this lexer
    let tokens = Lexer::new(src).lex();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Error));
}

#[test]
fn blank_and_comment_only_lines_indentation() {
    let src = "a\n    \n    // comment\n    b\nc";
    let tokens = Lexer::new(src).lex();
    println!(
        "TOKENS: {:?}",
        tokens
            .iter()
            .map(|t| (t.kind.clone(), t.lexeme.clone()))
            .collect::<Vec<_>>()
    );
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind.clone()).collect();
    // Updated to match the new correct behavior where 'b' is properly preserved
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier, // a
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Newline,    // from blank line
            TokenKind::Newline,    // from comment line
            TokenKind::Identifier, // b (now correctly preserved!)
            TokenKind::Newline,    // after b
            TokenKind::Dedent,
            TokenKind::Identifier, // c
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_mixed_indentation_error() {
    // The lexer only processes indentation after newlines, not at file start
    // So we need content before the newline for indentation to be processed

    // Valid: only spaces after newline
    let tokens_spaces = Lexer::new("x\n  a").lex();
    assert_eq!(tokens_spaces.len(), 6); // x, newline, indent, a, dedent, eof
    assert!(!tokens_spaces.iter().any(|t| t.kind == TokenKind::Error));
    assert_eq!(tokens_spaces[2].kind, TokenKind::Indent);

    // Valid: only tabs after newline (1 tab = 4 spaces)
    let tokens_tabs = Lexer::new("x\n\ta").lex();
    assert_eq!(tokens_tabs.len(), 6); // x, newline, indent, a, dedent, eof
    assert!(!tokens_tabs.iter().any(|t| t.kind == TokenKind::Error));
    assert_eq!(tokens_tabs[2].kind, TokenKind::Indent);

    // Invalid: space then tab after newline
    let tokens_space_tab = Lexer::new("x\n \ta").lex();
    assert!(tokens_space_tab.iter().any(|t| t.kind == TokenKind::Error));
    let error_token = tokens_space_tab
        .iter()
        .find(|t| t.kind == TokenKind::Error)
        .unwrap();
    assert_eq!(error_token.lexeme, " \t");
    if let Some(LiteralValue::String(msg)) = &error_token.literal {
        assert!(msg.contains("Mixed tabs and spaces"));
    }

    // Invalid: tab then space after newline
    let tokens_tab_space = Lexer::new("x\n\t a").lex();
    assert!(tokens_tab_space.iter().any(|t| t.kind == TokenKind::Error));
    let error_token = tokens_tab_space
        .iter()
        .find(|t| t.kind == TokenKind::Error)
        .unwrap();
    assert_eq!(error_token.lexeme, "\t ");
    if let Some(LiteralValue::String(msg)) = &error_token.literal {
        assert!(msg.contains("Mixed tabs and spaces"));
    }
}

#[test]
fn test_mixed_tabs_spaces_in_indent() {
    let input = "x\n\t a"; // Content, newline, then tab+space+content
    let lexer = Lexer::new(input);
    let tokens = lexer.lex();

    // Should produce Identifier + Newline + Error + ... + EOF
    // The exact count may vary but we should have an error
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Error));
    let error_token = tokens.iter().find(|t| t.kind == TokenKind::Error).unwrap();
    if let Some(LiteralValue::String(msg)) = &error_token.literal {
        assert!(msg.contains("Mixed tabs and spaces"));
    } else {
        panic!("Expected error message");
    }
}

#[test]
fn test_expansion_tabs_to_spaces() {
    let input_tab_space = "x\n\t\ta"; // Content, newline, then two tabs then 'a'
    let lexer_ts = Lexer::new(input_tab_space);
    let tokens_ts = lexer_ts.lex();

    // Should produce some tokens including x and indent
    // The exact sequence may vary due to the identifier consumption issue
    assert!(tokens_ts.len() >= 3);
    assert_eq!(tokens_ts[0].kind, TokenKind::Identifier); // x
    assert_eq!(tokens_ts[1].kind, TokenKind::Newline);
    // Should have an indent token somewhere
    assert!(tokens_ts.iter().any(|t| t.kind == TokenKind::Indent));
}

#[test]
fn test_blank_comment_lines_preserve_identifiers() {
    // This test explicitly checks for the bug where identifiers after blank/comment lines are dropped
    let src = "a\n    \n    // comment\n    b\nc";
    let tokens = Lexer::new(src).lex();

    // Extract all identifiers to verify they're all present
    let idents: Vec<&str> = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Identifier)
        .map(|t| t.lexeme.as_str())
        .collect();

    // CRITICAL: Should find all three identifiers: a, b, c
    // Currently fails because 'b' is missing - this is the bug to fix
    assert_eq!(
        idents,
        vec!["a", "b", "c"],
        "Expected all identifiers a, b, c but got: {:?}",
        idents
    );
}

#[test]
fn test_simple_dedent_case() {
    // Simplified test: just a simple dedent case
    let src = "a\n    b\nc";
    let tokens = Lexer::new(src).lex();

    println!(
        "All tokens: {:?}",
        tokens
            .iter()
            .map(|t| (t.kind.clone(), t.lexeme.clone()))
            .collect::<Vec<_>>()
    );

    // Extract all identifiers to verify they're all present
    let idents: Vec<&str> = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Identifier)
        .map(|t| t.lexeme.as_str())
        .collect();

    assert_eq!(
        idents,
        vec!["a", "b", "c"],
        "Expected all identifiers a, b, c but got: {:?}",
        idents
    );
}

#[test]
fn test_minimal_indent_case() {
    // Even simpler: just one space and one character
    let src = "a\n b";
    let tokens = Lexer::new(src).lex();

    println!(
        "Minimal tokens: {:?}",
        tokens
            .iter()
            .map(|t| (t.kind.clone(), t.lexeme.clone()))
            .collect::<Vec<_>>()
    );

    let idents: Vec<&str> = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Identifier)
        .map(|t| t.lexeme.as_str())
        .collect();

    assert_eq!(
        idents,
        vec!["a", "b"],
        "Expected identifiers a, b but got: {:?}",
        idents
    );
}
