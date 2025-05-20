# Ferra AI APIs: `ai::refactor` & `ai::verify` Design (Functional v0.1)

> **Status:** Initial Draft - Module 3.3 (Steps 3.3.1 - 3.3.3)

## 1. Introduction and Goals

This document specifies the functional design for two key AI-assisted development APIs in Ferra: `ai::refactor` and `ai::verify`. These APIs are intended to be invoked from Ferra code (e.g., in build scripts, development tools, or potentially via annotations processed by a compiler-integrated tool), leveraging AI capabilities to automate common coding tasks and enhance code quality. The design prioritizes clear interaction patterns, robust provenance tracking, and integration with Ferra's existing compiler infrastructure.

*   **Purpose**:
    *   To define a set of standardized, "functional" AI APIs within the Ferra ecosystem for code refactoring and verification tasks.
    *   To provide developers with powerful tools that can intelligently assist in maintaining and improving their Ferra codebases.
    *   To lay the groundwork for a suite of AI-native development experiences.

*   **Relationship to `AI_API_AST.md`**:
    *   This design builds upon the foundational concepts established in `AI_API_AST.md`, particularly the `ai::ast()` API for providing code context (AST, semantic information) to AI services.
    *   The `.note.ai` section and the principles of using semantic tags (see `IR_SEMANTIC_TAGS.md`) are also central, especially for embedding `X-AI-Provenance` information.

*   **Core Requirements (from `Steps.md` & `comprehensive_plan.md`)**:
    *   **Step 3.3.1**: Design `ai::refactor::<goal>()` for common refactoring goals. This involves defining a clear syntax for invocation (e.g., `ai::refactor::<"extract_function">(range = (42, 58))` as per `Steps.md`, Item 4) and specifying a set of initial, actionable refactoring goals.
    *   **Step 3.3.2**: Design `ai::verify()` for tasks such as re-type-checking, fuzz-testing, and coverage enforcement (e.g., `ai::verify(coverage = 0.8, fuzz = 1_000)` as per `Steps.md`, Item 4).
    *   **Step 3.3.3**: Specify the mechanisms for embedding and verifying `X-AI-Provenance` signatures for any code generated or modified by these AI APIs (Ed25519 signatures, attestations via Sigstore, as per `Steps.md`, Item 4).

*   **Goals for this Document**:
    *   Define the conceptual syntax, parameters, and semantics for the `ai::refactor` and `ai::verify` APIs and their initial sub-functions/goals.
    *   Outline the expected interaction model: inputs required by the APIs (e.g., code context, parameters), the operations they perform (conceptually), and the outputs or effects they produce (e.g., proposed code changes, verification reports).
    *   Specify how these APIs will interact with the Ferra compiler, particularly concerning access to AST/IR information (likely via `ai::ast()` or similar compiler services) and the use of semantic tags for provenance.
    *   Identify a practical initial set of common refactoring goals for `ai::refactor` for v0.1.
    *   Detail the intended capabilities and parameters for the initial `ai::verify` sub-functions.
    *   Elaborate on the structure, content, embedding, and verification process for `X-AI-Provenance` data.

This document focuses on the "functional" design of these APIs – what they do and how they are conceptually used – rather than the specific implementation details of the underlying AI models or the compiler internals, which will be covered in subsequent implementation phases.

## 2. `ai::refactor::<goal>()` API (Step 3.3.1)

The `ai::refactor` API provides a standardized way for developers and tools to request automated code transformations on Ferra source code, guided by an AI service.

### 2.1. Core Concept

The core concept is to offer a high-level, goal-oriented API where the user specifies *what* they want to achieve (the refactoring goal, e.g., "extract this code block into a new function") and provides the necessary context (e.g., the code range). The AI, with access to semantic information about the code (potentially via `ai::ast()` or similar compiler services), then attempts to perform the transformation safely and correctly.

This API is designed to be functional, meaning it describes an operation and its parameters. The actual execution might involve compiler plugins, IDE services, or CLI tools that interface with an AI backend.

### 2.2. Conceptual Syntax

The invocation of the `ai::refactor` API is envisioned to be ergonomic and expressive, fitting naturally within Ferra's syntax if called from code (e.g., a build script or a meta-programming context), or easily translatable from tool-specific commands (e.g., an IDE's refactoring menu).

*   **Primary Invocation Style (Generic Goal Parameter)**:
    This style uses a generic parameter or a typed enum to specify the refactoring goal.
    ```ferra
    // Using a generic type parameter (conceptual, depends on Ferra's generic capabilities)
    ai::refactor::<ExtractFunctionParameters>(
        source_file: "my_module.ferra", 
        range: (start_line: 42, start_col: 5, end_line: 50, end_col: 10), // Example range
        new_name: "calculate_sub_total"
    );

    // Using a named parameter for the goal type (perhaps more flexible)
    // `RefactorGoal` would be an enum or a set of specific data types.
    ai::refactor(
        goal: RefactorGoal::ExtractFunction { 
            source_file: "my_module.ferra", 
            range: selection_range, // Assume `selection_range` is a Range type
            new_name: "process_items",
            visibility: Visibility::Public // Optional, could default to private/local
        }
    );
    ```
    The `Steps.md` (Item 4) example `ai::refactor::<"extract_function">(range = (42, 58))` suggests a string literal for the goal type within a generic invocation, which is also a valid conceptual direction.

*   **Alternative: Dedicated Functions per Goal (Less Likely for Core API)**:
    While possible (e.g., `ai::refactor::extract_function(...)`, `ai::refactor::rename_symbol(...)`), a single `ai::refactor` entry point with a goal specifier is generally more extensible for adding new refactoring types without expanding the `ai::` module's top-level API surface excessively. However, helper functions wrapping the generic API might be provided for convenience.

