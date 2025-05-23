use ferra_lexer::{Lexer, LiteralValue, TokenKind};

#[test]
fn float_literals() {
    let tokens = Lexer::new("3.14 2.0e10 1_000.5 42 1.0e-3 7. 0.5").lex();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::FloatLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::IntegerLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::Eof
        ]
    );
}

#[test]
fn integration_float_literals() {
    let tokens =
        Lexer::new("x = 3.14; y = 2.0e10; z = 1_000.5; a = 42; b = 1.0e-3; c = 7.; d = 0.5;").lex();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Identifier,
            &TokenKind::Equal,
            &TokenKind::FloatLiteral,
            &TokenKind::Semicolon,
            &TokenKind::Identifier,
            &TokenKind::Equal,
            &TokenKind::FloatLiteral,
            &TokenKind::Semicolon,
            &TokenKind::Identifier,
            &TokenKind::Equal,
            &TokenKind::FloatLiteral,
            &TokenKind::Semicolon,
            &TokenKind::Identifier,
            &TokenKind::Equal,
            &TokenKind::IntegerLiteral,
            &TokenKind::Semicolon,
            &TokenKind::Identifier,
            &TokenKind::Equal,
            &TokenKind::FloatLiteral,
            &TokenKind::Semicolon,
            &TokenKind::Identifier,
            &TokenKind::Equal,
            &TokenKind::FloatLiteral,
            &TokenKind::Semicolon,
            &TokenKind::Identifier,
            &TokenKind::Equal,
            &TokenKind::FloatLiteral,
            &TokenKind::Semicolon,
            &TokenKind::Eof
        ]
    );
    // Spot check a value
    let float_token = tokens
        .iter()
        .find(|t| t.kind == TokenKind::FloatLiteral && t.lexeme == "3.14")
        .unwrap();
    assert_eq!(float_token.literal, Some(LiteralValue::Float(3.14)));
}
