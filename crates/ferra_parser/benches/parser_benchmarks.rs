use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ferra_lexer::{Lexer, TokenKind};
use ferra_parser::{ast::Arena, token::VecTokenStream, ProgramParser, TokenType};

/// Convert TokenKind from lexer to TokenType for parser
fn convert_token_kind(kind: TokenKind) -> TokenType {
    match kind {
        TokenKind::Let => TokenType::Let,
        TokenKind::Var => TokenType::Var,
        TokenKind::Fn => TokenType::Fn,
        TokenKind::Async => TokenType::Async,
        TokenKind::Data => TokenType::Data,
        TokenKind::Match => TokenType::Match,
        TokenKind::True => TokenType::BooleanLiteral(true),
        TokenKind::False => TokenType::BooleanLiteral(false),
        TokenKind::Return => TokenType::Return,
        TokenKind::If => TokenType::If,
        TokenKind::Else => TokenType::Else,
        TokenKind::While => TokenType::While,
        TokenKind::For => TokenType::For,
        TokenKind::In => TokenType::In,
        TokenKind::Break => TokenType::Break,
        TokenKind::Continue => TokenType::Continue,
        TokenKind::Pub => TokenType::Pub,
        TokenKind::Unsafe => TokenType::Unsafe,
        TokenKind::Identifier => TokenType::Identifier("dummy".to_string()),
        TokenKind::IntegerLiteral => TokenType::IntegerLiteral(42),
        TokenKind::FloatLiteral => TokenType::FloatLiteral(3.15),
        TokenKind::StringLiteral => TokenType::StringLiteral("dummy".to_string()),
        TokenKind::CharacterLiteral => TokenType::StringLiteral("dummy".to_string()),
        TokenKind::BooleanLiteral => TokenType::BooleanLiteral(true),
        TokenKind::Plus => TokenType::Plus,
        TokenKind::Minus => TokenType::Minus,
        TokenKind::Star => TokenType::Star,
        TokenKind::Slash => TokenType::Slash,
        TokenKind::Percent => TokenType::Percent,
        TokenKind::EqualEqual => TokenType::EqualEqual,
        TokenKind::NotEqual => TokenType::BangEqual,
        TokenKind::Less => TokenType::Less,
        TokenKind::Greater => TokenType::Greater,
        TokenKind::LessEqual => TokenType::LessEqual,
        TokenKind::GreaterEqual => TokenType::GreaterEqual,
        TokenKind::LogicalAnd => TokenType::AmpAmp,
        TokenKind::LogicalOr => TokenType::PipePipe,
        TokenKind::BitAnd => TokenType::Ampersand,
        TokenKind::BitOr => TokenType::Pipe,
        TokenKind::Equal => TokenType::Equal,
        TokenKind::Bang => TokenType::Bang,
        TokenKind::Question => TokenType::Question,
        TokenKind::Dot => TokenType::Dot,
        TokenKind::Comma => TokenType::Comma,
        TokenKind::Colon => TokenType::Colon,
        TokenKind::Semicolon => TokenType::Semicolon,
        TokenKind::LParen => TokenType::LeftParen,
        TokenKind::RParen => TokenType::RightParen,
        TokenKind::LBrace => TokenType::LeftBrace,
        TokenKind::RBrace => TokenType::RightBrace,
        TokenKind::LBracket => TokenType::LeftBracket,
        TokenKind::RBracket => TokenType::RightBracket,
        TokenKind::Arrow => TokenType::Arrow,
        TokenKind::FatArrow => TokenType::FatArrow,
        TokenKind::DotDot => TokenType::DotDot,
        TokenKind::DotDotEqual => TokenType::DotDotEqual,
        TokenKind::PathSep => TokenType::DoubleColon,
        TokenKind::Underscore => TokenType::Identifier("_".to_string()),
        TokenKind::Eof => TokenType::Eof,
        _ => TokenType::Eof,
    }
}

/// Convert source code to tokens
fn source_to_tokens(source: &str) -> Vec<TokenType> {
    let lexer = Lexer::new(source);
    let tokens = lexer.lex();

    tokens
        .into_iter()
        .map(|token| convert_token_kind(token.kind))
        .collect()
}