*   **Key Components of the Call**:
    *   **`ai::refactor`**: The namespace and function indicating an AI-driven refactoring operation.
    *   **`GoalType` / `goal` parameter**: Clearly specifies the type of refactoring to be performed (e.g., `ExtractFunction`, `RenameSymbol`). This would likely map to an enum or a set of well-defined string constants or types.
    *   **Goal-Specific Parameters**: Each `GoalType` will have its own set of required and optional parameters. For example:
        *   `source_file: String` or `target_item: ItemId`: Specifies the context.
        *   `range: SourceRange` or `location: SourceLocation`: Specifies the exact code to be refactored.
        *   `new_name: String`: For renaming or extracting to a new named entity.
        *   `options: map[String]AnyValue`: For additional goal-specific options.

The precise syntax will depend on how these APIs are exposed (e.g., as compiler intrinsics, library functions callable from scripts, or abstract definitions for tool integration). For this design document, we focus on the conceptual structure and necessary information. All API calls will adhere to standard Ferra function call syntax as defined in **SYNTAX_GRAMMAR_V0.1.md** and will be represented by corresponding call expression nodes in the AST as per **AST_SPECIFICATION.md**.

*   **Conceptual EBNF for API Call Structure** (Illustrative):
    ```ebnf
    // Assumes ai::refactor is a function that takes a structured config object or named args
    // This EBNF is highly conceptual as the actual calls are standard Ferra function calls.
    RefactorCall ::= "ai::refactor" "(" RefactorConfig "|" NamedRefactorArgs ")" ";"?
    // RefactorConfig could be a struct literal, NamedRefactorArgs would be param: value, ...
    ```

### 2.3. Common Refactoring Goals (Initial Set for v0.1)

The v0.1 `ai::refactor` API will focus on a small set of high-value, common refactoring operations that are relatively well-defined and have a clear path for AI assistance.

*   **1. `extract_function`**:
    *   **Goal**: Extracts a specified block of code into a new function.
    *   **Conceptual Parameters**:
        *   `target`: `SourceRange` (e.g., file, start line/col, end line/col) or `NodeId` specifying the code to extract.
        *   `new_function_name: String`.
        *   `visibility: Option<Visibility>` (e.g., `private`, `public`, `module_level`; defaults to most restrictive, e.g., private to current scope).
        *   `options: Option<ExtractFunctionOptions>` (e.g., `extract_as_async: bool`).
    *   **AI Actions (Conceptual)**:
        1.  Analyze the selected code block and its surrounding context (data flow, variable usage, types via `ai::ast()` or compiler services).
        2.  Identify input parameters required by the new function (variables used in the block but defined outside).
        3.  Identify return values (variables defined or modified in the block and used outside after the block).
        4.  Determine the appropriate signature for the new function.
        5.  Generate the new function definition.
        6.  Replace the original code block with a call to the new function.
    *   **Example (from `Steps.md`)**: `ai::refactor::<"extract_function">(range = (42, 58))` implies a range-based selection.

*   **2. `rename_symbol`**:
    *   **Goal**: Renames a symbol (variable, function, type, field, etc.) consistently across its scope of use.
    *   **Conceptual Parameters**:
        *   `target`: `SourceLocation` (file, line, col of an instance of the symbol) or `SymbolIdentifier { name: String, scope_id: ScopeId }`.
        *   `new_name: String`.
    *   **AI Actions (Conceptual)**:
        1.  Identify the symbol definition at the target location or matching the identifier.
        2.  Perform scope analysis to find all valid references to this symbol.
        3.  Propose renaming all occurrences.
        4.  Handle potential name collisions in sub-scopes (e.g., by suggesting qualified names or warning).

*   **3. `inline_variable` / `inline_function` (Simplified for v0.1)**:
    *   **Goal**: Replaces uses of a variable with its initializer, or a function call with its body, where simple and safe.
    *   **Conceptual Parameters**:
        *   `target`: `SourceLocation` of the variable usage or function call to inline.
    *   **AI Actions (Conceptual - Simplified for v0.1)**:
        *   **Variable**: If the variable is immutable, initialized with a simple literal or constant expression, and used only once (or a few times nearby without intervening modification), replace its usage.
        *   **Function**: If the function is small, non-recursive, has no complex side effects, and inlining it wouldn't significantly bloat code or obscure logic, replace the call site with the function body (with appropriate parameter substitution).
        *   *v0.1 scope will be very conservative for these.*

*   **4. `add_doc_comment`**:
    *   **Goal**: Generates and inserts a basic documentation comment stub for a function, data class, or module.
    *   **Conceptual Parameters**:
        *   `target`: `SourceLocation` or `NodeId` of the item to document.
        *   `template: Option<String>` (e.g., a predefined style like "Summarize parameters and return value").
    *   **AI Actions (Conceptual)**:
        1.  Analyze the item's signature (e.g., function parameters, return type; data class fields).
        2.  Generate a placeholder doc comment (e.g., `/// [TODO: Briefly describe ${function_name}] \n/// \n/// Parameters:\n///   - `${param1_name}`: [TODO: Describe ${param1_name}]\n/// \n/// Returns: \n///   [TODO: Describe return value]`).
        3.  Insert the comment above the item.

*   **5. `convert_to_async` (Conceptual, Stretch Goal for v0.1)**:
    *   **Goal**: Attempts to convert a synchronous function to an `async fn`, potentially identifying blocking calls that need `await`.
    *   **Conceptual Parameters**:
        *   `target_function`: `SourceLocation` or `NodeId`.
    *   **AI Actions (Conceptual)**:
        1.  Analyze the function body for calls that have `async` counterparts or are known blocking operations.
        2.  Suggest adding `async` to the function signature and `await` to identified calls.
        3.  This is complex due to call chain implications and would be highly experimental.

This initial set balances utility with feasibility for AI-assisted implementation.

### 2.4. Interaction Model

Defines how the `ai::refactor` API interacts with the user, the compiler, and the underlying AI service.

