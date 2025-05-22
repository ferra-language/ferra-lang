# Ferra Project Structure

## Directory Structure
```
ferra/
├── compiler/                 # Main compiler implementation
│   ├── frontend/            # Frontend components
│   │   ├── src/
│   │   │   ├── lexer/      # Lexical analysis
│   │   │   ├── parser/     # Syntax analysis
│   │   │   └── type_inference/ # Type checking
│   │   └── tests/          # Frontend tests
│   ├── midend/             # Middle-end components
│   │   ├── src/
│   │   │   ├── ir/        # SSA IR implementation
│   │   │   └── semantic_tags/ # AI semantic tags
│   │   └── tests/
│   ├── backend/            # Backend components
│   │   ├── src/
│   │   │   └── llvm/      # LLVM code generation
│   │   └── tests/
│   └── diagnostics/        # Error handling
├── stdlib/                 # Standard library
│   ├── src/
│   │   ├── io/           # I/O operations
│   │   └── collections/  # Data structures
│   └── tests/
├── tools/                  # Command-line tools
│   ├── langc/            # Compiler driver
│   └── lang/             # Package manager
├── tests/                 # Integration tests
└── examples/             # Example programs
```

## Implementation Status

### Phase 1: MVP Compiler & Tooling (Q3 2025)

#### Module 1.1: Front-End - Lexer & Parser Design 🔄
- [x] Project structure setup
- [x] Initial lexer implementation
  - [x] Token definitions
  - [x] Basic lexer operations
  - [x] Test cases
- [ ] Parser implementation
  - [ ] Pratt parser for expressions
  - [ ] GLR fallback for ambiguous constructs
  - [ ] AST structure
- [ ] Integration tests

#### Module 1.2: Front-End - Type Inference & Basic Diagnostics Design
- [ ] Type inference system
  - [ ] Hindley-Milner implementation
  - [ ] Gradual typing support
- [ ] Diagnostic system
  - [ ] Error reporting
  - [ ] Warning system

#### Module 1.3: Mid-End - SSA IR Design
- [ ] SSA IR structure
- [ ] AST to IR conversion
- [ ] Semantic tags implementation

#### Module 1.4: Back-End - Initial Target Design
- [ ] LLVM backend setup
- [ ] IR to LLVM conversion
- [ ] Basic code generation

#### Module 1.5: Core Standard Library
- [ ] I/O APIs
- [ ] Basic data structures

#### Module 1.6: AI API
- [ ] `ai::ast()` implementation
- [ ] `.note.ai` section handling

#### Module 1.7: VSCode Plugin
- [ ] Syntax highlighting
- [ ] Error integration
- [ ] Project scaffolding

## Current Focus
Currently implementing Module 1.1 (Front-End - Lexer & Parser Design):
1. ✅ Basic lexer implementation with token definitions
2. 🔄 Working on parser implementation using Pratt/GLR approach
3. Next: AST structure and type inference system

## Dependencies
- LLVM 16.0
- Rust 2021 edition
- Key crates:
  - logos (lexer)
  - inkwell (LLVM bindings)
  - thiserror (error handling)
  - serde (serialization)
  - clap (CLI) 