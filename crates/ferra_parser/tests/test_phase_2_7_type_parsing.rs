//! Phase 2.7: Type Parsing Comprehensive Tests
//!
//! Tests for all type expression parsing capabilities introduced in Phase 2.7
//! including simple types, qualified identifiers, tuples, arrays, function types,
//! extern function types, pointer types, and complex nested combinations.

use ferra_parser::{
    ast::Type,
    token::{TokenType, VecTokenStream},
    types::parse_type,
};

fn create_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
    VecTokenStream::from_token_types(token_types)
}

/// Test Phase 2.7.1: Basic Type Expressions

#[test]
fn test_simple_identifier_types() {
    // Test basic built-in types
    let test_cases = vec![
        "int", "string", "bool", "float", "char", "void", "u8", "u16", "u32", "u64", "i8", "i16",
        "i32", "i64", "f32", "f64", "usize", "isize",
    ];

    for type_name in test_cases {
        let mut tokens = create_token_stream(vec![TokenType::Identifier(type_name.to_string())]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Identifier(name) => assert_eq!(name, type_name),
            _ => panic!("Expected identifier type for {}", type_name),
        }
    }
}

#[test]
fn test_custom_identifier_types() {
    // Test user-defined types
    let test_cases = vec![
        "MyStruct",
        "DatabaseConnection",
        "Vector3",
        "PlayerState",
        "GameEngine",
    ];

    for type_name in test_cases {
        let mut tokens = create_token_stream(vec![TokenType::Identifier(type_name.to_string())]);

        let result = parse_type(&mut tokens).unwrap();
        match result {
            Type::Identifier(name) => assert_eq!(name, type_name),
            _ => panic!("Expected identifier type for {}", type_name),
        }
    }
}