*   **Context Provision**:
    *   The AI service will require context to perform refactorings. This context will primarily be sourced via:
        *   The `ai::ast()` API (see `AI_API_AST.md`) to get a JSON representation of the relevant AST and semantic tags.
        *   Specific parameters passed to the `ai::refactor` call (e.g., source range, new names).
        *   Potentially, direct compiler services providing more detailed semantic information (e.g., type information, symbol resolution) for the targeted code region.
*   **Change Proposal and Application**:
    *   **Proposal First**: For non-trivial refactorings, the AI should ideally propose changes rather than applying them directly.
    *   **Diff Format**: Proposed changes could be presented as a standard diff (e.g., unified diff format) against the original source file(s).
    *   **IDE Integration**: IDEs would consume this diff and present it to the user for review and approval (e.g., using built-in diff viewers or refactoring preview windows).
    *   **Direct AST Modification (Controlled)**: While direct AST modification by AI is powerful, it carries risks. If pursued, it would need to be within a tightly controlled compiler plugin architecture with strong validation and sandboxing, and likely still require user confirmation for the resulting source code changes. For v0.1, source-level diffs are a safer starting point for user-facing tools.
*   **User Confirmation**: A user confirmation step is crucial before applying AI-suggested refactorings, especially those that alter code logic or structure significantly.
*   **Atomicity**: Refactorings should aim to be atomic. If a multi-file refactoring (like renaming a public symbol) cannot be completed fully, it should ideally be rolled back or not applied.

### 2.5. Underlying AI Service/Model Requirements (High-Level)

The effectiveness of `ai::refactor` depends on the capabilities of the AI model/service it interfaces with.

*   **Code Understanding**: The AI must have strong capabilities in parsing and understanding the syntax and semantics of Ferra code (leveraging the AST and semantic info provided).
*   **Transformation Logic**: It needs to be trained or programmed with the logic for specific refactoring operations (e.g., understanding data flow for `extract_function`).
*   **Code Generation**: For some refactorings, it must be able to generate new, syntactically correct Ferra code.
*   **Safety and Correctness**: A high priority is ensuring that refactorings preserve the existing behavior of the program (unless the goal is explicitly to change behavior in a controlled way, which is less common for pure refactoring).
*   **Contextual Awareness**: Ability to understand not just the selected code but also its surrounding context.

The architecture for this AI service (local model, cloud API, hybrid) is an open question (TBD AI-SERVICE-1).

### 2.6. Error Handling and Fallbacks

The `ai::refactor` API must handle situations where a requested refactoring cannot be performed.

*   **Failure Reasons**:
    *   Ambiguous selection or parameters from the user.
    *   Code that is too complex for the AI to analyze confidently for the given goal.
    *   Refactoring would introduce errors or break semantic invariants.
    *   Preconditions for the refactoring are not met.
*   **Reporting**:
    *   The API should return a clear status (e.g., success, failure, needs_confirmation).
    *   In case of failure, provide a reason (e.g., "Selected code cannot be extracted due to multiple return paths not assignable to a single variable," "Renaming this symbol would cause a name collision").
    *   Diagnostics should align with **DESIGN_DIAGNOSTICS.md** principles (e.g., "positive-first" messaging, structured error data, use of defined error codes from `diagnostic_codes.md`).
*   **No Change as Fallback**: If a refactoring cannot be safely applied, the default fallback is to make no changes to the source code.

## 3. `ai::verify()` API (Step 3.3.2)

The `ai::verify` API is designed to provide developers with AI-assisted tools for enhancing code quality, correctness, and robustness. Unlike `ai::refactor` which transforms code, `ai::verify` focuses on analysis and reporting, potentially suggesting areas for improvement or flagging issues.

### 3.1. Core Concept

The `ai::verify` API offers a suite of checks and analyses that leverage AI to go beyond traditional static analysis or linting. It aims to perform deeper semantic understanding, generate test inputs, or assess code against higher-level quality attributes like test coverage or adherence to specified invariants. The results are typically reports or diagnostics rather than direct code modifications, though it might suggest patches or test cases.

### 3.2. Conceptual Syntax

Similar to `ai::refactor`, the `ai::verify` API could be invoked through a general function with specific checks requested as parameters, or through more specialized sub-functions.

*   **General Invocation Style**:
    ```ferra
    // Example from Steps.md (Item 4)
    // This implies ai::verify can take multiple checks or a configuration object.
    if !ai::verify(coverage = 0.8, fuzz = 1_000_000) { // fuzz likely implies a target and iterations/duration
        panic("AI patch failed hard gate"); 
    }

    // More explicit check list:
    // `VerificationCheck` would be an enum or a set of specific data types for each check.
    let verification_report = ai::verify(
        target_module: "my_module.ferra",
        checks: [
            VerificationCheck::Fuzz { target_function: "my_func", iterations: 10000 },
            VerificationCheck::Coverage { min_percentage: 80.0 }
        ]
    );
    ```

*   **Specialized Sub-Functions (Potentially for convenience)**:
    ```ferra
    let fuzz_report = ai::verify::fuzz(target: "my_func", iterations: 10000);
    let coverage_status = ai::verify::coverage(min_percentage: 80.0, scope: "current_module");
    let type_soundness_report = ai::verify::retype_check(item: "my_complex_function"); 
    ```
    For v0.1, a combination might be suitable: a main `ai::verify(config)` function that can dispatch to various checks, and perhaps a few highly common checks exposed directly (like the `Steps.md` example suggests for `coverage` and `fuzz`).

*   **Key Components of the Call**:
    *   **`ai::verify`**: The namespace and function indicating an AI-driven verification operation.
    *   **`CheckType` / `checks` parameter**: Specifies the type(s) of verification to perform.
    *   **Check-Specific Parameters**: Each verification check will have its own parameters (e.g., target function for fuzzing, coverage threshold).
    *   **`target_scope`**: Often, verification will apply to a specific file, module, function, or item.

