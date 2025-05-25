use ferra_lexer::*;

fn lex_all(input: &str) -> Vec<Token> {
    Lexer::new(input).lex()
}

#[test]
fn test_line_comment() {
    let tokens = lex_all("// this is a comment\nlet");
    assert_eq!(tokens[0].kind, TokenKind::Newline);
    assert_eq!(tokens[1].kind, TokenKind::Let);
}

#[test]
fn test_block_comment() {
    let tokens = lex_all("/* block comment */let");
    assert_eq!(tokens[0].kind, TokenKind::Let);
}

#[test]
fn test_unterminated_block_comment() {
    let tokens = lex_all("/* unterminated");
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Unterminated block comment".to_string()
        ))
    );
}
