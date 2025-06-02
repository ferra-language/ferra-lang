# Ferra Parser User API Guide

**Version**: 1.0  
**Last Updated**: January 2025  
**Target Audience**: Developers integrating the Ferra parser into applications

---

## Overview

The Ferra Parser is a high-performance, arena-allocated recursive descent parser with Pratt expression parsing. It converts Ferra source code into a well-typed Abstract Syntax Tree (AST) with comprehensive error recovery and positive-first error messaging.

### Key Features
- **Arena Allocation**: Zero-copy AST nodes with automatic memory management
- **Pratt Expression Parsing**: Proper operator precedence and associativity
- **Error Recovery**: Intelligent error recovery with positive-first messaging
- **Rich Diagnostics**: Integration with `miette` for beautiful error reports
- **Thread Safe**: Pure functional parsing with no global state
- **Production Ready**: 429 parser tests covering all language features

---

## Quick Start

### Basic Setup

Add to your `Cargo.toml`:
```toml
[dependencies]
ferra_parser = "0.1"
ferra_lexer = "0.1"
```

### Simple Parsing Example

```rust
use ferra_parser::{Arena, ProgramParser};
use ferra_lexer::tokenize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create arena for AST allocation
    let arena = Arena::new();
    
    // 2. Tokenize source code
    let source = r#"
        fn main() {
            let greeting = "Hello, Ferra!";
            println!(greeting);
        }
    "#;
    
    let tokens = tokenize(source)?;
    
    // 3. Parse into AST
    let mut parser = ProgramParser::new(&arena, tokens);
    let ast = parser.parse_compilation_unit()?;
    
    // 4. Use the AST
    println!("Parsed {} top-level items", ast.items.len());
    
    Ok(())
}
```

## Architecture Overview

### Components Hierarchy

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Your App     │    │  ferra_parser   │    │  ferra_lexer    │
│                │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │    Code     │ │────▶│ │    Arena    │ │    │ │  Tokenizer  │ │
│ │ Generation  │ │    │ │             │ │    │ │             │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│                 │    │        ▲        │    │        ▲        │
│ ┌─────────────┐ │    │        │        │    │        │        │
│ │    AST      │ │◀───│ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ Processing  │ │    │ │   Parsers   │ │◀───│ │   Tokens    │ │
│ └─────────────┘ │    │ │             │ │    │ │             │ │
└─────────────────┘    │ └─────────────┘ │    │ └─────────────┘ │
                       └─────────────────┘    └─────────────────┘
```

### Memory Management

The parser uses arena allocation for zero-copy AST construction:

```rust
use ferra_parser::Arena;

// Arena owns all AST nodes
let arena = Arena::new();

// All parsed nodes reference arena memory
let ast = parse_with_arena(&arena, source)?;

// Arena automatically cleans up when dropped
// (ast references become invalid after arena drop)
```

## Parser Types

### 1. ProgramParser - Complete Programs

**Use Case**: Parsing entire source files  
**Input**: Complete Ferra programs  
**Output**: `CompilationUnit` with all top-level items

```rust
use ferra_parser::{Arena, ProgramParser};

let arena = Arena::new();
let mut parser = ProgramParser::new(&arena, tokens);

// Parse complete program
let compilation_unit = parser.parse_compilation_unit()?;

// Access top-level items
for item in compilation_unit.items {
    match item {
        Item::Function(func) => println!("Function: {}", func.name),
        Item::DataClass(data) => println!("Data class: {}", data.name),
        Item::ExternBlock(extern_) => println!("Extern block"),
    }
}
```

### 2. StatementParser - Individual Statements

**Use Case**: REPL, incremental parsing, statement-by-statement analysis  
**Input**: Single statements  
**Output**: `Statement` nodes

```rust
use ferra_parser::{Arena, StatementParser};

let arena = Arena::new();
let mut parser = StatementParser::new(&arena, tokens);

// Parse single statement
let statement = parser.parse_statement()?;