_Note: All `ai::verify` API calls, whether general or specialized, will use standard Ferra function call syntax (see **SYNTAX_GRAMMAR_V0.1.md**) and be represented by standard call expression nodes in the AST (see **AST_SPECIFICATION.md**)._

*   **Conceptual EBNF for API Call Structure** (Illustrative):
    ```ebnf
    // Assumes ai::verify is a function that takes a structured config object or named args
    // This EBNF is highly conceptual as the actual calls are standard Ferra function calls.
    VerifyCall   ::= "ai::verify" "(" VerifyConfig "|" NamedVerifyArgs ")" ";"?
    // VerifyConfig could be a struct literal, NamedVerifyArgs would be param: value, ... 
    // The example `ai::verify(coverage = 0.8, fuzz = 1_000_000)` from Steps.md fits NamedVerifyArgs.
    ```

### 3.3. Verification Sub-Functions/Checks (Initial Set for v0.1)

The initial set of verification checks for v0.1 will focus on areas where AI can provide significant value beyond traditional tools.

*   **1. Re-Type-Checking (Conceptual - `ai::verify::retype_check()`)**:
    *   **Purpose**: To perform a potentially more exhaustive or AI-guided type soundness check on a piece of code, especially after complex refactorings (manual or AI-driven) or when dealing with highly generic or dynamically-influenced code sections. This is distinct from the standard compiler type check.
    *   **Interaction**:
        *   Could involve using AI to generate challenging type scenarios or to analyze type flow in ways the standard checker might not.
        *   Might also verify consistency with `#[ai.assume(type_is="...")]` tags.
    *   **Output**: A report indicating type soundness or highlighting potential subtle type issues or ambiguities not caught by the main type checker.
    *   *v0.1 Scope*: This is highly conceptual and might be limited to very specific scenarios or deferred if too complex initially.

*   **2. Fuzz Testing (Simplified - `ai::verify::fuzz(target: FunctionRefOrName, config: FuzzConfig)`)**:
    *   **Purpose**: Automatically generate a wide range of inputs to test a given function for unexpected panics, assertion failures, or other erroneous behavior.
    *   **Conceptual Parameters (`FuzzConfig`)**:
        *   `target: String` (e.g., fully qualified function name like `"my_module::my_public_func"`).
        *   `input_types: Option<List<TypeHintOrSchema>>` (Hints for AI on generating valid inputs; could infer from function signature).
        *   `constraints: Option<List<Constraint>>` (e.g., "param1 > 0", "string_param_length < 100").
        *   `duration_ms: Option<Int>` or `iterations: Option<Int>` (e.g., `fuzz = 1_000_000` from `Steps.md` implies iterations or a similar metric).
        *   `seed: Option<Int>` for reproducibility.
    *   **AI Actions (Conceptual)**:
        1.  Analyze the target function's signature and potentially its body (via `ai::ast()`).
        2.  Generate a diverse set of valid (and near-valid/invalid edge case) inputs based on types and constraints.
        3.  Execute the function with these inputs in a controlled environment.
        4.  Monitor for panics, assertion failures, or other specified error conditions.
    *   **Output**: A report listing any failing inputs, the nature of the failure, and a summary of the fuzzing session.

*   **3. Coverage Analysis & Suggestion (Conceptual - `ai::verify::coverage(target_percentage: Float, scope: SourceScope)`)**:
    *   **Purpose**: To analyze existing test coverage (e.g., line, branch coverage from an external tool run) and, if below a `target_percentage`, potentially use AI to suggest areas needing more tests or even draft simple missing test cases.
    *   **Interaction**:
        *   Requires integration with a standard coverage data format (e.g., lcov, Cobertura). The API might take a path to a coverage report.
        *   The `ai::verify(coverage = 0.8)` from `Steps.md` implies checking against a coverage threshold.
    *   **AI Actions (Conceptual)**:
        1.  Parse the coverage report for the given `scope`.
        2.  Identify uncovered or poorly covered code regions.
        3.  For these regions, analyze the code (via `ai::ast()`) to understand conditions or paths not exercised.
        4.  Suggest specific scenarios or inputs that would improve coverage.
        5.  (Advanced) Attempt to generate skeleton test cases for these scenarios.
    *   **Output**: A report on current coverage vs. target, a list of under-tested regions, and AI-generated suggestions or test skeletons.

*   **4. Invariant Consistency Check (Interaction with `#[ai.verify(...)]` attributes - Future)**:
    *   **Purpose**: If code contains `#[ai.verify(invariant="...")]` attributes (see `AI_API_AST.md` and `IR_SEMANTIC_TAGS.md`), this check would attempt to:
        *   Statically analyze if the invariant is likely to hold.
        *   Suggest inputs for fuzzing that might violate the invariant.
    *   *This is more of a future integration point, making the `#[ai.verify(...)]` attributes actionable by the `ai::verify()` API.*

### 3.4. Interaction Model

*   **Context Provision**: Similar to `ai::refactor`, `ai::verify` functions will need code context.
    *   `ai::ast()` or similar compiler services for semantic information.
    *   Specific parameters defining the target and scope of verification.
    *   For checks like coverage, it might consume output from other tools (e.g., coverage reports).
*   **Output and Reporting**:
    *   Verification results will typically be reports (e.g., JSON, structured text).
    *   These reports should detail:
        *   Checks performed.
        *   Success/failure status for each check.
        *   Specific findings (e.g., failing inputs for fuzzing, uncovered lines for coverage, potential type issues).
        *   Suggestions for remediation, if applicable.
    *   Output formatting and diagnostic messages should adhere to the principles and, where applicable, the JSON schema outlined in **DESIGN_DIAGNOSTICS.md**.

### 3.5. Integration with Testing Frameworks and CI

