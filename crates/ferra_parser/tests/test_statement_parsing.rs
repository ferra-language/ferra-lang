use ferra_parser::{
    ast::{Arena, Expression, Item, Literal, Statement, Type},
    statement::parser::StatementParser,
    token::{TokenType, VecTokenStream},
};

// Helper functions
fn create_test_arena() -> Arena {
    Arena::new()
}

fn create_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
    VecTokenStream::from_token_types(token_types)
}

// Phase 2.3.1: Declaration Statements Tests

#[test]
fn test_variable_declarations() {
    let arena = create_test_arena();

    // Test: let x = 42
    let tokens = create_token_stream(vec![
        TokenType::Let,
        TokenType::Identifier("x".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(42),
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::VariableDecl(var_decl)) = result {
        assert_eq!(var_decl.name, "x");
        assert!(!var_decl.is_mutable);
        assert!(var_decl.var_type.is_none());
        assert!(var_decl.initializer.is_some());
        if let Some(Expression::Literal(Literal::Integer(42))) = &var_decl.initializer {
            // Success
        } else {
            panic!("Expected integer literal 42");
        }
    } else {
        panic!("Expected variable declaration");
    }

    // Test: var mut_x: i32 = 10
    let tokens = create_token_stream(vec![
        TokenType::Var,
        TokenType::Identifier("mut_x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(10),
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::VariableDecl(var_decl)) = result {
        assert_eq!(var_decl.name, "mut_x");
        assert!(var_decl.is_mutable);
        assert!(var_decl.var_type.is_some());
        if let Some(Type::Identifier(type_name)) = &var_decl.var_type {
            assert_eq!(type_name, "i32");
        } else {
            panic!("Expected i32 type");
        }
    } else {
        panic!("Expected variable declaration");
    }
}

#[test]
fn test_function_declarations() {
    let arena = create_test_arena();

    // Test: fn hello() { }
    let tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::Identifier("hello".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_item();
    assert!(result.is_ok());

    if let Ok(Item::FunctionDecl(func_decl)) = result {
        assert_eq!(func_decl.name, "hello");
        assert!(!func_decl.is_async);
        assert!(!func_decl.is_extern);
        assert!(func_decl.parameters.is_empty());
        assert!(func_decl.return_type.is_none());
        assert!(func_decl.body.is_some());
    } else {
        panic!("Expected function declaration");
    }

    // Test: async fn calculate(x: i32, y: i32) -> i32 { }
    let tokens = create_token_stream(vec![
        TokenType::Async,
        TokenType::Fn,
        TokenType::Identifier("calculate".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::Comma,
        TokenType::Identifier("y".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("i32".to_string()),
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_item();
    assert!(result.is_ok());

    if let Ok(Item::FunctionDecl(func_decl)) = result {
        assert_eq!(func_decl.name, "calculate");
        assert!(func_decl.is_async);
        assert_eq!(func_decl.parameters.len(), 2);
        assert_eq!(func_decl.parameters[0].name, "x");
        assert_eq!(func_decl.parameters[1].name, "y");
        assert!(func_decl.return_type.is_some());
        if let Some(Type::Identifier(return_type)) = &func_decl.return_type {
            assert_eq!(return_type, "i32");
        }
    } else {
        panic!("Expected async function declaration");
    }
}

#[test]
fn test_data_class_declarations() {
    let arena = create_test_arena();

    // Test: data Person { name: String, age: i32 }
    let tokens = create_token_stream(vec![
        TokenType::Data,
        TokenType::Identifier("Person".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("name".to_string()),
        TokenType::Colon,
        TokenType::Identifier("String".to_string()),
        TokenType::Comma,
        TokenType::Identifier("age".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_item();
    assert!(result.is_ok());

    if let Ok(Item::DataClassDecl(data_class)) = result {
        assert_eq!(data_class.name, "Person");
        assert_eq!(data_class.fields.len(), 2);
        assert_eq!(data_class.fields[0].name, "name");
        assert_eq!(data_class.fields[1].name, "age");
        if let Type::Identifier(field_type) = &data_class.fields[0].field_type {
            assert_eq!(field_type, "String");
        }
        if let Type::Identifier(field_type) = &data_class.fields[1].field_type {
            assert_eq!(field_type, "i32");
        }
    } else {
        panic!("Expected data class declaration");
    }
}

#[test]
fn test_extern_blocks() {
    let arena = create_test_arena();

    // Test: extern "C" { fn printf(format: char) -> i32; static errno: i32; }
    let tokens = create_token_stream(vec![
        TokenType::Extern,
        TokenType::StringLiteral("C".to_string()),
        TokenType::LeftBrace,
        TokenType::Fn,
        TokenType::Identifier("printf".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("format".to_string()),
        TokenType::Colon,
        TokenType::Identifier("char".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("i32".to_string()),
        TokenType::Semicolon,
        TokenType::Static,
        TokenType::Identifier("errno".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_item();
    assert!(result.is_ok());

    if let Ok(Item::ExternBlock(extern_block)) = result {
        assert_eq!(extern_block.abi, "C");
        assert_eq!(extern_block.items.len(), 2);
        // More detailed checks could be added for the extern items
    } else {
        panic!("Expected extern block");
    }
}

#[test]
fn test_modifiers() {
    let arena = create_test_arena();

    // Test: pub fn public_function() { }
    let tokens = create_token_stream(vec![
        TokenType::Pub,
        TokenType::Fn,
        TokenType::Identifier("public_function".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_item();
    assert!(result.is_ok());

    if let Ok(Item::FunctionDecl(func_decl)) = result {
        assert_eq!(func_decl.name, "public_function");
        assert!(func_decl.modifiers.is_public);
        assert!(!func_decl.modifiers.is_unsafe);
    } else {
        panic!("Expected public function declaration");
    }
}

// Phase 2.3.2: Control Flow Statements Tests

#[test]
fn test_if_statement() {
    let arena = create_test_arena();

    // Test: if true { }
    let tokens = create_token_stream(vec![
        TokenType::If,
        TokenType::BooleanLiteral(true),
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::If(if_stmt)) = result {
        if let Expression::Literal(Literal::Boolean(true)) = &if_stmt.condition {
            // Success
        } else {
            panic!("Expected boolean condition");
        }
        assert!(if_stmt.else_block.is_none());
    } else {
        panic!("Expected if statement");
    }
}

#[test]
fn test_while_statement() {
    let arena = create_test_arena();

    // Test: while true { }
    let tokens = create_token_stream(vec![
        TokenType::While,
        TokenType::BooleanLiteral(true),
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::While(while_stmt)) = result {
        if let Expression::Literal(Literal::Boolean(true)) = &while_stmt.condition {
            // Success
        } else {
            panic!("Expected boolean condition");
        }
    } else {
        panic!("Expected while statement");
    }
}

#[test]
fn test_for_statement() {
    let arena = create_test_arena();

    // Test: for item in collection { }
    let tokens = create_token_stream(vec![
        TokenType::For,
        TokenType::Identifier("item".to_string()),
        TokenType::In,
        TokenType::Identifier("collection".to_string()),
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::For(for_stmt)) = result {
        assert_eq!(for_stmt.variable, "item");
        if let Expression::Identifier(iterable) = &for_stmt.iterable {
            assert_eq!(iterable, "collection");
        } else {
            panic!("Expected identifier for iterable");
        }
    } else {
        panic!("Expected for statement");
    }
}

#[test]
fn test_return_statement() {
    let arena = create_test_arena();

    // Test: return 42
    let tokens = create_token_stream(vec![
        TokenType::Return,
        TokenType::IntegerLiteral(42),
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::Return(return_stmt)) = result {
        assert!(return_stmt.value.is_some());
        if let Some(Expression::Literal(Literal::Integer(42))) = &return_stmt.value {
            // Success
        } else {
            panic!("Expected integer return value");
        }
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_break_continue_statements() {
    let arena = create_test_arena();

    // Test: break
    let tokens = create_token_stream(vec![TokenType::Break, TokenType::Eof]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::Break(_)) = result {
        // Success
    } else {
        panic!("Expected break statement");
    }

    // Test: continue
    let tokens = create_token_stream(vec![TokenType::Continue, TokenType::Eof]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::Continue(_)) = result {
        // Success
    } else {
        panic!("Expected continue statement");
    }
}

// Phase 2.3.3: Expression and Block Statements Tests

#[test]
fn test_expression_statement() {
    let arena = create_test_arena();

    // Test: 42;
    let tokens = create_token_stream(vec![
        TokenType::IntegerLiteral(42),
        TokenType::Semicolon,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::Expression(expr)) = result {
        if let Expression::Literal(Literal::Integer(42)) = expr {
            // Success
        } else {
            panic!("Expected integer literal expression");
        }
    } else {
        panic!("Expected expression statement");
    }
}

#[test]
fn test_block_statements() {
    let arena = create_test_arena();

    // Test: { let x = 42; }
    let tokens = create_token_stream(vec![
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("x".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(42),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_statement();
    assert!(result.is_ok());

    if let Ok(Statement::Block(block)) = result {
        assert!(block.is_braced);
        assert_eq!(block.statements.len(), 1);
        if let Statement::VariableDecl(var_decl) = &block.statements[0] {
            assert_eq!(var_decl.name, "x");
        } else {
            panic!("Expected variable declaration in block");
        }
    } else {
        panic!("Expected block statement");
    }
}

#[test]
fn test_compilation_unit() {
    let arena = create_test_arena();

    // Test complete program: fn main() { let x = 42; }
    let tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::Identifier("main".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("x".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(42),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);

    let result = parser.parse_compilation_unit();
    assert!(result.is_ok());

    if let Ok(compilation_unit) = result {
        assert_eq!(compilation_unit.items.len(), 1);
        if let Item::FunctionDecl(func_decl) = &compilation_unit.items[0] {
            assert_eq!(func_decl.name, "main");
            assert!(func_decl.body.is_some());
            if let Some(body) = &func_decl.body {
                assert_eq!(body.statements.len(), 1);
            }
        } else {
            panic!("Expected function declaration");
        }
    } else {
        panic!("Expected compilation unit");
    }
}
