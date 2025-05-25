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
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Identifier, // a
            &TokenKind::Newline,
            &TokenKind::Indent,
            &TokenKind::Newline,
            &TokenKind::Indent,
            &TokenKind::Newline,
            &TokenKind::Dedent,
            &TokenKind::Newline,
            &TokenKind::Dedent,
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
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier, // a
            TokenKind::Newline,
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Slash,      // /
            TokenKind::Identifier, // comment
            TokenKind::Newline,
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Identifier, // c
            TokenKind::Eof,
        ]
    );
}