*   **CI Gate**: The `Steps.md` example `if !ai::verify(...) { panic(...) }` clearly positions `ai::verify` as a potential CI gate.
*   **Test Runner**: The Ferra test runner could have options to invoke specific `ai::verify` checks as part of a test run or a separate verification phase.
*   **Scheduled Jobs**: More intensive verification tasks (deep fuzzing, comprehensive coverage analysis) might be run as scheduled CI jobs.
*   **Conceptual CI Workflow Snippet (e.g., GitHub Actions)**:
    ```yaml
    # .github/workflows/ci.yml (excerpt)
    # ...
    # - name: Build Project
    #   run: ferra build --release

    - name: AI Code Verification
      run: |
        # Example: Run ai::verify for fuzzing and coverage as a CI gate
        # The actual command might be `ferra verify --config verify_config.ferra`
        # where verify_config.ferra calls ai::verify(...)
        # For this example, assume a direct CLI command for illustration:
        lang verify --target-module=src/core --fuzz-iterations=1000000 --coverage-min=0.80
        if [ $? -ne 0 ]; then
          echo "AI Verification failed!"
          # Potentially upload report artifact here
          exit 1
        fi
      continue-on-error: false # Fail the build if verification fails
    ```

### 3.6. Underlying AI Service/Model Requirements

*   **Code Analysis**: Deep semantic understanding of Ferra code.
*   **Test Case Generation**: Ability to generate valid and edge-case inputs based on type information and constraints (for fuzzing).
*   **Logical Reasoning**: For invariant checking or complex type analysis.
*   **Pattern Recognition**: For identifying areas needing better test coverage.
*   **Natural Language Output**: For generating readable reports and suggestions.

The specific AI models and techniques will vary depending on the verification check being implemented.

## 4. `X-AI-Provenance` Signatures (Step 3.3.3)

A critical aspect of integrating AI-driven code modifications or verifications is ensuring transparency, traceability, and trust. The `X-AI-Provenance` signature mechanism, as mandated by `Steps.md` (Item 4: "Every AI tool call attaches `X-AI-Provenance` (Ed25519, attestations baked by Sigstore)"), addresses this by providing a standardized way to record and cryptographically verify the origin and nature of AI-assisted code changes.

### 4.1. Purpose

The primary purposes of `X-AI-Provenance` are:

*   **Traceability**: To clearly identify which code segments were generated or modified by an AI tool, which specific tool/model was used, and under what circumstances (e.g., input prompts, parameters).
*   **Accountability**: To provide a basis for understanding the source of AI-generated code, which can be important for debugging, security reviews, and intellectual property considerations.
*   **Trust and Verification**: To allow developers and tools to verify that AI-generated code has not been tampered with since its generation/modification by a trusted AI service, and to understand the "chain of custody."
*   **Reproducibility (Conceptual)**: While full reproducibility of AI outputs can be challenging, storing input parameters can aid in attempting to reproduce or understand the generation process.
*   **Informing Subsequent AI Tools**: `ai::verify()` or other analysis tools might use provenance information to treat AI-generated code differently or to verify its specific properties.

### 4.2. Information to Embed

The provenance data associated with an AI tool call should be comprehensive enough to satisfy the purposes above. Based on `Steps.md` and general best practices, the following information should be considered for embedding:

*   **`ai_tool_name`: String**: Name of the AI tool or service (e.g., `"FerraRefactor::ExtractFunction"`, `"FerraVerify::FuzzInputGenerator"`).
*   **`ai_tool_version`: String**: Version of the AI tool or service.
*   **`ai_model_details`: Map (String -> String/Value)**: Information about the underlying AI model, if applicable (e.g., `model_name`, `model_version`, `training_data_summary_hash`). This needs careful consideration regarding proprietary model information.
*   **`timestamp`: DateTime (ISO 8601)**: Timestamp of when the AI operation was performed/completed.
*   **`invocation_id`: UUID/String**: A unique identifier for this specific AI tool invocation.
*   **`user_context`: Map (String -> String/Value)** (Optional):
    *   `user_id`: Identifier of the user invoking the tool (if applicable and privacy-preserving).
    *   `session_id`: Identifier for the development session.
*   **`input_parameters`: Map (String -> Value)**: Key-value map of all significant input parameters and configurations provided to the AI tool for this operation (e.g., for `ai::refactor::extract_function`, this would include `source_range`, `new_function_name`).
*   **`source_code_context`**:
    *   `original_code_hash`: Cryptographic hash (e.g., SHA-256) of the original code segment *before* modification by the AI (if applicable, e.g., for refactoring).
    *   `original_code_span`: Precise location (file, line/col range) of the original code segment.
*   **`output_code_context`**:
    *   `modified_code_hash`: Cryptographic hash of the AI-generated or AI-modified code segment (if applicable).
    *   `modified_code_span`: Precise location of the AI-generated/modified code.
    *   `diff_hash` (Optional): Hash of a diff representing the changes.
