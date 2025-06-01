//! Integration tests for expression parsing using the Pratt parser

use ferra_parser::{
    ast::{Arena, BinaryOperator, Expression, Literal, Pattern, UnaryOperator},
    pratt::parser::PrattParser,
    token::{TokenType, VecTokenStream},
};

#[test]
fn test_basic_literal_parsing() {
    let arena = Arena::new();
    let tokens =
        VecTokenStream::from_token_types(vec![TokenType::IntegerLiteral(42), TokenType::Eof]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Literal(Literal::Integer(value)) => {
                assert_eq!(*value, 42);
            }
            _ => panic!("Expected integer literal, got {:?}", expr),
        }
    }
}

#[test]
fn test_string_literal_parsing() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::StringLiteral("hello".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Literal(Literal::String(value)) => {
                assert_eq!(value, "hello");
            }
            _ => panic!("Expected string literal, got {:?}", expr),
        }
    }
}

#[test]
fn test_identifier_parsing() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("variable".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Identifier(name) => {
                assert_eq!(name, "variable");
            }
            _ => panic!("Expected identifier, got {:?}", expr),
        }
    }
}

#[test]
fn test_simple_binary_expression() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Binary(binary) => {
                assert!(matches!(binary.operator, BinaryOperator::Add));

                // Check left operand
                match binary.left.as_ref() {
                    Expression::Literal(Literal::Integer(value)) => assert_eq!(*value, 1),
                    _ => panic!("Expected left operand to be integer 1"),
                }

                // Check right operand
                match binary.right.as_ref() {
                    Expression::Literal(Literal::Integer(value)) => assert_eq!(*value, 2),
                    _ => panic!("Expected right operand to be integer 2"),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}

#[test]
fn test_unary_expression() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Minus,
        TokenType::IntegerLiteral(42),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Unary(unary) => {
                assert!(matches!(unary.operator, UnaryOperator::Minus));

                // Check operand
                match unary.operand.as_ref() {
                    Expression::Literal(Literal::Integer(value)) => assert_eq!(*value, 42),
                    _ => panic!("Expected operand to be integer 42"),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }
}

#[test]
fn test_grouped_expression() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftParen,
        TokenType::IntegerLiteral(42),
        TokenType::RightParen,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Grouped(inner) => match inner.as_ref() {
                Expression::Literal(Literal::Integer(value)) => assert_eq!(*value, 42),
                _ => panic!("Expected grouped expression to contain integer 42"),
            },
            _ => panic!("Expected grouped expression, got {:?}", expr),
        }
    }
}

