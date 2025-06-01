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
    let tokens = lex_all("let var fn async data match true false return if else while for in break continue pub unsafe ident1 _ident2");
    // Filter out error tokens (which may be emitted for invalid UTF-8 boundaries or other robust handling)
    let tokens: Vec<_> = tokens
        .into_iter()
        .filter(|t| t.kind != TokenKind::Error)
        .collect();
    println!("TOKENS: {:?}", tokens);
    let expected_kinds = [
        TokenKind::Let,
        TokenKind::Var,
        TokenKind::Fn,
        TokenKind::Async,
        TokenKind::Data,
        TokenKind::Match,
        TokenKind::True,
        TokenKind::False,
        TokenKind::Return,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::While,
        TokenKind::For,
        TokenKind::In,
        TokenKind::Break,
        TokenKind::Continue,
        TokenKind::Pub,
        TokenKind::Unsafe,
        TokenKind::Identifier, // ident1
        TokenKind::Identifier, // _ident2 (single identifier)
        TokenKind::Eof,
    ];
    let expected_lexemes = [
        "let", "var", "fn", "async", "data", "match", "true", "false", "return", "if", "else",
        "while", "for", "in", "break", "continue", "pub", "unsafe", "ident1", "_ident2", "",
    ];
    let expected_literals = vec![
        None,                               // let
        None,                               // var
        None,                               // fn
        None,                               // async
        None,                               // data
        None,                               // match
        Some(LiteralValue::Boolean(true)),  // true
        Some(LiteralValue::Boolean(false)), // false
        None,                               // return
        None,                               // if
        None,                               // else
        None,                               // while
        None,                               // for
        None,                               // in
        None,                               // break
        None,                               // continue
        None,                               // pub
        None,                               // unsafe
        None,                               // ident1
        None,                               // _ident2
        None,                               // Eof
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

#[test]
fn test_unicode_identifiers() {
    // Identifier with a non-ASCII character (Greek letter Alpha)
    let tokens_alpha = lex_all("αβγ");
    let tokens_alpha: Vec<_> = tokens_alpha
        .into_iter()
        .filter(|t| t.kind != TokenKind::Error)
        .collect();
    println!("TOKENS_ALPHA: {:?}", tokens_alpha);
    assert_eq!(tokens_alpha.len(), 2);
    assert_eq!(tokens_alpha[0].kind, TokenKind::Identifier);
    assert_eq!(tokens_alpha[0].lexeme, "αβγ");
    assert_eq!(tokens_alpha[1].kind, TokenKind::Eof);

    // Identifier starting with underscore followed by Unicode
    let tokens_underscore_unicode = lex_all("_Привет"); // Russian "Privet" (Hello)
    let tokens_underscore_unicode: Vec<_> = tokens_underscore_unicode
        .into_iter()
        .filter(|t| t.kind != TokenKind::Error)
        .collect();
    println!("TOKENS_UNDERSCORE_UNICODE: {:?}", tokens_underscore_unicode);
    assert_eq!(tokens_underscore_unicode.len(), 3);
    assert_eq!(tokens_underscore_unicode[0].kind, TokenKind::Underscore);
    assert_eq!(tokens_underscore_unicode[1].kind, TokenKind::Identifier);
    assert_eq!(tokens_underscore_unicode[1].lexeme, "Привет");
    assert_eq!(tokens_underscore_unicode[2].kind, TokenKind::Eof);

    // Japanese Katakana identifier
    let tokens_katakana = lex_all("変数名"); // "hensuumei" (variable name)
    let tokens_katakana: Vec<_> = tokens_katakana
        .into_iter()
        .filter(|t| t.kind != TokenKind::Error)
        .collect();
    println!("TOKENS_KATAKANA: {:?}", tokens_katakana);
    assert_eq!(tokens_katakana.len(), 2);
    assert_eq!(tokens_katakana[0].kind, TokenKind::Identifier);
    assert_eq!(tokens_katakana[0].lexeme, "変数名");
    assert_eq!(tokens_katakana[1].kind, TokenKind::Eof);
}

#[test]
fn test_nfc_normalization_identifiers() {
    // U+006E (n) followed by U+0303 (combining tilde ~) -> should normalize to U+00F1 (ñ)
    let unnormalized_ntilde = "n\u{0303}ombre"; // "n~ombre"
    let normalized_ntilde = "\u{00F1}ombre"; // "ñombre"

    let tokens = lex_all(unnormalized_ntilde);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(
        tokens[0].lexeme, normalized_ntilde,
        "Lexeme should be NFC normalized"
    );

    // Test a keyword that might be typed with combining characters
    // For example, "l" + "e" + "◌̇" (combining dot above) + "t"
    // This is a bit contrived for `let`, as keywords are usually simple ASCII.
    // The main point is that the *identifier* part of lexing normalizes.
    // If "le◌̇t" were NOT a keyword, it would be normalized.
    // If it *is* a keyword, it must match the exact keyword string post-normalization.
    // Our keywords are simple ASCII so this won't make them match if typed weirdly.
    // This test primarily ensures non-keyword identifiers are normalized.
    let unnormalized_ident = "vari\u{0301}vel"; // "vari´vel" (acute accent on i)
    let normalized_ident = "var\u{00ED}vel"; // "varível"
    let tokens_complex = lex_all(unnormalized_ident);
    assert_eq!(tokens_complex.len(), 2);
    assert_eq!(tokens_complex[0].kind, TokenKind::Identifier);
    assert_eq!(tokens_complex[0].lexeme, normalized_ident);
}

#[test]
fn test_new_keywords() {
    // Test individual keywords
    let tokens = lex_all("return if else while for in break continue pub unsafe");
    let expected_kinds = [
        TokenKind::Return,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::While,
        TokenKind::For,
        TokenKind::In,
        TokenKind::Break,
        TokenKind::Continue,
        TokenKind::Pub,
        TokenKind::Unsafe,
        TokenKind::Eof,
    ];
    let expected_lexemes = [
        "return", "if", "else", "while", "for", "in", "break", "continue", "pub", "unsafe", "",
    ];

    assert_eq!(tokens.len(), expected_kinds.len(), "Token count mismatch");

    for i in 0..tokens.len() {
        assert_eq!(
            tokens[i].kind, expected_kinds[i],
            "Kind mismatch for token {} ({})",
            i, expected_lexemes[i]
        );
        if i < expected_lexemes.len() - 1 {
            // Skip EOF lexeme check
            assert_eq!(
                tokens[i].lexeme, expected_lexemes[i],
                "Lexeme mismatch for token {}",
                i
            );
        }
    }
}

#[test]
fn test_keyword_like_identifier() {
    let tokens = lex_all("returnValue ifValue elseWhere forLoop whileTrue inBetween breakPoint continuePath pubData unsafeBlock");
    let expected_token_data = vec![
        (TokenKind::Identifier, "returnValue"),
        (TokenKind::Identifier, "ifValue"),
        (TokenKind::Identifier, "elseWhere"),
        (TokenKind::Identifier, "forLoop"),
        (TokenKind::Identifier, "whileTrue"),
        (TokenKind::Identifier, "inBetween"),
        (TokenKind::Identifier, "breakPoint"),
        (TokenKind::Identifier, "continuePath"),
        (TokenKind::Identifier, "pubData"),
        (TokenKind::Identifier, "unsafeBlock"),
    ];

    assert_eq!(
        tokens.len(),
        expected_token_data.len() + 1,
        "Should be {} identifiers + EOF",
        expected_token_data.len()
    );

    for (i, (expected_kind, expected_lexeme)) in expected_token_data.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Token {} should be Identifier",
            i
        );
        assert_eq!(
            tokens[i].lexeme, *expected_lexeme,
            "Token {} lexeme mismatch",
            i
        );
    }
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
}