match statement {
    Statement::VariableDecl(var) => {
        println!("Variable: {} = {:?}", var.name, var.initializer);
    }
    Statement::FunctionDecl(func) => {
        println!("Function: {} with {} parameters", 
                 func.name, func.parameters.len());
    }
    // ... handle other statement types
}
```

### 3. PrattParser - Expression Parsing

**Use Case**: Expression evaluation, calculator, macro expansion  
**Input**: Expression tokens  
**Output**: `Expression` nodes with proper precedence

```rust
use ferra_parser::{Arena, PrattParser};
use ferra_parser::token::VecTokenStream;

let arena = Arena::new();
let token_stream = VecTokenStream::from_token_types(expression_tokens);
let mut parser = PrattParser::new(&arena, token_stream);

// Parse expression with precedence
let expr = parser.parse_expression(0)?;  // 0 = minimum precedence

// Access expression tree
match expr {
    Expression::Binary { left, operator, right } => {
        println!("Binary op: {} {} {}", left, operator, right);
    }
    Expression::Literal(lit) => {
        println!("Literal: {:?}", lit);
    }
    // ... handle other expression types
}
```

### 4. BlockParser - Block Structures

**Use Case**: Parsing function bodies, control flow blocks  
**Input**: Block tokens (braced or indented)  
**Output**: `Block` with statements

```rust
use ferra_parser::{Arena, BlockParser};

let arena = Arena::new();
let mut parser = BlockParser::new(&arena, tokens);

// Parse block with automatic style detection
let block = parser.parse_block()?;

println!("Block contains {} statements", block.statements.len());
println!("Block style: {:?}", block.style); // Braced or Indented
```

---

## Core API Reference

### Parser Struct

The main entry point for parsing operations.

```rust
pub struct Parser<'arena, T: TokenStream> {
    // Private fields
}

impl<'arena, T: TokenStream> Parser<'arena, T> {
    /// Create a new parser with arena and token stream
    pub fn new(arena: &'arena Arena, tokens: T) -> Self;
    
    /// Parse a complete compilation unit (top-level program)
    pub fn parse_compilation_unit(&mut self) -> Result<&'arena CompilationUnit, Vec<ParseError>>;
    
    /// Parse a single expression
    pub fn parse_expression(&mut self) -> Result<&'arena Expression, ParseError>;
    
    /// Parse a single statement
    pub fn parse_statement(&mut self) -> Result<&'arena Statement, ParseError>;
    
    /// Parse a single top-level item (function, data class, etc.)
    pub fn parse_top_level_item(&mut self) -> Result<&'arena TopLevelItem, ParseError>;
}
```

### Arena Memory Management

Arena provides zero-copy AST allocation with automatic cleanup.

```rust
use ferra_parser::Arena;

// Create arena for AST nodes
let arena = Arena::new();

// All AST nodes are allocated in the arena
let ast = parser.parse_compilation_unit()?;

// Arena automatically deallocates when dropped
// No manual memory management required
```

### Token Stream Interface

The parser accepts any type implementing `TokenStream`:

```rust
pub trait TokenStream {
    /// Peek at the current token without consuming it
    fn peek(&self) -> &Token;
    
    /// Consume and return the current token
    fn consume(&mut self) -> Token;
    
