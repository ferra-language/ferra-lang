use ferra_lexer::*;

// Helper function to lex the entire input string
fn lex_all(input: &str) -> Vec<Token> {
    Lexer::new(input).lex()
}

#[test]
fn test_let_keyword() {
    let tokens = lex_all("let");
    println!("TOKENS: {:?}", tokens);
    assert_eq!(tokens.len(), 2); // let, EOF
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[0].lexeme, "let");
    // Only check that the span covers the input
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, 3);
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

#[test]
fn test_identifier() {
    let tokens = lex_all("my_var");
    println!("TOKENS: {:?}", tokens);
    assert_eq!(tokens.len(), 2); // identifier, EOF
    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[0].lexeme, "my_var");
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, 6);
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

#[test]
fn test_keywords_and_identifiers() {
    let tokens = lex_all("let var fn async data match true false ident1 _ident2");
    let expected_kinds = vec![
        TokenKind::Let,
        TokenKind::Var,
        TokenKind::Fn,
        TokenKind::Async,
        TokenKind::Data,
        TokenKind::Match,
        TokenKind::True,
        TokenKind::False,
        TokenKind::Identifier,
        TokenKind::Identifier,
        TokenKind::Eof,
    ];
    let expected_lexemes = vec![
        "let", "var", "fn", "async", "data", "match", "true", "false", "ident1", "_ident2", "",
    ];
    let expected_literals = vec![
        None,
        None,
        None,
        None,
        None,
        None,
        Some(LiteralValue::Boolean(true)),
        Some(LiteralValue::Boolean(false)),
        None,
        None,
        None,
    ];

    assert_eq!(
        tokens.len(),
        expected_kinds.len(),
        "Mismatch in token count"
    );

    for i in 0..tokens.len() - 1 {
        // Exclude EOF for literal check simplicity here
        assert_eq!(
            tokens[i].kind, expected_kinds[i],
            "Mismatch in kind for {}",
            expected_lexemes[i]
        );
        assert_eq!(
            tokens[i].lexeme, expected_lexemes[i],
            "Mismatch in lexeme for {}",
            expected_lexemes[i]
        );
        assert_eq!(
            tokens[i].literal, expected_literals[i],
            "Mismatch in literal for {}",
            expected_lexemes[i]
        );
    }
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn test_and_or_keywords() {
    let tokens = lex_all("and or");
    assert_eq!(tokens.len(), 3); // and, or, eof
    assert_eq!(tokens[0].kind, TokenKind::LogicalAnd);
    assert_eq!(tokens[0].lexeme, "and");
    assert_eq!(tokens[1].kind, TokenKind::LogicalOr);
    assert_eq!(tokens[1].lexeme, "or");
    assert_eq!(tokens[2].kind, TokenKind::Eof);
}