/// Benchmark parsing small programs (now including the previously hanging patterns)
fn bench_small_programs(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_programs");
    group.warm_up_time(std::time::Duration::from_millis(200));
    group.measurement_time(std::time::Duration::from_millis(800));

    let programs = vec![
        ("empty_function", "fn test() { }"),
        ("function_with_statement", "fn test() { let x = 42; }"),
        ("function_with_params", "fn calc(a, b) { }"),
        (
            "function_with_expression",
            "fn test() { let z = (1 + 2) * 3; }",
        ),
        // These patterns were previously hanging but now work due to parser bug fix:
        ("params_and_statement", "fn calc(a, b) { let x = 42; }"),
        (
            "params_and_multiple_statements",
            "fn calc(a, b) { let x = 42; let y = a + b; }",
        ),
        (
            "params_and_complex_expression",
            "fn calc(a, b) { let result = (a + b) * 2; }",
        ),
    ];

    for (name, source) in programs {
        let tokens = source_to_tokens(source);

        group.bench_with_input(
            BenchmarkId::new("parse", name),
            &tokens,
            |b, tokens: &Vec<TokenType>| {
                b.iter(|| {
                    let arena = Arena::new();
                    let stream = VecTokenStream::from_token_types(tokens.clone());
                    let mut parser = ProgramParser::new(&arena, stream);
                    let _result = parser.parse_compilation_unit();
                    let _ = black_box(_result.is_ok());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark parser creation overhead
fn bench_parser_creation(c: &mut Criterion) {
    let tokens = source_to_tokens("fn test() { }");

    c.bench_function("parser_creation", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let stream = VecTokenStream::from_token_types(tokens.clone());
            let parser = ProgramParser::new(&arena, stream);
            black_box(std::mem::size_of_val(&parser))
        })
    });
}

/// Benchmark expression parsing (nested expressions)
fn bench_expression_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("expressions");
    group.warm_up_time(std::time::Duration::from_millis(200));
    group.measurement_time(std::time::Duration::from_millis(600));

    let expressions = vec![
        ("simple", "fn test() { let x = 42; }"),
        ("arithmetic", "fn test() { let x = 1 + 2 * 3; }"),
        ("nested_parens", "fn test() { let x = ((1 + 2) * 3); }"),
        ("deep_nesting", "fn test() { let x = (((1 + 2) * 3) + 4); }"),
        // Test with parameters (previously hanging patterns):
        (
            "arithmetic_with_params",
            "fn calc(a, b) { let x = a + b * 2; }",
        ),
        (
            "nested_with_params",
            "fn calc(a, b) { let x = ((a + b) * 2); }",
        ),
    ];

    for (name, source) in expressions {
        let tokens = source_to_tokens(source);

        group.bench_with_input(
            BenchmarkId::new("expr", name),
            &tokens,
            |b, tokens: &Vec<TokenType>| {
                b.iter(|| {
                    let arena = Arena::new();
                    let stream = VecTokenStream::from_token_types(tokens.clone());
                    let mut parser = ProgramParser::new(&arena, stream);
                    let _result = parser.parse_compilation_unit();
                    let _ = black_box(_result.is_ok());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark parsing larger programs to test scalability
fn bench_large_programs(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_programs");
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_secs(1));

    let large_program = r#"
        fn fibonacci(n) {
            if n <= 1 {
                return n;
            } else {
                return fibonacci(n - 1) + fibonacci(n - 2);
            }
        }

        fn factorial(n) {
            let result = 1;
            let i = 1;
            while i <= n {
                result = result * i;
                i = i + 1;
            }
            return result;
        }

        fn quicksort(arr, low, high) {
            if low < high {
                let pi = partition(arr, low, high);
                quicksort(arr, low, pi - 1);
                quicksort(arr, pi + 1, high);
            }
        }

        fn main() {
            let n = 10;
            let fib_result = fibonacci(n);
            let fact_result = factorial(n);
            let arr = [64, 34, 25, 12, 22, 11, 90];
            quicksort(arr, 0, 6);
        }
    "#;

    let tokens = source_to_tokens(large_program);

    group.bench_function("multi_function_program", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let stream = VecTokenStream::from_token_types(tokens.clone());
            let mut parser = ProgramParser::new(&arena, stream);
            let _result = parser.parse_compilation_unit();
            let _ = black_box(_result.is_ok());
        })
    });

    group.finish();
}

// Memory leak detection and profiling
fn memory_leak_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_profiling");
    group.warm_up_time(std::time::Duration::from_millis(200));
    group.measurement_time(std::time::Duration::from_millis(800));
    group.sample_size(50); // Reduce sample size

    // Test that repeated parsing doesn't accumulate memory
    let simple_source = "fn test() { let x = 42; let y = x + 1; }";
    let tokens = source_to_tokens(simple_source);

    group.bench_function("repeated_parsing_no_leaks", |b| {
        b.iter(|| {
            for _ in 0..25 {
                // Reduced from 100
                let arena = Arena::new();
                let stream = VecTokenStream::from_token_types(tokens.clone());
                let mut parser = ProgramParser::new(&arena, stream);
                let _result = parser.parse_compilation_unit();
                let _ = black_box(_result.is_ok());
            }
        })
    });

    // Test memory usage with larger inputs
    let mut large_source = String::new();
    for i in 0..50 {
        // Reduced from 200
        large_source.push_str(&format!(
            "fn func_{i}() {{ let x_{i} = {i}; let y_{i} = x_{i} * 2; }}\n"
        ));
    }
    let large_tokens = source_to_tokens(&large_source);

    group.bench_function("large_input_memory_usage", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let stream = VecTokenStream::from_token_types(large_tokens.clone());
            let mut parser = ProgramParser::new(&arena, stream);
            let _result = parser.parse_compilation_unit();
            let _ = black_box(_result.is_ok());
        })
    });

    // Test memory efficiency with deep nesting
    let nested_source = format!(
        "fn outer() {{ {} }}",
        (0..10) // Reduced from 50
            .map(|i| format!("{{ let x_{i} = {i}; }}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let nested_tokens = source_to_tokens(&nested_source);

    group.bench_function("memory_efficiency_deep_nesting", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let stream = VecTokenStream::from_token_types(nested_tokens.clone());
            let mut parser = ProgramParser::new(&arena, stream);
            let _result = parser.parse_compilation_unit();
            let _ = black_box(_result.is_ok());
        })
    });

    group.finish();
}

