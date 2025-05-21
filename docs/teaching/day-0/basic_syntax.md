---
title: "Basic Syntax & Setup"
duration: "5m"
level: "beginner"
---

# Basic Syntax & Setup

> **Duration**: 5 minutes
> **Goal**: Get developers up and running with Ferra in under 5 minutes

## Overview

This tutorial covers the absolute basics of Ferra, focusing on getting you started quickly.

## 1. Prerequisites (1 minute)

### 1.1. System Requirements
- Modern operating system (Windows, macOS, Linux)
- VSCode with Ferra plugin
- Git

### 1.2. VSCode Plugin Setup
1. Open VSCode
2. Install Ferra plugin
3. Enable syntax highlighting
4. Enable live diagnostics

## 2. Installation (1 minute)

```bash
# Install Ferra
curl -sSL https://get.ferra.dev | sh

# Verify installation
lang --version
```

## 3. First Project (1 minute)

```bash
# Create a new project
lang new hello_ferra
cd hello_ferra
```

## 4. Write Your First Program (1 minute)

```ferra
// src/main.ferra
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

## 5. Build and Run (1 minute)

```bash
lang build
./hello_ferra
```

## Quiz

1. What is the correct way to declare a function in Ferra?
   - A. `function greet(name)`
   - B. `def greet(name)`
   - C. `fn greet(name: String)`
   - D. `func greet(name)`

2. Which command creates a new Ferra project?
   - A. `ferra new`
   - B. `lang new`
   - C. `create ferra`
   - D. `new ferra`

3. What is the correct way to build a Ferra project?
   - A. `ferra build`
   - B. `lang build`
   - C. `build ferra`
   - D. `compile ferra`

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Syntax Grammar](../../reference/SYNTAX_GRAMMAR_V0.1.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [First Program](./first_program.md)
- [VSCode Integration](./vscode_integration.md)
- [CLI Basics](./cli_basics.md)

## Video Content

- **Duration**: 5 minutes
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (30s)
  2. Installation (1m)
  3. Project Creation (1m)
  4. Code Walkthrough (1m)
  5. Build & Run (1m)
  6. Conclusion (30s)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration 