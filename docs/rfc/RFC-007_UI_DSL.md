---
number: RFC-007
title: "UI-DSL Design"
status: Draft
version: v0.3
authors: ["Amrit Doll <amritdoll@example.com>"]
last_reviewed: 2025-05-21
last_updated: 2025-05-21
---

# RFC-007: UI-DSL Design

## Table of Contents
1. [Summary](#1-summary)
2. [Motivation](#2-motivation)
3. [Impact](#3-impact)
   1. [Developer Experience](#31-developer-experience)
   2. [Ecosystem](#32-ecosystem)
   3. [Performance](#33-performance)
4. [Design Decisions](#4-design-decisions)
5. [Drawbacks](#5-drawbacks)
6. [Security & Privacy](#6-security--privacy)
7. [Implementation Plan](#7-implementation-plan)
8. [Migration Strategy](#8-migration-strategy)
9. [Unresolved Questions](#9-unresolved-questions)
10. [Future Possibilities](#10-future-possibilities)
11. [References](#11-references)

## 1. Summary
This RFC specifies the design for Ferra's UI-DSL, a declarative, type-safe domain-specific language for building cross-platform mobile UIs (iOS/Android) in Ferra. The UI-DSL enables developers to define user interfaces using a single, expressive syntax that maps to native UI frameworks (SwiftUI/UIKit for iOS, Jetpack Compose for Android), aiming for native look, feel, and performance. The preview version focuses on a core set of UI elements, state management, event handling, and a function-based component model, with a clear path for future extensibility.

## 2. Motivation
Modern mobile development requires building UIs that are both native-feeling and maintainable across platforms. Existing cross-platform solutions often force trade-offs between code reuse, developer experience, and native fidelity. Ferra's UI-DSL aims to:
- Provide a single, consistent way to define UIs for iOS and Android.
- Maximize code sharing while preserving native performance and idioms.
- Leverage Ferra's type system for safer, more predictable UI code.
- Lower the barrier for developers to build high-quality mobile apps in Ferra.

## 3. Impact
### 3.1 Developer Experience
- Declarative, readable syntax inspired by SwiftUI/Compose.
- Strong static typing and compile-time error checking for UI code.
- Component-based architecture for reusability and maintainability.
- Integrated state management and event handling.
- Tooling support (e.g., `lang doctor ios`) for platform-specific best practices.

### 3.2 Ecosystem
- Enables Ferra to target mobile platforms with a single codebase.
- Facilitates integration with native libraries and FFI.
- Lays groundwork for future UI-DSL extensions (web, desktop, advanced theming).

### 3.3 Performance
- UI-DSL compiles to native UI code (SwiftUI/Compose) for optimal performance.
- Minimal runtime overhead; leverages platform-native rendering and controls.
- Target runtime size for Android: ≤ 5 MB for Ferra-related code.
- Expected codegen latency: < 100ms for typical UI modules.
- `lang doctor ios` lint times: < 50ms per file.

## 4. Design Decisions

### 4.1 DSL Philosophy & Syntax
- Declarative, function-based component model: UI elements are Ferra functions returning `ui::Component`.
- Chained modifiers: Styling and layout via method chaining (e.g., `.font_size(18.0).padding(10.0)`).
- No custom `ui` blocks or macros for v0.1; standard Ferra syntax is used for all UI declarations.
- Reusable components: Defined as Ferra functions with typed parameters, returning `ui::Component`.
- Children passed as explicit `children: [ ... ]` lists; future versions may support trailing closure/builder syntax.

**Example:**
```ferra
fn MainScreen() -> ui::Component {
    #[State] var counter = 0;
    return ui::VStack(spacing: 15.0) {
        children: [
            ui::Text("Current count: " + String::from_int(counter)),
            ui::Button(text_label: "Increment", on_tap: || {
                counter = counter + 1;
            }),
            ui::HStack(spacing: 10.0) {
                children: [
                    ui::Image(ImageSource::Asset("icon.png")).frame(width: 30.0, height: 30.0),
                    ui::Text("An icon and text")
                ]
            }.padding(EdgeInsets::all(5.0))
        ]
    }.padding(EdgeInsets::all(16.0));
}
```

**EBNF Sketch (simplified):**
```
component ::= function_call | modifier_chain
function_call ::= identifier "(" [args] ")" ["{" children "}"]
modifier_chain ::= component "." identifier "(" [args] ")"
children ::= "children:" "[" component_list "]"
component_list ::= component {"," component}
```

### 4.2 Supported UI Elements & Modifiers (Preview Set)
- **Layout**: `VStack`, `HStack`, `ZStack`, `Spacer`, `Grid`
- **Text & Images**: `Text`, `Image`
- **Controls**: `Button`, `TextField`, `SecureField`, `Toggle`, `Slider`, `ProgressView`
- **Lists/Scroll**: `ScrollView`, `List`
- **Navigation**: `NavigationView`, `Link`
- **Modifiers**: `.frame()`, `.padding()`, `.background()`, `.foreground_color()`, `.font_size()`, `.font_weight()`, `.opacity()`, `.corner_radius()`, `.border()`, `.text_alignment()`, etc.
- **Styling types**: `ColorValue`, `FontWeight`, `EdgeInsets`, `Alignment`, `ShapeType`
- **Theming**: Semantic colors (e.g., `Color::TextPrimary`), mapped to platform-native themes.

### 4.3 State Management
- Local state: `#[State] var x = ...` or `let (x, set_x) = ui::state(...)` triggers UI recomposition on change.
- Two-way binding: Input elements accept `&mut` references to state variables.
- Derived state: Computed via regular Ferra functions/properties.
- Shared/app-level state: Prop drilling for v0.1; future: observable objects, actor integration.
- Immutability encouraged for predictable updates.

### 4.4 Event Handling
- Event handlers: Passed as Ferra functions/closures (e.g., `on_tap: || { ... }`).
- Type-checked signatures, FFI-compatible for native bridging.
- Async handlers supported (`async fn`), integrated with Ferra's concurrency model.
- No direct DOM-like manipulation; state changes drive UI updates.
- Simple event propagation for v0.1; advanced bubbling/capturing is future work.

### 4.5 Mapping to Native Platforms
- **iOS (SwiftUI/UIKit)**:
  - Direct mapping of core elements/modifiers to SwiftUI.
  - State: Ferra `#[State]` maps to SwiftUI `@State`; two-way binding via FFI.
  - Event handlers: SwiftUI actions call Ferra functions via C ABI (see `FFI_C_CPP.md`).
  - UIKit fallback via `UIViewRepresentable` for unsupported elements.
  - Codegen: Ferra compiler emits Swift code, FFI headers, and build scripts for Xcode integration.
- **Android (Jetpack Compose)**:
  - Core elements/modifiers map to Compose `@Composable` functions and `Modifier` chains.
  - State: Ferra state bridges to Compose `mutableStateOf` via JNI/FFI.
  - Event handlers: Compose lambdas call Ferra functions via JNI.
  - Codegen: Ferra compiler emits Kotlin code, JNI glue, and Gradle integration scripts.
- **Runtime size target**: ≤ 5 MB for Ferra code on Android (enforced by CI/build tooling).

### 4.6 Static Analysis & Tooling
- `lang doctor ios`: Static analyzer for iOS best practices, layout/performance hints, HIG adherence, accessibility, and resource checks.
- CLI output: Human- and machine-readable diagnostics, integrated with Ferra's main diagnostic system.
- Diagnostic codes registered in `diagnostic_codes.md` (see [diagnostic_codes.md](../diagnostic_codes.md) for UI-DSL-specific codes).

### 4.7 FFI & Build Integration
- FFI bridge: All event handlers and state accessors exported with C ABI (see `FFI_C_CPP.md`).
- Data marshalling: Strings, structs, and callbacks follow FFI conventions for safety and performance.
- Build system: `lang build` orchestrates codegen, native compilation, and integration with Xcode/Gradle.
- CI checks: Enforce runtime size, codegen correctness, and diagnostics coverage.

> For full details, see [UI_DSL_MOBILE.md](../UI_DSL_MOBILE.md) and referenced backend/FFI specs.

### 4.2 State and Binding Grammar
```ebnf
StateDecl     ::= "state" IDENT ":" TypeExpr
BindingExpr   ::= "@" IDENT | "@" "(" Expr ")"
StateUpdate   ::= "set" IDENT "=" Expr
StateEffect   ::= "effect" IDENT "->" Block

// Example usage:
state count: Int
state user: User?
state items: List<Item>

// Binding examples
Text(@count)  // Direct binding
Text(@(count + 1))  // Computed binding
Button("Increment") {
    set count = count + 1
}
```

### 4.3 Platform Extensions
The UI-DSL supports platform-specific extensions through feature flags:

```ferra
// Enable iOS-specific features
#[cfg(target_os = "ios")]
fn ios_specific() {
    // iOS-only code
}

// Enable Android-specific features
#[cfg(target_os = "android")]
fn android_specific() {
    // Android-only code
}
```

## 5. Drawbacks
- **Complexity**: Adds a new abstraction layer and codegen path; requires robust FFI and build integration.
- **FFI Edge Cases**: State/event bridging can introduce subtle bugs if FFI contracts are violated.
- **Preview Limitations**: v0.1 supports only a core subset of UI elements, basic navigation, and simple state patterns.
- **Learning Curve**: Developers must learn new DSL idioms and platform mapping details.
- **Performance Tuning**: FFI overhead and state sync may require tuning for large/complex UIs.

## 6. Security & Privacy
- **FFI Safety**: All FFI calls (state/event) follow strict type and memory safety rules (see [FFI_C_CPP.md](../FFI_C_CPP.md)).
- **Data Handling**: UI state/data is not persisted or transmitted unless explicitly coded; privacy depends on app logic.
- **Platform Permissions**: UI-DSL does not grant extra permissions; platform-specific permissions are managed via manifest (see [SECURITY_MODEL.md](../SECURITY_MODEL.md)).
- **Error Handling**: FFI errors and panics are caught and surfaced as UI errors or logs.
- **Accessibility**: Basic checks in `lang doctor ios`; comprehensive support is future work.
- **Diagnostic Codes**: UI-DSL-specific diagnostic codes are registered in [diagnostic_codes.md](../diagnostic_codes.md).

## 7. Implementation Plan
- **Phase 1 (Q1 2026)**: Core UI-DSL syntax, codegen for SwiftUI/Compose, FFI bridge, minimal test suite. Finish SwiftUI codegen prototype by end of Q1 2026.
- **Phase 2 (Q2 2026)**: Static analysis (`lang doctor ios`), expanded UI elements, Android runtime size CI, docs/examples.
- **Phase 3 (Q3 2026)**: Advanced state patterns, navigation improvements, accessibility, performance tuning.
- **Test Harness**: End-to-end UI tests, FFI contract tests, static analyzer regression suite.
- **CI**: Enforce runtime size, codegen correctness, and diagnostics coverage.
- **Metrics**: UI render latency < 16ms, FFI roundtrip < 1ms, test coverage > 80%.

## 8. Migration Strategy
The migration to the UI-DSL will be gradual and opt-in:

```ferra
// Legacy UI code
#[feature(ui_dsl)]  // Opt-in to UI-DSL features
mod legacy_ui {
    // Existing imperative UI code
    fn render_button() -> View {
        let button = Button::new();
        button.set_text("Click me");
        button.on_click(|| println!("clicked"));
        return button;
    }
}

// New UI-DSL code
mod new_ui {
    use legacy_ui;  // Can still use legacy components
    
    fn MainScreen() -> View {
        VStack {
            Text("Welcome")
            Button("Click me") {
                println!("clicked")
            }
        }
    }
}
```

Migration phases:
1. **Phase 1: Opt-in DSL**
   - Enable `#[feature(ui_dsl)]` per module
   - Legacy UI code continues to work
   - New code can use DSL syntax

2. **Phase 2: Component Wrappers**
   - Wrap legacy components in DSL syntax
   - Maintain backward compatibility
   - Example:
   ```ferra
   // Legacy component wrapper
   fn LegacyButton(text: String, on_click: Fn()) -> View {
       let button = legacy_ui::render_button();
       button.set_text(text);
       button.on_click(on_click);
       return button;
   }
   
   // DSL usage
   Button("Click me") {
       LegacyButton("Legacy", || println!("legacy"))
   }
   ```

3. **Phase 3: Full Migration**
   - Convert all UI code to DSL
   - Remove legacy UI support
   - Enable strict DSL mode

## 9. Unresolved Questions
- **High Priority**:
  - Final UI-DSL syntax for children/builders (UI-DSL-1).
  - Navigation state and destination modeling.
- **Medium Priority**:
  - Advanced shared state/observable objects.
  - FFI optimizations for async/event-heavy UIs.
- **Low Priority**:
  - Accessibility and localization best practices.
  - Live preview/devtools integration.

## 10. Future Possibilities
- Web/desktop UI-DSL targets (e.g., WASM, Electron, native desktop toolkits).
- Richer theming/animation APIs.
- Visual design tools and live preview.
- Deep platform-specific UI extensions via FFI.
- Advanced accessibility and localization.
- Integration with AI-driven UI generation/refactoring.
- See [UI_DSL_ROADMAP.md](../UI_DSL_ROADMAP.md) for detailed future plans.

## 11. References
1. [UI_DSL_MOBILE.md](../UI_DSL_MOBILE.md)
2. [FFI_C_CPP.md](../FFI_C_CPP.md)
3. [SECURITY_MODEL.md](../SECURITY_MODEL.md)
4. [SYNTAX_GRAMMAR_V0.1.md](../SYNTAX_GRAMMAR_V0.1.md)
5. [AST_SPECIFICATION.md](../AST_SPECIFICATION.md)
6. [CONCURRENCY_MODEL.md](../CONCURRENCY_MODEL.md)
7. [BACKEND_EXPANDED_TARGETS.md](../BACKEND_EXPANDED_TARGETS.md)
8. [diagnostic_codes.md](../diagnostic_codes.md)
9. [UI_DSL_ROADMAP.md](../UI_DSL_ROADMAP.md)
10. [Steps.md](../Steps.md) 