# Ferra UI-DSL Potential Roadmap (Post-Preview v0.1)

> **Status:** Planning - Companion to `UI_DSL_MOBILE.md` (Module 3.1)

This document outlines potential areas for enhancement and expansion of the Ferra UI-DSL beyond the initial v0.1 preview defined in `UI_DSL_MOBILE.md`. The prioritization and inclusion of these features will depend on feedback from the preview, community needs, and overall project goals.

## 1. Expanded UI Component Library

Gradually add more common and advanced UI elements to the core DSL based on developer feedback and platform capabilities. Examples:

*   Date/Time Pickers
*   Map Views (with basic interaction)
*   Web Views (for embedding web content)
*   Charting Components
*   Specialized Collection Views (Carousels, Paged Scrollers, Expandable Lists)
*   Alerts and Dialogs (more sophisticated than basic system popups)
*   Menus and Context Menus

## 2. Richer Styling and Theming

*   Introduce a more comprehensive styling system.
*   Support for style sheets or cascading styles.
*   Develop a Ferra-based theming engine for custom app themes.
*   Access to more system font properties and custom font loading.
*   Low-level drawing primitives or a canvas API for custom graphics.

## 3. Animation and Transition APIs

*   Design a common Ferra API for defining property animations (implicit and explicit).
*   Support for view transitions (e.g., between screens or states).
*   Physics-based animations.

## 4. Advanced Gesture Support

*   Add support for a wider range of gestures (long press, swipe, pinch, rotate, pan).
*   API for creating custom gesture recognizers.
*   Mechanisms for gesture conflict resolution.

## 5. Platform-Specific Extension Points

*   Formalize mechanisms for developers to create or use platform-specific components and APIs within the Ferra UI-DSL structure (e.g., `#[ios(...)]`, `#[android(...)]` attributes or dedicated modules).
*   Provide clearer FFI patterns for embedding fully native (Swift/Kotlin) custom UI views within a Ferra UI hierarchy.

## 6. Improved State Management Solutions

*   Introduce more sophisticated built-in solutions for shared and application-level state management.
*   Deeper integration with Ferra's actor model for state.
*   Patterns inspired by established state management libraries (e.g., Redux-like stores, MVVM helpers).

## 7. Tooling Enhancements

*   **Live Preview**: Investigate and develop live preview capabilities within IDEs (e.g., VSCode plugin) for faster UI development iterations.
*   **Visual Designer**: Explore the potential for a visual UI design tool that can output or round-trip with the Ferra UI-DSL code.
*   **Enhanced `lang doctor`**: Expand `lang doctor` capabilities for both iOS and Android with more checks, deeper analysis, and potentially automated refactoring suggestions.
*   **Debugging Tools**: Specific tools for debugging Ferra UI layouts, state, and event flow.

## 8. Optimized FFI Bridge

Refine and optimize the FFI layer between Ferra and native UI code for better performance, reduced boilerplate, and richer data exchange.

## 9. Testing Utilities

Provide Ferra-based utilities and frameworks for UI testing that can interact with the DSL components and simulate user interactions.

## 10. Accessibility (A11y) Enhancements

Deeper integration with platform accessibility features, providing Ferra APIs to control ARIA-like attributes, focus order, and other a11y concerns.

## 11. Localization (i18n) and Internationalization

DSL-level support or tight integration with Ferra i18n libraries for localized strings, right-to-left (RTL) layout support, and locale-specific formatting.

## 12. Support for Other Platforms

Explore extending the UI-DSL concepts to other platforms like desktop (Windows, macOS, Linux) and web (via WebAssembly).

---
This roadmap is indicative and will evolve based on the project's trajectory and community input. 