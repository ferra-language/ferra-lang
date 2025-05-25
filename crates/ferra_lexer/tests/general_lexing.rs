use ferra_lexer::*;

// Helper function to lex the entire input string
fn lex_all(input: &str) -> Vec<Token> {
    Lexer::new(input).lex()
}

#[test]
fn test_empty_input() {
    assert_eq!(lex_all(""), vec![Token::eof_dummy()]);
}

#[test]
fn test_whitespace_only() {
    let tokens = lex_all("   \t\n  ");
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenKind::Newline);
    assert_eq!(tokens[1].kind, TokenKind::Indent);
    assert_eq!(tokens[2].kind, TokenKind::Dedent);
    assert_eq!(tokens[3].kind, TokenKind::Eof);
}

#[test]
fn test_unrecognized_character() {
    let tokens = lex_all("$");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "$".to_string());
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

#[test]
fn test_shebang_skipped() {
    let src = "#! /usr/bin/env ferra\nlet x = 42;";
    let tokens = ferra_lexer::Lexer::new(src).lex();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &ferra_lexer::TokenKind::Let,
            &ferra_lexer::TokenKind::Identifier,
            &ferra_lexer::TokenKind::Equal,
            &ferra_lexer::TokenKind::IntegerLiteral,
            &ferra_lexer::TokenKind::Semicolon,
            &ferra_lexer::TokenKind::Eof,
        ]
    );
}