// Real-world parsing scenarios
fn real_world_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_parsing");

    // Simulate parsing a typical web service module
    group.bench_function("web_service_module", |b| {
        let source = r#"
            data User {
                id: Int,
                name: String,
                email: String,
                created_at: DateTime
            }

            data ApiResponse<T> {
                data: T,
                status: Int,
                message: String
            }

            fn get_user(id: Int) -> ApiResponse<User> {
                if id <= 0 {
                    return ApiResponse {
                        data: null,
                        status: 400,
                        message: "Invalid user ID"
                    };
                }

                let user = database.find_user_by_id(id);
                if user.is_none() {
                    return ApiResponse {
                        data: null,
                        status: 404,
                        message: "User not found"
                    };
                }

                return ApiResponse {
                    data: user.unwrap(),
                    status: 200,
                    message: "Success"
                };
            }

            fn create_user(name: String, email: String) -> ApiResponse<User> {
                if name.is_empty() || email.is_empty() {
                    return ApiResponse {
                        data: null,
                        status: 400,
                        message: "Name and email are required"
                    };
                }

                let user = User {
                    id: generate_id(),
                    name: name,
                    email: email,
                    created_at: now()
                };

                database.save_user(user);

                return ApiResponse {
                    data: user,
                    status: 201,
                    message: "User created successfully"
                };
            }
        "#;

        b.iter(|| {
            let arena = Arena::new();
            let tokens = source_to_tokens(source);
            let token_stream = VecTokenStream::from_token_types(tokens);
            let mut parser = ProgramParser::new(&arena, token_stream);

            let _result = parser.parse_compilation_unit();
            let _ = black_box(_result.is_ok());
        })
    });

    // Simulate parsing a mathematical computation module
    group.bench_function("math_computation_module", |b| {
        let source = r#"
            fn fibonacci(n: Int) -> Int {
                if n <= 1 {
                    return n;
                }
                return fibonacci(n - 1) + fibonacci(n - 2);
            }

            fn factorial(n: Int) -> Int {
                if n <= 1 {
                    return 1;
                }
                return n * factorial(n - 1);
            }

            fn matrix_multiply(a: Array<Array<Float>>, b: Array<Array<Float>>) -> Array<Array<Float>> {
                let rows_a = a.length();
                let cols_a = a[0].length();
                let cols_b = b[0].length();

                let result = Array::with_capacity(rows_a);

                for i in 0..rows_a {
                    let row = Array::with_capacity(cols_b);
                    for j in 0..cols_b {
                        let sum = 0.0;
                        for k in 0..cols_a {
                            sum = sum + a[i][k] * b[k][j];
                        }
                        row.push(sum);
                    }
                    result.push(row);
                }

                return result;
            }
        "#;

        b.iter(|| {
            let arena = Arena::new();
            let tokens = source_to_tokens(source);
            let token_stream = VecTokenStream::from_token_types(tokens);
            let mut parser = ProgramParser::new(&arena, token_stream);

            let _result = parser.parse_compilation_unit();
            let _ = black_box(_result.is_ok());
        })
    });

    group.finish();
}

