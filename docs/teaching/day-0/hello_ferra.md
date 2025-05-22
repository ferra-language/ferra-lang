---
title: "Day-0 Lab: Hello, Ferra!"
duration: "5m"
level: "beginner"
---

# Day-0 Lab: Hello, Ferra!

> **Duration**: 5 minutes
> **Goal**: Get up and running with Ferra in under 5 minutes

## Overview

This lab will guide you through installing Ferra, creating your first project, and running a simple "Hello, Ferra!" program.

## 1. Prerequisites (1 minute)

*   **System Requirements**:
    *   macOS, Linux, or Windows
    *   VSCode (recommended)
    *   Git

*   **VSCode Setup**:
    *   Install the [Ferra VSCode Plugin](../../reference/VSCODE_PLUGIN_ALPHA_SPEC.md) for syntax highlighting and live diagnostics

*   **Installation**:
    ```bash
    curl -sSL https://get.ferra.dev | sh
    ```

## 2. Create Your First Project (1 minute)

```bash
lang new hello_ferra
cd hello_ferra
```

**CLI Grammar**:
```ebnf
NewCmd  ::= "lang" "new" IDENT
```
See [SYNTAX_GRAMMAR_V0.1.md](../../reference/SYNTAX_GRAMMAR_V0.1.md) for details.

## 3. Write the Greet Function (1 minute)

Create `src/main.ferra`:
```ferra
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

**Cross-References**:
- [Function Syntax](../../reference/SYNTAX_GRAMMAR_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)

## 4. Build and Run (1 minute)

```bash
lang build
./hello_ferra
```

**CLI Grammar**:
```ebnf
BuildCmd ::= "lang" "build" ( "--release" )?
```
See [SYNTAX_GRAMMAR_V0.1.md](../../reference/SYNTAX_GRAMMAR_V0.1.md) for details.

## 5. Next Steps (1 minute)

*   [Week-1 Tutorial: Building a REST API](../week-1/rest_api.md)
*   [Ferra Documentation](../../reference/)
*   [Community Forums](https://community.ferra.dev)

## Quiz

1. What command is used to create a new Ferra project?
   - A. `lang new <name>`
   - B. `ferra init <name>`
   - C. `lang create <name>`
   - D. `ferra new <name>`

2. What is the purpose of the `greet` function in the Day-0 lab?
   - A. To validate input data
   - B. To return a greeting message
   - C. To handle errors
   - D. To connect to a database

3. Which file contains the main entry point for a Ferra project?
   - A. `src/main.ferra`
   - B. `src/app.ferra`
   - C. `src/index.ferra`
   - D. `src/entry.ferra`

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Common Issues

If you run into any problems:

1. **Installation fails**: Make sure you have curl installed
2. **Build errors**: Check that your code matches exactly
3. **Permission denied**: Run `chmod +x hello_ferra` after building

## Resources

- [Ferra Documentation](https://docs.ferra.dev)
- [Language Reference](../../reference/README.md)
- [Community Discord](https://discord.gg/ferra)
- [GitHub Repository](https://github.com/Starr2591/ferra-lang) 
