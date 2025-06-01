//! Additional comprehensive tests to restore missing Phase 2.1 coverage

use ferra_parser::{
    ast::{Arena, BinaryOperator, Expression, Item, Literal, Statement, UnaryOperator},
    pratt::parser::PrattParser,
    statement::parser::StatementParser,
    token::{TokenType, VecTokenStream},
    Parser,
};

// Test 1: Advanced AST Construction Tests
#[test]
fn test_compilation_unit_with_multiple_items() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Fn,
        TokenType::Identifier("func1".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Data,
        TokenType::Identifier("Point".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);

    let mut parser = Parser::new(&arena, tokens);
    let result = parser.parse_compilation_unit();
    assert!(result.is_ok());

    if let Ok(compilation_unit) = result {
        assert_eq!(compilation_unit.items.len(), 2);

        // First item should be function
        if let Item::FunctionDecl(func) = &compilation_unit.items[0] {
            assert_eq!(func.name, "func1");
        } else {
            panic!("Expected function declaration");
        }

        // Second item should be data class
        if let Item::DataClassDecl(data_class) = &compilation_unit.items[1] {
            assert_eq!(data_class.name, "Point");
            assert_eq!(data_class.fields.len(), 1);
        } else {
            panic!("Expected data class declaration");
        }
    }
}

// Test 2: Complex Expression Precedence
#[test]
fn test_complex_precedence_chain() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::Star,
        TokenType::IntegerLiteral(3),
        TokenType::EqualEqual,
        TokenType::IntegerLiteral(6),
        TokenType::AmpAmp,
        TokenType::BooleanLiteral(true),
        TokenType::Eof,
    ]);

    let mut pratt_parser = PrattParser::new(&arena, tokens);
    let result = pratt_parser.parse_expression(0);
    assert!(result.is_ok());

    // Should parse as: ((1 + (2 * 3)) == 6) && true
    if let Ok(Expression::Binary(and_expr)) = result {
        assert!(matches!(and_expr.operator, BinaryOperator::And));

        // Left side should be equality comparison
        if let Expression::Binary(eq_expr) = and_expr.left.as_ref() {
            assert!(matches!(eq_expr.operator, BinaryOperator::Equal));

            // Left of equality should be addition
            if let Expression::Binary(add_expr) = eq_expr.left.as_ref() {
                assert!(matches!(add_expr.operator, BinaryOperator::Add));

                // Right of addition should be multiplication
                if let Expression::Binary(mul_expr) = add_expr.right.as_ref() {
                    assert!(matches!(mul_expr.operator, BinaryOperator::Mul));
                } else {
                    panic!("Expected multiplication in precedence chain");
                }
            } else {
                panic!("Expected addition in precedence chain");
            }
        } else {
            panic!("Expected equality in precedence chain");
        }
    } else {
        panic!("Expected complex precedence expression");
    }
}

// Test 3: Nested Function Calls with Member Access
#[test]
fn test_nested_function_calls_with_member_access() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("obj".to_string()),
        TokenType::Dot,
        TokenType::Identifier("get_inner".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::Dot,
        TokenType::Identifier("process".to_string()),
        TokenType::LeftParen,
        TokenType::IntegerLiteral(42),
        TokenType::RightParen,
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(0),
        TokenType::RightBracket,
        TokenType::Eof,
    ]);

    let mut pratt_parser = PrattParser::new(&arena, tokens);
    let result = pratt_parser.parse_expression(0);
    assert!(result.is_ok());

    // Should parse as: obj.get_inner().process(42)[0]
    if let Ok(Expression::Index(index_expr)) = result {
        // Object should be obj.get_inner().process(42)
        if let Expression::Call(call_expr) = index_expr.object.as_ref() {
            assert_eq!(call_expr.arguments.len(), 1);

            // Callee should be obj.get_inner().process
            if let Expression::MemberAccess(member_expr) = call_expr.callee.as_ref() {
                assert_eq!(member_expr.member, "process");

                // Object should be obj.get_inner()
                if let Expression::Call(inner_call) = member_expr.object.as_ref() {
                    assert_eq!(inner_call.arguments.len(), 0);
                } else {
                    panic!("Expected inner function call");
                }
            } else {
                panic!("Expected member access for process");
            }
        } else {
            panic!("Expected function call in index expression");
        }
    } else {
        panic!("Expected index expression");
    }
}