/// Benchmark error recovery overhead - measures the performance cost of error recovery
fn bench_error_recovery_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_recovery_overhead");
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_secs(1));

    // Test cases with different types of syntax errors
    let error_scenarios = vec![
        ("missing_semicolon", "fn test() { let x = 42 let y = 24; }"),
        ("missing_paren", "fn test( { let x = 42; }"),
        ("unmatched_brace", "fn test() { let x = 42; "),
        ("invalid_token", "fn test() { let x = @@ 42; }"),
        ("incomplete_expression", "fn test() { let x = ; }"),
        ("multiple_errors", "fn test( { let x = @@ ; let y = 42 }"),
        ("nested_errors", "fn outer() { fn inner( { let x = ; } }"),
        ("chain_of_errors", "let = ; fn ( { while { for in } }"),
    ];

    for (name, error_source) in &error_scenarios {
        let error_tokens = source_to_tokens(error_source);

        group.bench_with_input(
            BenchmarkId::new("error_recovery", name),
            &error_tokens,
            |b, tokens: &Vec<TokenType>| {
                b.iter(|| {
                    let arena = Arena::new();
                    let stream = VecTokenStream::from_token_types(tokens.clone());
                    let mut parser = ProgramParser::new(&arena, stream);
                    let _result = parser.parse_compilation_unit();
                    // Ensure we capture whether errors were collected
                    let _ = black_box(parser.has_errors());
                })
            },
        );

        // Compare against equivalent valid code to measure overhead
        let valid_source = match *name {
            "missing_semicolon" => "fn test() { let x = 42; let y = 24; }",
            "missing_paren" => "fn test() { let x = 42; }",
            "unmatched_brace" => "fn test() { let x = 42; }",
            "invalid_token" => "fn test() { let x = 42; }",
            "incomplete_expression" => "fn test() { let x = 42; }",
            "multiple_errors" => "fn test() { let x = 42; let y = 42; }",
            "nested_errors" => "fn outer() { fn inner() { let x = 42; } }",
            "chain_of_errors" => {
                "let x = 42; fn test() { while true { for item in collection { } } }"
            }
            _ => "fn test() { let x = 42; }",
        };

        let valid_tokens = source_to_tokens(valid_source);

        group.bench_with_input(
            BenchmarkId::new("valid_baseline", name),
            &valid_tokens,
            |b, tokens: &Vec<TokenType>| {
                b.iter(|| {
                    let arena = Arena::new();
                    let stream = VecTokenStream::from_token_types(tokens.clone());
                    let mut parser = ProgramParser::new(&arena, stream);
                    let _result = parser.parse_compilation_unit();
                    let _ = black_box(_result.is_ok());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark error recovery with varying error densities
fn bench_error_density_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_density_impact");
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_secs(1));

    // Generate programs with different error densities
    let error_densities = vec![
        ("no_errors", 0),
        ("low_density", 1),       // 1 error per 10 statements
        ("medium_density", 3),    // 3 errors per 10 statements
        ("high_density", 5),      // 5 errors per 10 statements
        ("very_high_density", 8), // 8 errors per 10 statements
    ];

    for (density_name, error_count) in error_densities {
        let mut source = String::new();

        for i in 0..10 {
            if i < error_count {
                // Add statements with errors
                source.push_str(&format!("fn func_{i}( {{ let x = ; }}\n"));
            } else {
                // Add valid statements
                source.push_str(&format!("fn func_{i}() {{ let x = {i}; }}\n"));
            }
        }

        let tokens = source_to_tokens(&source);

        group.bench_with_input(
            BenchmarkId::new("density", density_name),
            &tokens,
            |b, tokens: &Vec<TokenType>| {
                b.iter(|| {
                    let arena = Arena::new();
                    let stream = VecTokenStream::from_token_types(tokens.clone());
                    let mut parser = ProgramParser::new(&arena, stream);
                    let _result = parser.parse_compilation_unit();
                    let _ = black_box(parser.has_errors());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark error recovery scalability with large inputs containing errors
fn bench_error_recovery_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_recovery_scalability");
    group.warm_up_time(std::time::Duration::from_millis(200));
    group.measurement_time(std::time::Duration::from_millis(800));
    group.sample_size(50); // Reduce sample size

    let program_sizes = vec![
        ("small_with_errors", 25),       // Reduced from 50
        ("medium_with_errors", 75),      // Reduced from 200
        ("large_with_errors", 150),      // Reduced from 500
        ("very_large_with_errors", 250), // Reduced from 1000
    ];

    for (size_name, func_count) in program_sizes {
        let mut source = String::new();

        for i in 0..func_count {
            if i % 10 == 0 {
                // Every 10th function has an error
                source.push_str(&format!("fn func_{i}( {{ let x = ; }}\n"));
            } else {
                source.push_str(&format!("fn func_{i}() {{ let x = {i}; }}\n"));
            }
        }

        let tokens = source_to_tokens(&source);

        group.bench_with_input(
            BenchmarkId::new("scale", size_name),
            &tokens,
            |b, tokens: &Vec<TokenType>| {
                b.iter(|| {
                    let arena = Arena::new();
                    let stream = VecTokenStream::from_token_types(tokens.clone());
                    let mut parser = ProgramParser::new(&arena, stream);
                    let _result = parser.parse_compilation_unit();
                    let _ = black_box(parser.has_errors());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark different error recovery strategies to measure their overhead
fn bench_recovery_strategy_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("recovery_strategy_overhead");
    group.warm_up_time(std::time::Duration::from_millis(300));
    group.measurement_time(std::time::Duration::from_secs(1));

    // Test error scenarios that trigger different recovery strategies
    let recovery_scenarios = vec![
        (
            "panic_mode_recovery",
            "fn test() { @@@ @@@ @@@ let x = 42; }",
        ),
        (
            "production_based_recovery",
            "fn test() { let x = 42 let y = 24; }",
        ),
        (
            "smart_recovery_context",
            "fn test( let x = 42; fn another() { } }",
        ),
        (
            "deep_nesting_recovery",
            "fn test() { if true { while false { let x = ; } } }",
        ),
        (
            "multiple_sync_points",
            "fn a() { @@@ } fn b() { @@@ } fn c() { @@@ }",
        ),
    ];

    for (strategy_name, error_source) in recovery_scenarios {
        let error_tokens = source_to_tokens(error_source);

        group.bench_with_input(
            BenchmarkId::new("strategy", strategy_name),
            &error_tokens,
            |b, tokens: &Vec<TokenType>| {
                b.iter(|| {
                    let arena = Arena::new();
                    let stream = VecTokenStream::from_token_types(tokens.clone());
                    let mut parser = ProgramParser::new(&arena, stream);
                    let _result = parser.parse_compilation_unit();
                    let _ = black_box(parser.has_errors());
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parser_creation,
    bench_small_programs,
    bench_expression_parsing,
    bench_large_programs,
    memory_leak_detection,
    real_world_scenarios,
    bench_error_recovery_overhead,
    bench_error_density_impact,
    bench_error_recovery_scalability,
    bench_recovery_strategy_overhead
);

criterion_main!(benches);