    /// Check if we're at the end of the token stream
    fn is_at_end(&self) -> bool;
}

// Built-in implementation for Vec<Token>
let tokens: Vec<Token> = lexer.tokenize()?;
let parser = Parser::new(&arena, VecTokenStream::new(tokens));
```

### Error Handling with Recovery

```rust
use ferra_parser::{Parser, Arena, ParseError, VecTokenStream};
use ferra_lexer;

fn parse_with_error_handling(source: &str) {
    let arena = Arena::new();
    let tokens = ferra_lexer::tokenize(source).expect("Lexing failed");
    let token_stream = VecTokenStream::new(tokens);
    let mut parser = Parser::new(&arena, token_stream);
    
    match parser.parse_compilation_unit() {
        Ok(ast) => {
            println!("Successfully parsed {} items", ast.items.len());
        }
        Err(errors) => {
            for error in errors {
                eprintln!("Parse error: {}", error);
                // Rich diagnostic formatting with miette
                eprintln!("{:?}", error);
            }
        }
    }
}
```

---

## AST Node Reference

### Compilation Unit

The root AST node representing a complete Ferra source file.

```rust
pub struct CompilationUnit {
    pub items: Vec<&'arena TopLevelItem>,
    pub span: Span,
}

// Usage example
let ast = parser.parse_compilation_unit()?;
for item in &ast.items {
    match item {
        TopLevelItem::Function(func) => println!("Function: {}", func.name),
        TopLevelItem::DataClass(class) => println!("Data class: {}", class.name),
        TopLevelItem::VariableDecl(var) => println!("Variable: {}", var.name),
    }
}
```

### Expressions

All expression types with proper precedence handling:

```rust
pub enum Expression {
    // Literals
    StringLiteral(StringLiteral),
    IntegerLiteral(IntegerLiteral),
    FloatLiteral(FloatLiteral),
    BooleanLiteral(BooleanLiteral),
    
    // Complex expressions
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    FunctionCall(FunctionCall),
    MemberAccess(MemberAccess),
    ArrayIndex(ArrayIndex),
    ArrayLiteral(ArrayLiteral),
    
    // Control flow
    IfExpression(IfExpression),
    MatchExpression(MatchExpression),
}

// Example: Binary expression access
match expression {
    Expression::Binary(bin_expr) => {
        println!("Left: {:?}", bin_expr.left);
        println!("Operator: {:?}", bin_expr.operator);
        println!("Right: {:?}", bin_expr.right);
    }
    Expression::FunctionCall(call) => {
        println!("Function: {:?}", call.function);
        println!("Args: {} arguments", call.arguments.len());
    }
    _ => println!("Other expression type"),
}
```

### Statements

All statement types for declarations and control flow:

```rust
pub enum Statement {
    // Declarations
    VariableDecl(VariableDeclaration),
    FunctionDecl(FunctionDeclaration),
    DataClassDecl(DataClassDeclaration),
    
    // Control flow
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),
    ForStatement(ForStatement),
    ReturnStatement(ReturnStatement),
    
    // Other
    ExpressionStatement(ExpressionStatement),
    Block(Block),
}
```

### Array Indexing Operations

The parser supports comprehensive array indexing with proper precedence:

```rust
// Basic array indexing
let item = arr[0];
let item = arr[index];

// Complex indexing expressions
let result = matrix[row + 1][col * 2];
let value = get_array()[calculate_index()];

// Chained operations
let data = obj.get_array()[index].field;
```

**AST Structure for Array Indexing:**

```rust
// arr[index] becomes:
Expression::ArrayIndex(ArrayIndex {
    array: Expression::Identifier("arr"),
    index: Expression::Identifier("index"),
    span: Span { ... }
})

// matrix[row + 1][col * 2] becomes:
Expression::ArrayIndex(ArrayIndex {
    array: Expression::ArrayIndex(ArrayIndex {
        array: Expression::Identifier("matrix"),
        index: Expression::Binary(/* row + 1 */),
        span: Span { ... }
    }),
    index: Expression::Binary(/* col * 2 */),
    span: Span { ... }
})
```

**Precedence Handling:**
Array indexing has the highest precedence among postfix operators:
1. `arr[i]` - Array indexing
2. `obj.field` - Member access  
3. `func()` - Function call

**Complex Expression Examples:**
```rust
// All valid expressions with proper precedence:
obj.method()[0]           // (obj.method())[0]
arr[i + 1].field         // (arr[i + 1]).field
get_matrix()[x][y]       // ((get_matrix())[x])[y]
items[0..5][2]          // (items[0..5])[2]
```

---

## Advanced Integration Patterns

### 1. Incremental Parsing

For editors and IDEs that need to reparse on changes:

```rust
use ferra_parser::{Arena, ProgramParser, ParseError};

