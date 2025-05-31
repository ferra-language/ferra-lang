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
    let input = "/* unterminated\nblock\ncomment"; // Spans multiple lines
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2); // Error + EOF
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Unterminated block comment: expected closing */ before end of file.".to_string()
        ))
    );
    assert_eq!(tokens[0].lexeme, input); // The whole thing is the lexeme of the error

    // Check span details
    // "/* unterminated\nblock\ncomment"
    //  ^----------------------------^
    // Line 1, Col 1, Offset 0 --> Line 3, Col 8, Offset 27 (length of string)
    let span = &tokens[0].span;
    assert_eq!(span.start.line, 1);
    assert_eq!(span.start.column, 1);
    assert_eq!(span.start.offset, 0);
    assert_eq!(span.end.line, 3); // Should end on line 3
    assert_eq!(span.end.column, 8); // Column after 't' in "comment"
    assert_eq!(span.end.offset, input.len()); // Offset is the length of the input string
}

#[test]
fn test_unterminated_block_comment_eof() {
    let input = "/* unterminated"; // Single line, ends at EOF
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2); // Error + EOF
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Unterminated block comment: expected closing */ before end of file.".to_string()
        ))
    );
    assert_eq!(tokens[0].lexeme, input);
    let span = &tokens[0].span;
    assert_eq!(span.start.line, 1);
    assert_eq!(span.start.column, 1);
    assert_eq!(span.start.offset, 0);
    assert_eq!(span.end.line, 1);
    assert_eq!(span.end.column, 16); // Column after 'd' in "unterminated"
    assert_eq!(span.end.offset, input.len());
}

#[test]
fn test_nested_block_comments() {
    let src = "let /* outer /* inner */ outer_again */ var;";
    let tokens = Lexer::new(src).lex();
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Var);
    assert_eq!(tokens[2].kind, TokenKind::Semicolon);
    assert_eq!(tokens[3].kind, TokenKind::Eof);
}

#[test]
fn test_unterminated_nested_block_comment() {
    let src = "let /* outer /* inner */ unterminated"; // Missing closing */ for outer
    let tokens = Lexer::new(src).lex();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Error);
    assert_eq!(tokens[1].lexeme, "/* outer /* inner */ unterminated");
    assert_eq!(tokens[2].kind, TokenKind::Eof);
}

#[test]
fn test_unterminated_inner_nested_block_comment() {
    let src = "let /* outer /* unterminated inner */ var;"; // Missing closing */ for inner
    let tokens = Lexer::new(src).lex();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Error);
    assert_eq!(tokens[1].lexeme, "/* outer /* unterminated inner */ var;");
    assert_eq!(tokens[2].kind, TokenKind::Eof);
}