#[test]
fn test_tuple_types_various_lengths() {
    // Empty tuple: ()
    let mut tokens = create_token_stream(vec![TokenType::LeftParen, TokenType::RightParen]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Tuple(types) => assert_eq!(types.len(), 0),
        _ => panic!("Expected empty tuple type"),
    }

    // Single element tuple: (int,)
    let mut tokens = create_token_stream(vec![
        TokenType::LeftParen,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::RightParen,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Tuple(types) => assert_eq!(types.len(), 1),
        _ => panic!("Expected single element tuple"),
    }

    // Triple tuple: (int, string, bool)
    let mut tokens = create_token_stream(vec![
        TokenType::LeftParen,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("string".to_string()),
        TokenType::Comma,
        TokenType::Identifier("bool".to_string()),
        TokenType::RightParen,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Tuple(types) => {
            assert_eq!(types.len(), 3);
            match (&types[0], &types[1], &types[2]) {
                (Type::Identifier(t1), Type::Identifier(t2), Type::Identifier(t3)) => {
                    assert_eq!(t1, "int");
                    assert_eq!(t2, "string");
                    assert_eq!(t3, "bool");
                }
                _ => panic!("Expected identifier types in tuple"),
            }
        }
        _ => panic!("Expected triple tuple type"),
    }
}

#[test]
fn test_array_types_simple_and_nested() {
    // Simple array: [int]
    let mut tokens = create_token_stream(vec![
        TokenType::LeftBracket,
        TokenType::Identifier("int".to_string()),
        TokenType::RightBracket,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Array(element_type) => match element_type.as_ref() {
            Type::Identifier(name) => assert_eq!(name, "int"),
            _ => panic!("Expected int element type"),
        },
        _ => panic!("Expected array type"),
    }

    // Nested array: [[string]]
    let mut tokens = create_token_stream(vec![
        TokenType::LeftBracket,
        TokenType::LeftBracket,
        TokenType::Identifier("string".to_string()),
        TokenType::RightBracket,
        TokenType::RightBracket,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Array(outer_type) => match outer_type.as_ref() {
            Type::Array(inner_type) => match inner_type.as_ref() {
                Type::Identifier(name) => assert_eq!(name, "string"),
                _ => panic!("Expected string in nested array"),
            },
            _ => panic!("Expected nested array"),
        },
        _ => panic!("Expected array type"),
    }

    // Triple nested array: [[[bool]]]
    let mut tokens = create_token_stream(vec![
        TokenType::LeftBracket,
        TokenType::LeftBracket,
        TokenType::LeftBracket,
        TokenType::Identifier("bool".to_string()),
        TokenType::RightBracket,
        TokenType::RightBracket,
        TokenType::RightBracket,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Array(l1) => match l1.as_ref() {
            Type::Array(l2) => match l2.as_ref() {
                Type::Array(l3) => match l3.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "bool"),
                    _ => panic!("Expected bool in triple nested array"),
                },
                _ => panic!("Expected third level array"),
            },
            _ => panic!("Expected second level array"),
        },
        _ => panic!("Expected array type"),
    }
}

/// Test Phase 2.7.2: Function and Advanced Types

#[test]
fn test_function_types_various_signatures() {
    // No parameters, no return: fn()
    let mut tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::RightParen,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Function(func_type) => {
            assert_eq!(func_type.parameters.len(), 0);
            assert!(!func_type.is_extern);
            assert!(func_type.abi.is_none());
            // Should default to unit type
            match func_type.return_type.as_ref() {
                Type::Tuple(types) => assert_eq!(types.len(), 0),
                _ => panic!("Expected unit return type"),
            }
        }
        _ => panic!("Expected function type"),
    }

    // Single parameter with return: fn(int) -> string
    let mut tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::Identifier("int".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("string".to_string()),
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Function(func_type) => {
            assert_eq!(func_type.parameters.len(), 1);
            match &func_type.parameters[0] {
                Type::Identifier(name) => assert_eq!(name, "int"),
                _ => panic!("Expected int parameter"),
            }
            match func_type.return_type.as_ref() {
                Type::Identifier(name) => assert_eq!(name, "string"),
                _ => panic!("Expected string return type"),
            }
        }
        _ => panic!("Expected function type"),
    }

    // Multiple parameters: fn(int, string, bool) -> float
    let mut tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("string".to_string()),
        TokenType::Comma,
        TokenType::Identifier("bool".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("float".to_string()),
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Function(func_type) => {
            assert_eq!(func_type.parameters.len(), 3);
            match (
                &func_type.parameters[0],
                &func_type.parameters[1],
                &func_type.parameters[2],
            ) {
                (Type::Identifier(p1), Type::Identifier(p2), Type::Identifier(p3)) => {
                    assert_eq!(p1, "int");
                    assert_eq!(p2, "string");
                    assert_eq!(p3, "bool");
                }
                _ => panic!("Expected identifier parameter types"),
            }
            match func_type.return_type.as_ref() {
                Type::Identifier(name) => assert_eq!(name, "float"),
                _ => panic!("Expected float return type"),
            }
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_extern_function_types() {
    // Extern with ABI: extern "C" fn(int) -> void
    let mut tokens = create_token_stream(vec![
        TokenType::Extern,
        TokenType::StringLiteral("C".to_string()),
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::Identifier("int".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("void".to_string()),
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Function(func_type) => {
            assert!(func_type.is_extern);
            assert_eq!(func_type.abi, Some("C".to_string()));
            assert_eq!(func_type.parameters.len(), 1);
            match func_type.return_type.as_ref() {
                Type::Identifier(name) => assert_eq!(name, "void"),
                _ => panic!("Expected void return type"),
            }
        }
        _ => panic!("Expected extern function type"),
    }

    // Extern without explicit ABI: extern fn() -> int
    let mut tokens = create_token_stream(vec![
        TokenType::Extern,
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("int".to_string()),
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Function(func_type) => {
            assert!(func_type.is_extern);
            assert!(func_type.abi.is_none());
            assert_eq!(func_type.parameters.len(), 0);
        }
        _ => panic!("Expected extern function type"),
    }
}

#[test]
fn test_pointer_types() {
    // Simple pointer: *int
    let mut tokens = create_token_stream(vec![
        TokenType::Star,
        TokenType::Identifier("int".to_string()),
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Pointer(ptr_type) => {
            assert!(ptr_type.is_mutable); // Default to mutable for now
            match ptr_type.target.as_ref() {
                Type::Identifier(name) => assert_eq!(name, "int"),
                _ => panic!("Expected int pointer target"),
            }
        }
        _ => panic!("Expected pointer type"),
    }

    // Pointer to pointer: **string
    let mut tokens = create_token_stream(vec![
        TokenType::Star,
        TokenType::Star,
        TokenType::Identifier("string".to_string()),
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Pointer(outer_ptr) => match outer_ptr.target.as_ref() {
            Type::Pointer(inner_ptr) => match inner_ptr.target.as_ref() {
                Type::Identifier(name) => assert_eq!(name, "string"),
                _ => panic!("Expected string in nested pointer"),
            },
            _ => panic!("Expected nested pointer"),
        },
        _ => panic!("Expected pointer type"),
    }
}

/// Test Complex Combinations

#[test]
fn test_array_of_tuples() {
    // [(int, string)]
    let mut tokens = create_token_stream(vec![
        TokenType::LeftBracket,
        TokenType::LeftParen,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("string".to_string()),
        TokenType::RightParen,
        TokenType::RightBracket,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Array(element_type) => match element_type.as_ref() {
            Type::Tuple(types) => {
                assert_eq!(types.len(), 2);
                match (&types[0], &types[1]) {
                    (Type::Identifier(t1), Type::Identifier(t2)) => {
                        assert_eq!(t1, "int");
                        assert_eq!(t2, "string");
                    }
                    _ => panic!("Expected int and string in tuple"),
                }
            }
            _ => panic!("Expected tuple element in array"),
        },
        _ => panic!("Expected array type"),
    }
}

#[test]
fn test_tuple_of_arrays() {
    // ([int], [string], [bool])
    let mut tokens = create_token_stream(vec![
        TokenType::LeftParen,
        TokenType::LeftBracket,
        TokenType::Identifier("int".to_string()),
        TokenType::RightBracket,
        TokenType::Comma,
        TokenType::LeftBracket,
        TokenType::Identifier("string".to_string()),
        TokenType::RightBracket,
        TokenType::Comma,
        TokenType::LeftBracket,
        TokenType::Identifier("bool".to_string()),
        TokenType::RightBracket,
        TokenType::RightParen,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Tuple(types) => {
            assert_eq!(types.len(), 3);
            for (i, expected) in ["int", "string", "bool"].iter().enumerate() {
                match &types[i] {
                    Type::Array(element_type) => match element_type.as_ref() {
                        Type::Identifier(name) => assert_eq!(name, expected),
                        _ => panic!("Expected {} array element", expected),
                    },
                    _ => panic!("Expected array type in tuple position {}", i),
                }
            }
        }
        _ => panic!("Expected tuple type"),
    }
}

#[test]
fn test_function_with_complex_parameters() {
    // fn([int], (string, bool)) -> *int
    let mut tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::LeftBracket,
        TokenType::Identifier("int".to_string()),
        TokenType::RightBracket,
        TokenType::Comma,
        TokenType::LeftParen,
        TokenType::Identifier("string".to_string()),
        TokenType::Comma,
        TokenType::Identifier("bool".to_string()),
        TokenType::RightParen,
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Star,
        TokenType::Identifier("int".to_string()),
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Function(func_type) => {
            assert_eq!(func_type.parameters.len(), 2);

            // First parameter: [int]
            match &func_type.parameters[0] {
                Type::Array(element_type) => match element_type.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "int"),
                    _ => panic!("Expected int array"),
                },
                _ => panic!("Expected array parameter"),
            }

            // Second parameter: (string, bool)
            match &func_type.parameters[1] {
                Type::Tuple(types) => {
                    assert_eq!(types.len(), 2);
                    match (&types[0], &types[1]) {
                        (Type::Identifier(t1), Type::Identifier(t2)) => {
                            assert_eq!(t1, "string");
                            assert_eq!(t2, "bool");
                        }
                        _ => panic!("Expected string and bool in tuple"),
                    }
                }
                _ => panic!("Expected tuple parameter"),
            }

            // Return type: *int
            match func_type.return_type.as_ref() {
                Type::Pointer(ptr_type) => match ptr_type.target.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "int"),
                    _ => panic!("Expected int pointer"),
                },
                _ => panic!("Expected pointer return type"),
            }
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_higher_order_functions() {
    // fn(fn(int) -> string) -> bool
    let mut tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::Identifier("int".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("string".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("bool".to_string()),
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Function(outer_func) => {
            assert_eq!(outer_func.parameters.len(), 1);

            // Parameter: fn(int) -> string
            match &outer_func.parameters[0] {
                Type::Function(inner_func) => {
                    assert_eq!(inner_func.parameters.len(), 1);
                    match &inner_func.parameters[0] {
                        Type::Identifier(name) => assert_eq!(name, "int"),
                        _ => panic!("Expected int parameter in inner function"),
                    }
                    match inner_func.return_type.as_ref() {
                        Type::Identifier(name) => assert_eq!(name, "string"),
                        _ => panic!("Expected string return in inner function"),
                    }
                }
                _ => panic!("Expected function parameter"),
            }

            // Return type: bool
            match outer_func.return_type.as_ref() {
                Type::Identifier(name) => assert_eq!(name, "bool"),
                _ => panic!("Expected bool return type"),
            }
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_array_of_function_pointers() {
    // [fn(int) -> string]
    let mut tokens = create_token_stream(vec![
        TokenType::LeftBracket,
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::Identifier("int".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("string".to_string()),
        TokenType::RightBracket,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Array(element_type) => match element_type.as_ref() {
            Type::Function(func_type) => {
                assert_eq!(func_type.parameters.len(), 1);
                match &func_type.parameters[0] {
                    Type::Identifier(name) => assert_eq!(name, "int"),
                    _ => panic!("Expected int parameter"),
                }
                match func_type.return_type.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "string"),
                    _ => panic!("Expected string return type"),
                }
            }
            _ => panic!("Expected function type in array"),
        },
        _ => panic!("Expected array type"),
    }
}

#[test]
fn test_extremely_complex_type() {
    // [fn(*[int], (string, bool)) -> *(string, [bool])]
    let mut tokens = create_token_stream(vec![
        TokenType::LeftBracket,
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::Star,
        TokenType::LeftBracket,
        TokenType::Identifier("int".to_string()),
        TokenType::RightBracket,
        TokenType::Comma,
        TokenType::LeftParen,
        TokenType::Identifier("string".to_string()),
        TokenType::Comma,
        TokenType::Identifier("bool".to_string()),
        TokenType::RightParen,
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Star,
        TokenType::LeftParen,
        TokenType::Identifier("string".to_string()),
        TokenType::Comma,
        TokenType::LeftBracket,
        TokenType::Identifier("bool".to_string()),
        TokenType::RightBracket,
        TokenType::RightParen,
        TokenType::RightBracket,
    ]);

    let result = parse_type(&mut tokens).unwrap();
    match result {
        Type::Array(element_type) => {
            match element_type.as_ref() {
                Type::Function(func_type) => {
                    assert_eq!(func_type.parameters.len(), 2);

                    // First parameter: *[int]
                    match &func_type.parameters[0] {
                        Type::Pointer(ptr_type) => match ptr_type.target.as_ref() {
                            Type::Array(arr_type) => match arr_type.as_ref() {
                                Type::Identifier(name) => assert_eq!(name, "int"),
                                _ => panic!("Expected int in array"),
                            },
                            _ => panic!("Expected array in pointer"),
                        },
                        _ => panic!("Expected pointer parameter"),
                    }

                    // Second parameter: (string, bool)
                    match &func_type.parameters[1] {
                        Type::Tuple(types) => {
                            assert_eq!(types.len(), 2);
                        }
                        _ => panic!("Expected tuple parameter"),
                    }

                    // Return type: *(string, [bool])
                    match func_type.return_type.as_ref() {
                        Type::Pointer(ptr_type) => match ptr_type.target.as_ref() {
                            Type::Tuple(types) => {
                                assert_eq!(types.len(), 2);
                                match (&types[0], &types[1]) {
                                    (Type::Identifier(t1), Type::Array(arr)) => {
                                        assert_eq!(t1, "string");
                                        match arr.as_ref() {
                                            Type::Identifier(name) => assert_eq!(name, "bool"),
                                            _ => panic!("Expected bool array"),
                                        }
                                    }
                                    _ => panic!("Expected string and bool array"),
                                }
                            }
                            _ => panic!("Expected tuple in return pointer"),
                        },
                        _ => panic!("Expected pointer return type"),
                    }
                }
                _ => panic!("Expected function type in array"),
            }
        }
        _ => panic!("Expected array type"),
    }
}

/// Test Error Cases

#[test]
fn test_type_parsing_error_cases() {
    // Invalid token for type
    let mut tokens = create_token_stream(vec![
        TokenType::Plus, // Invalid token for type
    ]);

    let result = parse_type(&mut tokens);
    assert!(result.is_err());

    // Incomplete array type
    let mut tokens = create_token_stream(vec![
        TokenType::LeftBracket,
        // Missing element type and closing bracket
    ]);

    let result = parse_type(&mut tokens);
    assert!(result.is_err());

    // Incomplete function type
    let mut tokens = create_token_stream(vec![
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::Identifier("int".to_string()),
        // Missing closing paren
    ]);

    let result = parse_type(&mut tokens);
    assert!(result.is_err());
}

/// Test Integration with Existing Parsers

#[test]
fn test_type_parsing_integration() {
    use ferra_parser::{ast::Arena, program::ProgramParser, token::VecTokenStream};

    // Test that the enhanced type parser works with existing parsers
    let tokens = vec![
        TokenType::Fn,
        TokenType::Identifier("complex_function".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("data".to_string()),
        TokenType::Colon,
        TokenType::LeftBracket,
        TokenType::Identifier("int".to_string()),
        TokenType::RightBracket,
        TokenType::Comma,
        TokenType::Identifier("callback".to_string()),
        TokenType::Colon,
        TokenType::Fn,
        TokenType::LeftParen,
        TokenType::Identifier("string".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("bool".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Star,
        TokenType::Identifier("int".to_string()),
        TokenType::LeftBrace,
        TokenType::RightBrace,
    ];

    let arena = Arena::new();
    let token_stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, token_stream);

    let result = parser.parse_compilation_unit();
    assert!(result.is_ok());

    let compilation_unit = result.unwrap();
    assert_eq!(compilation_unit.items.len(), 1);

    match &compilation_unit.items[0] {
        ferra_parser::ast::Item::FunctionDecl(func_decl) => {
            assert_eq!(func_decl.name, "complex_function");
            assert_eq!(func_decl.parameters.len(), 2);

            // First parameter should be array type
            match &func_decl.parameters[0].param_type {
                Type::Array(element_type) => match element_type.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "int"),
                    _ => panic!("Expected int array parameter"),
                },
                _ => panic!("Expected array parameter type"),
            }

            // Second parameter should be function type
            match &func_decl.parameters[1].param_type {
                Type::Function(func_type) => {
                    assert_eq!(func_type.parameters.len(), 1);
                    match func_type.return_type.as_ref() {
                        Type::Identifier(name) => assert_eq!(name, "bool"),
                        _ => panic!("Expected bool return type"),
                    }
                }
                _ => panic!("Expected function parameter type"),
            }

            // Return type should be pointer
            match func_decl.return_type.as_ref().unwrap() {
                Type::Pointer(ptr_type) => match ptr_type.target.as_ref() {
                    Type::Identifier(name) => assert_eq!(name, "int"),
                    _ => panic!("Expected int pointer return"),
                },
                _ => panic!("Expected pointer return type"),
            }
        }
        _ => panic!("Expected function declaration"),
    }
}
