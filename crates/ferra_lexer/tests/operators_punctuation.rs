use ferra_lexer::*;

// Helper function to lex the entire input string
fn lex_all(input: &str) -> Vec<Token> {
    Lexer::new(input).lex()
}

#[test]
fn test_single_char_operators() {
    let ops = vec![
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Star),
        ("/", TokenKind::Slash),
        ("=", TokenKind::Equal),
        (";", TokenKind::Semicolon),
        ("(", TokenKind::LParen),
        (")", TokenKind::RParen),
        ("{", TokenKind::LBrace),
        ("}", TokenKind::RBrace),
        (",", TokenKind::Comma),
        (":", TokenKind::Colon),
        // Add other single char ops as needed from TokenKind
        ("<", TokenKind::Less),
        (">", TokenKind::Greater),
        ("&", TokenKind::BitAnd),
        ("|", TokenKind::BitOr),
        ("^", TokenKind::Caret),
        ("!", TokenKind::Bang),
        ("?", TokenKind::Question),
        (".", TokenKind::Dot),
        ("_", TokenKind::Underscore),
        ("%", TokenKind::Percent),
    ];

    for (op_str, kind) in ops {
        let tokens = lex_all(op_str);
        assert_eq!(tokens.len(), 2, "Failed for op: {}", op_str);
        assert_eq!(tokens[0].kind, kind, "Mismatch in kind for {}", op_str);
        assert_eq!(
            tokens[0].lexeme, op_str,
            "Mismatch in lexeme for {}",
            op_str
        );
    }
}

#[test]
fn test_multi_char_operators() {
    let ops = vec![
        ("==", TokenKind::EqualEqual),
        ("!=", TokenKind::NotEqual),
        ("<=", TokenKind::LessEqual),
        (">=", TokenKind::GreaterEqual),
        ("&&", TokenKind::LogicalAnd),
        ("||", TokenKind::LogicalOr),
        ("+=", TokenKind::PlusEqual),
        ("-=", TokenKind::MinusEqual),
        ("*=", TokenKind::StarEqual),
        ("/=", TokenKind::SlashEqual),
        ("%=", TokenKind::PercentEqual),
        ("&=", TokenKind::BitAndEqual),
        ("|=", TokenKind::BitOrEqual),
        ("^=", TokenKind::CaretEqual),
        ("<<=", TokenKind::ShiftLeftEqual),
        ("<<", TokenKind::ShiftLeft),
        (">>=", TokenKind::ShiftRightEqual),
        (">>", TokenKind::ShiftRight),
        ("->", TokenKind::Arrow),
        ("=>", TokenKind::FatArrow),
        ("..=", TokenKind::DotDotEqual),
        ("..", TokenKind::DotDot),
        ("::", TokenKind::PathSep),
        ("??", TokenKind::Coalesce),
    ];

    for (op_str, kind) in ops {
        let tokens = lex_all(op_str);
        assert_eq!(tokens.len(), 2, "Failed for op: {}", op_str);
        assert_eq!(tokens[0].kind, kind, "Mismatch in kind for {}", op_str);
        assert_eq!(
            tokens[0].lexeme, op_str,
            "Mismatch in lexeme for {}",
            op_str
        );
    }
    // Specific test for >> because > is also a token
    let tokens_shr = lex_all(">>");
    assert_eq!(tokens_shr.len(), 2);
    assert_eq!(tokens_shr[0].kind, TokenKind::ShiftRight);
    assert_eq!(tokens_shr[0].lexeme, ">>");
}

#[test]
fn test_mixed_operators_and_others() {
    let input = "let x = a + b == c && d - 1.0;";
    let kinds: Vec<TokenKind> = lex_all(input).into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Identifier, // x
            TokenKind::Equal,
            TokenKind::Identifier, // a
            TokenKind::Plus,
            TokenKind::Identifier, // b
            TokenKind::EqualEqual,
            TokenKind::Identifier, // c
            TokenKind::LogicalAnd,
            TokenKind::Identifier, // d
            TokenKind::Minus,
            TokenKind::FloatLiteral, // 1.0
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}