// Test 4: Complex Array Literal with Expressions
#[test]
fn test_array_with_complex_expressions() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::Comma,
        TokenType::Identifier("func".to_string()),
        TokenType::LeftParen,
        TokenType::IntegerLiteral(3),
        TokenType::Star,
        TokenType::IntegerLiteral(4),
        TokenType::RightParen,
        TokenType::Comma,
        TokenType::Bang,
        TokenType::BooleanLiteral(false),
        TokenType::RightBracket,
        TokenType::Eof,
    ]);

    let mut pratt_parser = PrattParser::new(&arena, tokens);
    let result = pratt_parser.parse_expression(0);
    assert!(result.is_ok());

    // Should parse as: [1 + 2, func(3 * 4), !false]
    if let Ok(Expression::Array(array)) = result {
        assert_eq!(array.elements.len(), 3);

        // First element: 1 + 2
        if let Expression::Binary(binary) = &array.elements[0] {
            assert!(matches!(binary.operator, BinaryOperator::Add));
        } else {
            panic!("Expected binary expression in array");
        }

        // Second element: func(3 * 4)
        if let Expression::Call(call) = &array.elements[1] {
            assert_eq!(call.arguments.len(), 1);
            if let Expression::Binary(arg_binary) = &call.arguments[0] {
                assert!(matches!(arg_binary.operator, BinaryOperator::Mul));
            } else {
                panic!("Expected multiplication in function argument");
            }
        } else {
            panic!("Expected function call in array");
        }

        // Third element: !false
        if let Expression::Unary(unary) = &array.elements[2] {
            assert!(matches!(unary.operator, UnaryOperator::Not));
        } else {
            panic!("Expected unary expression in array");
        }
    } else {
        panic!("Expected array literal");
    }
}

// Test 5: Statement Integration with Complex Expressions
#[test]
fn test_variable_declaration_with_complex_initializer() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Let,
        TokenType::Identifier("result".to_string()),
        TokenType::Equal,
        TokenType::Identifier("calculate".to_string()),
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::RightParen,
        TokenType::Star,
        TokenType::IntegerLiteral(3),
        TokenType::Eof,
    ]);

    let mut statement_parser = StatementParser::new(&arena, tokens);
    let result = statement_parser.parse_statement();
    assert!(result.is_ok());

    // Should parse as: let result = calculate(1 + 2) * 3;
    if let Ok(Statement::VariableDecl(var_decl)) = result {
        assert_eq!(var_decl.name, "result");
        assert!(var_decl.initializer.is_some());

        if let Some(Expression::Binary(binary)) = &var_decl.initializer {
            assert!(matches!(binary.operator, BinaryOperator::Mul));

            // Left should be calculate(1 + 2)
            if let Expression::Call(call) = binary.left.as_ref() {
                assert_eq!(call.arguments.len(), 1);
                if let Expression::Binary(arg_binary) = &call.arguments[0] {
                    assert!(matches!(arg_binary.operator, BinaryOperator::Add));
                } else {
                    panic!("Expected addition in function call");
                }
            } else {
                panic!("Expected function call in variable initializer");
            }
        } else {
            panic!("Expected binary expression in variable initializer");
        }
    } else {
        panic!("Expected variable declaration");
    }
}

// Test 6: Error Handling with Specific Error Types
#[test]
fn test_expected_expression_error() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::RightParen, // Invalid - should be expression
        TokenType::Eof,
    ]);

    let mut pratt_parser = PrattParser::new(&arena, tokens);
    let result = pratt_parser.parse_expression(0);
    assert!(result.is_err());
}