*   **`attestation_uri`: Option<String>`**: A URI pointing to external attestations or more detailed provenance records, potentially managed by a service like Sigstore's Rekor.
*   **`custom_metadata`: Option<Map (String -> Value)>**: For any other tool-specific or extensible metadata.

The exact schema for this data needs to be standardized, likely as a CBOR map to align with `IR_SEMANTIC_TAGS.md`. (TBD AI-PROV-FORMAT-1).

### 4.3. Signature Mechanism

To ensure integrity and authenticity, the collected provenance data must be cryptographically signed.

*   **Algorithm**: Ed25519, as specified in `Steps.md`. This is a modern, fast, and secure elliptic curve signature algorithm.
*   **Signing Entity**:
    *   If the AI tool runs locally (e.g., as a compiler plugin or IDE extension with a local model), it might sign with a locally managed key.
    *   If the AI tool calls a remote AI service, that service would sign the provenance data with its key.
*   **Key Management**: The management, distribution, and verification of public keys used for provenance signatures are critical (TBD AI-PROV-KEY-1). This might involve:
    *   Published public keys for official Ferra AI tools/services.
    *   A trust model for third-party AI tools providing Ferra support.
*   **Integration with Sigstore (Attestations)**:
    *   `Steps.md` mentions "attestations baked by Sigstore." This implies that after signing the provenance data, a record of this signature and the data itself (or a hash of it) could be submitted to a Sigstore transparency log (like Rekor).
    *   The `attestation_uri` field in the provenance data could then point to this Rekor entry.
    *   This provides a publicly verifiable, immutable record of the AI operation, enhancing trust and auditability.

### 4.4. Embedding Location

Provenance information (the data and its signature) needs to be associated with the code it pertains to. Several locations are possible:

1.  **Source Code Comments**:
    *   For direct code modifications (e.g., by `ai::refactor`), the provenance data (or a summary/hash and a link to full data) and its signature can be embedded as a structured comment immediately preceding or following the modified code block.
    *   **Pros**: Directly visible with the code; travels with the source file.
    *   **Cons**: Can clutter source; might be accidentally modified or removed by developers.
    *   **Format**: A standardized comment format (e.g., `// X-AI-PROVENANCE:BASE64_CBOR_SIGNED_DATA`) would be needed.

2.  **Semantic Tags in `.note.ai` / `ferra.ai` Section (`IR_SEMANTIC_TAGS.md`)**:
    *   The CBOR-encoded signed provenance data can be stored as a specific semantic tag associated with the relevant AST/IR nodes that were generated or modified.
    *   **Pros**: Persists with the compiled artifact; robust against source formatting changes; structured and machine-readable.
    *   **Cons**: Not directly visible in the source code without tooling.
    *   **Action**: This would likely require defining a new numeric key in `IR_SEMANTIC_TAGS.md` specifically for `X-AI-Provenance` data.

3.  **Commit Metadata**:
    *   For refactorings that span multiple files or are significant, a summary of the AI operation and a hash of the full provenance data could be included in the Git commit message.
    *   The full signed provenance data could be stored as a Git note or a separate artifact linked from the commit.
    *   **Pros**: Good for tracking larger changesets.
    *   **Cons**: Not as granular as source comments or IR tags.

4.  **IDE/Tool-Specific Storage**:
    *   IDEs might maintain their own databases linking AI operations to code locations and provenance data.

A combination of these might be used (e.g., an IR tag for compiled artifacts, and optional source comments for immediate visibility if the AI tool directly modifies source). The primary focus for compiler-integrated AI tools would likely be semantic tags.

### 4.5. Verification of Provenance

Developers and tools need to be able to verify the `X-AI-Provenance` signatures.

*   **Tooling (`lang verify-provenance`)**:
    *   A command-line tool, e.g., `lang verify-provenance <file_or_artifact_path>`, could be provided.
    *   This tool would:
        1.  Extract embedded provenance data and signatures (from comments or `.note.ai` sections).
        2.  Verify the signature against the known public key of the signing AI tool/service.
        3.  Optionally, if an `attestation_uri` is present, query the Sigstore log to verify the attestation's inclusion.
        4.  Display the verified provenance information to the user.
        5.  Report any signature failures or tampering.
*   **IDE Integration**:
    *   IDEs could perform this verification automatically and visually indicate the provenance status of AI-generated/modified code (e.g., a special marker or gutter icon).
*   **`ai::verify()` API Interaction**:
    *   The `ai::verify()` API itself could have an option or a specific check to analyze the provenance of the code it is verifying. For example, it might apply different rules or levels of scrutiny based on whether code has trusted AI provenance, unknown provenance, or human-authored provenance.
    *   It could also re-verify the integrity of embedded provenance data as part of its checks.

Robust provenance tracking and verification are essential for building confidence in AI-assisted software development and for integrating AI tools responsibly into the Ferra ecosystem.

## 5. General API Design Considerations

Beyond the specifics of `ai::refactor` and `ai::verify`, several general design considerations apply to these AI-driven APIs to ensure they are robust, usable, and integrate well into the Ferra ecosystem.

*   **Synchronous vs. Asynchronous Operations**:
    *   **Default Expectation**: AI operations, especially those involving complex analysis or calls to potentially remote AI models, can be time-consuming. Therefore, these APIs should ideally be designed with an asynchronous interaction pattern in mind.
    *   **Invocation**:
        *   Calling `ai::refactor(...)` or `ai::verify(...)` might return immediately with a handle or a `Future`/`Promise`-like object.
        *   The developer can then `await` this handle for the results (e.g., proposed diff, verification report).
    *   **Blocking Mode (Optional)**: For simpler scripting or CLI tools, a synchronous (blocking) mode might be offered as a convenience wrapper, but the underlying mechanism should support asynchronicity to avoid freezing user interfaces (like IDEs) or build processes.
    *   **Callback Mechanism**: For very long-running tasks, a callback or event-based notification system could be considered for providing progress updates or final results.

*   **User Experience (UX)**: How are these APIs invoked by developers (CLI, IDE plugin, direct code annotations)? (TBD AI-API-UX-1)
    *   IDE Integration (Primary UX): The most common way users will interact with `ai::refactor` is likely through IDE context menus (e.g., right-click -> "Refactor with AI -> Extract Function") or quick fixes. `ai::verify` results would appear in problem panes or as inline diagnostics. IDEs would translate these UI actions into API calls.
    *   CLI Tools: `lang refactor <goal> ...` and `lang verify <check> ...` commands could provide command-line access for scripting or users who prefer the terminal. These CLIs would wrap the core API calls.
        *   Example CLI Invocations (Conceptual):
            ```sh
            # Refactoring Examples
            lang refactor extract_function --file src/utils.ferra --range 42:5-50:10 --new-name process_data
            lang refactor rename_symbol --file src/models.ferra --location 15:8 --new-name UserModelV2

            # Verification Examples
            lang verify fuzz --target-function "my_app::utils::parse_input" --iterations 1000000
            lang verify coverage --min-percentage 0.85 --scope ./src
            lang verify retype_check --file src/complex_logic.ferra
            ```
    *   Direct Code Annotations/Calls ...
    *   Consistency: ...
    *   _Permissioning Note_: The invocation of these powerful AI APIs might in the future be subject to specific capability-based permissions (e.g., `ai:invoke_refactor`, `ai:invoke_verify`, `ai:access_model:<model>`) declared in `manifest.perms`, as detailed in the overall Ferra **SECURITY_MODEL.md** (Section 5.2).

