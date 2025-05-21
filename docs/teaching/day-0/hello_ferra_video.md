---
title: "Day-0 Lab: Hello, Ferra! (Video Script)"
duration: "5m"
format: "Screen recording with voice-over"
---

# Day-0 Lab: Hello, Ferra! (Video Script)

> **Duration**: 5 minutes
> **Format**: Screen recording with voice-over
> **Resolution**: 1920x1080
> **Audio**: 48kHz, 16-bit
> **Font**: JetBrains Mono
> **See**: [hello_ferra.md](./hello_ferra.md) for the written lab

## 1. Introduction (30s)

*   **Visual**: Title screen with Ferra logo
*   **Narration**: "Welcome to the Day-0 lab for Ferra. In this 5-minute tutorial, you'll install Ferra, create your first project, and run a simple 'Hello, Ferra!' program."

## 2. Installation (1m)

*   **Visual**: Terminal window
*   **Narration**: "First, let's install Ferra. Open your terminal and run this command:"
*   **Code**: `curl -sSL https://get.ferra.dev | sh`
*   **Narration**: "This will install the Ferra compiler, package manager, and basic development tools."

## 3. Project Creation (1m)

*   **Visual**: Terminal window
*   **Narration**: "Now, let's create a new Ferra project. Run:"
*   **Code**: `lang new hello_ferra`
*   **Narration**: "This creates a new project with a basic structure. Navigate into the project directory:"
*   **Code**: `cd hello_ferra`

## 4. Code Walkthrough (1m)

*   **Visual**: VSCode window
*   **Narration**: "Open `src/main.ferra` in your editor. Here's a simple greeting function:"
*   **Code**:
    ```ferra
    fn greet(name: String) -> String {
        "Hello, " + name + "!"
    }

    fn main() {
        println(greet("Ferra"))
    }
    ```
*   **Narration**: "This function takes a name and returns a greeting. The `main` function calls it with 'Ferra'."

## 5. Build & Run (1m)

*   **Visual**: Terminal window
*   **Narration**: "Let's build and run the program. Run:"
*   **Code**: `lang build`
*   **Narration**: "Then run the executable:"
*   **Code**: `./hello_ferra`
*   **Narration**: "You should see 'Hello, Ferra!' printed to the console."

## 6. Conclusion (30s)

*   **Visual**: Summary screen
*   **Narration**: "Congratulations! You've written and run your first Ferra program. Check out the Week-1 tutorial to build a REST API, and join our community forums for help and discussion."

## Subtitles

*   **Format**: `.vtt` file with accurate timestamps
*   **Template**: [hello_ferra.vtt](./hello_ferra.vtt)

## Accessibility Notes

*   **High Contrast**: Use high contrast for code and UI elements
*   **Screen Reader**: Ensure all visual elements are described in the narration
*   **Transcript**: Full text transcript available in [hello_ferra.md](./hello_ferra.md)

## Accessibility Notes

- All code blocks are shown in high contrast
- Terminal text is enlarged for visibility
- Narration is clear and well-paced
- Subtitles are provided in .vtt format
- Screen reader friendly with descriptive narration 