struct IncrementalParser {
    arena: Arena,
    last_valid_ast: Option<CompilationUnit>,
}

impl IncrementalParser {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            last_valid_ast: None,
        }
    }
    
    pub fn parse_update(&mut self, source: &str) -> Result<&CompilationUnit, ParseError> {
        // Try to parse new version
        let tokens = ferra_lexer::tokenize(source)?;
        let mut parser = ProgramParser::new(&self.arena, tokens);
        
        match parser.parse_compilation_unit() {
            Ok(ast) => {
                self.last_valid_ast = Some(ast);
                Ok(self.last_valid_ast.as_ref().unwrap())
            }
            Err(e) => {
                // Keep last valid AST for partial functionality
                if let Some(ref ast) = self.last_valid_ast {
                    eprintln!("Parse error, using last valid AST: {}", e);
                    Ok(ast)
                } else {
                    Err(e)
                }
            }
        }
    }
}
```

### 2. Error Collection and Recovery

Collect multiple errors instead of stopping at first error:

```rust
use ferra_parser::{Arena, ProgramParser, ParseError};

fn parse_with_error_collection(source: &str) -> (Option<CompilationUnit>, Vec<ParseError>) {
    let arena = Arena::new();
    let tokens = match ferra_lexer::tokenize(source) {
        Ok(tokens) => tokens,
        Err(e) => return (None, vec![e.into()]),
    };
    
    let mut parser = ProgramParser::new(&arena, tokens);
    
    // Enable error recovery mode
    parser.set_error_recovery(true);
    
    match parser.parse_compilation_unit() {
        Ok(ast) => {
            let errors = parser.collect_errors(); // Get non-fatal errors
            (Some(ast), errors)
        }
        Err(fatal_error) => {
            let mut errors = parser.collect_errors();
            errors.push(fatal_error);
            (None, errors)
        }
    }
}
```

### 3. AST Visitor Pattern

For traversing and analyzing parsed AST:

```rust
use ferra_parser::ast::{CompilationUnit, Statement, Expression};

trait AstVisitor {
    fn visit_compilation_unit(&mut self, unit: &CompilationUnit) {
        for item in &unit.items {
            self.visit_item(item);
        }
    }
    
    fn visit_statement(&mut self, stmt: &Statement);
    fn visit_expression(&mut self, expr: &Expression);
    fn visit_item(&mut self, item: &Item);
}

// Example: Count function definitions
struct FunctionCounter {
    count: usize,
}

impl AstVisitor for FunctionCounter {
    fn visit_item(&mut self, item: &Item) {
        if let Item::Function(_) = item {
            self.count += 1;
        }
    }
    
    fn visit_statement(&mut self, _stmt: &Statement) {}
    fn visit_expression(&mut self, _expr: &Expression) {}
}

// Usage
let mut counter = FunctionCounter { count: 0 };
counter.visit_compilation_unit(&ast);
println!("Found {} functions", counter.count);
```

### 4. Custom Error Handling

Integrate with your application's error handling:

```rust
use ferra_parser::ParseError;
use miette::{Diagnostic, SourceSpan};

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum MyAppError {
    #[error("Failed to parse Ferra source")]
    #[diagnostic(help("Check syntax and try again"))]
    ParseError {
        #[from]
        source: ParseError,
        #[label("Error occurred here")]
        span: SourceSpan,
    },
    
    #[error("Semantic analysis failed")]
    SemanticError { message: String },
}

fn parse_and_analyze(source: &str) -> Result<AnalysisResult, MyAppError> {
    // Parse with custom error conversion
    let arena = Arena::new();
    let tokens = ferra_lexer::tokenize(source)
        .map_err(|e| MyAppError::ParseError { 
            source: e.into(), 
            span: (0, source.len()).into() 
        })?;
    
    let mut parser = ProgramParser::new(&arena, tokens);
    let ast = parser.parse_compilation_unit()
        .map_err(|e| MyAppError::ParseError { 
            source: e, 
            span: (0, source.len()).into() 
        })?;
    
    // Continue with semantic analysis...
    analyze_ast(ast)
}
```

## Performance Optimization

### 1. Arena Reuse

Reuse arenas for better performance in batch processing:

```rust
use ferra_parser::Arena;

struct BatchParser {
    arena: Arena,
}

impl BatchParser {
    pub fn new() -> Self {
        Self { arena: Arena::new() }
    }
    
    pub fn parse_batch(&mut self, sources: &[&str]) -> Vec<Option<CompilationUnit>> {
        let mut results = Vec::new();
        
        for source in sources {
            // Parse with reused arena
            let result = self.parse_single(source);
            results.push(result);
            
            // Reset arena for next parse (keeps memory allocated)
            self.arena.reset();
        }
        
        results
    }
    
    fn parse_single(&self, source: &str) -> Option<CompilationUnit> {
        let tokens = ferra_lexer::tokenize(source).ok()?;
        let mut parser = ProgramParser::new(&self.arena, tokens);
        parser.parse_compilation_unit().ok()
    }
}
```

### 2. Streaming Parser

For large files, parse in chunks:

```rust
use std::io::{BufReader, BufRead};
use ferra_parser::{Arena, StatementParser};

fn parse_large_file(file_path: &str) -> Result<Vec<Statement>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let arena = Arena::new();
    let mut statements = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        
        // Parse line as statement
        let tokens = ferra_lexer::tokenize(&line)?;
        let mut parser = StatementParser::new(&arena, tokens);
        
        if let Ok(stmt) = parser.parse_statement() {
            statements.push(stmt);
        }
        
        // Reset arena after each statement to limit memory usage
        arena.reset();
    }
    
    Ok(statements)
}
```

## Testing Integration

### Unit Testing with Parser

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ferra_parser::{Arena, PrattParser};
    use ferra_parser::token::VecTokenStream;
    
    fn parse_expression(input: &str) -> Expression {
        let arena = Arena::new();
        let tokens = ferra_lexer::tokenize(input).unwrap();
        let token_stream = VecTokenStream::from_tokens(tokens);
        let mut parser = PrattParser::new(&arena, token_stream);
        parser.parse_expression(0).unwrap()
    }
    
    #[test]
    fn test_arithmetic_expressions() {
        let expr = parse_expression("1 + 2 * 3");
        
        // Verify correct precedence
        if let Expression::Binary { left, operator, right } = expr {
            assert_eq!(operator, BinaryOperator::Add);
            assert!(matches!(**left, Expression::Literal(_)));
            assert!(matches!(**right, Expression::Binary { .. }));
        } else {
            panic!("Expected binary expression");
        }
    }
}
```

### Integration Testing

```rust
#[test]
fn test_complete_program_parsing() {
    let source = r#"
        data Person {
            name: String,
            age: i32
        }
        
        fn create_person(name: String, age: i32) -> Person {
            Person { name, age }
        }
        
        fn main() {
            let person = create_person("Alice", 30);
            println!(person.name);
        }
    "#;
    
    let arena = Arena::new();
    let tokens = ferra_lexer::tokenize(source).unwrap();
    let mut parser = ProgramParser::new(&arena, tokens);
    let ast = parser.parse_compilation_unit().unwrap();
    
    assert_eq!(ast.items.len(), 3); // data, fn, fn
    
    // Verify specific items
    assert!(matches!(ast.items[0], Item::DataClass(_)));
    assert!(matches!(ast.items[1], Item::Function(_)));
    assert!(matches!(ast.items[2], Item::Function(_)));
}
```

## Common Patterns and Best Practices

### 1. Resource Management

