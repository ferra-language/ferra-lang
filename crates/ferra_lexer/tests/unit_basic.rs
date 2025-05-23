use ferra_lexer::{Lexer, Token, TokenKind};

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