*   **Security and Trust**:
    *   **Code Modification**: `ai::refactor` involves code modification, which carries inherent security risks if the AI service is compromised or produces malicious/buggy code.
        *   **User Confirmation**: Mandatory user review and confirmation for all but the most trivial, demonstrably safe refactorings.
        *   **Sandboxing (for AI execution)**: If AI models are executed locally, they might need to run in a sandboxed environment with limited permissions.
        *   **Provenance**: `X-AI-Provenance` is key to tracing changes back to the AI tool.
    *   **Data Privacy**: When code context is sent to an AI service (especially a remote one), data privacy implications must be considered. Users need to be aware of what data is being sent. On-premise or local AI model options could mitigate this.
    *   **Trusted AI Services**: Ferra might need a mechanism to define or recognize trusted AI providers/models for these features.

*   **Resource Consumption**:
    *   AI models can be computationally and memory intensive.
    *   **Local vs. Remote Execution**: The choice of local AI models versus cloud-based APIs (TBD AI-SERVICE-1) will significantly impact resource usage on the developer's machine and cost.
    *   **Cancellation/Timeout**: APIs should support cancellation for long-running operations and have configurable timeouts.
    *   **Feedback**: Provide feedback to the user if an operation is taking a long time.

*   **Modularity and Extensibility**:
    *   The `ai::refactor` and `ai::verify` systems should be designed to be extensible.
    *   Adding new refactoring `GoalType`s or new `VerificationCheck`s should be possible without major changes to the core API structure. This might involve a plugin architecture or a registration mechanism for new AI capabilities.

*   **Configuration and Customization**:
    *   Users or projects might need to configure aspects of the AI interaction, such_as:
        *   Preferred AI model/provider (if choices are available).
        *   Default behavior for user confirmation.
        *   Specific rules or styles for AI-generated code (e.g., formatting preferences).

### 6. Relationship to Other AI Features

The `ai::refactor` and `ai::verify` APIs are part of a broader suite of AI-native features in Ferra. Their relationship to other planned or existing AI features is important:

*   **`ai::ast()` (from `AI_API_AST.md`)**:
    *   This is a foundational API. `ai::refactor` and `ai::verify` will likely rely on `ai::ast()` (or an equivalent internal compiler service providing similar data) to obtain the necessary Abstract Syntax Tree and semantic information about the code they are operating on.
    *   The JSON AST representation defined in `AI_API_AST.md` provides a standardized format for code context that AI models can consume.

*   **`ai::explain(err)` (from `FRONTEND_ENHANCEMENTS.md`)**:
    *   **Different Focus**: `ai::explain(err)` is focused on providing natural language explanations for compiler diagnostics (errors and warnings).
    *   `ai::refactor` *changes* code, and `ai::verify` *analyzes* code for broader quality attributes beyond simple diagnostics.
    *   **Synergy**: An error explained by `ai::explain(err)` might lead a developer to invoke `ai::refactor` to apply a suggested fix. `ai::verify` might produce diagnostics that could then be piped into `ai::explain(err)`.

*   **Semantic Tags (`IR_SEMANTIC_TAGS.md`)**:
    *   **Input for AI**: Tags like `#[ai.assume(...)]` or `#[ai.verify(invariant="...")]` (defined in `AI_API_AST.md` and stored via `IR_SEMANTIC_TAGS.md`) can provide explicit hints or conditions from the developer that `ai::verify` or even `ai::refactor` could use as input.
    *   **Output/Storage for Provenance**: As discussed in Section 4, semantic tags are a key mechanism for embedding `X-AI-Provenance` information into compiled artifacts.
    *   **Communication Channel**: Tags can act as a persistent communication channel between different AI tools or between the developer and AI tools operating at different stages of compilation or analysis.

*   **Future AI APIs**:
    *   Other AI APIs might be developed in the future (e.g., `ai::generate_tests()`, `ai::document_code()`, `ai::optimize_perf()`). The design of `ai::refactor` and `ai::verify` should consider common patterns (like context provision, result reporting, provenance) that could be reused for such future APIs to maintain consistency within the `ai::` namespace.

A cohesive strategy across all AI APIs will ensure they work well together and provide a consistent developer experience.

## 7. Open Questions / TBD

This section consolidates the "To Be Determined" items identified throughout this document. These require further design, prototyping, and community feedback during the implementation of Module 3.3.

*   **(AI-REF-GOAL-1)**: **Final Curated List of Refactoring Goals**:
    *   The definitive, prioritized list of common refactoring goals to be supported by `ai::refactor` in its v0.1 implementation (from the candidates in Section 2.3).
*   **(AI-REF-APPLY-1)**: **Precise Change Application Mechanism**:
    *   The exact mechanism for how `ai::refactor` proposes and applies changes (e.g., standardized diff format, specific IDE Language Server Protocol extensions, direct AST modification protocol if deemed safe and necessary).
*   **(AI-VER-RETYPE-1)**: **`ai::verify::retype_check()` Utility & Strategy**:
    *   The specific utility, value proposition, and implementation strategy for an AI-assisted `retype_check()` that goes beyond the compiler's standard type checking. What concrete problems does it solve?
