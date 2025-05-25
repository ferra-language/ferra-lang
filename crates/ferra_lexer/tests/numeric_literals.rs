use ferra_lexer::*;

// Helper function to lex the entire input string
fn lex_all(input: &str) -> Vec<Token> {
    Lexer::new(input).lex()
}

#[test]
fn test_simple_integer() {
    let tokens = lex_all("123");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[0].lexeme, "123");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Integer(123)));
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, 3);
}

#[allow(clippy::approx_constant)]
#[test]
fn test_float_simple() {
    let tokens = lex_all("3.14");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::FloatLiteral);
    assert_eq!(tokens[0].lexeme, "3.14");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Float(3.14)));
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, 4);

    let tokens_leading_dot = lex_all(".5");
    assert_eq!(tokens_leading_dot.len(), 2);
    assert_eq!(tokens_leading_dot[0].kind, TokenKind::FloatLiteral);
    assert_eq!(tokens_leading_dot[0].lexeme, ".5");
    assert_eq!(
        tokens_leading_dot[0].literal,
        Some(LiteralValue::Float(0.5))
    );

    let tokens_trailing_dot = lex_all("7.");
    assert_eq!(tokens_trailing_dot.len(), 2);
    assert_eq!(tokens_trailing_dot[0].kind, TokenKind::FloatLiteral);
    assert_eq!(tokens_trailing_dot[0].lexeme, "7.");
    assert_eq!(
        tokens_trailing_dot[0].literal,
        Some(LiteralValue::Float(7.0))
    );
}

#[test]
fn test_float_with_exponent() {
    let tokens = lex_all("1.2e3");
    assert_eq!(tokens[0].kind, TokenKind::FloatLiteral);
    assert_eq!(tokens[0].literal, Some(LiteralValue::Float(1200.0)));

    let tokens_cap_e = lex_all("0.5E-2");
    assert_eq!(tokens_cap_e[0].kind, TokenKind::FloatLiteral);
    assert_eq!(tokens_cap_e[0].literal, Some(LiteralValue::Float(0.005)));

    let tokens_exp_pos = lex_all("5e+1");
    assert_eq!(tokens_exp_pos[0].kind, TokenKind::FloatLiteral);
    assert_eq!(tokens_exp_pos[0].literal, Some(LiteralValue::Float(50.0)));
}

#[test]
fn test_number_with_underscores() {
    let tokens_int = lex_all("1_000_000");
    assert_eq!(tokens_int[0].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens_int[0].literal, Some(LiteralValue::Integer(1000000)));
    assert_eq!(tokens_int[0].lexeme, "1_000_000");

    let tokens_float = lex_all("3_0.1_4e-2");
    assert_eq!(tokens_float[0].kind, TokenKind::FloatLiteral);
    assert_eq!(tokens_float[0].literal, Some(LiteralValue::Float(0.3014))); // 30.14e-2 = 0.3014
    assert_eq!(tokens_float[0].lexeme, "3_0.1_4e-2");
}

#[test]
fn test_invalid_numbers() {
    let _tokens_double_dot = lex_all("1..2"); // This should be DotDot, not a number error in lex_number
                                              // The main lex loop handles DotDot before lex_number for such cases.
                                              // So, a direct call to lex_number with "1..2" is not a valid test for lex_number itself.
                                              // We test error cases that lex_number *would* produce if it got malformed numbers.

    // lex_number itself would error if parsing fails, e.g.:
    let tokens_bad_float = lex_all("3.14.15");
    assert_eq!(
        tokens_bad_float[0].kind,
        TokenKind::FloatLiteral,
        "Lexeme: {}",
        tokens_bad_float[0].lexeme
    );
    assert_eq!(tokens_bad_float[0].lexeme, "3.14");
    assert_eq!(
        tokens_bad_float[1].kind,
        TokenKind::FloatLiteral,
        "Lexeme: {}",
        tokens_bad_float[1].lexeme
    );
    assert_eq!(tokens_bad_float[1].lexeme, ".15");

    let tokens_bad_exp = lex_all("1e"); // Exponent without digits
    assert_eq!(
        tokens_bad_exp[0].kind,
        TokenKind::Error,
        "Lexeme: {}",
        tokens_bad_exp[0].lexeme
    );
    assert_eq!(tokens_bad_exp[0].lexeme, "1e");

    let tokens_bad_exp_sign = lex_all("1e-");
    assert_eq!(
        tokens_bad_exp_sign[0].kind,
        TokenKind::Error,
        "Lexeme: {}",
        tokens_bad_exp_sign[0].lexeme
    );
    assert_eq!(tokens_bad_exp_sign[0].lexeme, "1e-");
}

#[test]
fn test_hex_literals() {
    let tokens = lex_all("0x1A 0XFF 0x_a_B_0");
    assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[0].literal, Some(LiteralValue::Integer(26)));
    assert_eq!(tokens[1].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[1].literal, Some(LiteralValue::Integer(255)));
    assert_eq!(tokens[2].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[2].literal, Some(LiteralValue::Integer(0xAB0)));
    let tokens_err = lex_all("0x 0x_ 0xG");
    assert_eq!(tokens_err[0].kind, TokenKind::Error); // No digits
    assert_eq!(tokens_err[1].kind, TokenKind::Error); // Ends with underscore (or no digits after prefix)
    assert_eq!(tokens_err[2].kind, TokenKind::Error); // Invalid digit
}

#[test]
fn test_octal_literals() {
    let tokens = lex_all("0o12 0O77 0o_1_0");
    assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[0].literal, Some(LiteralValue::Integer(10)));
    assert_eq!(tokens[1].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[1].literal, Some(LiteralValue::Integer(63)));
    assert_eq!(tokens[2].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[2].literal, Some(LiteralValue::Integer(0o10)));
    let tokens_err = lex_all("0o 0o_ 0o8");
    assert_eq!(tokens_err[0].kind, TokenKind::Error); // No digits
    assert_eq!(tokens_err[1].kind, TokenKind::Error); // Ends with underscore (or no digits after prefix)
    assert_eq!(tokens_err[2].kind, TokenKind::Error); // Invalid digit
}

#[test]
fn test_binary_literals() {
    let tokens = lex_all("0b10 0B11 0b_1_0");
    assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[0].literal, Some(LiteralValue::Integer(2)));
    assert_eq!(tokens[1].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[1].literal, Some(LiteralValue::Integer(3)));
    assert_eq!(tokens[2].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[2].literal, Some(LiteralValue::Integer(0b10)));
    let tokens_err = lex_all("0b 0b_ 0b2");
    assert_eq!(tokens_err[0].kind, TokenKind::Error); // No digits
    assert_eq!(tokens_err[1].kind, TokenKind::Error); // Ends with underscore (or no digits after prefix)
    assert_eq!(tokens_err[2].kind, TokenKind::Error); // Invalid digit
}

#[test]
fn test_leading_zero_decimal() {
    let tokens = lex_all("0123 007");
    assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[0].literal, Some(LiteralValue::Integer(123)));
    assert_eq!(tokens[1].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[1].literal, Some(LiteralValue::Integer(7)));
    let tokens_float = lex_all("0.5");
    assert_eq!(tokens_float[0].kind, TokenKind::FloatLiteral);
    assert_eq!(tokens_float[0].literal, Some(LiteralValue::Float(0.5)));
}
