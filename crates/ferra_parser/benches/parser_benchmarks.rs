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
                    let result = parser.parse_compilation_unit();
                    black_box(result.is_ok())
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
                    let result = parser.parse_compilation_unit();
                    black_box(result.is_ok())
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
            let result = parser.parse_compilation_unit();
            black_box(result.is_ok())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_programs,
    bench_parser_creation,
    bench_expression_parsing,
    bench_large_programs
);

criterion_main!(benches);
