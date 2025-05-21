---
title: "Ferra Teaching Materials - Initial Outline v0.1"
status: "Initial Draft - Module 3.7 (Steps 3.7.1 - 3.7.3)"
---

# Ferra Teaching Materials - Initial Outline v0.1

> **Status:** Initial Draft - Module 3.7 (Steps 3.7.1 - 3.7.3)

## 1. Introduction and Goals

This document outlines the initial teaching materials for Ferra, focusing on the Day-0 lab and Week-1 tutorial content. The goal is to provide a clear, engaging, and accessible learning path for new Ferra developers, with a strong emphasis on practical, hands-on experience.

*   **Purpose**:
    *   To provide a structured learning path for new Ferra developers
    *   To ensure consistent, high-quality teaching materials across different formats
    *   To establish clear standards for documentation and video content
    *   To create an engaging, practical introduction to Ferra's key features

*   **Core Requirements (from `Steps.md`)**:
    *   **Day-0 lab**: Compile & run `greet` function in 5 minutes
    *   **Week-1**: Build a REST API
    *   **All tutorials**: Mirrored in Markdown and narrated video with subtitles (`.vtt`)

## 2. Day-0 Lab: "Hello, Ferra!" (Step 3.7.1)

The Day-0 lab is designed to get developers up and running with Ferra in under 5 minutes, focusing on the absolute basics.

### 2.1. Learning Objectives

*   Install Ferra toolchain
*   Create a new Ferra project
*   Write and run a simple program
*   Understand basic syntax and compilation

### 2.2. Lab Structure

1.  **Prerequisites** (1 minute)
    *   System requirements
    *   Required tools (VSCode, Git)
    *   Installation commands
    *   **VSCode Plugin**: Enable syntax highlighting and live diagnostics by following the [VSCode Plugin Alpha Spec](../../reference/VSCODE_PLUGIN_ALPHA_SPEC.md)

2.  **Create Your First Project** (1 minute)
    ```bash
    # Install Ferra
    curl -sSL https://get.ferra.dev | sh

    # Create a new project
    lang new hello_ferra
    cd hello_ferra
    ```

    **CLI Grammar**:
    ```ebnf
    NewCmd  ::= "lang" "new" IDENT
    ```
    See [SYNTAX_GRAMMAR_V0.1.md](../../reference/SYNTAX_GRAMMAR_V0.1.md) for details.

3.  **Write the Greet Function** (1 minute)
    ```ferra
    // src/main.ferra
    fn greet(name: String) -> String {
        "Hello, " + name + "!"
    }

    fn main() {
        println(greet("Ferra"))
    }
    ```

    **Cross-References**:
    - [Function Syntax](../../reference/SYNTAX_GRAMMAR_V0.1.md)
    - [Standard Library](../../reference/STDLIB_CORE_V0.1.md)

4.  **Build and Run** (1 minute)
    ```bash
    lang build
    ./hello_ferra
    ```

    **CLI Grammar**:
    ```ebnf
    BuildCmd ::= "lang" "build" ( "--release" )?
    ```
    See [SYNTAX_GRAMMAR_V0.1.md](../../reference/SYNTAX_GRAMMAR_V0.1.md) for details.

5.  **Next Steps** (1 minute)
    *   Links to Week-1 tutorial
    *   Community resources
    *   Documentation references

### 2.3. Video Content

*   **Duration**: 5 minutes
*   **Format**: Screen recording with voice-over
*   **Sections**:
    1.  Introduction (30s)
    2.  Installation (1m)
    3.  Project Creation (1m)
    4.  Code Walkthrough (1m)
    5.  Build & Run (1m)
    6.  Conclusion (30s)

### 2.4. Accessibility Features

*   **Subtitles**: `.vtt` file with accurate timestamps
*   **Transcript**: Full text transcript in Markdown
*   **Code Blocks**: High contrast, syntax highlighted
*   **Audio**: Clear, well-paced narration

## 3. Week-1 Tutorial: Building a REST API (Step 3.7.2)

The Week-1 tutorial builds on the Day-0 lab to create a practical REST API using Ferra's core features.

### 3.1. Learning Objectives

*   Understand Ferra's HTTP server capabilities
*   Work with JSON data
*   Implement basic CRUD operations
*   Handle errors and edge cases
*   Test API endpoints

### 3.2. Tutorial Structure

> **Total Estimated Time**: ≈6 hours

1.  **Project Setup** (30 minutes)
    ```bash
    lang new todo_api
    cd todo_api
    ```