*   **(AI-VER-FUZZ-1)**: **`FuzzConfig` Details & AI Fuzzing Techniques**:
    *   Detailed specification for the `FuzzConfig` parameters for `ai::verify::fuzz()`.
    *   High-level approach or requirements for the AI techniques used in input generation and fuzz campaign management.
*   **(AI-VER-COV-1)**: **`ai::verify::coverage()` Integration Strategy**:
    *   How `ai::verify::coverage()` will integrate with existing external coverage tools and data formats (e.g., reading lcov files, interacting with coverage runners).
    *   The nature of AI suggestions for improving coverage (e.g., identifying uncovered paths, generating test case stubs).
*   **(AI-PROV-FORMAT-1)**: **`X-AI-Provenance` Data Schema**:
    *   The exact schema (e.g., specific CBOR map keys and value types) for the `X-AI-Provenance` data to ensure consistent structure.
*   **(AI-PROV-KEY-1)**: **Provenance Signature Key Management**:
    *   Strategy for managing and distributing public keys used for verifying `X-AI-Provenance` signatures, including trust model considerations for first-party and third-party AI tools.
    *   Details of Sigstore integration for attestations.
*   **(AI-API-UX-1)**: **Primary Invocation Method & UX (v0.1)**:
    *   The primary intended method(s) for developers to invoke `ai::refactor` and `ai::verify` in v0.1 (e.g., focus on IDE integration first, or provide robust CLI wrappers).
*   **(AI-SERVICE-1)**: **Underlying AI Service Architecture**:
    *   The high-level architecture for the AI service(s) that will power these APIs: Will it rely on local models, cloud-based APIs, or a hybrid approach? What are the implications for performance, cost, and data privacy?
*   **(AI-TAG-PROV-1)**: **Provenance Tag in `IR_SEMANTIC_TAGS.md`**:
    *   Define a specific numeric key and detailed structure within `IR_SEMANTIC_TAGS.md` for storing `X-AI-Provenance` data in compiled artifacts.

Addressing these TBDs through further design, prototyping, and targeted RFCs where appropriate will be crucial for a successful and robust v0.1 implementation of these AI APIs.

---
## Appendix A: Conceptual Core Data Structures for AI APIs

This appendix outlines conceptual data structures for common inputs and outputs of the `ai::refactor` and `ai::verify` APIs. The exact Ferra type definitions will be refined during implementation and depend on standard library types for ranges, paths, etc.

### A.1. `SourceLocation` and `SourceRange`

These are fundamental for specifying code to operate on.

```ferra
// Conceptual data Location
data SourceLocation {
    file_path: String,
    line: Int,      // 1-indexed
    column: Int,    // 1-indexed, UTF-8 character offset on the line
}

data SourceRange {
    file_path: String,
    start_loc: SourceLocation,
    end_loc: SourceLocation, // Typically exclusive of the end character
}
```

### A.2. `CodeChange` and `CodeChangeSet` (for `ai::refactor`)

Represents proposed modifications to source code.

```ferra
enum ChangeType { Insert, Delete, Replace }

data CodeChange {
    file_path: String,
    range: SourceRange, // For Delete/Replace, the range to be modified
                       // For Insert, range.start_loc is the insertion point (range.end_loc might be same)
    change_type: ChangeType,
    new_text: Option<String>, // Content for Insert/Replace
}

// A collection of changes, possibly across multiple files, for a single refactoring operation.
data CodeChangeSet {
    changes: Vector<CodeChange>,
    summary_message: Option<String>, // e.g., "Extracted function 'new_func' from main.ferra"
}
```

### A.3. `VerificationCheckConfig` (Union type for `ai::verify` input)

Represents the configuration for a specific verification check.

```ferra
// Example configurations for checks mentioned in Section 3.3
data FuzzTargetConfig {
    target_function_name: String, // Fully qualified if necessary
    iterations: Option<Int>,
    duration_ms: Option<Int>,
    // ... other fuzzing parameters like input constraints, seed ...
}

data CoverageCheckConfig {
    min_target_percentage: Float, // e.g., 0.80 for 80%
    scope_path: Option<String>,    // File or directory to check coverage for
    coverage_report_path: Option<String>, // Path to an existing coverage report file
}

data RetypeCheckConfig {
    target_item_path: String, // e.g., function or module path
    // ... other re-type-checking specific options ...
}

// Union type for all possible verification check configurations
enum VerificationCheckConfig {
    Fuzz(FuzzTargetConfig),
    Coverage(CoverageCheckConfig),
    Retype(RetypeCheckConfig),
    // ... other check types ...
}
```

### A.4. `VerificationReport` and `VerificationFinding` (for `ai::verify` output)

Represents the outcome of verification checks.

```ferra
enum FindingSeverity { Error, Warning, Info, Suggestion }

data VerificationFinding {
    check_type: String, // e.g., "Fuzz", "Coverage", "RetypeCheck"
    severity: FindingSeverity,
    message: String,
    location: Option<SourceLocation>, // If the finding pertains to a specific code location
    details: Option<map[String]AnyValue>, // e.g., failing input for fuzz, uncovered lines for coverage
    // Potentially a suggested `CodeChangeSet` for auto-fixable findings
    suggested_fix: Option<CodeChangeSet> 
}

data VerificationReport {
    summary_status: String, // e.g., "All checks passed", "Verification failed: 2 issues found"
    findings: Vector<VerificationFinding>,
    // Overall metrics, e.g.:
    coverage_achieved: Option<Float>,
    fuzz_inputs_run: Option<Int>,
}
```

_Note: These are conceptual. `AnyValue` would map to Ferra's dynamic type or an enum of supported literal types for structured data. Actual fields and types will be finalized during the detailed API and implementation design._

---
This document will guide the design of the functional `ai::refactor` and `ai::verify` APIs in Ferra.