---
title: "CLI Basics"
duration: "5m"
level: "beginner"
---

# CLI Basics

> **Duration**: 5 minutes
> **Goal**: Learn essential Ferra CLI commands

## Overview

This tutorial covers the basic command-line interface (CLI) commands for Ferra development.

## 1. Basic Commands (2 minutes)

```bash
# Create a new project
lang new my_project

# Build a project
lang build

# Run a project
lang run

# Check for errors
lang check

# Format code
lang fmt
```

### Command Grammar
```ebnf
NewCmd  ::= "lang" "new" IDENT
BuildCmd ::= "lang" "build" ( "--release" )?
RunCmd  ::= "lang" "run"
CheckCmd ::= "lang" "check"
FmtCmd  ::= "lang" "fmt"
```

## 2. Project Management (2 minutes)

```bash
# Add a dependency
lang add http

# Update dependencies
lang update

# Clean build artifacts
lang clean

# Show project info
lang info
```

### Command Grammar
```ebnf
AddCmd    ::= "lang" "add" IDENT
UpdateCmd ::= "lang" "update"
CleanCmd  ::= "lang" "clean"
InfoCmd   ::= "lang" "info"
```

## 3. Development Tools (1 minute)

```bash
# Start REPL
lang repl

# Generate documentation
lang doc

# Run tests
lang test

# Show help
lang help
```

### Command Grammar
```ebnf
ReplCmd  ::= "lang" "repl"
DocCmd   ::= "lang" "doc"
TestCmd  ::= "lang" "test"
HelpCmd  ::= "lang" "help" ( IDENT )?
```

## Quiz

1. What command creates a new Ferra project?
   - A. `ferra new`
   - B. `lang new`
   - C. `create ferra`
   - D. `new ferra`

2. Which command builds a project in release mode?
   - A. `lang build --release`
   - B. `lang build -r`
   - C. `lang build release`
   - D. `lang build -m release`

3. How do you add a dependency to your project?
   - A. `lang install`
   - B. `lang add`
   - C. `lang dependency`
   - D. `lang require`

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Syntax Grammar](../../reference/SYNTAX_GRAMMAR_V0.1.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [REST API Basics](../week-1/rest_api_basics.md)
- [Data Models](../week-1/data_models.md)
- [Error Handling](../week-1/error_handling.md)

## Video Content

- **Duration**: 5 minutes
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (30s)
  2. Basic Commands (2m)
  3. Project Management (2m)
  4. Development Tools (30s)
  5. Conclusion (30s)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration 

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
} 