#[test]
fn test_precedence_parsing() {
    let arena = Arena::new();

    // Test: 1 + 2 * 3 should parse as 1 + (2 * 3)
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::Star,
        TokenType::IntegerLiteral(3),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Binary(binary) => {
                assert!(matches!(binary.operator, BinaryOperator::Add));
                // Left should be 1
                match binary.left.as_ref() {
                    Expression::Literal(Literal::Integer(1)) => {}
                    _ => panic!("Expected left operand to be 1"),
                }
                // Right should be (2 * 3)
                match binary.right.as_ref() {
                    Expression::Binary(right_binary) => {
                        assert!(matches!(right_binary.operator, BinaryOperator::Mul));
                        match (right_binary.left.as_ref(), right_binary.right.as_ref()) {
                            (
                                Expression::Literal(Literal::Integer(2)),
                                Expression::Literal(Literal::Integer(3)),
                            ) => {}
                            _ => panic!("Expected right operand to be (2 * 3)"),
                        }
                    }
                    _ => panic!("Expected right operand to be binary expression"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }
}

#[test]
fn test_left_associativity() {
    let arena = Arena::new();

    // Test: 1 - 2 - 3 should parse as (1 - 2) - 3
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::IntegerLiteral(1),
        TokenType::Minus,
        TokenType::IntegerLiteral(2),
        TokenType::Minus,
        TokenType::IntegerLiteral(3),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Binary(binary) => {
                assert!(matches!(binary.operator, BinaryOperator::Sub));
                // Left should be (1 - 2)
                match binary.left.as_ref() {
                    Expression::Binary(left_binary) => {
                        assert!(matches!(left_binary.operator, BinaryOperator::Sub));
                        match (left_binary.left.as_ref(), left_binary.right.as_ref()) {
                            (
                                Expression::Literal(Literal::Integer(1)),
                                Expression::Literal(Literal::Integer(2)),
                            ) => {}
                            _ => panic!("Expected left operand to be (1 - 2)"),
                        }
                    }
                    _ => panic!("Expected left operand to be binary expression"),
                }
                // Right should be 3
                match binary.right.as_ref() {
                    Expression::Literal(Literal::Integer(3)) => {}
                    _ => panic!("Expected right operand to be 3"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }
}

#[test]
fn test_comparison_operators() {
    let arena = Arena::new();

    // Test: a == b
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("a".to_string()),
        TokenType::EqualEqual,
        TokenType::Identifier("b".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Binary(binary) => {
                assert!(matches!(binary.operator, BinaryOperator::Equal));
            }
            _ => panic!("Expected binary expression"),
        }
    }
}

#[test]
fn test_logical_operators() {
    let arena = Arena::new();

    // Test: a && b || c
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("a".to_string()),
        TokenType::AmpAmp,
        TokenType::Identifier("b".to_string()),
        TokenType::PipePipe,
        TokenType::Identifier("c".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Binary(binary) => {
                assert!(matches!(binary.operator, BinaryOperator::Or));
                // Left should be (a && b)
                match binary.left.as_ref() {
                    Expression::Binary(left_binary) => {
                        assert!(matches!(left_binary.operator, BinaryOperator::And));
                    }
                    _ => panic!("Expected left operand to be (a && b)"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }
}

#[test]
fn test_complex_nested_expression() {
    let arena = Arena::new();

    // Test: (1 + 2) * 3
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::RightParen,
        TokenType::Star,
        TokenType::IntegerLiteral(3),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Binary(binary) => {
                assert!(matches!(binary.operator, BinaryOperator::Mul));
                // Left should be (1 + 2)
                match binary.left.as_ref() {
                    Expression::Grouped(grouped) => match grouped.as_ref() {
                        Expression::Binary(inner_binary) => {
                            assert!(matches!(inner_binary.operator, BinaryOperator::Add));
                        }
                        _ => panic!("Expected grouped expression to contain binary expression"),
                    },
                    _ => panic!("Expected left operand to be grouped expression"),
                }
                // Right should be 3
                match binary.right.as_ref() {
                    Expression::Literal(Literal::Integer(3)) => {}
                    _ => panic!("Expected right operand to be 3"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }
}

#[test]
fn test_multiple_unary_operators() {
    let arena = Arena::new();

    // Test: --42 (double negative)
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Minus,
        TokenType::Minus,
        TokenType::IntegerLiteral(42),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Unary(unary) => {
                assert!(matches!(unary.operator, UnaryOperator::Minus));
                // Operand should be another unary expression
                match unary.operand.as_ref() {
                    Expression::Unary(inner_unary) => {
                        assert!(matches!(inner_unary.operator, UnaryOperator::Minus));
                        match inner_unary.operand.as_ref() {
                            Expression::Literal(Literal::Integer(42)) => {}
                            _ => panic!("Expected inner operand to be 42"),
                        }
                    }
                    _ => panic!("Expected operand to be unary expression"),
                }
            }
            _ => panic!("Expected unary expression"),
        }
    }
}

#[test]
fn test_boolean_literals() {
    let arena = Arena::new();

    // Test true
    let tokens =
        VecTokenStream::from_token_types(vec![TokenType::BooleanLiteral(true), TokenType::Eof]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Literal(Literal::Boolean(true)) => {}
            _ => panic!("Expected boolean literal true"),
        }
    }

    // Test false
    let tokens =
        VecTokenStream::from_token_types(vec![TokenType::BooleanLiteral(false), TokenType::Eof]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Literal(Literal::Boolean(false)) => {}
            _ => panic!("Expected boolean literal false"),
        }
    }
}

#[test]
fn test_float_literals() {
    let arena = Arena::new();

    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::FloatLiteral(std::f64::consts::PI),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);

    assert!(result.is_ok());
    if let Ok(expr) = result {
        match expr {
            Expression::Literal(Literal::Float(value)) => {
                // Use an appropriate tolerance for floating point comparison
                assert!((*value - std::f64::consts::PI).abs() < f64::EPSILON);
            }
            _ => panic!("Expected float literal"),
        }
    }
}

// Tests for Phase 2.2.2 - Advanced Primary Expressions

#[test]
fn test_qualified_identifier() {
    let arena = Arena::new();

    // Test: module.function
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("module".to_string()),
        TokenType::Dot,
        TokenType::Identifier("function".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::MemberAccess(member_access) => {
                assert_eq!(member_access.member, "function");
                match member_access.object.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "module"),
                    _ => panic!("Expected object to be identifier 'module'"),
                }
            }
            _ => panic!("Expected member access, got {:?}", expr),
        }
    }
}

