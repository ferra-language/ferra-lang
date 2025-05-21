---
title: "First Program"
duration: "5m"
level: "beginner"
---

# First Program

> **Duration**: 5 minutes
> **Goal**: Write and run your first Ferra program

## Overview

This tutorial guides you through writing and running your first Ferra program, focusing on the essential concepts.

## 1. Project Structure (1 minute)

```
hello_ferra/
├── src/
│   └── main.ferra
├── tests/
│   └── main_test.ferra
├── Ferra.toml
└── README.md
```

## 2. Write the Program (2 minutes)

```ferra
// src/main.ferra
#[ai::tag(core_component)]
fn greet(name: String) -> String {
    "Hello, " + name + "!"
}

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
    println(greet("Ferra"))
}
```

### Code Explanation

1. `fn greet(name: String) -> String`: Function declaration with:
   - Parameter: `name` of type `String`
   - Return type: `String`

2. `fn main()`: Entry point function
   - No parameters
   - No return type (implicit `Unit`)

3. `println()`: Standard library function for output

## 3. Build and Run (2 minutes)

```bash
# Build the program
lang build

# Run the program
./hello_ferra
```

### Expected Output
```
Hello, Ferra!
```

## Quiz

1. What is the correct way to declare a function with a return type?
   - A. `fn greet(name) -> String`
   - B. `fn greet(name: String) -> String`
   - C. `function greet(name: String): String`
   - D. `def greet(name: String) -> String`

2. Which function is the entry point in Ferra?
   - A. `start()`
   - B. `init()`
   - C. `main()`
   - D. `entry()`

3. What is the correct way to print output in Ferra?
   - A. `print()`
   - B. `console.log()`
   - C. `println()`
   - D. `echo()`

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Syntax Grammar](../../reference/SYNTAX_GRAMMAR_V0.1.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [VSCode Integration](./vscode_integration.md)
- [CLI Basics](./cli_basics.md)
- [REST API Basics](../week-1/rest_api_basics.md)

## Video Content

- **Duration**: 5 minutes
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (30s)
  2. Project Structure (1m)
  3. Code Walkthrough (2m)
  4. Build & Run (1m)
  5. Conclusion (30s)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration 