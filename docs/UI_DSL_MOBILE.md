# Ferra UI-DSL for Mobile (iOS/Android) Design - Preview v0.1

> **Status:** Initial Draft - Module 3.1 (Steps 3.1.1 - 3.1.4)

## 1. Introduction and Goals

This document outlines the design for a User Interface Domain Specific Language (UI-DSL) embedded within Ferra, specifically targeting the creation of mobile applications for iOS and Android. The "preview" nature of this v0.1 specification emphasizes establishing a foundational common DSL and demonstrating its viability by mapping to native UI frameworks on both platforms.

*   **Purpose**:
    *   To enable Ferra developers to define user interfaces for mobile applications using a single, consistent syntax and component model within the Ferra language.
    *   To abstract away common UI patterns and components, allowing for a significant portion of UI code to be shared across iOS and Android.
    *   To compile or translate these Ferra UI descriptions into native UI code for each target platform (SwiftUI/UIKit for iOS, Jetpack Compose for Android), aiming for native look, feel, and performance.

*   **Goals for the Preview Version (v0.1 of this UI-DSL spec)**:
    1.  **Design a Common UI-DSL (Step 3.1.1)**: Create a minimal yet expressive declarative syntax within Ferra for defining UI structures, components, basic styling, state management, and event handling.
    2.  **Focus on Core Elements**: Identify and specify a core set of common UI elements (text, buttons, images, lists, layout containers, basic inputs) and layout principles (stacks, spacers, alignment) that are readily available and conceptually similar on both iOS and Android.
    3.  **Specify iOS Mapping (Step 3.1.2)**: Detail the strategy for translating the Ferra UI-DSL into functional SwiftUI views and modifiers. Outline how UIKit might be used as a fallback or for specific needs.
    4.  **Specify Android Mapping & Runtime Target (Step 3.1.4)**: Detail the strategy for translating the Ferra UI-DSL into Jetpack Compose composables and modifiers. Crucially, address strategies to meet the **target runtime size of ≤ 5 MB** for the Ferra-related portion of an Android application.
    5.  **Design `lang doctor ios` (Step 3.1.3)**: Outline the capabilities of a static analysis tool (`lang doctor ios`) to assist developers by checking Ferra UI code for iOS-specific best practices, potential performance issues, or common pitfalls before native compilation.
    6.  **Enable a "Preview" Functionality**: The overall goal is to achieve a "preview" level of functionality that demonstrates the core concepts and viability of using Ferra for cross-platform mobile UI development, rather than a production-ready, feature-complete UI framework at this stage.

*   **Non-Goals for Preview Version (v0.1 of this UI-DSL spec)**:
    *   **Full Feature Parity**: This preview will not attempt to replicate every feature or component available in SwiftUI or Jetpack Compose.
    *   **Advanced UI Capabilities**: Complex custom drawing, intricate animations, advanced gesture recognition systems, and augmented reality (AR) features are out of scope for this initial preview.
    *   **Deep Platform-Specific UI**: Highly platform-specific or niche UI elements that do not have a clear, common abstraction across both iOS and Android will generally be excluded from the common DSL in this version.
    *   **Visual UI Builder Tools**: No visual design tools or IDE plugins for visual UI construction are planned for this phase. The focus is on the code-based DSL.
    *   **Pixel-Perfect Cross-Platform Duplication**: The goal is native look and feel on each platform, not necessarily making the UI look identical pixel-for-pixel across iOS and Android.

## 2. Common UI-DSL Design (Step 3.1.1)

This section details the design of the common UI-DSL in Ferra. The aim is to create a language layer that allows developers to define UIs in a platform-agnostic way for a core set of functionalities, which can then be mapped to native iOS and Android UI frameworks.

### 2.0 Conceptual DSL Syntax (EBNF Teaser)

While Section 2.6 discusses the DSL syntax in detail (favoring a function-based component model with chained modifiers for v0.1), the following high-level conceptual EBNF illustrates the anticipated structure. The actual syntax will rely on standard Ferra function calls and method chaining, potentially augmented by attributes. (See **SYNTAX_GRAMMAR_V0.1.md** for Ferra's core grammar, particularly function calls and attribute syntax, and **AST_SPECIFICATION.md** for corresponding AST node definitions, e.g., `CallExpr`, `MethodCallExpr`, `Attribute`.)

```ebnf
// Conceptual EBNF - illustrative of structure, actual syntax is standard Ferra.
// A UI component declaration is a Ferra function returning a special UI type.
UIDecl          ::= AttributeListOpt "fn" IDENTIFIER "(" ParamListOpt ")" "->" "ui::Component" BlockExpr ;

// Using a UI element (built-in or custom) involves calling its function and then chaining modifier methods.
UIElement       ::= PathToUIFunction "(" ArgListOpt ")" MethodCallChainOpt ;
MethodCallChainOpt ::= ( "." IDENTIFIER "(" ArgListOpt ")" )* ; // Standard Ferra method calls

// Containers might take children via a named argument or a builder block if Ferra syntax evolves.
// Block           ::= "{" (UIElement | UIDecl)* "}" ; // Conceptual for a builder, actual is Ferra block/expr

// PathToUIFunction: e.g., `ui::Text`, `my_module::CustomButton`
// ParamListOpt, ArgListOpt: Standard Ferra parameter and argument lists.
```
This EBNF is a sketch. The actual parsing of UI definitions will be handled by Ferra's standard parser, interpreting function calls to `ui::*` modules (or user-defined component functions) and their chained methods.

### 2.1. Design Philosophy

The Ferra UI-DSL will be guided by several core philosophies to ensure it is modern, developer-friendly, and effective for cross-platform mobile development:

*   **Declarative Syntax**:
    *   UI structures will be defined by declaring the desired state and appearance of components, rather than through imperative, step-by-step manipulation of UI objects.
    *   This approach, common in frameworks like SwiftUI, Jetpack Compose, Flutter, and React, leads to more readable, predictable, and maintainable UI code.
    *   The UI should automatically update (re-render or recompose) when underlying state data changes.

*   **Component-Based Architecture**:
    *   UIs will be constructed by composing smaller, independent, and reusable components.
    *   A component can be a basic UI element (like a `Text` or `Button`) or a custom component built by combining other components.
    *   Ferra functions will likely serve as the primary mechanism for defining these reusable UI components, encapsulating their structure and local logic.

*   **Strongly Typed**:
    *   The UI-DSL will be deeply integrated with Ferra's static type system.
    *   This means component parameters (properties), state variables, and event handler signatures will be type-checked at compile time, catching many common UI programming errors early.
    *   For example, passing a `String` to a property expecting an `Int` would be a compile-time error.

*   **Common Subset Focus (for v0.1 Preview)**:
    *   The initial version of the DSL will focus on a carefully selected subset of UI elements, layout patterns, styling attributes, and interaction models that have clear, common, and readily available equivalents on both iOS (SwiftUI/Compose) and Android (Jetpack Compose).
    *   This pragmatic approach aims to maximize code reusability for common UI tasks in the preview version.

*   **Readability and Python-Ease**:
    *   The syntax for defining UI components and their modifiers should be intuitive, concise, and aim for a developer experience that approaches the "Python-ease" goal of Ferra.
    *   This doesn't mean replicating Python syntax, but rather striving for a low cognitive overhead and a gentle learning curve for common UI tasks, while still leveraging Ferra's safety and performance.

*   **Extensibility (Future Consideration)**:
    *   While the v0.1 preview focuses on a common subset, the DSL's design should not preclude future extensibility to accommodate platform-specific UI elements or functionalities.
    *   Potential mechanisms for this could include:
        *   Platform-specific attributes or modifiers.
        *   Conditional compilation blocks (`#[cfg(target_os = "ios")] { ... }`).
        *   A well-defined FFI mechanism for bridging to custom native UI components written in Swift or Kotlin.

*   **Unidirectional Data Flow (Preferred Pattern)**:
    *   To manage UI complexity and make state changes more predictable, the DSL will encourage patterns aligning with unidirectional data flow.
    *   This typically means:
        *   **State flows down**: Parent components pass data (state) to child components as immutable properties or through controlled bindings.
        *   **Events flow up**: Child components communicate changes or user interactions back to parent components (or state handlers) via callbacks or event emissions, rather than directly modifying parent or global state.
    *   This approach, seen in frameworks like React/Redux, Elm, and often in Compose with ViewModels, helps in reasoning about state transitions and debugging.

These philosophies will guide the specific design choices for syntax, core elements, state management, and other aspects of the Ferra UI-DSL.

### 2.2. Core UI Elements (Initial Set for Preview)

The preview version of the Ferra UI-DSL will support a minimal but essential set of UI elements. These are chosen for their fundamental role in building UIs and their relatively straightforward conceptual mapping to both SwiftUI and Jetpack Compose. The conceptual Ferra syntax presented here assumes a function-based component model (details in Section 2.6).

_Note: The invocation of these UI elements (e.g., `ui::Text(...)`) and their modifiers (e.g., `.font_size(...)`) will follow standard Ferra function and method call syntax. Their representation in the Abstract Syntax Tree (AST) will correspond to Ferra's general AST nodes for function calls, method calls, and their arguments. Refer to `SYNTAX_GRAMMAR_V0.1.md` for Ferra's core grammar and `AST_SPECIFICATION.md` for AST node definitions._

*   **Layout Containers**:
    *   **`VStack(alignment: HorizontalAlignment = .center, spacing: Float = 0.0) { ...children }`**: Arranges child components vertically.
        *   `alignment`: How children are aligned horizontally within the VStack (e.g., `.leading`, `.center`, `.trailing`).
        *   `spacing`: Vertical space between child elements.
        *   *Maps to SwiftUI `VStack`, Compose `Column`.*
    *   **`HStack(alignment: VerticalAlignment = .center, spacing: Float = 0.0) { ...children }`**: Arranges child components horizontally.
        *   `alignment`: How children are aligned vertically within the HStack (e.g., `.top`, `.center`, `.bottom`).
        *   `spacing`: Horizontal space between child elements.
        *   *Maps to SwiftUI `HStack`, Compose `Row`.*
    *   **`ZStack(alignment: Alignment = .center) { ...children }`**: Overlays child components, aligning them along both X and Y axes within the ZStack's bounds.
        *   `alignment`: How children are aligned (e.g., `.topLeading`, `.center`, `.bottomTrailing`).
        *   *Maps to SwiftUI `ZStack`, Compose `Box`.*
    *   **`Spacer(min_length: Float = 0.0)`**: A flexible space that expands along the major axis of its parent stack.
        *   `min_length`: Optional minimum size for the spacer.
        *   *Maps to SwiftUI `Spacer`, Compose `Spacer` or weighted elements.*
    *   **`Grid(columns: Int, spacing: Float = 0.0) { ...children }`** (Simplified for v0.1):
        *   A basic grid that arranges children into a specified number of columns.
        *   More advanced grid layouts (e.g., adaptive columns, row/column spans) are future work.
        *   *Maps to SwiftUI `LazyVGrid`/`LazyHGrid` with fixed columns, Compose `LazyVerticalGrid` or custom layout.*

*   **Text Display**:
    *   **`Text(content: String)`**: Displays a piece of static or dynamic text.
        *   **Modifiers**: `.font(name: String?, size: Float, weight: FontWeight)`, `.color(ColorValue)`, `.alignment(TextAlignment)`, `.line_limit(Int?)`, `.truncate_mode(TruncateMode)`.
        *   `FontWeight`: e.g., `.light`, `.regular`, `.medium`, `.bold`.
        *   `TextAlignment`: e.g., `.leading`, `.center`, `.trailing`.
        *   `TruncateMode`: e.g., `.head`, `.middle`, `.tail`.

*   **Images**:
    *   **`Image(source: ImageSource)`**: Displays an image.
        *   `ImageSource`: An enum or similar type, e.g., `ImageSource::Asset("image_name.png")`, `ImageSource::Url("http://...")`.
        *   **Modifiers**: `.resizable(Bool)`, `.aspect_ratio(RatioMode, content_mode: ContentMode)`, `.content_mode(ContentMode)`, `.frame(width: Float?, height: Float?)`, `.clip_shape(ShapeType)`.
        *   `RatioMode`: e.g., `.fit`, `.fill`.
        *   `ContentMode`: e.g., `.scale_to_fit`, `.scale_to_fill`.
        *   `ShapeType`: e.g., `.circle`, `.rounded_rect(cornerRadius: Float)`.

*   **Buttons**:
    *   **`Button(on_tap: fn() -> Unit) { ...label_content }`**: An interactive button.
        *   `on_tap`: A Ferra function or closure executed when the button is tapped.
        *   The `label_content` block defines the button's appearance (e.g., a `Text` or `Image` component).
    *   **`Button(text_label: String, on_tap: fn() -> Unit)`**: A convenience for simple text buttons.

*   **Input Fields**:
    *   **`TextField(placeholder: String, text_binding: &mut String, on_commit: fn() -> Unit = ||{})`**: A single-line plain text input field.
        *   `placeholder`: Hint text displayed when the field is empty.
        *   `text_binding`: A mutable reference to a Ferra `String` for two-way data binding.
        *   `on_commit`: Optional action when user finalizes input (e.g., presses return).
    *   **`SecureField(placeholder: String, text_binding: &mut String, on_commit: fn() -> Unit = ||{})`**: A single-line secure text input for passwords.
        *   Parameters similar to `TextField`.

*   **Switches/Toggles**:
    *   **`Toggle(label: String?, is_on_binding: &mut Bool)`**: A boolean switch.
        *   `label`: Optional text label for the toggle.
        *   `is_on_binding`: A mutable reference to a Ferra `Bool` for two-way data binding.

*   **Sliders**:
    *   **`Slider(value_binding: &mut Float, range: (Float, Float), step: Float = 1.0)`**: A slider for selecting a continuous or discrete value within a range.
        *   `value_binding`: Mutable reference to a Ferra `Float` for two-way data binding.
        *   `range`: A tuple `(min_value, max_value)`.
        *   `step`: Optional step increment for discrete sliders.

*   **Progress Indicators**:
    *   **`ProgressView(value: Option<Float>)`**: Displays a progress bar.
        *   `value`: An `Option<Float>`. If `Some(float_val)` (0.0 to 1.0), it's a determinate progress bar. If `None`, it's an indeterminate (spinning) progress indicator.

*   **Lists/Scrollable Views**:
    *   **`ScrollView(axes: ScrollAxes = .vertical) { ...content }`**: A container that allows its content to be scrolled if it exceeds the available space.
        *   `axes`: Enum specifying scroll direction(s) (e.g., `.vertical`, `.horizontal`, `.both`).
    *   **`List<ItemData>(items: Vector<ItemData>, row_builder: fn(index: Int, item: ItemData) -> Component)`**: Displays a vertically scrolling list of items. `ItemData` is a generic type parameter.
        *   `items`: A Ferra `Vector` containing the data for each row.
        *   `row_builder`: A Ferra function that takes the index and an `item` of `ItemData` and returns the UI `Component` to render for that row.
        *   This is a simplified list for v0.1; it implies efficient rendering of potentially large lists (like SwiftUI `List` or Compose `LazyColumn`). More advanced features like sections, item IDs for diffing, and pull-to-refresh are future considerations.

*   **Navigation Elements (Basic Preview)**:
    *   **`NavigationView { ...root_view }`**: A container that provides a context for navigation, typically displaying a navigation bar. (This is a high-level concept for v0.1; detailed behavior like stack management is TBD UI-DSL-1).
    *   **`Link<DestinationView: View>(label: Component) { ...destination_view_constructor }`** (Highly Conceptual):
        *   Represents a UI element (e.g., a button or styled text) that, when tapped, navigates to a `DestinationView`.
        *   The exact mechanism for defining destinations and triggering navigation is a significant TBD (UI-DSL-1) and will depend heavily on how state and view transitions are managed. For v0.1, this might be very limited.

This initial set provides a foundation for building simple UIs. Each element will also support a common set of basic modifiers for styling and layout (detailed in Section 2.5).

### 2.3. State Management

A declarative UI DSL requires a robust and intuitive way to manage state, as UI is a function of state (`UI = f(state)`). Changes in state should automatically trigger UI updates.

*   **Local Component State**:
    *   **Purpose**: For state that is owned and managed exclusively by a single UI component and does not need to be shared widely. Examples include whether a dropdown is expanded, the current text in a search field before submission, or animation states.
    *   **Declaration**: Ferra will provide a mechanism to declare state variables within a component-defining function such_that changes to these variables trigger a re-evaluation/re-composition of that component's UI.
        *   **Conceptual Syntax (TBD UI-DSL-2)**:
            *   Attribute-based: `#[State] var user_input: String = "";`
            *   Hook-like function: `let (user_input, set_user_input) = ui::state("");` (Inspired by React Hooks).
        *   The chosen syntax must integrate cleanly with Ferra's variable declaration (`var` for mutable state). The `#[State]` attribute, if adopted, would be parsed as a standard Ferra attribute (see **SYNTAX_GRAMMAR_V0.1.md** Section 1.3.5) and represented in the AST (see **AST_SPECIFICATION.md** Section 5 ItemKind::Attribute). Its semantic meaning—identifying a variable whose changes trigger UI recomposition—would be handled by the UI-DSL compilation logic. This might involve specific IR semantic tags (e.g., conceptually, an `#[ai::tag(ui_state_variable)]` or similar, detailed in **IR_SEMANTIC_TAGS.md**) if the runtime or later compiler passes need to identify these state variables for special handling (e.g., observability, optimized updates).
    *   **Reactivity**: When a `#[State]` variable (or a variable managed by `ui::state`) is modified, the Ferra UI runtime will ensure that the component function is re-executed, and the UI is updated to reflect the new state. Only the parts of the UI that depend on the changed state should ideally be re-rendered for efficiency.

*   **Data Binding**:
    *   **Two-Way Binding**: For input elements like `TextField`, `Toggle`, `Slider`, the DSL will support two-way data binding. This means the UI element both displays the state variable and updates the state variable upon user interaction.
    *   **Mechanism**: This will likely be achieved by passing mutable references (`&mut FerraType`) to `#[State]` variables into the UI element constructors. The use of mutable references aligns with Ferra's ownership and borrowing principles (see `OWNERSHIP_MODEL_PRINCIPLES_V0.1.md` and `OWNERSHIP_BORROW_CHECKER.md`), and the UI framework would need to ensure these accesses are safe and correctly managed in the context of UI updates.
        *   Example: `ui::TextField(placeholder: "Name", text_binding: &mut user_name_state)`
    *   **Underlying Implementation**: The UI element, when its value changes due to user input, will call a setter or modify the value through the mutable reference, which in turn triggers the state change and UI update mechanism.

*   **Derived State**:
    *   Often, some UI aspects are derived from other state variables. Ferra's regular computed properties or functions can be used for this. If these computations depend on `#[State]` variables, they will be re-evaluated when the underlying state changes, leading to UI updates.
    *   Example: `let is_form_valid = username.len() > 3 && password.len() > 5; // Recomputed if username/password are state`

*   **Shared and Application-Level State (Conceptual for v0.1 Preview)**:
    *   Managing state that needs to be shared across multiple components, sibling components, or globally within the application is more complex.
    *   **Prop Drilling (Passing State Down)**: For simple cases, state can be owned by a common ancestor component and passed down to child components as parameters (props). Callbacks can be used to send events/changes back up. This is a fundamental pattern in declarative UI.
    *   **Observable Objects / State Holders**: For more complex shared state, a dedicated state holder object might be used. This object would manage a piece of shared state, and components could observe it for changes.
        *   Ferra `data` classes, possibly with mechanisms to notify observers of changes, could serve this role.
        *   The exact mechanism for "observability" in Ferra (e.g., compiler-generated notifications, or manual implementation via a pattern) is TBD UI-DSL-2.
    *   **Leveraging Ferra's Actor Model**: For complex application logic and state that needs to be managed asynchronously or has intricate business rules, Ferra's actor model (`CONCURRENCY_MODEL.md`) could be a powerful backend for UI state.
        *   UI components could send messages to actors to perform actions or request data.
        *   Actors could update shared state stores, which then notify the UI to re-render.
        *   This keeps complex logic separate from UI descriptions and leverages Ferra's concurrency strengths. For the v0.1 preview, direct integration might be simplified to basic patterns.
    *   **Focus for v0.1 Preview**: The preview will primarily focus on robust local component state and simple prop drilling. Advanced shared state management patterns will be prototyped but may not have dedicated DSL sugar yet.

*   **Immutability and State Updates**:
    *   To make state changes predictable, especially with shared state, encouraging the use of immutable data structures for state is often beneficial. Updates would produce new state instances rather than mutating existing ones in place.
    *   Ferra's `let` vs. `var` and its `data` class semantics will influence this. If `#[State]` variables are mutable, updates are direct. If they are immutable data structures, updates involve creating new instances.

The state management approach will aim for a balance between simplicity for common cases and scalability for more complex applications, drawing inspiration from established patterns in modern UI frameworks while fitting naturally within Ferra's existing semantics.

### 2.4. Event Handling

User interactions with UI elements (e.g., tapping a button, typing in a field) generate events that the application logic, written in Ferra, needs to respond to. The UI-DSL must provide a clear and type-safe way to define these interactions.

*   **Callback Functions as Event Handlers**:
    *   The primary mechanism for event handling will be through callback functions (or closures) passed as parameters to UI components.
    *   UI elements that are interactive will expose parameters for common events they can generate, such as `on_tap`, `on_change`, `on_submit`, etc.
    *   **Example**:
        ```ferra
        // Button with an on_tap event handler
        ui::Button(text_label: "Save", on_tap: || {
            println("Save button tapped!");
            // Call a Ferra function to perform save logic
            // save_data_action(); 
        })

        // TextField with an on_change event handler
        #[State] var current_text = "";
        ui::TextField(
            placeholder: "Enter message", 
            text_binding: &mut current_text,
            on_change: fn(new_value: String) {
                println("Text changed to: " + new_value);
                // Note: new_value is already in current_text due to two-way binding.
                // This callback could be used for additional validation or side effects.
            },
            on_commit: || {
                println("Text field committed with: " + current_text);
                // send_message(current_text);
            }
        )
        ```
    *   _Callbacks, being Ferra functions or closures, are represented as standard function/closure nodes in the AST (`AST_SPECIFICATION.md`). Their invocation from the native side via FFI (see `FFI_C_CPP.md`) is a key part of the UI-DSL to native bridge._

*   **Event Handler Signatures**:
    *   The Ferra functions or closures passed as event handlers will have well-defined, FFI-compatible (or translatable) signatures.
    *   Some event handlers might take no arguments (e.g., `on_tap: fn() -> Unit`).
    *   Others might receive event-specific data (e.g., `on_change: fn(new_value: String) -> Unit` for a text field, or `on_slide: fn(new_value: Float) -> Unit` for a slider).
    *   These signatures will be type-checked by the Ferra compiler.

*   **Asynchronous Event Handlers**:
    *   Event handlers can be standard Ferra functions or `async fn`s.
    *   Using `async fn` allows event handlers to perform non-blocking operations (like network requests, complex computations, or interactions with Ferra actors) without freezing the UI.
    *   **Example**:
        ```ferra
        #[State] var server_response = "";

        async fn handle_fetch_data() {
            // server_response = "Loading..."; // Update UI to show loading state
            // let result = await fetch_data_from_network_async("my_api_endpoint");
            // server_response = result; // Update UI with fetched data
            // For the above to work, server_response must be a state variable that triggers UI recomposition.
        }

        ui::Button(text_label: "Fetch Data", on_tap: handle_fetch_data) 
        // Or as a closure:
        // ui::Button(text_label: "Fetch Data", on_tap: async || {
        //     server_response = "Loading...";
        //     let result = await fetch_data_from_network_async("my_api_endpoint");
        //     server_response = result;
        // })
        ```
    *   The Ferra UI runtime, in conjunction with its concurrency model (see `CONCURRENCY_MODEL.md`), will manage the execution of these `async` event handlers. UI updates resulting from completed asynchronous operations would typically flow through state variable changes.

*   **No Direct DOM-like Manipulation**:
    *   Following the declarative philosophy, event handlers should primarily focus on updating state variables or calling business logic functions.
    *   They should not attempt to directly manipulate UI elements imperatively (e.g., directly changing a text label's content from an event handler without going through state). The UI should re-render based on state changes.

*   **Event Propagation (Simplified for v0.1)**:
    *   For v0.1, event propagation (e.g., bubbling, capturing) will be kept simple. Events are typically handled by the component that defines the handler.
    *   More complex event propagation or global event listeners are future considerations.

This event handling model aims to be idiomatic Ferra, type-safe, and capable of integrating with Ferra's asynchronous capabilities for responsive UIs.

### 2.5. Styling and Theming (Basic Modifiers)

While the primary focus of the v0.1 UI-DSL is on structure and core components, basic styling capabilities are essential for creating usable UIs. A modifier-based approach, common in declarative UI frameworks, will be adopted.

*   **Modifier-Based Styling**:
    *   Styling properties will be applied to UI elements using chained method calls (modifiers) that return a new, modified version of the component (or describe the modification).
    *   **Example Syntax**:
        ```ferra
        ui::Text("Styled Text")
            .font_size(18.0)
            .font_weight(FontWeight::Bold)
            .foreground_color(Color::Named("blue"))
            .padding(EdgeInsets::all(10.0))
            .background(Color::Hex("#EEEEEE"))
            .frame(min_width: 100.0, max_height: 50.0, alignment: .center)
        ```

*   **Core Styling Properties & Modifiers (Initial Set for v0.1)**:
    *   **Sizing and Framing**:
        *   `.frame(width: Option<Float>, height: Option<Float>, min_width: Option<Float>, min_height: Option<Float>, max_width: Option<Float>, max_height: Option<Float>, alignment: Option<Alignment>)`: Specifies fixed, minimum, or maximum dimensions and alignment within the offered space.
    *   **Padding**:
        *   `.padding(EdgeInsets)`: Applies padding around an element.
        *   `.padding(uniform: Float)`: Convenience for equal padding on all sides.
        *   `.padding(horizontal: Float, vertical: Float)`
        *   `.padding(top: Float, leading: Float, bottom: Float, trailing: Float)`
        *   `EdgeInsets` type: e.g., `EdgeInsets { top: 0.0, leading: 0.0, bottom: 0.0, trailing: 0.0 }` or helper constructors like `EdgeInsets::symmetric(horizontal: 10, vertical: 5)`.
    *   **Background**:
        *   `.background(ColorValue)`: Sets the background color.
        *   `.background(shape: ShapeType, color: ColorValue)`: Sets a shaped background.
    *   **Foreground/Text Color**:
        *   `.foreground_color(ColorValue)`: Sets the color for text or an image tint.
    *   **Font Properties (for `Text` elements)**:
        *   `.font_size(Float)`
        *   `.font_weight(FontWeight)` (Enum: e.g., `Thin`, `Light`, `Regular`, `Medium`, `Semibold`, `Bold`, `Heavy`, `Black`)
        *   `.font_style(FontStyle)` (Enum: e.g., `italic`)
        *   `.font_family(String)` (Specifies a font family name; font availability is platform-dependent).
    *   **Text Alignment (for `Text` within its bounds)**:
        *   `.text_alignment(TextAlignment)` (Enum: e.g., `leading`, `center`, `trailing`, `justify`).
    *   **Opacity**:
        *   `.opacity(Float)`: Sets transparency (0.0 for fully transparent, 1.0 for fully opaque).
    *   **Corner Radius (for elements with a background/border)**:
        *   `.corner_radius(Float)`
    *   **Border**:
        *   `.border(color: ColorValue, width: Float = 1.0, shape: ShapeType = .rectangle)`
    *   **Alignment (within parent layout containers)**:
        *   Layout containers like `VStack`, `HStack`, `ZStack` will have alignment parameters. Individual elements might also have modifiers like `.align_self(Alignment)` if needed to override parent alignment for that specific child, though this is less common in SwiftUI/Compose and might be deferred.

*   **Supporting Types for Styling**:
    *   `ColorValue`: An enum or struct to represent colors.
        *   `Color::Rgb(r: u8, g: u8, b: u8)`
        *   `Color::Rgba(r: u8, g: u8, b: u8, a: u8)`
        *   `Color::Hex(string_code: String)` (e.g., `"#RRGGBBAA"` or `"#RGB"`)
        *   `Color::Named(PredefinedColor)` (e.g., `PredefinedColor::Red`, `Blue`, `TextPrimary`, `BackgroundSurface` for theming).
    *   `FontWeight`, `FontStyle`, `TextAlignment`: Enums as described above.
    *   `EdgeInsets`: Struct for padding.
    *   `Alignment`: Enum for layout alignment (e.g., `.top`, `.center`, `.bottom`, `.leading`, `.trailing`, `.top_leading`, etc.).
    *   `ShapeType`: Enum for basic shapes (e.g., `.rectangle`, `.circle`, `.rounded_rect(cornerRadius: Float)`).

*   **Conceptual Approach to Theming (v0.1 Preview)**:
    *   **Semantic Colors**: Introduce a small set of semantic `Color::Named` values (e.g., `Color::TextPrimary`, `Color::TextSecondary`, `Color::BackgroundPrimary`, `Color::BackgroundSecondary`, `Color::Accent`).
    *   **Platform Adaptation**: The mapping layer (Section 3 & 4) would translate these semantic Ferra colors to the appropriate platform-specific dynamic colors that adapt to light/dark mode and system accent colors.
        *   iOS: Maps to SwiftUI `Color.primary`, `Color.secondary`, `Color(UIColor.systemBackground)`, etc.
        *   Android: Maps to Material Theme colors (`MaterialTheme.colors.onBackground`, `MaterialTheme.colors.surface`, etc.).
    *   **No Custom Theming Engine**: For v0.1, Ferra will not have its own custom theming engine. It will rely on the target platform's built-in capabilities for basic light/dark mode adaptation through these semantic color definitions.

This basic set of styling modifiers and a simple approach to theming should provide enough flexibility for the preview version while keeping the DSL and implementation manageable.

### 2.6. DSL Syntax in Ferra

The exact way UI elements are declared and composed in Ferra code is fundamental to the DSL's ergonomics and integration with the language. The goal is a syntax that is clear, declarative, and leverages Ferra's strengths.

*   **Primary Syntactic Approach: Function-Based Components with Chained Modifiers**:
    *   For v0.1, the primary approach will be to represent UI elements and custom components as Ferra functions.
    *   These functions would conceptually return a special opaque `ui::Component` type (or a similar internal representation of a UI node/tree).
    *   Modifiers (for styling, layout, event handling) will be implemented as methods on this `ui::Component` type, allowing for a chained, fluent API.
    *   **Example**:
        ```ferra
        // Assuming a hypothetical `ui` module
        // `ui::Text`, `ui::VStack`, etc., are functions returning `ui::Component` instances.

        fn GreetingView(name: String) -> ui::Component {
            return ui::Text("Hello, " + name + "!")
                .font_size(20.0)
                .foreground_color(Color::Named(PredefinedColor::TextPrimary));
        }

        fn MainScreen() -> ui::Component {
            #[State] var counter = 0;

            return ui::VStack(spacing: 15.0) {
                children: [ // Child components are passed as a list/vector
                    GreetingView("Ferra Developer"),
                    ui::Text("Current count: " + String::from_int(counter)),
                    ui::Button(text_label: "Increment", on_tap: || {
                        counter = counter + 1;
                    }),
                    ui::HStack(spacing: 10.0) {
                        children: [
                            ui::Image(ImageSource::Asset("icon.png")).frame(width:30.0, height:30.0),
                            ui::Text("An icon and text")
                        ]
                    }.padding(EdgeInsets::all(5.0))
                ]
            }.padding(EdgeInsets::all(16.0));
        }
        ```
    *   **Benefits**:
        *   Leverages Ferra's existing function and method call syntax.
        *   Integrates well with Ferra's type system (function signatures, modifier return types).
        *   Familiar pattern from other declarative UI frameworks (SwiftUI, Compose modifiers).

*   **Container Components and Children**:
    *   Layout containers like `VStack`, `HStack`, `ZStack`, `ScrollView`, and `List` will accept their child components, typically as a list or vector passed to a `children` parameter or through a builder-style trailing closure if Ferra syntax supports it elegantly.
    *   **Trailing Closure for Children (Conceptual Future Syntax)**:
        If Ferra evolves to have convenient trailing closure syntax for builders (similar to Swift or Kotlin), it could look like:
        ```ferra
        // Highly conceptual if Ferra supports this style for builders
        // ui::VStack(spacing: 10.0) {
        //     ui::Text("First item")
        //     ui::Text("Second item")
        // }
        ```
        For v0.1, passing children as an explicit named parameter (e.g., `children: [ ... ]`) is more straightforward given current presumed Ferra syntax.

*   **No Special `ui` Blocks or Custom Parsing for v0.1**:
    *   While a dedicated `ui { ... }` block with custom parsing rules (like JSX or QML) could offer more syntactic sugar, this adds significant complexity to the Ferra parser and compiler for v0.1.
    *   Similarly, a heavy macro-based DSL (like some Rust UI libraries) also adds its own layer of complexity and tooling challenges.
    *   The function-based approach is chosen for v0.1 for its simplicity of integration into the existing language.

*   **Defining Reusable Custom Components**:
    *   Developers will create reusable custom UI components simply by defining Ferra functions that take parameters (props) and return an `ui::Component` (the UI description for that component).
    *   **Example**:
        ```ferra
        // Reusable component defined as a Ferra function
        fn LabeledTextField(label: String, placeholder: String, text_binding: &mut String) -> ui::Component { // ui::Component is conceptual
            return ui::VStack(alignment: .leading, spacing: 4.0) {
                children: [
                    ui::Text(label).font_size(14.0),
                    ui::TextField(placeholder, text_binding: text_binding)
                ]
            };
        }

        // Using the custom component
        fn MyForm() -> ui::Component {
            #[State] var email = "";
            #[State] var phone = "";
            return ui::VStack(spacing: 10.0) {
                children: [
                    LabeledTextField("Email:", "Enter your email", &mut email),
                    LabeledTextField("Phone:", "Enter your phone", &mut phone)
                ]
            };
        }
        ```

This approach aims to provide a declarative and composable way to define UIs directly within Ferra, leveraging its existing syntax and type system as much as possible for the preview version. The TBD (UI-DSL-1) regarding final syntax will be resolved through prototyping based on these principles.

## 3. Mapping to iOS - SwiftUI/UIKit (Step 3.1.2)

This section details how the common Ferra UI-DSL will be translated into native UI code for the iOS platform. The primary target for this mapping is SwiftUI, due to its modern, declarative nature which aligns well with the Ferra UI-DSL philosophy. UIKit will serve as a fallback or for integration where SwiftUI lacks specific capabilities or when interacting with existing UIKit codebases.

### 3.1. Primary Target: SwiftUI

SwiftUI is Apple's modern declarative framework for building UIs across all Apple platforms. Its design principles are very similar to those proposed for the Ferra UI-DSL, making it a natural primary target.

*   **Direct Translation of Components and Modifiers**:
    *   Most core Ferra UI elements (Section 2.2) and their modifiers (Section 2.5) are expected to have direct or very close equivalents in SwiftUI.
    *   **Layout Containers**:
        *   Ferra `VStack` -> SwiftUI `VStack`
        *   Ferra `HStack` -> SwiftUI `HStack`
        *   Ferra `ZStack` -> SwiftUI `ZStack`
        *   Ferra `Spacer` -> SwiftUI `Spacer`
        *   Ferra `Grid` -> SwiftUI `LazyVGrid` or `LazyHGrid` (with appropriate column configuration).
    *   **Content Elements**:
        *   Ferra `Text` -> SwiftUI `Text` (with modifiers for font, color, etc., mapping to SwiftUI text modifiers).
        *   Ferra `Image` -> SwiftUI `Image` (source mapping from Ferra `ImageSource` to `Image(systemName:)`, `Image(named:)`, or async image loading for URLs).
        *   Ferra `Button` -> SwiftUI `Button` (mapping the Ferra `on_tap` closure to the button's action).
    *   **Controls**:
        *   Ferra `TextField` -> SwiftUI `TextField`
        *   Ferra `SecureField` -> SwiftUI `SecureField`
        *   Ferra `Toggle` -> SwiftUI `Toggle`
        *   Ferra `Slider` -> SwiftUI `Slider`
        *   Ferra `ProgressView` -> SwiftUI `ProgressView`
    *   **Collections**:
        *   Ferra `ScrollView` -> SwiftUI `ScrollView`
        *   Ferra `List` -> SwiftUI `List` or `ForEach` within a `ScrollView` for dynamic content.
    *   **Navigation**:
        *   Ferra `NavigationView` -> SwiftUI `NavigationView` (or `NavigationStack` for newer iOS versions).
        *   Ferra `Link` -> SwiftUI `NavigationLink`.
    *   **Modifiers**: Ferra UI modifiers (e.g., `.padding()`, `.font_size()`, `.background_color()`) will be translated into their corresponding SwiftUI view modifiers (e.g., `.padding()`, `.font(.system(size:))`, `.background()`).

*   **Mapping Ferra State Management to SwiftUI**:
    *   The goal is to make state management feel natural within the Ferra UI-DSL while being compatible with SwiftUI's reactive update model.
    *   **Local Component State**:
        *   Ferra `#[State] var counter = 0;` or `let (count, set_count) = ui::state(0);` needs to map to SwiftUI's state-driving properties.
        *   This will likely involve the generated Swift code for a Ferra component using SwiftUI's `@State` property wrapper for state declared within that component.
        *   When the Ferra state variable is updated (e.g., from an event handler), the generated Swift code will update the corresponding `@State` variable, triggering a SwiftUI view update.
    *   **Two-Way Data Binding**:
        *   Ferra `&mut String` passed to `TextField(text_binding: &mut username)` will map to SwiftUI's `Binding<String>`.
        *   The generated Swift code will create a `Binding` that reads from and writes to the underlying Ferra state (which might be managed via FFI calls to get/set the Ferra variable's value if the state logic resides entirely in Ferra).
    *   **Shared State / Observable Objects (Conceptual for v0.1 Preview)**:
        *   If Ferra UI-DSL supports a concept of observable objects for shared state, these could map to SwiftUI's `@ObservedObject` or `@EnvironmentObject`.
        *   The Ferra object would need to conform to a pattern that can be bridged to Swift's `ObservableObject` protocol (e.g., Ferra object calls a Swift-side delegate on change, which then triggers `objectWillChange.send()`). This involves FFI.
        *   Alternatively, complex state might be managed by Ferra actors, with UI updates driven by messages from actors to the UI layer (again, via an FFI bridge).

*   **Event Handling**:
    *   Ferra callback functions (e.g., for `Button.on_tap`) will be invoked from the Swift side.
    *   This typically involves:
        1.  The Ferra UI-DSL compiler generating Swift UI code.
        2.  The Ferra business logic (including event handler functions) being compiled into a native library (e.g., a static or dynamic library for iOS).
        3.  The generated Swift UI code making FFI calls (see `FFI_C_CPP.md`) to the Ferra library to execute the Ferra event handler function when a UI event occurs.
        4.  If the Ferra handler is `async`, the FFI call would need to manage the asynchronous nature, potentially returning immediately while the Ferra async task runs, and then Ferra calling back into Swift (via another FFI mechanism) to update state upon completion.

The direct conceptual parallels between the proposed Ferra UI-DSL and SwiftUI make SwiftUI an ideal primary compilation target for iOS, aiming for a high degree of fidelity and native performance.

### 3.2. UIKit Fallback/Integration

While SwiftUI is the primary target for its declarative nature, UIKit remains a mature and comprehensive framework on iOS. There might be scenarios where:
1.  A specific UI component or feature required by the Ferra UI-DSL does not have a direct or adequate equivalent in SwiftUI.
2.  Deeper platform integration is needed that is more readily available through UIKit APIs.
3.  The Ferra UI needs to integrate with existing UIKit-based application parts.

In such cases, a strategy for UIKit fallback or integration is necessary.

*   **Leveraging SwiftUI's UIKit Interoperability**:
    *   SwiftUI provides mechanisms to embed UIKit views and view controllers into a SwiftUI view hierarchy using `UIViewRepresentable` and `UIViewControllerRepresentable` protocols.
    *   **`UIViewRepresentable`**: If a specific Ferra UI element needs to be backed by a UIKit `UIView` (e.g., a specialized map view, a web view for v0.1 if not directly in DSL, or a complex custom control), the Ferra-to-SwiftUI code generator can create a Swift struct conforming to `UIViewRepresentable`.
        *   This struct would be responsible for creating and configuring the underlying `UIView` (`makeUIView`) and updating it when Ferra state changes (`updateUIView`).
        *   Data and callbacks would be passed between Ferra and this Swift representable struct, likely via an FFI bridge and a coordinator class if complex interaction is needed.
    *   **`UIViewControllerRepresentable`**: Similarly, if a whole screen or a more complex piece of UI is better managed by a UIKit `UIViewController`, it can be wrapped for use in SwiftUI.

*   **Strategy for Ferra UI-DSL**:
    *   **Identify Gaps**: As the common Ferra UI-DSL is mapped to SwiftUI, any elements or features that cannot be satisfactorily implemented with pure SwiftUI will be candidates for a UIKit-backed implementation.
    *   **Ferra Component to UIKit Wrapper**:
        *   The Ferra UI-DSL would still define the component abstractly (e.g., `ui::MapView(...)`).
        *   The code generator for the iOS target would then produce the necessary `UIViewRepresentable` (or `UIViewControllerRepresentable`) Swift wrapper for that specific Ferra component.
        *   This wrapper would instantiate and manage the corresponding UIKit view (e.g., `MKMapView`).
    *   **FFI for Control**: Ferra logic would control the UIKit view via FFI calls that interact with methods exposed by the Swift wrapper or a coordinator. For example, to update map coordinates or handle annotations.

*   **Considerations**:
    *   **Complexity**: Bridging to UIKit adds a layer of complexity compared to direct SwiftUI mapping. This should be done judiciously for components where it provides significant value or is unavoidable.
    *   **State Management**: Managing state and data flow between Ferra, the SwiftUI host, and the embedded UIKit view/controller needs careful design to maintain a reactive UI. SwiftUI's `Binding` can often be used to communicate with the representable.
    *   **Preview Limitations**: For the v0.1 preview of the UI-DSL, extensive UIKit fallbacks will be minimized to keep the scope manageable. The focus is on components that map well to SwiftUI. However, acknowledging this fallback path is important for future extensibility.

This approach allows the Ferra UI-DSL to benefit from the full capabilities of the iOS platform by using UIKit where SwiftUI might be insufficient, while still aiming for a primarily SwiftUI-driven architecture from the Ferra DSL's perspective.

### 3.3. Code Generation Strategy (iOS)

The Ferra UI-DSL, once parsed into an AST (or a dedicated UI tree representation), needs to be translated into executable Swift code that utilizes SwiftUI and potentially UIKit. This code generation step is critical for bridging the Ferra domain with the native iOS UI layer.

*   **Input**: The Ferra compiler will take the Ferra UI-DSL code (likely represented as a specific part of the Ferra AST or a derived UI-specific intermediate representation) as input.
*   **Output**: The primary output will be Swift source files (`.swift`). These files will contain:
    *   SwiftUI `View` structs corresponding to Ferra UI components.
    *   SwiftUI `ViewModifier`s corresponding to Ferra UI modifiers.
    *   Swift code for state management bridges (e.g., creating SwiftUI `Binding`s that interact with Ferra state via FFI).
    *   FFI declarations (`@_cdecl` or bridging headers if using Objective-C shims) to make Ferra functions (like event handlers or state accessors) callable from Swift.
    *   Swift code to invoke these FFI-exposed Ferra functions.

*   **Code Generation Process**:
    1.  **AST Traversal/Transformation**: The Ferra compiler will traverse the UI-DSL portion of the AST.
    2.  **Component Mapping**: Each Ferra UI element (e.g., `ui::VStack`, `ui::Text`) will be mapped to a corresponding Swift code generator that produces the SwiftUI equivalent (`VStack{...}`, `Text("...")`).
    3.  **Modifier Translation**: Ferra modifiers (e.g., `.padding()`, `.font_size()`) will be translated into chained SwiftUI modifier calls on the generated views.
    4.  **State Bridging**:
        *   For Ferra state variables (`#[State] var ...`) used to drive the UI, the Swift code will need to interact with this state.
        *   This might involve generating Swift `ObservableObject` classes or using `@State` / `@ObservedObject` / `@EnvironmentObject` property wrappers that are synchronized with the Ferra state via FFI.
        *   For two-way bindings (e.g., `TextField`'s `text_binding`), the generated Swift `Binding` will need `get` and `set` closures that make FFI calls to Ferra functions to read/write the underlying Ferra state variable.
    5.  **Event Handler Bridging**:
        *   Ferra event handler functions (e.g., `on_tap` callbacks) will be exposed from the compiled Ferra native library with a C ABI (as per `FFI_C_CPP.md`).
        *   The generated Swift UI code (e.g., a SwiftUI `Button`) will, in its action closure, make an FFI call to the corresponding exported Ferra event handler.
        *   If the Ferra event handler is `async`, the Swift FFI call might need to handle this by setting up a mechanism for the Ferra async task to call back into Swift upon completion if UI updates are needed immediately from the result (e.g., via another FFI callback or a shared state update mechanism).

*   **FFI Layer for Ferra Logic**:
    *   The core application logic, state management beyond simple UI state, and event handler implementations will reside in Ferra code, compiled into a native library (e.g., a `.framework` or static/dynamic library).
    *   `FFI_C_CPP.md` defines how Ferra functions are exported with a C ABI. These exported functions (and any necessary data structures defined with `#[repr(C)]`) will be the bridge. Adherence to the conventions in **FFI_C_CPP.md** (e.g., for string marshalling, error propagation, memory management for FFI-passed data) is crucial.
    *   **Swift Interop with C ABI**: Swift has excellent interoperability with C. It can directly call C functions (and thus Ferra functions exported with `extern "C"`).
    *   **Data Marshalling**: Data passed between Swift and Ferra via FFI (e.g., state values, event parameters) must be marshalled according to the rules and types defined in **FFI_C_CPP.md**. Helper functions on both the Ferra and Swift side might be needed for complex types.

*   **Project Structure and Build Integration**:
    *   A Ferra mobile project targeting iOS would likely consist of:
        1.  Ferra code (UI-DSL, application logic).
        2.  Generated Swift UI code.
        3.  A native iOS project (e.g., Xcode project) that includes the generated Swift code and links against the compiled Ferra native library.
    *   The Ferra build tool (`lang build`) would orchestrate the compilation of Ferra code, generation of Swift UI code, and potentially trigger the Xcode build process or provide clear instructions/artifacts for integration. For details on generating Apple Bitcode suitable for App Store submission, see **BACKEND_EXPANDED_TARGETS.md** (Section 4).

*   **Future: Direct Swift Interop?**
    *   While initial interop will rely on the C ABI via `FFI_C_CPP.md`, future Ferra versions might explore more direct or richer interoperability with Swift if technically feasible, potentially reducing some FFI boilerplate. For v0.1, C ABI is the pragmatic choice.

This strategy aims to keep the UI rendering native (SwiftUI) while allowing significant UI logic and application state to be written and managed in Ferra.

### 3.4. Specific Component Mappings (iOS Examples)

This subsection provides a few illustrative examples of how core Ferra UI-DSL components and their properties would map to SwiftUI. These are conceptual and subject to refinement based on the final DSL syntax and FFI capabilities.

*   **Ferra `Text(content: String)` to SwiftUI `Text`**:
    *   **Ferra DSL**: `ui::Text("Hello, SwiftUI!")`
    *   **Generated Swift Code (Conceptual)**: `Text("Hello, SwiftUI!")`
    *   **Modifiers**:
        *   Ferra: `.font_size(18.0).font_weight(FontWeight::Bold).foreground_color(Color::Named("primary"))`
        *   SwiftUI: `.font(.system(size: 18, weight: .bold)).foregroundColor(.primary)`
        *   Mapping `Color::Named("primary")` to SwiftUI `Color.primary` would be part of the theming/color system translation.

*   **Ferra `Button(text_label: String, on_tap: fn() -> Unit)` to SwiftUI `Button`**:
    *   **Ferra DSL**:
        ```ferra
        ui::Button(text_label: "Tap Me", on_tap: || {
            // my_ferra_action_handler();
            println("Button tapped in Ferra!");
        })
        ```
    *   **Generated Swift Code (Conceptual)**:
        ```swift
        Button("Tap Me") {
            // FFI call to a Ferra function that wraps the on_tap closure/logic
            // For example: ferra_generated_button_tap_action_id123()
            ferra_button_actions.action_for_id("id123")?() // Conceptual
        }
        ```
    *   The `ferra_generated_button_tap_action_id123()` would be an `extern "C"` Ferra function. The Swift code needs a way to identify and call the correct Ferra handler. This might involve registering handlers with IDs or using a more direct FFI call if the `on_tap` Ferra function itself is exported.

*   **Ferra `VStack { children: [...] }` to SwiftUI `VStack`**:
    *   **Ferra DSL**:
        ```ferra
        ui::VStack(alignment: .leading, spacing: 10.0) {
            children: [
                ui::Text("Item 1"),
                ui::Text("Item 2")
            ]
        }
        ```
    *   **Generated Swift Code (Conceptual)**:
        ```swift
        VStack(alignment: .leading, spacing: 10.0) {
            // Generated Text("Item 1")
            // Generated Text("Item 2")
        }
        ```
    *   Child components are recursively translated.

*   **Ferra `TextField(placeholder: String, text_binding: &mut String)` to SwiftUI `TextField` with `Binding`**:
    *   **Ferra DSL**:
        ```ferra
        #[State] var user_name = "";
        // ...
        ui::TextField(placeholder: "Enter name", text_binding: &mut user_name)
        ```
    *   **Generated Swift Code (Conceptual)**:
        A Swift `View` struct would be generated, holding a mechanism to bridge to the Ferra `user_name` state.
        ```swift
        struct FerraTextFieldWrapperView_idXYZ: View {
            // This binding would internally use FFI to get/set the Ferra state variable
            @ObservedObject var ferraStateBridge: FerraStateBridge_idXYZ // Manages FFI communication

            var body: some View {
                TextField("Enter name", text: $ferraStateBridge.userNameBinding)
            }
        }

        // Conceptual Swift bridge object
        // class FerraStateBridge_idXYZ: ObservableObject {
        //     @Published var userNameSwift: String = "" // Local Swift copy, or direct FFI calls in Binding
        //
        //     var userNameBinding: Binding<String> {
        //         Binding(
        //             get: {
        //                 // FFI call: ferra_get_user_name_idXYZ() -> CString
        //                 // Convert CString to Swift String
        //                 return self.userNameSwift; // Simplified
        //             },
        //             set: { newValue in
        //                 self.userNameSwift = newValue;
        //                 // FFI call: ferra_set_user_name_idXYZ(newValue.cString(using: .utf8))
        //             }
        //         )
        //     }
        //     // Needs logic to update userNameSwift if Ferra state changes externally
        // }
        ```
    *   The exact bridging for two-way bindings (`&mut T`) will be one of the more complex FFI interactions, requiring careful design of the FFI interface for state accessors.

These examples illustrate the general approach. The code generator will need a comprehensive mapping for all supported DSL elements and their modifiers to their SwiftUI counterparts, including handling the FFI calls for event handlers and state synchronization.

## 4. Mapping to Android - Jetpack Compose (Step 3.1.4)

This section details how the common Ferra UI-DSL will be translated into native UI code for the Android platform. Jetpack Compose is the primary target, being Android's modern declarative UI toolkit, which shares many principles with SwiftUI and the proposed Ferra UI-DSL.

### 4.1. Primary Target: Jetpack Compose

Jetpack Compose allows developers to build native Android UIs with Kotlin in a declarative way. Its compositional approach and state management system are conceptually similar to SwiftUI, making it a suitable target for the Ferra UI-DSL.

*   **Direct Translation of Components and Modifiers**:
    *   Similar to the iOS mapping, many core Ferra UI elements and their modifiers are expected to have direct or close equivalents in Jetpack Compose.
    *   **Layout Containers**:
        *   Ferra `VStack` -> Compose `Column`
        *   Ferra `HStack` -> Compose `Row`
        *   Ferra `ZStack` -> Compose `Box` (Compose's `Box` is more general but can achieve `ZStack` behavior with appropriate modifiers).
        *   Ferra `Spacer` -> Compose `Spacer` modifier or `Modifier.weight(1f)`.
        *   Ferra `Grid` -> Compose `LazyVerticalGrid` or a custom `Layout` composable.
    *   **Content Elements**:
        *   Ferra `Text` -> Compose `Text` (with modifiers for font, color, etc., mapping to Compose `TextStyle` and other text attributes).
        *   Ferra `Image` -> Compose `Image` (handling `ImageSource` to load from drawables, assets, or network via libraries like Coil/Glide).
        *   Ferra `Button` -> Compose `Button` (mapping the Ferra `on_tap` closure to `onClick`).
    *   **Controls**:
        *   Ferra `TextField` -> Compose `TextField` or `OutlinedTextField`.
        *   Ferra `SecureField` -> Compose `TextField` with `visualTransformation = PasswordVisualTransformation()`.
        *   Ferra `Toggle` -> Compose `Switch` or `Checkbox`.
        *   Ferra `Slider` -> Compose `Slider`.
        *   Ferra `ProgressView` -> Compose `LinearProgressIndicator` or `CircularProgressIndicator`.
    *   **Collections**:
        *   Ferra `ScrollView` -> Compose `LazyColumn` / `LazyRow` or `Modifier.verticalScroll()` / `Modifier.horizontalScroll()`.
        *   Ferra `List` -> Compose `LazyColumn` or `LazyRow` for dynamic content.
    *   **Navigation**:
        *   Ferra `NavigationView` -> Compose Navigation components (e.g., using `NavController` and defining routes/destinations). This mapping is more complex than simple view swapping and will require a robust navigation abstraction in the Ferra UI-DSL.
        *   Ferra `Link` -> Compose `Button` or clickable `Text` that triggers navigation via `NavController`.
    *   **Modifiers**: Ferra UI modifiers will be translated into their corresponding Jetpack Compose `Modifier` extensions (e.g., `Modifier.padding()`, `Modifier.background()`, `Modifier.fillMaxSize()`).

*   **Mapping Ferra State Management to Jetpack Compose**:
    *   Compose has its own reactive state system that Ferra's state management needs to integrate with.
    *   **Local Component State**:
        *   Ferra `#[State] var counter = 0;` or `let (count, set_count) = ui::state(0);` will need to bridge to Compose's `State<T>` objects, typically created with `remember { mutableStateOf(...) }`.
        *   When Ferra state changes, the corresponding Compose `State` object must be updated (likely via an FFI bridge), triggering recomposition of affected composables.
    *   **Two-Way Data Binding**:
        *   Ferra `&mut String` for `TextField` will map to passing a `MutableState<String>` to the Compose `TextField`'s `value` and `onValueChange` parameters. The `onValueChange` lambda will call back into Ferra (via FFI) to update the Ferra state variable.
    *   **Shared State / ViewModels**:
        *   For more complex or shared state, the Android mapping might involve generating or interacting with Android `ViewModel`s.
        *   Ferra application logic (perhaps in actors) could update data in a `ViewModel` (potentially via an FFI bridge to Kotlin code that owns the ViewModel), and Compose UI would observe `StateFlow` or `LiveData` from this `ViewModel`.

### 4.2. Code Generation Strategy (Android)

The process will involve generating Kotlin code that uses Jetpack Compose.

*   **Input**: The Ferra UI-DSL representation (AST or UI-IR).
*   **Output**: Kotlin source files (`.kt`) containing:
    *   `@Composable` functions corresponding to Ferra UI components.
    *   Compose `Modifier` chains based on Ferra modifiers.
    *   Code for bridging state and event handling between Ferra and Kotlin/Compose via JNI/FFI.
    *   JNI declarations if Ferra logic is compiled to a native library.

*   **Code Generation Process**:
    1.  **AST/UI-IR Traversal**: Similar to iOS, the compiler traverses the Ferra UI description.
    2.  **Component and Modifier Mapping**: Translate Ferra elements and modifiers to their Compose Kotlin equivalents.
    3.  **State Bridging**: Generate Kotlin code that uses `mutableStateOf` and `remember` for local state, and connect these to Ferra state via JNI calls to exported Ferra accessor/mutator functions.
    4.  **Event Handler Bridging**: Ferra event handlers (exported as C functions via `FFI_C_CPP.md`) will be called from Kotlin through JNI. For `async` Ferra handlers, the JNI call might trigger the async task, and completion could notify the Kotlin/Compose layer (e.g., via a callback passed through JNI or by updating shared state).

*   **FFI Layer (JNI)**:
    *   Interaction between Kotlin (for Jetpack Compose) and compiled Ferra native code will occur via the Java Native Interface (JNI).
    *   Ferra functions (event handlers, state accessors) will be exported with a C ABI, as detailed in **FFI_C_CPP.md**. These C functions then become the target for JNI bridge calls.
    *   Kotlin code will use `external fun` declarations to define the JNI signatures for these C-exported Ferra functions.
    *   A C/C++ JNI glue layer (itself adhering to **FFI_C_CPP.md** on the Ferra side) might be necessary to adapt the exported Ferra C functions to the specific function signatures and types expected by JNI (e.g., converting JNI types like `jstring`, `jobject` to C types that Ferra can understand, and vice-versa).
    *   Data marshalling across JNI will follow rules similar to C FFI (as per **FFI_C_CPP.md**), but with JNI-specific considerations for Java/Kotlin types.

*   **Project Structure and Build Integration**:
    *   A Ferra Android project would involve:
        1.  Ferra code (UI-DSL, application logic).
        2.  Generated Kotlin/Compose UI code.
        3.  Ferra code compiled as a native library (`.so` files for different Android ABIs).
        4.  An Android Gradle project that includes the generated Kotlin code and the Ferra native libraries.
    *   The Ferra build tool (`lang build`) would need to orchestrate these steps, possibly generating a Gradle project structure or providing easy integration points. For specifics on generating Android App Bundles (AABs), refer to **BACKEND_EXPANDED_TARGETS.md** (Section 5).

### 4.3. Specific Component Mappings (Android Examples)

This subsection provides conceptual examples mirroring those for iOS, but for Jetpack Compose.

*   **Ferra `Text(content: String)` to Compose `Text`**:
    *   **Ferra DSL**: `ui::Text("Hello, Compose!")`
    *   **Generated Kotlin Code (Conceptual)**: `Text("Hello, Compose!")`
    *   **Modifiers**: Ferra modifiers map to Compose `Modifier` extensions or `TextStyle` properties.

*   **Ferra `Button(text_label: String, on_tap: fn() -> Unit)` to Compose `Button`**:
    *   **Ferra DSL**:
        ```ferra
        ui::Button(text_label: "Press Me", on_tap: || {
            println("Button pressed in Ferra!");
        })
        ```
    *   **Generated Kotlin Code (Conceptual)**:
        ```kotlin
        Button(onClick = {
            // JNI call to the Ferra on_tap handler
            FerraBridge.buttonTapAction("buttonId_abc") 
        }) {
            Text("Press Me")
        }
        ```

*   **Ferra `VStack { children: [...] }` to Compose `Column`**:
    *   **Ferra DSL**:
        ```ferra
        ui::VStack(alignment: .center, spacing: 8.0) { /* ... */ }
        ```
    *   **Generated Kotlin Code (Conceptual)**:
        ```kotlin
        Column(
            horizontalAlignment = Alignment.CenterHorizontally, 
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) { /* ... */ }
        ```

*   **Ferra `TextField(text_binding: &mut String)` to Compose `TextField`**:
    *   **Ferra DSL**:
        ```ferra
        #[State] var searchText = "";
        ui::TextField(placeholder: "Search...", text_binding: &mut searchText)
        ```
    *   **Generated Kotlin Code (Conceptual)**:
        ```kotlin
        var searchTextState by remember { mutableStateOf(FerraBridge.getSearchText("searchFieldId")) } // Initial value via FFI
        TextField(
            value = searchTextState,
            onValueChange = { newValue ->
                searchTextState = newValue
                FerraBridge.setSearchText("searchFieldId", newValue) // Update Ferra state via FFI
            },
            label = { Text("Search...") }
        )
        ```
        The `FerraBridge` is a conceptual Kotlin object handling JNI calls.

### 4.4. Runtime Size Consideration (Target: ≤ 5 MB for Ferra portion)

Achieving the ≤ 5 MB target for the Ferra-related portion of an Android application (as per `comprehensive_plan.md` Module 2.3.3, which was a WASM target, but the principle applies here for native mobile) requires diligent optimization.

*   **Minimal Ferra Runtime**: The parts of the Ferra runtime compiled into the native library must be as small as possible. This means aggressive stripping of unused runtime features.
*   **Efficient Code Generation**: The Ferra compiler's native backend (e.g., LLVM as per `BACKEND_LLVM_X86-64.md`, adapted for Android ABIs like ARM) must optimize for size when targeting mobile.
*   **Tree-Shaking / Dead Code Elimination**:
    *   Applied to the compiled Ferra native library to remove any unused Ferra code (application logic, UI-DSL helpers, stdlib parts).
    *   Applied by R8/ProGuard to the Kotlin/Java bytecode, including any generated Kotlin UI code and JNI bridge code.
*   **Native Library Stripping**: Standard tools (e.g., `strip`) should be used on the compiled Ferra `.so` files to remove unnecessary symbols and debug information for release builds.
*   **Avoid Large Dependencies in Ferra Code**: Ferra code contributing to the UI or core logic should be mindful of pulling in large Ferra library dependencies if they are not essential.
*   **FFI Overhead**: The JNI layer itself adds some overhead, but this should be manageable. The bulk of the size will come from the compiled Ferra code and its runtime.
*   **WASM as an Alternative? (Cross-reference to Module 2.3)**:
    *   While direct native compilation is the current focus for this section, `BACKEND_WASM_WASI.md` discusses WASM. If a highly optimized Ferra-to-WASM toolchain exists with a very small WASM runtime, embedding Ferra logic as a WASM module callable via JNI from Kotlin could be an alternative path to explore for size, though it introduces another layer of indirection. For v0.1 native preview, direct native compilation of Ferra logic is assumed. The principles for achieving a small binary size (e.g., ≤ 200 kB baseline for WASM as per `Steps.md` Item 9 and the CI rule mentioned in `BACKEND_WASM_WASI.md` Section 5) are relevant here.
*   **Measurement and Profiling**: The build process must include tools to measure the size contribution of the Ferra native libraries to the final APK/AAB, allowing for tracking against the 5MB target.
*   **CI Enforcement**: A CI check (e.g., named `check_android_ferra_runtime_size` in the project's CI configuration) should be implemented to fail builds if the Ferra-related portion of a release Android build exceeds the 5 MB target. This echoes the CI rule approach for WASM size (see `BACKEND_WASM_WASI.md` Section 5 and `Steps.md` Item 9).
    *   *Conceptual CI Invocation/Check Example*:
        ```sh
        # Example: Command that might be run in CI
        # ferra_size_checker --target=android --build=release --max-mb=5.0
        # Or, a badge/status from a CI job step:
        # [CI Job: android_size_check] Status: PASS (4.78MB / 5.00MB)
        ```
    *   *Conceptual CI Output Example (on failure)*:
        ```
        ERROR: Ferra Android runtime size check failed.
        Expected <= 5.00 MB, Actual = 5.23 MB.
        Ferra native libraries: libferra_applogic.so (3.10MB), libferra_ui_bridge.so (2.13MB).
        ```

Strategies like careful stdlib design, compiler optimizations for size, and aggressive stripping will be key to meeting this runtime size target on Android.

## 5. `lang doctor ios` Static Analyzer (Step 3.1.3)

To improve the developer experience and help create higher-quality iOS applications with the Ferra UI-DSL, a dedicated static analysis tool, `lang doctor ios`, will be designed. This tool will inspect Ferra UI code specifically for iOS-related best practices, potential performance issues, and common pitfalls before the code is compiled to native Swift/UIKit.

### 5.1. Purpose and Goals

*   **Primary Purpose**: To provide early feedback to developers on their Ferra UI-DSL code concerning its suitability and optimality for the iOS platform (SwiftUI/UIKit).
*   **Key Goals**:
    *   **Proactive Issue Detection**: Identify common iOS-specific UI/UX antipatterns or performance concerns that can be inferred from the Ferra UI-DSL source.
    *   **Actionable Recommendations**: Offer clear, constructive advice and suggestions on how to improve the UI code for the iOS target.
    *   **Educational Tool**: Help Ferra developers learn about iOS UI best practices, even if they are not deeply familiar with native iOS development.
    *   **Reduce Debugging Cycles**: Catch potential problems before they manifest as runtime issues or suboptimal user experiences on iOS devices.
    *   **Complement to Generic Ferra Linter**: While Ferra will have a general linter, `lang doctor ios` provides checks specific to the iOS UI mapping.

### 5.2. Checks to Perform (Examples for v0.1 Preview)

The initial "preview" version of `lang doctor ios` will focus on a manageable set of checks. More sophisticated analyses can be added in the future.

*   **Layout and Performance Hints**:
    *   **Deeply Nested Stacks**: Detect excessively deep nesting of `VStack` or `HStack` components, which can sometimes lead to performance issues in SwiftUI. Suggest alternatives like using `LazyVStack`/`LazyHStack` (if directly mappable from Ferra `List` or similar) or restructuring the view.
    *   **Misuse of `Spacer`**: Identify patterns where `Spacer` might be used in a way that leads to unexpected layout behavior or inefficient space distribution.
    *   **Image Sizing**: Warn if `Image` components are used without explicit sizing (`.frame()`) or appropriate content modes (`.resizable().aspectRatio()`), which could lead to performance issues with large images or unexpected layouts.
    *   **List Item Complexity**: For `List` components, warn if the `row_builder` function appears to generate overly complex or deeply nested views for each row, which can impact scrolling performance. Suggest simplifying row components or using techniques like `drawingGroup()` (if a Ferra equivalent exists).

*   **iOS Human Interface Guidelines (HIG) Adherence (Basic)**:
    *   **Navigation Patterns**: Check for common misuses of `NavigationView` or `Link` that might deviate from standard iOS navigation paradigms (e.g., too many nested navigation views).
    *   **Standard Control Usage**: If a Ferra component maps to a standard iOS control, check if it's being used in a way that is highly unconventional for iOS (e.g., a `Toggle` used for a non-boolean action).
    *   **Touch Target Sizes**: Heuristically check if interactive elements like `Button` or list rows might result in touch targets smaller than Apple's recommended minimums (e.g., by analyzing explicit frame sizes or font sizes).

*   **Accessibility (Basic Checks)**:
    *   **Missing Labels for Interactive Elements**: Warn if `Button`s, `Toggle`s, `Slider`s, etc., lack descriptive text labels or accessibility identifiers that would be crucial for VoiceOver and other assistive technologies. (Requires the DSL to have a way to specify accessibility labels).
    *   **Dynamic Type Support**: Encourage the use of relative font sizing or provide warnings if fixed font sizes are used extensively, which can hinder Dynamic Type support.

*   **Resource Management Hints**:
    *   **Large Local Assets**: If image assets referenced via `ImageSource::Asset(...)` are found to be very large, suggest optimizing them for mobile. (Requires tool access to project assets).

*   **API Usage (Ferra UI-DSL specific for iOS mapping)**:
    *   **Deprecated Ferra UI Components/Modifiers (Future)**: If parts of the Ferra UI-DSL become deprecated for iOS mapping, `lang doctor ios` can warn about their usage.
    *   **Platform-Specific API Suggestions**: If Ferra gains platform-specific UI extensions, the doctor could suggest using an iOS-specific variant where beneficial.

### 5.3. Output and Integration

*   **Command-Line Interface**:
    *   The tool will be invoked via `lang doctor ios [path_to_ferra_files_or_project]`.
    *   If run within a Ferra project directory, it will analyze the relevant UI code in that project.
    *   **Conceptual EBNF for the command**:
        ```ebnf
        DoctorCmd  ::= "lang" "doctor" TargetOS ( FilePath | DirPath ) Options*
        TargetOS   ::= "ios" (* | "android" - future *)
        Options    ::= FormatOption | LevelOption | FixOption | OtherFlags*
        FormatOption ::= "--format" "=" ("text" | "json")
        LevelOption  ::= "--level" "=" ("error" | "warning" | "info" | "hint")
        FixOption    ::= "--fix" (* Attempt to auto-apply suggestions - future *)
        // FilePath, DirPath, OtherFlags are standard terminal argument patterns.
        ```
    *   **Example Invocation with Flags**:
        ```sh
        lang doctor ios ./src/my_ui_module.ferra
        lang doctor ios --format=json ./my_app_project
        lang doctor ios --level=warning
        lang doctor ios --fix (* Future: Attempt auto-fixes *)
        ```
*   **Output Format**:
    *   **Human-Readable**: Console output listing issues found, with file paths, line numbers, suggestions, and severity levels (e.g., ERROR, WARNING, INFO/HINT). This output should follow the "positive-first" principles and general diagnostic style outlined in **DESIGN_DIAGNOSTICS.md**.
    *   **Machine-Readable (Future)**: Optionally output diagnostics in JSON format, ideally conforming to the schema specified in **DESIGN_DIAGNOSTICS.md** (Section 3.2 JSON Line Protocol), for integration with IDEs like VSCode. This allows issues to be displayed directly in the code editor.
*   **Integration with Main Diagnostics System**:
    *   `lang doctor ios` should utilize the same underlying diagnostic reporting infrastructure as the main Ferra compiler (see **DESIGN_DIAGNOSTICS.md**), ensuring consistent message formatting, severity levels, and potential for de-duplication or filtering. The specific mechanisms for de-duplication (e.g., Bloom filter as per `FRONTEND_ENHANCEMENTS.md`) would apply.
    *   Diagnostic codes for `lang doctor ios` issues would be distinct (e.g., `DCI_XXX` for Doctor Check iOS) and registered in `diagnostic_codes.md`.
*   **Severity Levels**:
    *   **Error**: Indicates a strong violation of best practices or something likely to cause significant runtime issues or poor UX on iOS.
    *   **Warning**: Suggests an area for improvement or a potential issue that might not be critical.
    *   **Hint/Info**: Provides advice or points out opportunities for optimization or better adherence to HIG.

The `lang doctor ios` tool is envisioned as a helpful companion for Ferra developers targeting iOS, aiming to improve the quality and platform-appropriateness of their UIs. A similar `lang doctor android` could be a future consideration.

## 6. Cross-Platform Considerations & Abstractions

While the primary goal of the Ferra UI-DSL is to provide a common language for UI development across iOS and Android, inherent platform differences must be acknowledged and addressed. The DSL aims for a high degree of code sharing by abstracting common patterns, but it also needs a strategy for handling divergences.

*   **Handling Platform Differences**:
    *   **Navigation Paradigms**: iOS and Android have distinct navigation patterns (e.g., iOS's typical navigation controller stack vs. Android's Activities/Fragments and Compose Navigation). The Ferra UI-DSL's navigation components (`NavigationView`, `Link`) must be abstract enough to map appropriately to each platform's idiomatic navigation flow. This might mean the Ferra DSL describes *intent* (e.g., "navigate to X screen," "go back") rather than dictating a specific visual stack presentation that only fits one platform.
    *   **Specific Iconography and System Glyphs**: Common icons (e.g., share, back arrow, settings) often have platform-standard representations. The DSL might provide abstract icon identifiers (e.g., `Icon::Share`, `Icon::Settings`) that map to SF Symbols on iOS and Material Design Icons on Android.
    *   **Standard Controls and Behaviors**: While core elements like buttons and text fields are common, their detailed behavior, animations, or associated system dialogs (e.g., date pickers, file pickers if added later) can vary. The Ferra DSL components will aim to provide a common API surface, with the native mapping layer responsible for invoking the platform-idiomatic control.
    *   **Gestures**: Basic tap gestures are straightforward. More complex gestures (long press, swipe, pinch-to-zoom) have platform-specific implementations and sensitivities. The v0.1 DSL will likely focus on simple tap/click events, deferring complex gesture abstractions.
    *   **Styling and Theming Nuances**: While basic styling can be common, default control appearances, font rendering, and animation curves differ. The DSL should not try to enforce pixel-perfect similarity but rather map Ferra styling intent to the native styling system effectively (as discussed in Section 2.5).

*   **Mechanisms for Conditional UI (v0.1 Preview - Limited Scope)**:
    *   For the v0.1 preview, the emphasis is on the *common* DSL. Extensive features for deeply platform-specific UI variations within the same Ferra codebase will be limited.
    *   **Conditional Compilation**: Ferra's existing conditional compilation features (`#[cfg(target_os = "ios")]`, `#[cfg(target_os = "android")]`) could be used at a coarse-grained level within Ferra UI component functions to include or exclude certain sub-components or apply different modifiers.
        ```ferra
        fn MyPlatformAwareComponent() -> ui::Component {
            let base_view = ui::VStack { /* ... common elements ... */ };
            
            #[cfg(target_os = "ios")]
            {
                // base_view = base_view.add_child(ui::AppleSpecificFooter()); // Conceptual
            }

            #[cfg(target_os = "android")]
            {
                // base_view = base_view.add_child(ui::MaterialDesignSnackbarTrigger()); // Conceptual
            }
            return base_view;
        }
        ```
    *   **Platform-Specific Implementations via FFI (Future)**: A more robust long-term approach for highly divergent UI sections would be to define an interface in the common Ferra UI-DSL and provide separate Ferra files or modules that implement this interface by calling out to completely native (Swift or Kotlin) UI code via FFI. This is beyond v0.1 preview scope.
    *   **Querying Platform at Runtime (Limited Use)**: Some minor adaptations might be possible by querying the target platform at runtime (if such a Ferra API exists, e.g., `core::os::current()`) and adjusting component parameters. However, this is less ideal for structural UI differences than compile-time approaches.

*   **Abstraction Level of the Common DSL**:
    *   The Ferra UI-DSL aims for a "common denominator plus" approach. It will provide abstractions for UI concepts that are broadly shared between modern iOS (SwiftUI) and Android (Jetpack Compose) development.
    *   It will intentionally *not* attempt to abstract every single feature of both native frameworks, as this would lead to a leaky or overly complex common DSL.
    *   The abstraction should be high enough to provide significant code reuse and developer convenience but low enough that the mapping to native components remains efficient and allows for a native look and feel.
    *   Where a feature is highly platform-specific and has no reasonable common abstraction (e.g., specific Apple Watch complications or Android widgets for the home screen), it would fall outside the common DSL and would require platform-native code or future FFI-based extensions.

*   **Philosophy: "Write Once, Adapt Natively"**:
    *   The goal is not a "write once, run anywhere identically" UI, which often leads to a lowest-common-denominator or non-native feel.
    *   Instead, it's "write the core UI structure and logic once in Ferra, and have it map to an excellent, idiomatic native experience on each platform."
    *   The DSL provides the shared language, and the platform-specific code generators (to SwiftUI, to Compose) are responsible for the idiomatic native translation.

Balancing the desire for a common abstraction with the need to respect platform-specific conventions and capabilities is a key challenge for any cross-platform UI toolkit. Ferra's UI-DSL preview will start with a focused common core.

## 7. Limitations and Future Work (UI-DSL Preview)

The v0.1 preview of the Ferra UI-DSL is intended as a foundational step, demonstrating viability and core concepts. As such, it will have inherent limitations, with many advanced features deferred for future iterations. A more detailed potential roadmap is outlined in Appendix A.

*   **Explicitly Out of Scope for v0.1 Preview (Summary)**:
    *   **Comprehensive UI Element Set**: Beyond the core elements in Section 2.2 (e.g., no date pickers, maps, web views for v0.1).
    *   **Advanced Styling & Theming**: Limited to basic modifiers (Section 2.5) and semantic color mapping; no custom theming engine.
    *   **Complex Animations & Transitions**: Minimal to no explicit support beyond default platform behaviors.
    *   **Advanced Gesture Recognition**: Focus on basic tap/click events.
    *   **Deep Platform-Specific Integrations**: No support for features like App Clips, Widgets, Share Extensions, or platform services like HealthKit/ARKit.
    *   **Advanced Accessibility**: Basic considerations only; comprehensive support is a larger effort.
    *   **DSL-Level Localization Features**: Handled by Ferra application logic, not the UI-DSL itself.
    *   **Visual Design Tools / Live Preview**: Development is code-centric for v0.1.
    *   **Mature & Highly Optimized FFI Bridge for UI**: Foundational FFI bridge; deep optimizations are future work.
    *   **Tablet / Desktop / Web UI Adaptation**: Focus is solely on mobile (iOS/Android) for this preview.

*   **Key Immediate Future Considerations (Post-Preview v0.1)**:
    *   Expanding the core UI component library.
    *   Developing richer styling and a basic theming API.
    *   Introducing foundational animation support.
    *   Improving state management solutions for shared/complex state.
    *   Enhancing developer tooling (e.g., `lang doctor android`, basic live preview investigations).

## 8. Open Questions / TBD (UI-DSL Mobile Preview v0.1)

This section lists open questions and items marked as "To Be Determined" (TBD) during the drafting of this initial UI-DSL specification. These will need to be resolved through further design, prototyping, and implementation feedback.

*   **(UI-DSL-1) Final UI-DSL Syntax and Component Model**:
    *   The precise Ferra syntax for declaring UI elements (e.g., function calls, dedicated `ui` blocks, macros), defining modifiers, and composing children (Section 2.6).
    *   The exact nature and API of the conceptual `ui::Component` type returned by UI-defining functions.
    *   Final design for how navigation state and destinations are represented and triggered in the DSL (Section 2.2 - Navigation Elements).

*   **(UI-DSL-2) State Management Primitives and Data Binding Syntax**:
    *   The definitive syntax and mechanism for declaring local component state that triggers UI updates (e.g., `#[State]` attribute vs. `ui::state()` hook-like function) (Section 2.3).
    *   The exact API for two-way data binding, especially how `&mut T` interacts with the underlying state mechanism and native UI elements.
    *   Mechanisms for "observability" if shared state objects (beyond simple prop drilling) are to be supported more formally in v0.1 (Section 2.3).

*   **(UI-DSL-3) Ferra-Native Interop Details (Swift/Kotlin)**:
    *   The detailed FFI contract and bridge implementation for passing complex data (beyond simple primitives), state updates, and asynchronous callbacks between Ferra logic and the generated Swift/Kotlin UI code (Sections 3.3 & 4.2). This includes specifics of marshalling, memory management for shared data, and handling async completion.
    *   Defining the specific patterns for how Swift `Binding` and Compose `MutableState` will be synchronized with Ferra state variables via FFI/JNI.

*   **(UI-DSL-4) Scope and Depth of `lang doctor ios` Analyzer**:
    *   Finalizing the initial set of checks for the v0.1 `lang doctor ios` (Section 5.2).
    *   Determining the depth of analysis (e.g., simple pattern matching vs. more complex flow analysis) for these checks.
    *   Specific diagnostic codes and detailed message content for each check.

*   **(UI-DSL-5) Cross-Platform Asset Management**:
    *   A common strategy for referencing and managing UI assets (images, fonts, etc.) in a Ferra project such that they can be correctly bundled and accessed by both iOS and Android native builds (Section 2.2 - Images, Section 2.5 - Font Properties). This includes how `ImageSource::Asset("name")` resolves to platform-specific asset catalogs or drawable resources.

*   **(UI-DSL-6) Color Representation and Theming Details**:
    *   Finalizing the `ColorValue` representation (e.g., specific named colors available in `PredefinedColor`) and the precise mapping of semantic colors to SwiftUI and Jetpack Compose theme attributes (Section 2.5).

*   **(UI-DSL-7) Fixed-Size Grid and List Implementation Details**:
    *   The precise API and underlying implementation strategy for `Grid` to ensure reasonable mapping to both platforms for v0.1.
    *   The internal mechanics of `List` for efficient rendering (e.g., how it maps to `LazyVGrid`/`LazyHGrid` or `LazyColumn`/`LazyRow`).

*   **(UI-DSL-8) Standard Library UI Module**:
    *   Defining the structure and contents of the `ui` module (or `core::ui` / `std::ui`) that will house the DSL components, modifiers, and supporting types like `ColorValue`, `FontWeight`, `EdgeInsets`.

Addressing these TBDs will be crucial for producing a functional and developer-friendly UI-DSL preview.