#[test]
fn test_deeply_qualified_identifier() {
    let arena = Arena::new();

    // Test: std.collections.HashMap
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("std".to_string()),
        TokenType::Dot,
        TokenType::Identifier("collections".to_string()),
        TokenType::Dot,
        TokenType::Identifier("HashMap".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::MemberAccess(outer_access) => {
                assert_eq!(outer_access.member, "HashMap");
                match outer_access.object.as_ref() {
                    Expression::MemberAccess(inner_access) => {
                        assert_eq!(inner_access.member, "collections");
                        match inner_access.object.as_ref() {
                            Expression::Identifier(name) => assert_eq!(name, "std"),
                            _ => panic!("Expected base object to be 'std'"),
                        }
                    }
                    _ => panic!("Expected inner object to be member access"),
                }
            }
            _ => panic!("Expected member access, got {:?}", expr),
        }
    }
}

#[test]
fn test_array_literals() {
    let arena = Arena::new();

    // Test empty array: []
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBracket,
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Array(array) => {
                assert_eq!(array.elements.len(), 0);
            }
            _ => panic!("Expected array literal, got {:?}", expr),
        }
    }

    // Test array with elements: [1, 2, 3]
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(1),
        TokenType::Comma,
        TokenType::IntegerLiteral(2),
        TokenType::Comma,
        TokenType::IntegerLiteral(3),
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Array(array) => {
                assert_eq!(array.elements.len(), 3);
                // Verify first element is 1
                match &array.elements[0] {
                    Expression::Literal(Literal::Integer(1)) => {}
                    _ => panic!("Expected first element to be integer 1"),
                }
            }
            _ => panic!("Expected array literal, got {:?}", expr),
        }
    }
}

#[test]
fn test_array_with_trailing_comma() {
    let arena = Arena::new();

    // Test: [1, 2,]
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(1),
        TokenType::Comma,
        TokenType::IntegerLiteral(2),
        TokenType::Comma,
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Array(array) => {
                assert_eq!(array.elements.len(), 2);
            }
            _ => panic!("Expected array literal, got {:?}", expr),
        }
    }
}

// Tests for Phase 2.2.3 - Postfix Operators

#[test]
fn test_function_calls() {
    let arena = Arena::new();

    // Test: func()
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("func".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Call(call) => {
                assert_eq!(call.arguments.len(), 0);
                match call.callee.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "func"),
                    _ => panic!("Expected callee to be identifier 'func'"),
                }
            }
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    // Test: func(1, 2)
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("func".to_string()),
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Comma,
        TokenType::IntegerLiteral(2),
        TokenType::RightParen,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Call(call) => {
                assert_eq!(call.arguments.len(), 2);
                match &call.arguments[0] {
                    Expression::Literal(Literal::Integer(1)) => {}
                    _ => panic!("Expected first argument to be integer 1"),
                }
                match &call.arguments[1] {
                    Expression::Literal(Literal::Integer(2)) => {}
                    _ => panic!("Expected second argument to be integer 2"),
                }
            }
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }
}

#[test]
fn test_member_access() {
    let arena = Arena::new();

    // Test: obj.field
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("obj".to_string()),
        TokenType::Dot,
        TokenType::Identifier("field".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::MemberAccess(member_access) => {
                assert_eq!(member_access.member, "field");
                match member_access.object.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "obj"),
                    _ => panic!("Expected object to be identifier 'obj'"),
                }
            }
            _ => panic!("Expected member access, got {:?}", expr),
        }
    }
}

#[test]
fn test_index_expressions() {
    let arena = Arena::new();

    // Test: arr[0]
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("arr".to_string()),
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(0),
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Index(index) => {
                match index.object.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "arr"),
                    _ => panic!("Expected object to be identifier 'arr'"),
                }
                match index.index.as_ref() {
                    Expression::Literal(Literal::Integer(0)) => {}
                    _ => panic!("Expected index to be integer 0"),
                }
            }
            _ => panic!("Expected index expression, got {:?}", expr),
        }
    }
}