2.  **Dependencies** (30 minutes)
    ```toml
    # Ferra.toml
    [dependencies]
    http = "1.0.0"
    json = "1.0.0"
    ```

3.  **Data Model** (1 hour)
    ```ferra
    data Todo {
        id: Int
        title: String
        completed: Bool
    }
    ```

4.  **API Endpoints** (2 hours)
    *   GET /todos
    *   POST /todos
    *   GET /todos/:id
    *   PUT /todos/:id
    *   DELETE /todos/:id

5.  **Error Handling** (1 hour)
    *   Input validation
    *   Error responses
    *   Status codes

6.  **Testing** (1 hour)
    *   Unit tests
    *   Integration tests
    *   API testing

### 3.3. Video Content

*   **Duration**: 6 hours (split into 12 30-minute episodes)
*   **Format**: Screen recording with voice-over
*   **Episodes**:
    1.  Introduction & Setup
    2.  Project Structure
    3.  Data Models
    4.  HTTP Server Basics
    5.  GET Endpoints
    6.  POST Endpoint
    7.  PUT Endpoint
    8.  DELETE Endpoint
    9.  Error Handling
    10. Input Validation
    11. Testing Basics
    12. API Testing

### 3.4. Accessibility Features

*   **Subtitles**: `.vtt` files for each episode
*   **Transcripts**: Full text transcripts in Markdown
*   **Code Blocks**: High contrast, syntax highlighted
*   **Audio**: Clear, well-paced narration
*   **Chapter Markers**: For easy navigation

## 4. Content Standards (Step 3.7.3)

### 4.1. Markdown Standards

*   **File Structure**:
    *   One file per tutorial/lab
    *   Clear section hierarchy
    *   Consistent formatting

*   **Code Blocks**:
    *   Language-specific syntax highlighting
    *   Line numbers for longer examples
    *   Clear comments and explanations

*   **Images and Diagrams**:
    *   Alt text for all images
    *   SVG format preferred
    *   High contrast colors

### 4.2. Video Standards

*   **Technical Requirements**:
    *   Resolution: 1920x1080 minimum (recommended), 720p minimum
    *   Frame rate: 30fps
    *   Audio: 48kHz, 16-bit
    *   Code font: JetBrains Mono or Fira Code

*   **Content Guidelines**:
    *   Clear introduction and conclusion
    *   Progress indicators
    *   Code zoom for important sections
    *   Consistent pacing

*   **Accessibility**:
    *   `.vtt` subtitles with accurate timestamps
    *   High contrast UI elements
    *   Clear audio with minimal background noise
    *   Screen reader friendly

### 4.3. Quality Assurance

*   **Review Process**:
    *   Technical accuracy check
    *   Accessibility compliance
    *   Content clarity
    *   Code correctness

*   **Feedback Loop**:
    *   Student feedback collection
    *   Regular content updates
    *   Issue tracking and resolution

## 5. Next Steps

*   **Month-1 Tutorial**: Cross-platform GUI + serverless backend (Module 4.5)
*   **Comprehensive Documentation**: Language reference and API docs (Module 4.5)
*   **Interactive Examples**: Browser-based code playground
*   **Community Contributions**: Guidelines for tutorial submissions

## 6. References

*   `STDLIB_CORE_V0.1.md`: Core library documentation
*   `FFI_C_CPP.md`: FFI capabilities for API integration
*   `CONCURRENCY_MODEL.md`: Async/await for API endpoints
*   `SECURITY_MODEL.md`: Security considerations for APIs
*   `VSCODE_PLUGIN_ALPHA_SPEC.md`: IDE integration for tutorials

## 7. Quiz Template

### Day-0 Lab Quiz

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

### Week-1 Tutorial Quiz

1. What is the purpose of the `validate` method in the Todo model?
   - A. To ensure the title is not empty and not too long
   - B. To check if the todo is completed
   - C. To generate a unique ID
   - D. To format the created_at timestamp

2. Which HTTP status code is returned when a todo is successfully created?
   - A. 200
   - B. 201
   - C. 204
   - D. 400

3. What is the purpose of the `error_response` helper function?
   - A. To log errors to a file
   - B. To centralize error responses for consistency
   - C. To validate input data
   - D. To handle database errors

### Quiz Stub Example

```json
{
  "id": "day0_quiz_1",
  "question": "What command is used to create a new Ferra project?",
  "options": [
    "lang new <name>",
    "ferra init <name>",
    "lang create <name>",
    "ferra new <name>"
  ],
  "correct": 0
}
```

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the full JSON schema used for auto-grading integration. 