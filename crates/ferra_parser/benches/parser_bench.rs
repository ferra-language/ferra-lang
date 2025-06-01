use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ferra_parser::{
    token::{TokenType, VecTokenStream},
    Parser,
};

fn benchmark_token_stream_creation(c: &mut Criterion) {
    c.bench_function("token_stream_creation", |b| {
        b.iter(|| {
            let tokens = vec![
                TokenType::Let,
                TokenType::Identifier("x".to_string()),
                TokenType::Equal,
                TokenType::IntegerLiteral(42),
                TokenType::Semicolon,
                TokenType::Eof,
            ];
            black_box(VecTokenStream::from_token_types(tokens))
        })
    });
}

fn benchmark_parser_creation(c: &mut Criterion) {
    c.bench_function("parser_creation", |b| {
        b.iter(|| {
            let tokens = vec![
                TokenType::Let,
                TokenType::Identifier("x".to_string()),
                TokenType::Equal,
                TokenType::IntegerLiteral(42),
                TokenType::Eof,
            ];
            let stream = VecTokenStream::from_token_types(tokens);
            black_box(Parser::new(stream))
        })
    });
}

// TODO: Add more benchmarks as parsing functionality is implemented
// fn benchmark_expression_parsing(c: &mut Criterion) { ... }
// fn benchmark_statement_parsing(c: &mut Criterion) { ... }
// fn benchmark_large_file_parsing(c: &mut Criterion) { ... }

criterion_group!(
    benches,
    benchmark_token_stream_creation,
    benchmark_parser_creation
);
criterion_main!(benches);