#[test]
fn test_chained_postfix_operations() {
    let arena = Arena::new();

    // Test: obj.method()[0]
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("obj".to_string()),
        TokenType::Dot,
        TokenType::Identifier("method".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(0),
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expr) = result {
        match expr {
            Expression::Index(index) => {
                // The object should be a function call
                match index.object.as_ref() {
                    Expression::Call(call) => {
                        // The callee should be member access
                        match call.callee.as_ref() {
                            Expression::MemberAccess(member_access) => {
                                assert_eq!(member_access.member, "method");
                                match member_access.object.as_ref() {
                                    Expression::Identifier(name) => assert_eq!(name, "obj"),
                                    _ => panic!("Expected base object to be 'obj'"),
                                }
                            }
                            _ => panic!("Expected callee to be member access"),
                        }
                    }
                    _ => panic!("Expected object to be function call"),
                }
            }
            _ => panic!("Expected index expression, got {:?}", expr),
        }
    }
}

// Tests for Phase 2.2.4 - Pattern Parsing

#[test]
fn test_literal_patterns() {
    let arena = Arena::new();

    // Test integer literal pattern
    let tokens =
        VecTokenStream::from_token_types(vec![TokenType::IntegerLiteral(42), TokenType::Eof]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Literal(Literal::Integer(42)) => {}
            _ => panic!("Expected integer literal pattern 42, got {:?}", pattern),
        }
    }

    // Test string literal pattern
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::StringLiteral("hello".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_identifier_patterns() {
    let arena = Arena::new();

    // Test simple identifier pattern
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("x".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Identifier(name) => assert_eq!(name, "x"),
            _ => panic!("Expected identifier pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_wildcard_pattern() {
    let arena = Arena::new();

    // Test wildcard pattern
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("_".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Wildcard => {}
            _ => panic!("Expected wildcard pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_data_class_patterns() {
    let arena = Arena::new();

    // Test empty data class pattern: Person {}
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("Person".to_string()),
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::DataClass(data_class) => {
                assert_eq!(data_class.name, "Person");
                assert_eq!(data_class.fields.len(), 0);
                assert!(!data_class.has_rest);
            }
            _ => panic!("Expected data class pattern, got {:?}", pattern),
        }
    }

    // Test data class pattern with fields: Person { name, age }
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("Person".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("name".to_string()),
        TokenType::Comma,
        TokenType::Identifier("age".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::DataClass(data_class) => {
                assert_eq!(data_class.name, "Person");
                assert_eq!(data_class.fields.len(), 2);
                assert_eq!(data_class.fields[0].name, "name");
                assert!(data_class.fields[0].pattern.is_none());
                assert_eq!(data_class.fields[1].name, "age");
                assert!(data_class.fields[1].pattern.is_none());
            }
            _ => panic!("Expected data class pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_data_class_pattern_with_bindings() {
    let arena = Arena::new();

    // Test: Person { name: n, age: 25 }
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("Person".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("name".to_string()),
        TokenType::Colon,
        TokenType::Identifier("n".to_string()),
        TokenType::Comma,
        TokenType::Identifier("age".to_string()),
        TokenType::Colon,
        TokenType::IntegerLiteral(25),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::DataClass(data_class) => {
                assert_eq!(data_class.name, "Person");
                assert_eq!(data_class.fields.len(), 2);

                // Check first field: name: n
                assert_eq!(data_class.fields[0].name, "name");
                assert!(data_class.fields[0].pattern.is_some());
                match &data_class.fields[0].pattern {
                    Some(Pattern::Identifier(name)) => assert_eq!(name, "n"),
                    _ => panic!("Expected identifier pattern for name field"),
                }

                // Check second field: age: 25
                assert_eq!(data_class.fields[1].name, "age");
                assert!(data_class.fields[1].pattern.is_some());
                match &data_class.fields[1].pattern {
                    Some(Pattern::Literal(Literal::Integer(25))) => {}
                    _ => panic!("Expected integer literal pattern for age field"),
                }
            }
            _ => panic!("Expected data class pattern, got {:?}", pattern),
        }
    }
}
