use ferra_lexer::{Lexer, LiteralValue, TokenKind};

#[test]
#[allow(clippy::approx_constant)]
fn float_literals() {
    let tokens = Lexer::new("3.14 2.0e10 1_000.5 42 1.0e-3 7. .5").lex();
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
    assert_eq!(tokens[0].literal, Some(LiteralValue::Float(3.14)));
    assert_eq!(tokens[1].literal, Some(LiteralValue::Float(2.0e10)));
    assert_eq!(tokens[2].literal, Some(LiteralValue::Float(1000.5)));
    assert_eq!(tokens[4].literal, Some(LiteralValue::Float(1.0e-3)));
    assert_eq!(tokens[5].literal, Some(LiteralValue::Float(7.0)));
    assert_eq!(tokens[6].literal, Some(LiteralValue::Float(0.5)));
}

#[test]
#[allow(clippy::approx_constant)]
fn integration_float_literals() {
    let tokens =
        Lexer::new("x = 3.14; y = 2.0e10; z = 1_000.5; a = 42; b = 1.0e-3; c = 7.; d = .5;").lex();
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
    let token_c = tokens.iter().find(|t| t.lexeme == "7.").unwrap();
    assert_eq!(token_c.literal, Some(LiteralValue::Float(7.0)));
    let token_d = tokens.iter().find(|t| t.lexeme == ".5").unwrap();
    assert_eq!(token_d.literal, Some(LiteralValue::Float(0.5)));
    let float_token = tokens
        .iter()
        .find(|t| t.kind == TokenKind::FloatLiteral && t.lexeme == "3.14")
        .unwrap();
    assert_eq!(float_token.literal, Some(LiteralValue::Float(3.14)));
}
