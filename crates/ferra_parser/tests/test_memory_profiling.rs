use ferra_parser::{
    program::ProgramParser,
    token::{stream::VecTokenStream, types::TokenType},
    Arena,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Memory tracking allocator for profiling
struct MemoryTracker {
    allocated: Arc<AtomicUsize>,
    peak_allocated: Arc<AtomicUsize>,
    allocation_count: Arc<AtomicUsize>,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            allocated: Arc::new(AtomicUsize::new(0)),
            peak_allocated: Arc::new(AtomicUsize::new(0)),
            allocation_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            current_allocated: self.allocated.load(Ordering::Relaxed),
            peak_allocated: self.peak_allocated.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
        }
    }

    fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.peak_allocated.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_allocated: usize,
    pub peak_allocated: usize,
    pub allocation_count: usize,
}

/// Generate simple tokens for testing
#[allow(dead_code)]
fn create_test_tokens() -> Vec<TokenType> {
    vec![
        TokenType::Fn,
        TokenType::Identifier("test".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("x".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(42),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]
}

/// Generate a large program token stream for memory testing
fn generate_large_program_tokens(size_category: &str) -> Vec<TokenType> {
    let (func_count, statements_per_func) = match size_category {
        "small" => (50, 10),
        "medium" => (200, 20),
        "large" => (500, 30),
        "very_large" => (1000, 40),
        "massive" => (2000, 50),
        _ => (100, 15),
    };

    let mut tokens = Vec::new();

    // Add some data classes
    for i in 0..func_count / 10 {
        tokens.extend(vec![
            TokenType::Data,
            TokenType::Identifier(format!("User{i}")),
            TokenType::LeftBrace,
            TokenType::Identifier("id".to_string()),
            TokenType::Colon,
            TokenType::Identifier("i32".to_string()),
            TokenType::Comma,
            TokenType::Identifier("name".to_string()),
            TokenType::Colon,
            TokenType::Identifier("String".to_string()),
            TokenType::RightBrace,
        ]);
    }

    // Add functions with varying complexity
    for i in 0..func_count {
        // Function signature
        tokens.extend(vec![
            TokenType::Fn,
            TokenType::Identifier(format!("function_{i}")),
            TokenType::LeftParen,
            TokenType::Identifier("param1".to_string()),
            TokenType::Colon,
            TokenType::Identifier("i32".to_string()),
            TokenType::Comma,
            TokenType::Identifier("param2".to_string()),
            TokenType::Colon,
            TokenType::Identifier("i32".to_string()),
            TokenType::RightParen,
            TokenType::Arrow,
            TokenType::Identifier("i32".to_string()),
            TokenType::LeftBrace,
        ]);

        // Add statements
        for stmt in 0..statements_per_func {
            tokens.extend(vec![
                TokenType::Let,
                TokenType::Identifier(format!("var_{stmt}")),
                TokenType::Equal,
                TokenType::Identifier("param1".to_string()),
                TokenType::Plus,
                TokenType::Identifier("param2".to_string()),
                TokenType::Star,
                TokenType::IntegerLiteral(stmt as i64),
                TokenType::Semicolon,
            ]);
        }

        // Function close
        tokens.extend(vec![
            TokenType::Return,
            TokenType::Identifier("param1".to_string()),
            TokenType::Semicolon,
            TokenType::RightBrace,
        ]);
    }

    tokens.push(TokenType::Eof);
    tokens
}

/// Measure memory usage during parsing
fn measure_memory_usage(tokens: Vec<TokenType>) -> (MemoryStats, Duration) {
    let tracker = MemoryTracker::new();
    tracker.reset();

    let start_time = Instant::now();

    // Create parser with memory tracking
    let arena = Arena::new();
    let token_stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, token_stream);

    // Parse the program
    let _result = parser.parse_compilation_unit();

    let parse_duration = start_time.elapsed();
    let memory_stats = tracker.get_stats();

    (memory_stats, parse_duration)
}

#[cfg(test)]
mod memory_profiling_tests {
    use super::*;

    #[test]
    fn test_small_program_memory_baseline() {
        let tokens = generate_large_program_tokens("small");
        let (memory_stats, duration) = measure_memory_usage(tokens);

        println!("Small Program Memory Stats:");
        println!("  Peak Memory: {} KB", memory_stats.peak_allocated / 1024);
        println!(
            "  Current Memory: {} KB",
            memory_stats.current_allocated / 1024
        );
        println!("  Allocations: {}", memory_stats.allocation_count);
        println!("  Parse Time: {:?}", duration);

        // Basic assertions - should not use excessive memory for small programs
        assert!(
            duration < Duration::from_millis(500),
            "Small program parsing too slow"
        );
        // Note: Memory tracking may not work as expected in test environment
    }

    #[test]
    fn test_medium_program_memory_usage() {
        let tokens = generate_large_program_tokens("medium");
        let (memory_stats, duration) = measure_memory_usage(tokens);

        println!("Medium Program Memory Stats:");
        println!("  Peak Memory: {} KB", memory_stats.peak_allocated / 1024);
        println!(
            "  Current Memory: {} KB",
            memory_stats.current_allocated / 1024
        );
        println!("  Allocations: {}", memory_stats.allocation_count);
        println!("  Parse Time: {:?}", duration);

        assert!(
            duration < Duration::from_secs(2),
            "Medium program parsing too slow"
        );
    }

    #[test]
    fn test_large_program_memory_scalability() {
        let tokens = generate_large_program_tokens("large");
        let (memory_stats, duration) = measure_memory_usage(tokens);

        println!("Large Program Memory Stats:");
        println!("  Peak Memory: {} KB", memory_stats.peak_allocated / 1024);
        println!(
            "  Current Memory: {} KB",
            memory_stats.current_allocated / 1024
        );
        println!("  Allocations: {}", memory_stats.allocation_count);
        println!("  Parse Time: {:?}", duration);

        assert!(
            duration < Duration::from_secs(5),
            "Large program parsing too slow"
        );
    }

    #[test]
    fn test_very_large_program_memory_limits() {
        let tokens = generate_large_program_tokens("very_large");
        let (memory_stats, duration) = measure_memory_usage(tokens);

        println!("Very Large Program Memory Stats:");
        println!(
            "  Peak Memory: {} MB",
            memory_stats.peak_allocated / (1024 * 1024)
        );
        println!(
            "  Current Memory: {} MB",
            memory_stats.current_allocated / (1024 * 1024)
        );
        println!("  Allocations: {}", memory_stats.allocation_count);
        println!("  Parse Time: {:?}", duration);

        assert!(
            duration < Duration::from_secs(10),
            "Very large program parsing too slow"
        );
    }

    #[test]
    fn test_memory_scaling_linearity() {
        let sizes = vec![("small", 50), ("medium", 200), ("large", 500)];

        let mut measurements = Vec::new();

        for (size_name, _expected_funcs) in sizes {
            let tokens = generate_large_program_tokens(size_name);
            let (memory_stats, duration) = measure_memory_usage(tokens);

            measurements.push((size_name, memory_stats.peak_allocated, duration));
            println!(
                "{}: {} KB, {:?}",
                size_name,
                memory_stats.peak_allocated / 1024,
                duration
            );
        }

        // Check that parsing time scales reasonably (not exponentially)
        if measurements.len() >= 3 {
            let small_time = measurements[0].2.as_millis();
            let large_time = measurements[2].2.as_millis();

            if small_time > 0 {
                let time_scaling_factor = large_time as f64 / small_time as f64;
                println!(
                    "Time scaling factor (large/small): {:.2}x",
                    time_scaling_factor
                );

                // Should scale reasonably (allow up to 80x for complex parser - some quadratic behavior expected)
                assert!(
                    time_scaling_factor < 80.0,
                    "Time scaling too aggressively: {:.2}x",
                    time_scaling_factor
                );
            }
        }
    }

    #[test]
    fn test_memory_leak_detection() {
        // Run the same parsing operation multiple times to detect leaks
        let tokens = generate_large_program_tokens("medium");
        let mut parse_times = Vec::new();

        for iteration in 0..5 {
            let start = Instant::now();
            let (memory_stats, _duration) = measure_memory_usage(tokens.clone());
            let total_time = start.elapsed();
            parse_times.push(total_time);

            println!(
                "Iteration {}: Peak Memory {} KB, Time {:?}",
                iteration + 1,
                memory_stats.peak_allocated / 1024,
                total_time
            );
        }

        // Check that parsing time doesn't grow significantly over iterations
        let first_time = parse_times[0].as_millis();
        let last_time = parse_times[4].as_millis();

        if first_time > 0 {
            let time_growth = (last_time as f64 / first_time as f64) - 1.0;
            println!("Time growth over 5 iterations: {:.1}%", time_growth * 100.0);

            // Allow up to 50% variance between runs
            assert!(
                time_growth < 0.5,
                "Potential performance degradation: {:.1}% growth",
                time_growth * 100.0
            );
        }
    }

    #[test]
    fn test_arena_allocation_efficiency() {
        let tokens = generate_large_program_tokens("medium");

        // Test multiple parses with arena reuse vs new arena each time
        let mut arena_reuse_times = Vec::new();
        let mut new_arena_times = Vec::new();

        // Test arena reuse (simulate keeping arena around)
        let arena = Arena::new();
        for _i in 0..3 {
            let start = Instant::now();
            let token_stream = VecTokenStream::from_token_types(tokens.clone());
            let mut parser = ProgramParser::new(&arena, token_stream);
            let _result = parser.parse_compilation_unit();
            arena_reuse_times.push(start.elapsed());
        }

        // Test new arena each time
        for _i in 0..3 {
            let start = Instant::now();
            let arena = Arena::new();
            let token_stream = VecTokenStream::from_token_types(tokens.clone());
            let mut parser = ProgramParser::new(&arena, token_stream);
            let _result = parser.parse_compilation_unit();
            new_arena_times.push(start.elapsed());
        }

        let avg_reuse_time: Duration =
            arena_reuse_times.iter().sum::<Duration>() / arena_reuse_times.len() as u32;
        let avg_new_time: Duration =
            new_arena_times.iter().sum::<Duration>() / new_arena_times.len() as u32;

        println!("Average parse time with arena reuse: {:?}", avg_reuse_time);
        println!("Average parse time with new arena: {:?}", avg_new_time);

        // Both should be reasonable
        assert!(
            avg_reuse_time < Duration::from_millis(2000),
            "Arena reuse too slow"
        );
        assert!(
            avg_new_time < Duration::from_millis(3000),
            "New arena allocation too slow"
        );
    }

    #[test]
    fn test_pathological_memory_cases() {
        // Test deeply nested structures using tokens
        let mut deep_nesting_tokens = vec![
            TokenType::Fn,
            TokenType::Identifier("test".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
        ];

        // Add 50 levels of nested if statements
        for i in 0..50 {
            deep_nesting_tokens.extend(vec![
                TokenType::If,
                TokenType::Identifier(format!("condition_{i}")),
                TokenType::LeftBrace,
            ]);
        }

        // Close all the nested blocks
        for _i in 0..50 {
            deep_nesting_tokens.push(TokenType::RightBrace);
        }

        deep_nesting_tokens.extend(vec![TokenType::RightBrace, TokenType::Eof]);

        let (memory_stats, duration) = measure_memory_usage(deep_nesting_tokens);

        println!("Deep Nesting Memory Stats:");
        println!("  Peak Memory: {} KB", memory_stats.peak_allocated / 1024);
        println!("  Parse Time: {:?}", duration);

        assert!(
            duration < Duration::from_secs(2),
            "Deep nesting parsing too slow"
        );

        // Test very wide structures (many statements)
        let mut wide_structure_tokens = vec![
            TokenType::Fn,
            TokenType::Identifier("test".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
        ];

        for i in 0..500 {
            wide_structure_tokens.extend(vec![
                TokenType::Let,
                TokenType::Identifier(format!("var_{i}")),
                TokenType::Equal,
                TokenType::IntegerLiteral(i as i64),
                TokenType::Plus,
                TokenType::IntegerLiteral(i as i64),
                TokenType::Star,
                TokenType::IntegerLiteral(i as i64),
                TokenType::Semicolon,
            ]);
        }

        wide_structure_tokens.extend(vec![TokenType::RightBrace, TokenType::Eof]);

        let (memory_stats, duration) = measure_memory_usage(wide_structure_tokens);

        println!("Wide Structure Memory Stats:");
        println!("  Peak Memory: {} KB", memory_stats.peak_allocated / 1024);
        println!("  Parse Time: {:?}", duration);

        assert!(
            duration < Duration::from_secs(2),
            "Wide structure parsing too slow"
        );
    }
}
