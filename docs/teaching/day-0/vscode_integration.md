---
title: "VSCode Integration"
duration: "5m"
level: "beginner"
---

# VSCode Integration

> **Duration**: 5 minutes
> **Goal**: Set up and use VSCode with Ferra

## Overview

This tutorial covers setting up VSCode for Ferra development, including syntax highlighting, live diagnostics, and essential features.

## 1. Install VSCode (1 minute)

1. Download VSCode from [code.visualstudio.com](https://code.visualstudio.com)
2. Install for your operating system
3. Launch VSCode

## 2. Install Ferra Plugin (2 minutes)

1. Open VSCode
2. Click Extensions icon (or press `Cmd+Shift+X` on macOS, `Ctrl+Shift+X` on Windows/Linux)
3. Search for "Ferra"
4. Click "Install" on the official Ferra plugin

### Plugin Features
- Syntax highlighting
- Live diagnostics
- Code completion
- Go to definition
- Find references
- Hover documentation

## 3. Configure Settings (2 minutes)

```json
// settings.json
{
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "ferra.ferra",
    "editor.fontFamily": "JetBrains Mono, Fira Code, monospace",
    "editor.fontSize": 14,
    "editor.lineHeight": 1.5,
    "editor.rulers": [80, 100],
    "editor.tabSize": 4,
    "editor.insertSpaces": true,
    "files.trimTrailingWhitespace": true,
    "files.insertFinalNewline": true
}
```

### Key Settings Explained
1. `formatOnSave`: Auto-format code on save
2. `defaultFormatter`: Use Ferra's formatter
3. `fontFamily`: Recommended monospace fonts
4. `rulers`: Visual guides for line length
5. `tabSize`: Consistent indentation

## Quiz

1. How do you install the Ferra plugin in VSCode?
   - A. Download from ferra.dev
   - B. Use the Extensions marketplace
   - C. Copy files to plugins folder
   - D. Run install script

2. Which setting enables auto-formatting on save?
   - A. `editor.autoFormat`
   - B. `editor.formatOnSave`
   - C. `editor.formatOnType`
   - D. `editor.autoIndent`

3. What is the recommended font for Ferra development?
   - A. Arial
   - B. Times New Roman
   - C. JetBrains Mono
   - D. Comic Sans

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [VSCode Plugin Alpha Spec](../../reference/VSCODE_PLUGIN_ALPHA_SPEC.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)
- [Syntax Grammar](../../reference/SYNTAX_GRAMMAR_V0.1.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)

## Next Steps

- [CLI Basics](./cli_basics.md)
- [REST API Basics](../week-1/rest_api_basics.md)
- [Data Models](../week-1/data_models.md)

## Video Content

- **Duration**: 5 minutes
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (30s)
  2. VSCode Installation (1m)
  3. Plugin Setup (2m)
  4. Configuration (1m)
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