```rust
// ✅ Good: Arena tied to processing scope
fn process_source(source: &str) -> Result<ProcessingResult, Error> {
    let arena = Arena::new();  // Arena created in processing scope
    let ast = parse_with_arena(&arena, source)?;
    let result = analyze_ast(ast)?;
    Ok(result)  // Arena dropped here, AST becomes invalid
}

// ❌ Avoid: Long-lived arena with temporary AST
struct Parser {
    arena: Arena,  // Long-lived arena
}

impl Parser {
    fn parse(&self, source: &str) -> Result<CompilationUnit, Error> {
        // AST references long-lived arena - memory never freed
        parse_with_arena(&self.arena, source)
    }
}
```

### 2. Error Handling

```rust
// ✅ Good: Specific error handling
match parser.parse_compilation_unit() {
    Ok(ast) => process_ast(ast),
    Err(ParseError::UnexpectedToken { expected, found, .. }) => {
        eprintln!("Expected {}, found {}", expected, found);
    }
    Err(ParseError::UnmatchedDelimiter { delimiter, .. }) => {
        eprintln!("Unmatched {}", delimiter);
    }
    Err(e) => {
        eprintln!("Parse error: {}", e);
    }
}

// ❌ Avoid: Generic error handling
if let Err(e) = parser.parse_compilation_unit() {
    eprintln!("Parse failed: {}", e);  // Not actionable
}
```

### 3. Performance Monitoring

```rust
use std::time::Instant;

fn parse_with_timing(source: &str) -> Result<(CompilationUnit, Duration), Error> {
    let start = Instant::now();
    
    let arena = Arena::new();
    let tokens = ferra_lexer::tokenize(source)?;
    let mut parser = ProgramParser::new(&arena, tokens);
    let ast = parser.parse_compilation_unit()?;
    
    let duration = start.elapsed();
    
    if duration > Duration::from_millis(100) {
        eprintln!("Warning: Slow parse took {:?}", duration);
    }
    
    Ok((ast, duration))
}
```

## Troubleshooting

### Common Issues and Solutions

#### 1. "AST references invalid after arena drop"

```rust
// ❌ Problem: AST outlives arena
fn parse_and_extract(source: &str) -> CompilationUnit {
    let arena = Arena::new();
    let ast = parse_with_arena(&arena, source).unwrap();
    ast  // ❌ Error: AST references arena memory that's about to be dropped
}

// ✅ Solution: Process AST before arena drop
fn parse_and_extract(source: &str) -> ProcessedData {
    let arena = Arena::new();
    let ast = parse_with_arena(&arena, source).unwrap();
    extract_data_from_ast(ast)  // Process before arena drop
}
```

#### 2. "Unexpected token" errors

```rust
// Check token stream manually
let tokens = ferra_lexer::tokenize(source)?;
for (i, token) in tokens.iter().enumerate() {
    println!("{}: {:?}", i, token);
}

// Then identify where parsing diverges from expectation
```

#### 3. Memory usage growing over time

```rust
// ✅ Reset arena between parses
let mut arena = Arena::new();
for source in sources {
    let ast = parse_with_arena(&arena, source)?;
    process_ast(ast);
    arena.reset();  // Free memory but keep arena capacity
}
```

## API Reference Summary

### Core Types

- `Arena` - Memory management for AST nodes
- `ProgramParser` - Parse complete programs
- `StatementParser` - Parse individual statements  
- `PrattParser` - Parse expressions with precedence
- `BlockParser` - Parse block structures

### Key Traits

- `TokenStream` - Abstract token consumption
- `AstNode` - Common interface for AST nodes

### Error Types

- `ParseError` - All parser error conditions
- `TokenError` - Lexer integration errors
- `RecoveryError` - Error recovery failures

---

**Next Steps**: See [ERROR_CATALOG.md](./ERROR_CATALOG.md) for error handling details and [CONTRIBUTOR_GUIDE.md](./CONTRIBUTOR_GUIDE.md) for development guidelines. 