// Test 7: Complex Control Flow Statement
#[test]
fn test_nested_if_with_complex_conditions() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::If,
        TokenType::Identifier("x".to_string()),
        TokenType::Greater,
        TokenType::IntegerLiteral(0),
        TokenType::AmpAmp,
        TokenType::Identifier("y".to_string()),
        TokenType::Less,
        TokenType::IntegerLiteral(10),
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::BooleanLiteral(true),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);

    let mut statement_parser = StatementParser::new(&arena, tokens);
    let result = statement_parser.parse_statement();
    assert!(result.is_ok());

    // Should parse as: if x > 0 && y < 10 { return true; }
    if let Ok(Statement::If(if_stmt)) = result {
        // Condition should be x > 0 && y < 10
        if let Expression::Binary(and_expr) = &if_stmt.condition {
            assert!(matches!(and_expr.operator, BinaryOperator::And));

            // Left should be x > 0
            if let Expression::Binary(left_binary) = and_expr.left.as_ref() {
                assert!(matches!(left_binary.operator, BinaryOperator::Greater));
            } else {
                panic!("Expected greater than comparison");
            }

            // Right should be y < 10
            if let Expression::Binary(right_binary) = and_expr.right.as_ref() {
                assert!(matches!(right_binary.operator, BinaryOperator::Less));
            } else {
                panic!("Expected less than comparison");
            }
        } else {
            panic!("Expected binary AND expression in if condition");
        }

        // Body should have return statement
        assert_eq!(if_stmt.then_block.statements.len(), 1);
        if let Statement::Return(_) = &if_stmt.then_block.statements[0] {
            // Success
        } else {
            panic!("Expected return statement in if body");
        }
    } else {
        panic!("Expected if statement");
    }
}

// Test 8: Multiple Unary Operators
#[test]
fn test_multiple_unary_operators() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Bang,
        TokenType::Bang,
        TokenType::BooleanLiteral(true),
        TokenType::Eof,
    ]);

    let mut pratt_parser = PrattParser::new(&arena, tokens);
    let result = pratt_parser.parse_expression(0);
    assert!(result.is_ok());

    // Should parse as: !!true
    if let Ok(Expression::Unary(outer_not)) = result {
        assert!(matches!(outer_not.operator, UnaryOperator::Not));

        if let Expression::Unary(inner_not) = outer_not.operand.as_ref() {
            assert!(matches!(inner_not.operator, UnaryOperator::Not));

            if let Expression::Literal(Literal::Boolean(true)) = inner_not.operand.as_ref() {
                // Success
            } else {
                panic!("Expected boolean literal true");
            }
        } else {
            panic!("Expected inner unary not");
        }
    } else {
        panic!("Expected outer unary not");
    }
}

// Test 9: Chained Assignment Operations
#[test]
fn test_chained_member_access_with_indexing() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("matrix".to_string()),
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(0),
        TokenType::RightBracket,
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(1),
        TokenType::RightBracket,
        TokenType::Dot,
        TokenType::Identifier("value".to_string()),
        TokenType::Eof,
    ]);

    let mut pratt_parser = PrattParser::new(&arena, tokens);
    let result = pratt_parser.parse_expression(0);
    assert!(result.is_ok());

    // Should parse as: matrix[0][1].value
    if let Ok(Expression::MemberAccess(member)) = result {
        assert_eq!(member.member, "value");

        // Object should be matrix[0][1]
        if let Expression::Index(outer_index) = member.object.as_ref() {
            // Object should be matrix[0]
            if let Expression::Index(inner_index) = outer_index.object.as_ref() {
                // Object should be matrix
                if let Expression::Identifier(name) = inner_index.object.as_ref() {
                    assert_eq!(name, "matrix");
                } else {
                    panic!("Expected matrix identifier");
                }
            } else {
                panic!("Expected inner index expression");
            }
        } else {
            panic!("Expected outer index expression");
        }
    } else {
        panic!("Expected member access expression");
    }
}

// Test 10: Parser State Management
#[test]
fn test_parser_sequential_parsing() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Let,
        TokenType::Identifier("x".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(1),
        TokenType::Let,
        TokenType::Identifier("y".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(2),
        TokenType::Eof,
    ]);

    let mut statement_parser = StatementParser::new(&arena, tokens);

    // Parse first statement
    let result1 = statement_parser.parse_statement();
    assert!(result1.is_ok());

    if let Ok(Statement::VariableDecl(var_decl)) = result1 {
        assert_eq!(var_decl.name, "x");
    } else {
        panic!("Expected first variable declaration");
    }

    // Parse second statement
    let result2 = statement_parser.parse_statement();
    assert!(result2.is_ok());

    if let Ok(Statement::VariableDecl(var_decl)) = result2 {
        assert_eq!(var_decl.name, "y");
    } else {
        panic!("Expected second variable declaration");
    }
}

// Test 11: Comprehensive Function Declaration
#[test]
fn test_comprehensive_function_with_all_features() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Pub,
        TokenType::Async,
        TokenType::Fn,
        TokenType::Identifier("advanced_func".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("param1".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::Comma,
        TokenType::Identifier("param2".to_string()),
        TokenType::Colon,
        TokenType::Identifier("String".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("bool".to_string()),
        TokenType::LeftBrace,
        TokenType::If,
        TokenType::Identifier("param1".to_string()),
        TokenType::Greater,
        TokenType::IntegerLiteral(0),
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::BooleanLiteral(true),
        TokenType::RightBrace,
        TokenType::Return,
        TokenType::BooleanLiteral(false),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);

    let mut statement_parser = StatementParser::new(&arena, tokens);
    let result = statement_parser.parse_item();
    assert!(result.is_ok());

    if let Ok(Item::FunctionDecl(func)) = result {
        assert_eq!(func.name, "advanced_func");
        assert!(func.is_async);
        assert!(func.modifiers.is_public);
        assert_eq!(func.parameters.len(), 2);
        assert!(func.return_type.is_some());
        assert!(func.body.is_some());

        // Check body has if statement and return
        if let Some(body) = &func.body {
            assert_eq!(body.statements.len(), 2);

            if let Statement::If(_) = &body.statements[0] {
                // Success
            } else {
                panic!("Expected if statement in function body");
            }

            if let Statement::Return(_) = &body.statements[1] {
                // Success
            } else {
                panic!("Expected return statement in function body");
            }
        }
    } else {
        panic!("Expected comprehensive function declaration");
    }
}

// Test 12: Error Recovery Testing
#[test]
fn test_basic_error_handling() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Let,
        TokenType::Equal, // Error: missing identifier
        TokenType::IntegerLiteral(42),
        TokenType::Eof,
    ]);

    let mut statement_parser = StatementParser::new(&arena, tokens);
    let result = statement_parser.parse_statement();
    assert!(result.is_err());

    // Verify we get a parse error
    if let Err(error) = result {
        // Just verify we get some kind of error
        assert!(!format!("{}", error).is_empty());
    }
}

// Test 13: Large Expression Tree
#[test]
fn test_large_expression_tree() {
    let arena = Arena::new();
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftParen,
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::RightParen,
        TokenType::Star,
        TokenType::LeftParen,
        TokenType::IntegerLiteral(3),
        TokenType::Plus,
        TokenType::IntegerLiteral(4),
        TokenType::RightParen,
        TokenType::RightParen,
        TokenType::Plus,
        TokenType::LeftParen,
        TokenType::IntegerLiteral(5),
        TokenType::Star,
        TokenType::IntegerLiteral(6),
        TokenType::RightParen,
        TokenType::Eof,
    ]);

    let mut pratt_parser = PrattParser::new(&arena, tokens);
    let result = pratt_parser.parse_expression(0);
    assert!(result.is_ok());

    // Should parse as: ((1 + 2) * (3 + 4)) + (5 * 6)
    if let Ok(Expression::Binary(outer_add)) = result {
        assert!(matches!(outer_add.operator, BinaryOperator::Add));

        // Left should be ((1 + 2) * (3 + 4))
        if let Expression::Grouped(left_grouped) = outer_add.left.as_ref() {
            if let Expression::Binary(left_mul) = left_grouped.as_ref() {
                assert!(matches!(left_mul.operator, BinaryOperator::Mul));
            } else {
                panic!("Expected multiplication in left grouped expression");
            }
        } else {
            panic!("Expected grouped expression on left");
        }

        // Right should be (5 * 6)
        if let Expression::Grouped(right_grouped) = outer_add.right.as_ref() {
            if let Expression::Binary(right_mul) = right_grouped.as_ref() {
                assert!(matches!(right_mul.operator, BinaryOperator::Mul));
            } else {
                panic!("Expected multiplication in right grouped expression");
            }
        } else {
            panic!("Expected grouped expression on right");
        }
    } else {
        panic!("Expected large binary expression tree");
    }
}
