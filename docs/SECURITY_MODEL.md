# Ferra Security Model v0.1

> **Status:** Initial Draft - Module 3.4 (Steps 3.4.1 - 3.4.2)

## 1. Introduction and Goals

This document specifies the foundational security model for the Ferra programming language and its ecosystem. The primary objective is to provide mechanisms that enhance the safety, trustworthiness, and robustness of Ferra applications by controlling their capabilities and sandboxing their execution environments. This model aims to give developers and users fine-grained control over what a program can do, thereby minimizing potential harm from bugs, vulnerabilities, or malicious dependencies.

The design is guided by the principle of "compile-time deny by default" for capabilities, meaning a program only has the permissions it explicitly requests and is granted. This is complemented by runtime sandboxing mechanisms to enforce these restrictions.

*   **Purpose**:
    *   To define a comprehensive security architecture for Ferra applications.
    *   To specify how programs declare their required permissions (capabilities).
    *   To detail the mechanisms for enforcing these permissions, both at compile-time (where possible) and runtime.
    *   To outline how Ferra will leverage sandboxing technologies (like WebAssembly/WASI and OS-level features such as seccomp-bpf) to isolate programs and limit their potential impact on the host system.
    *   To ensure security considerations are integrated throughout the Ferra ecosystem, from the language and standard library to the package manager and tooling.

*   **Core Requirements (from `Steps.md` & `comprehensive_plan.md`)**:
    *   **Step 3.4.1: Design capability-based permissions (`manifest.perms`)**: This involves defining a system where programs declare necessary permissions (e.g., network access, file system I/O) in their manifest file. `Steps.md` (Item 6) provides an example: `manifest.perms = ["net:fetch", "fs:read:~/downloads"]`.
    *   **Step 3.4.2: Design sandboxing mechanisms (Wasm+WASI, seccomp-bpf)**: This requires specifying how Ferra will utilize:
        *   The inherent sandboxing of WebAssembly and the capability model of WASI for programs compiled to WASM.
        *   OS-level sandboxing mechanisms like seccomp-bpf (for Linux ELF binaries) to restrict syscalls. `Steps.md` (Item 6) notes a performance target of "< 5 µs switch" for sandboxing, indicating efficiency is a concern.
    *   **Principle: Compile-time deny by default**: Capabilities should not be implicitly available; they must be explicitly requested.

*   **Goals for this Document**:
    1.  **Specify the Capability-Based Permission System**: Detail the syntax for permission strings, their integration into the package manifest (`Ferra.toml`), the scope and granularity of permissions, and the mechanisms for their compile-time and runtime enforcement.
    2.  **Detail Sandboxing Mechanisms**: Describe the design and integration of Wasm/WASI sandboxing and OS-level sandboxing (e.g., seccomp-bpf), including how they interact with the permission system.
    3.  **Discuss Interactions**: Analyze how the security model interacts with other Ferra features such as FFI, the concurrency model, and AI APIs.
    4.  **Outline Developer Experience**: Consider how developers will declare, manage, and debug security-related aspects of their applications.
    5.  **Identify Limitations and Future Work**: Acknowledge the scope of the v0.1 security model and outline potential areas for future enhancement.

This document aims to establish a robust yet practical security model for Ferra v0.1, prioritizing safety and user control while enabling the development of powerful and diverse applications.

## 2. Guiding Principles for Ferra Security

The design of Ferra's security model is guided by several well-established principles to create a robust and trustworthy platform:

*   **Principle of Least Privilege (PoLP)**:
    *   By default, Ferra programs and their components (e.g., packages, actors) should only be granted the minimum permissions necessary to perform their intended functions.
    *   This minimizes the potential damage if a component is compromised or contains a vulnerability.
    *   The `manifest.perms` system is a direct implementation of this principle at the package level.

*   **Deny by Default**:
    *   Access to sensitive resources or capabilities (e.g., filesystem, network, FFI) is denied unless explicitly permitted.
    *   This is a cornerstone of the `manifest.perms` design and contrasts with models where programs have broad access by default.

*   **Clear and Understandable Permissions**:
    *   The permission system should be designed so that developers can easily understand what capabilities their program or its dependencies are requesting.
    *   Permission strings should be granular yet human-readable.
    *   Tooling should assist in auditing and visualizing the permission footprint of an application.

*   **Defense in Depth**:
    *   Security should not rely on a single mechanism. Ferra aims for multiple layers:
        1.  **Language Safety**: Ferra's core memory safety (ownership/borrowing) and type safety prevent many common vulnerabilities.
        2.  **Capability-Based Permissions**: Compile-time and runtime checks based on declared `manifest.perms`.
        3.  **Sandboxing**: OS-level or Wasm/WASI runtime enforcement to further restrict program behavior.
    *   These layers work together to provide comprehensive protection.

*   **Tooling Support for Security**:
    *   The Ferra compiler and associated tooling (e.g., `lang` CLI, IDE plugins) will play an active role in security by:
        *   Enforcing permission declarations.
        *   Providing clear diagnostics for security-related issues.
        *   Assisting developers in understanding and managing their application's security posture (e.g., via `lang doctor security` or similar).

*   **Minimizing Attack Surface**:
    *   The standard library and runtime will be designed with a minimal trusted computing base (TCB).
    *   Features that grant broad access will require explicit permissions.

*   **Secure Defaults**:
    *   Where choices exist, Ferra will favor secure defaults (e.g., FFI calls are inherently `unsafe` and might require specific permissions).

*   **Transparency**:
    *   Mechanisms like `X-AI-Provenance` for AI-generated code and clear permission declarations contribute to transparency about code origins and capabilities.

These principles will inform the specific design choices detailed in the subsequent sections of this document.

## 3. Capability-Based Permissions (`manifest.perms`) (Step 3.4.1)

This section details the design of Ferra's capability-based permission system, a cornerstone of its security model. This system allows developers to explicitly declare the sensitive operations their package requires, enabling finer-grained control and better security auditing.

### 3.1. Overview and Rationale

Capability-based security, in this context, means that a program or package must explicitly declare the permissions it needs to interact with its environment (e.g., network, filesystem, FFI). Without such a declaration, access is denied by default.

*   **Why Capability-Based Permissions?**
    *   **Enhanced Security**: Limits the potential impact of bugs or malicious code by restricting what a program can do to only what it legitimately needs.
    *   **Improved Auditing**: Makes it easier for developers, users, and security auditors to understand what resources and operations a package will attempt to access.
    *   **Principle of Least Privilege**: Directly implements PoLP by forcing developers to think about and declare minimal necessary permissions.
    *   **Supply Chain Security**: Helps in assessing the risk profile of dependencies by inspecting their requested permissions.

*   **Goals**:
    *   Control access to sensitive operations, including but not limited to:
        *   Network connections (listening, connecting to specific hosts/ports).
        *   Filesystem access (reading, writing, creating, deleting specific paths or patterns).
        *   Foreign Function Interface (FFI) calls (loading specific libraries, calling specific symbols).
        *   Environment variable access (reading, writing specific variables).
        *   Subprocess/command execution.
        *   Access to privileged system information or hardware.

### 3.2. Manifest File Integration (`Ferra.toml`)

Permissions are declared in the project's manifest file, `Ferra.toml` (as defined in **PACKAGE_MANAGER_SPEC.md**, particularly Section 5.2 which details the manifest structure). This provides a centralized, human-readable, and machine-parseable location for permission declarations.

*   **Syntax in `Ferra.toml`**:
    *   A dedicated section, e.g., `[package.permissions]`, will house the permission declarations.
    *   Permissions will be declared as a list of strings.
    *   Example (conceptual, extending the example from `Steps.md`):
        ```toml
        [package]
        name = "my_app"
        version = "0.1.0"
        # ... other metadata ...

        [package.permissions]
        # Network permissions
        net_fetch = ["https://api.example.com/data", "https://cdn.example.com"] # Allow fetching from specific URLs/domains
        net_connect = ["localhost:8080", "database.internal:5432"]          # Allow connecting to specific host:port
        net_listen = ["0.0.0.0:3000"]                                     # Allow listening on a specific address:port

        # Filesystem permissions
        fs_read = ["./config.json", "/data/input/*.csv", "~/documents/report.txt"] # Read specific files or patterns
        fs_write = ["./output/", "/tmp/my_app_cache/"]                             # Write to specific directories or patterns
        # fs_access = ["~/special_file:rw"] # Alternative combined read/write

        # Environment variable permissions
        env_read = ["USER_CONFIG_PATH", "API_KEY_VAULT_PATH"]
        env_write = ["MY_APP_TEMP_DIR"] # Less common, use with caution
        env_all = false # Default, true would allow access to all env vars (discouraged)

        # Command execution permissions
        cmd_exec = ["/usr/bin/git status", "/usr/local/bin/my_helper --input {}"] # Allow specific commands, potentially with arg patterns

        # FFI permissions
        ffi_load = ["libssl.so.*", "./local_libs/libcustom.dylib"] # Allow loading specific shared libraries
        # ffi_call = ["my_c_lib::do_dangerous_thing"] # Granular FFI symbol permission (more advanced)
        
        # Special permissions
        # high_res_time = true # Access to high-resolution timers (if deemed sensitive)
        # introspection_unsafe = true # Access to unsafe reflection or runtime modification APIs (if any)
        ```
    *   The exact structure (e.g., simple list of strings vs. sub-tables for categories) is TBD (SEC-PERM-SYNTAX-1). A list of strings with a defined prefix (e.g., `net:`, `fs:`) is a strong candidate.

### 3.3. Permission String Syntax and Semantics

Permission strings need to be both expressive enough to cover various scenarios and simple enough for developers to understand and use correctly.

*   **Conceptual EBNF for Permission Strings**:
    ```ebnf
    PermString      ::= Category ":" Action ( ":" ResourcePattern )? ( ":" OptionList )?
    Category        ::= "net" | "fs" | "env" | "cmd" | "ffi" | "proc" | "time" | "meta" | "stdio" // Extensible
    Action          ::= IDENTIFIER // e.g., "connect", "read", "write", "exec", "load", "info", "unsafe_all"
    ResourcePattern ::= <glob-like_pattern_or_specific_identifier> // e.g., "*.example.com:443", "./config.json", "USER_*". Syntax TBD.
    OptionList      ::= IDENTIFIER ( "," IDENTIFIER )* // e.g., "recursive", "no_follow_symlinks". Syntax TBD.
    ```

*   **Structure**: `category:operation[:resource_specifier[:options]]`
    *   **`category`**: Broad area (e.g., `net`, `fs`, `env`, `cmd`, `ffi`, `stdio`).
    *   **`operation`**: Specific action (e.g., `connect`, `read`, `write`, `exec`, `load`, `info`, `unsafe_all`).
    *   **`resource_specifier`** (Optional): The target of the operation (e.g., hostname, port, file path, variable name, command name, library name). This part can include wildcards or patterns.
    *   **`options`** (Optional): Further qualifiers (e.g., `recursive` for directory operations).

*   **Summary Table of Proposed Permission Categories (v0.1 - Illustrative)**:

    | Category        | Example Operations                       | Example `PermString`                                    | Notes                                                                       |
    |-----------------|------------------------------------------|---------------------------------------------------------|-----------------------------------------------------------------------------|
    | `net`           | `connect`, `listen`, `resolve`           | `net:connect:api.example.com:443`                       | Network access.                                                             |
    | `fs`            | `read`, `write`, `create`, `delete`, `metadata` | `fs:read:/etc/app.conf`, `fs:write:./output/**`         | Filesystem operations. Paths support glob-like patterns & env vars.         |
    | `env`           | `read`, `write`, `list`                  | `env:read:USER_CONFIG_PATH`, `env:list`                 | Environment variable access. `write` is discouraged.                        |
    | `cmd`           | `exec`                                   | `cmd:exec:/usr/bin/git[:status]`                        | Subprocess execution. Pattern matching for command and args is critical.    |
    | `ffi`           | `load`, `call` (future)                  | `ffi:load:libssl.so.*`                                  | Foreign Function Interface access. `call` is more granular (future).        |
    | `proc`          | `info`, `signal`                         | `proc:info:self`, `proc:signal:<pid>:<signum>`          | Process-related operations.                                                 |
    | `time`          | `high_resolution`, `set_system_time`     | `time:high_resolution`                                  | Access to sensitive time functions.                                         |
    | `stdio`         | `access` (or `read`, `write` granularly) | `stdio:access`                                          | Access to standard input, output, error streams.                            |
    | `meta`          | `unsafe_all`                             | `meta:unsafe_all`                                       | Escape hatch for all permissions; highly discouraged.                       |

*   **Proposed Initial Permission Categories and Strings (Illustrative List - TBD SEC-PERM-SYNTAX-1)**:
    *   **Network (`net`)**:
        *   `net:connect:<host_pattern>:<port_pattern>` (e.g., `net:connect:*.example.com:443`, `net:connect:192.168.1.100:any`)
        *   `net:listen:<ip_pattern>:<port_pattern>` (e.g., `net:listen:0.0.0.0:8080`)
        *   `net:resolve:<host_pattern>` (Permission to perform DNS lookups)
    *   **Filesystem (`fs`)**:
        *   `fs:read:<path_pattern>` (e.g., `fs:read:/etc/app.conf`, `fs:read:./data/*.json`, `fs:read:$HOME/.config/app/**`)
        *   `fs:write:<path_pattern>` (e.g., `fs:write:/var/log/app.log`, `fs:write:./output/**`)
        *   `fs:create:<path_pattern>`
        *   `fs:delete:<path_pattern>`
        *   `fs:metadata:<path_pattern>` (Read metadata like size, timestamps, type)
        *   *Path patterns could use glob-like syntax. Special variables like `$HOME`, `$TMPDIR`, `$CWD` might be supported.*
    *   **Environment Variables (`env`)**:
        *   `env:read:<VAR_NAME_PATTERN>` (e.g., `env:read:PATH`, `env:read:MYAPP_*`)
        *   `env:write:<VAR_NAME_PATTERN>` (Generally discouraged)
        *   `env:list` (Permission to list all environment variables)
    *   **Command Execution (`cmd`)**:
        *   `cmd:exec:<command_path_pattern>[:<arg_pattern_list>]` (e.g., `cmd:exec:/usr/bin/convert`, `cmd:exec:my_script.sh --input {}`)
        *   Careful validation of patterns is needed to prevent trivial bypasses.
    *   **Foreign Function Interface (`ffi`)**:
        *   `ffi:load:<library_name_pattern>` (e.g., `ffi:load:libz.so.1`, `ffi:load:*custom_helper*`)
        *   `ffi:call:<library_name_pattern>:<symbol_name_pattern>` (More granular, potentially for future refinement if `ffi:load` is too broad).
    *   **Process (`proc`)**:
        *   `proc:info:<pid_or_self>` (Access process information)
        *   `proc:signal:<pid_or_self>:<signal_num_or_name>` (Send signals)
    *   **Time (`time`)**:
        *   `time:high_resolution` (Access to high-precision timers that could be used for side-channel attacks).
        *   `time:set_system_time` (Highly privileged).
    *   **Special/Meta (`meta`)**:
        *   `meta:unsafe_all` (A dangerous "escape hatch" that grants all permissions, for trusted contexts or specific tooling use only, heavily discouraged for general applications).
    *   **Standard I/O (`stdio`)** (New Category):
        *   `stdio:access` (or `stdio:read_stdin`, `stdio:write_stdout`, `stdio:write_stderr` for more granularity - initial v0.1 will likely have a single `stdio:access` for simplicity).

*   **Use of Wildcards and Patterns**:
    *   Simple glob-like patterns (`*`, `**`, `?`, character sets `[]`) should be supported in resource specifiers where appropriate (e.g., file paths, hostnames).
    *   The exact pattern syntax needs to be clearly defined and consistently implemented.

*   **Semantics**: Each permission string grants a specific, narrow capability. The absence of a permission string implies denial of that capability.

### 3.4. Enforcement Mechanism

Declaring permissions is only effective if they are robustly enforced. Ferra aims for a multi-layered enforcement approach, prioritizing compile-time checks where feasible, supplemented by runtime checks and sandboxing.

*   **Compile-Time Checks (Primary Goal where Possible)**:
    *   **Mechanism**: The Ferra compiler (specifically during semantic analysis or a dedicated permissions-checking pass) will attempt to verify if the code uses operations that would require permissions not declared in the `manifest.perms`.
    *   **How it Works**:
        1.  **Standard Library Annotation**: Functions in the Ferra standard library that perform sensitive operations (e.g., `std::fs::read_file`, `std::net::connect_tcp`) will be internally annotated with the specific permission(s) they require.
        2.  **FFI Call Analysis**: Declarations of FFI functions (`extern "C" { ... }` blocks as per `FFI_C_CPP.md`) might inherently require a general `ffi:load:<library_name>` permission if a library is specified, or more granular `ffi:call:<symbol>` if possible to analyze. The use of FFI itself is a strong signal for potential capability use.
        3.  **Static Analysis**: The compiler will track calls to these annotated stdlib functions or FFI functions.
        4.  **Verification**: If a call requiring permission `P` is found, the compiler checks if `P` (or a compatible, broader permission) is present in the `[package.permissions]` section of `Ferra.toml`.
        5.  **Violation**: If the required permission is missing, a compile-time error is generated, adhering to the principles (e.g., positive-first messaging, structured data) and potentially using a specific error code range defined in **DESIGN_DIAGNOSTICS.md** and registered in `diagnostic_codes.md` (e.g., SExxx codes).
    *   **Benefits**: Catches permission issues early in the development cycle; provides strong guarantees before code is even run.
    *   **Limitations**:
        *   Cannot easily detect dynamically constructed parameters (e.g., a file path or URL built from runtime inputs) to verify against specific path/host patterns in permissions. For `fs:read:$USER_INPUT_PATH`, compile-time checks can only verify that *some* `fs:read` permission is present, not that it specifically matches the dynamic path.
        *   Complex FFI calls where the exact native functions called are determined dynamically might be hard to check statically.
        *   Certain operations might not have a clear, single point-of-call in the stdlib (e.g., some low-level system interactions if exposed).

*   **Runtime Checks (Complementary to Compile-Time)**:
    *   **Mechanism**: For operations where compile-time verification of specific resource access is insufficient (e.g., opening a file path determined at runtime), runtime checks are necessary.
    *   **How it Works**:
        1.  When a sensitive stdlib function is called (e.g., `File::open(runtime_path)`), before performing the actual OS operation, the Ferra runtime (or the stdlib function itself) queries the permission manager with the specific resource being accessed (e.g., the resolved `runtime_path`).
        2.  The permission manager checks this against the granted `manifest.perms`.
        3.  If the operation is not permitted for that specific resource, a runtime error/exception is raised (e.g., `PermissionDeniedError`). This error should also be structured consistently with **DESIGN_DIAGNOSTICS.md** for any user-facing messages.
    *   **Implementation**: This requires the compiled program to have access to its own declared permissions at runtime. This could be achieved by embedding a representation of the `manifest.perms` into the binary or having the OS/environment provide it.
    *   **Focus**: Runtime checks are particularly important for validating path patterns, network host/port patterns, environment variable names, and command execution arguments against dynamic values.

*   **Interaction with Chosen Sandboxing Mechanisms**:
    *   Compile-time and runtime permission checks within the Ferra application are the first line of defense.
    *   Sandboxing mechanisms (WASI, seccomp-bpf, detailed in Section 4) act as a second, lower-level enforcement layer.
    *   Ideally, if the Ferra permission system works correctly, a sandboxed program should rarely, if ever, hit a sandbox violation because its own internal runtime checks should have already prevented the unauthorized operation.
    *   Sandbox violations would then indicate either a bug in the Ferra permission checking logic, an attempt to bypass it (e.g., via unsafe FFI), or an operation not covered by Ferra's permission model but restricted by the sandbox.

### 3.5. Deny by Default Principle in Practice

The "deny by default" principle is fundamental to Ferra's security posture.

*   **No Explicit Permissions**: If a `Ferra.toml` file contains no `[package.permissions]` section or an empty one, the package is assumed to have *no* special capabilities by default. It can only:
    *   Perform pure computations.
    *   Allocate and manage its own memory.
    *   Use standard library features that do not require specific permissions (e.g., collection types, string manipulation, basic math).
    *   Access to standard I/O (`stdin`, `stdout`, `stderr`) **will require explicit permission** for v0.1 to maintain strict "deny by default". A simple, broad permission like `stdio:access` will be defined. Common project templates generated by `lang new` may include this by default for convenience in typical applications.
*   **Compile-Time Failure**: If code attempts to call a stdlib function annotated as requiring a permission (e.g., `std::net::connect_tcp` requiring `net:connect`, or `std::io::println` requiring `stdio:access`), and that permission is absent, the compilation will fail with a clear error message indicating the missing permission.
*   **Runtime Failure**: If a runtime check determines that an operation on a specific resource (e.g., opening `/etc/secret_file`) is not covered by the declared permissions (even if a general `fs:read` might be present but not for that specific path, or if path patterns don't match), the operation will fail at runtime with a `PermissionDeniedError`.

This proactive denial forces developers to be explicit about their program's needs, making the security profile transparent.

### 3.6. Dependency Management and Permissions

A package's true capabilities are a function of its own permissions and those of its entire dependency tree. Managing this is crucial for supply chain security.

*   **Permission Declaration in Dependencies**: Every Ferra package (library or application) declares its own required permissions in its `Ferra.toml`.
*   **Transitive Permissions (TBD SEC-PERM-DEPS-1)**: The strategy for how an application inherits or is affected by the permissions of its dependencies needs careful definition. Options include:
    1.  **Union**: The application effectively operates with the union of all permissions declared by itself and all its direct and transitive dependencies.
        *   *Pros*: Simpler for library authors.
        *   *Cons*: Can lead to privilege escalation if a dependency requests overly broad permissions. The application might gain unintended capabilities.
    2.  **Intersection (Less Likely for Usability)**: The application can only use permissions that are declared *both* by itself *and* by the dependency needing it.
        *   *Pros*: More secure, application explicitly allows what dependencies can do.
        *   *Cons*: Very verbose; application manifest would need to re-declare many permissions for its dependencies.
    3.  **Explicit Re-declaration/Delegation (Recommended Hybrid)**:
        *   An application must declare all permissions it *directly* needs.
        *   For permissions required *only* by its dependencies, the application's manifest might need to explicitly acknowledge or delegate these.
        *   Alternatively, tooling could clearly show the *full set* of effective permissions (own + deps) and require user/developer sign-off or a mechanism to restrict/override dependency permissions.
        *   The package manager (`lang` tool) could enforce that an application cannot have *fewer* permissions than what a critical dependency requires to function, or it must explicitly override/restrict that dependency's feature set.
*   **Auditing and Displaying Permission Footprint**:
    *   The `lang` package manager tool should provide commands to inspect the full permission footprint of a project:
        *   `lang permissions show`: Lists all permissions declared by the current package.
        *   `lang permissions audit` or `lang dependencies --permissions`: Lists all unique permissions requested by the current package *and all its direct and transitive dependencies*. This gives a clear picture of the total capabilities the final application might possess or require.
        *   This audit tool could also flag:
            *   Unused declared permissions in the main package.
            *   Highly sensitive permissions requested by dependencies.
*   **Lockfile (`Ferra.lock`)**: The lockfile could potentially store a hash or summary of the resolved permissions for each dependency at the time of locking, to detect if a dependency later tries to escalate its permissions in a patch version without a manifest change.

The goal is to make permission requirements transparent throughout the dependency chain and give the application developer ultimate control and awareness, while not being overly burdensome.

### 3.7. User Experience and Tooling

The success of a permission system depends heavily on its usability for developers.

*   **Compiler Diagnostics**:
    *   When a compile-time permission check fails, the error message must be clear, indicating:
        *   The operation that requires the permission.
        *   The specific permission string(s) that would satisfy the requirement.
        *   A reference to the `Ferra.toml` file where permissions should be declared.
    *   Diagnostics should align with `DESIGN_DIAGNOSTICS.md` (e.g., positive-first messaging).
*   **Tooling for Permission Inspection**:
    *   As mentioned in 3.6, `lang permissions audit` (or similar) is crucial for understanding the full permission set.
    *   IDE integration could show required permissions for stdlib calls on hover or highlight calls requiring undeclared permissions.
    *   **CLI for Permissions Management (Conceptual)**:
        *   The `lang` CLI could include subcommands for inspecting and potentially (with care) modifying permissions.
        *   **Conceptual EBNF for `lang permissions` subcommand**:
            ```ebnf
            PermCmd     ::= "lang" "permissions" PermSubcmd Options*
            PermSubcmd  ::= "show"    (* Show current package permissions *)
                        | "audit"   (* Show full permission footprint including dependencies *)
                        | "grant" PermString (* Add a permission to Ferra.toml - requires confirmation *)
                        | "revoke" PermString (* Remove a permission - requires confirmation *)
            // PermString defined in Section 3.3
            ```
        *   **Example CLI Invocations**:
            ```bash
            # Show permissions for the current package
            lang permissions show

            # Audit all permissions for the project and its dependencies
            lang permissions audit

            # Grant a specific permission (would prompt for confirmation)
            lang permissions grant fs:read:./config.json

            # Revoke a permission (would prompt for confirmation)
            lang permissions revoke net:connect:evil.com:*
            ```
*   **`lang doctor security` (Conceptual)**:
    *   A potential future subcommand of `lang doctor` could analyze a project for security best practices related to permissions:
        *   Detecting overly broad permissions (e.g., `fs:read:**` when only `fs:read:./config.json` is used).
        *   Flagging unused declared permissions.
        *   Highlighting dependencies that request sensitive permissions.
*   **Documentation**: Clear documentation on all available permission strings, their meaning, and best practices for requesting minimal permissions.
*   **Runtime Error Messages**: `PermissionDeniedError` at runtime should clearly state which permission was missing for what specific resource access attempt.
*   **Ease of Declaration**: Adding permissions to `Ferra.toml` should be straightforward. Tooling might even suggest adding a missing permission as a quick fix in the IDE.

The aim is to make security an understandable and manageable part of the Ferra development workflow, not an opaque barrier.

## 4. Sandboxing Mechanisms (Step 3.4.2)

Sandboxing provides a crucial runtime enforcement layer to Ferra's security model, complementing the capability-based permissions declared in the manifest. While permissions define a program's *intended* access, sandboxing aims to strictly confine its *actual* operational capabilities, especially concerning interactions with the underlying operating system or host environment.

### 4.1. Overview and Rationale for Sandboxing

*   **Purpose**:
    *   **Limit Blast Radius**: To contain the impact of vulnerabilities or malicious code within a Ferra program or one of its dependencies. If a compromised component attempts an unauthorized action, the sandbox should prevent it.
    *   **Enforce Least Privilege at Runtime**: To ensure that even if a program *could* theoretically perform an action (due to a bug in compile-time permission checks or dynamic code generation), it is blocked at the OS or Wasm runtime interface if that action is outside its granted sandbox profile.
    *   **Stronger Isolation**: Provides a harder boundary than in-process permission checks alone, especially against sophisticated exploits or misbehaving FFI code.
    *   **Defense in Depth**: Acts as a critical layer in Ferra's multi-layered security approach.

*   **General Strategy**: Ferra will leverage existing, robust sandboxing technologies tailored to the target compilation environment:
    *   WebAssembly (Wasm) with the WebAssembly System Interface (WASI) for Wasm targets.
    *   OS-level mechanisms like seccomp-bpf for native ELF binaries (initially focusing on Linux).

### 4.2. WebAssembly (Wasm) + WASI Sandboxing

When Ferra code is compiled to WebAssembly, it inherently runs within a sandboxed environment provided by the Wasm runtime. WASI extends this by providing a capability-based model for system interactions. This aligns perfectly with Ferra's security goals.

*   **Leveraging WASI's Capability-Based Model**:
    *   WASI itself is designed around capabilities. A WASI-compliant runtime (e.g., Wasmtime, Wasmer) instantiates a Wasm module with a specific set of pre-opened file descriptors, environment variables, and other limited capabilities.
    *   The Wasm module cannot access any system resource for which it hasn't been explicitly granted a capability by the host runtime during instantiation.
    *   This is detailed further in `BACKEND_WASM_WASI.md` (Section 3.3 "WASI Capabilities and Security").

*   **Informing WASI Runtime Configuration via `manifest.perms`**:
    *   While the Ferra `manifest.perms` do not directly translate into WASI capabilities embedded in the `.wasm` binary, they serve as a crucial declaration of intent.
    *   **Tooling Bridge**: Ferra tooling (e.g., a `ferra run-wasm <module.wasm>` command, or deployment scripts) can read the `manifest.perms` from the package associated with `module.wasm`.
    *   Based on these declared permissions, the tool can then configure the WASI runtime appropriately when instantiating the module.
        *   Example: If `manifest.perms` includes `fs:read:/data/input`, the `ferra run-wasm` tool could configure the WASI runtime to pre-open `/data/input` and grant the Wasm module read access to it.
        *   If `net:connect:api.example.com:443` is declared, future WASI socket capabilities could be configured accordingly by the runner.
    *   This provides a user-friendly way to translate high-level Ferra permissions into low-level WASI runtime configurations.

*   **Interaction with Ferra-to-WASM Compilation**:
    *   The Ferra compiler, when targeting `wasm32-wasi`, generates a Wasm module that imports required WASI functions (e.g., `fd_write`, `path_open`).
    *   The Ferra standard library's system-interacting functions are implemented by calling these imported WASI functions.
    *   If a Ferra program attempts an operation (e.g., opening a file) for which the host WASI runtime did not grant the necessary capability, the WASI call will fail, and this failure will propagate back into the Ferra code (typically as an `IOError`).

*   **Limitations of WASI Sandboxing for Non-WASI Host Interactions**:
    *   WASI standardizes access to system resources. If a Wasm module needs to interact with a host environment in ways not covered by WASI (e.g., calling arbitrary JavaScript functions in a browser beyond simple imports, or interacting with custom host APIs in an embedded Wasm runtime), these interactions are governed by the specific host's embedding and security policies, not directly by WASI.
    *   For such non-WASI interactions, the security relies on the interface defined between the Wasm module and the host, and any additional sandboxing the host itself might apply. Ferra's `manifest.perms` could still conceptually declare intent for these (e.g., `host:js_function:alert`), but enforcement is host-dependent.

By targeting Wasm/WASI, Ferra programs benefit from a strong, standardized sandboxing model that aligns well with the capability-based permissions defined in the manifest.

### 4.3. OS-Level Sandboxing for Native ELF Binaries (e.g., seccomp-bpf)

For Ferra programs compiled to native ELF binaries (initially targeting Linux), OS-level sandboxing provides an additional layer of security by restricting the system calls (syscalls) the program can make.

*   **Mechanism Choice**:
    *   **Primary Focus: `seccomp-bpf` (Linux)**: Secure Computing mode (seccomp) with Berkeley Packet Filter (BPF) allows for fine-grained filtering of syscalls and their arguments. It's a mature and widely used sandboxing mechanism on Linux.
    *   **Conceptual Applicability to Other OS**: While the detailed design will focus on seccomp-bpf, the principles could be adapted to similar technologies on other platforms if Ferra targets them natively in the future (e.g., macOS Sandbox framework, Windows sandbox features like AppContainers or less direct mechanisms). For v0.1, Linux/seccomp-bpf is the primary native sandboxing target.

*   **Profile Generation (TBD SEC-SANDBOX-SECCOMP-1)**:
    A seccomp-bpf profile defines the set of allowed syscalls and any restrictions on their arguments.
    *   **Static Generation Based on `manifest.perms` and Code Analysis (Preferred)**:
        1.  The Ferra compiler or a dedicated Ferra security tool could analyze the declared `manifest.perms` and the compiled code (e.g., calls to stdlib functions that map to specific syscalls, FFI declarations).
        2.  Based on this analysis, it would generate a minimal seccomp-bpf profile that allows only the necessary syscalls for the declared permissions. For example:
            *   `fs:read:<path>` permission might translate to allowing `openat`, `read`, `close`, `stat` syscalls with argument filters related to read-only flags.
            *   `net:connect:<host>:<port>` might allow `socket`, `connect`, `sendto`, `recvfrom`, `close`.
            *   If no filesystem permissions are declared, syscalls like `openat` could be denied by default.
        3.  This generated profile would be embedded in or distributed with the Ferra application.
    *   **Conceptual Profile Snippet (Illustrative)**:
        If `Ferra.toml` has `[package.permissions]
fs_read = ["/etc/app.conf"]
net_connect=["api.example.com:443"]`,
        a generated seccomp profile might conceptually include rules allowing:
        ```text
        # Simplified, conceptual representation of part of a seccomp profile
        allow_syscall: openat(flags=O_RDONLY, path="/etc/app.conf")
        allow_syscall: read
        allow_syscall: close
        allow_syscall: socket(domain=AF_INET, type=SOCK_STREAM)
        allow_syscall: connect(addr_family=AF_INET, host="api.example.com", port=443)
        deny_all_other_syscalls_by_default
        ```
        The actual BPF bytecode would be much more complex.
    *   **User-Provided Profiles**: For advanced use cases, allow developers to provide their own custom seccomp-bpf profiles (e.g., in BPF assembly or a higher-level C-like language for BPF). This offers maximum flexibility but requires deep security expertise.
    *   **Tools to Assist Profile Creation**: Ferra could provide tools to help generate or refine profiles, perhaps by tracing syscalls during a test run in a permissive mode and then suggesting a minimal profile.

*   **Profile Application**:
    *   **Program Self-Sandboxing**: The compiled Ferra program could apply the seccomp-bpf filter to itself early in its startup sequence (e.g., in the Ferra runtime initialization code, before executing user `main`). This is a common approach.
    *   **External Launcher/Wrapper**: Alternatively, a dedicated launcher program could set up the seccomp sandbox and then execute the Ferra application within it. This might be useful for system-wide policies or more complex sandbox setups.
    *   **Default Behavior**: Ferra programs should aim to apply a restrictive default sandbox profile (derived from `manifest.perms`) automatically unless explicitly configured otherwise.

*   **Granularity**:
    *   **Syscall Whitelisting**: The primary approach should be to whitelist only necessary syscalls and deny all others by default.
    *   **Argument Filtering**: Where possible and practical, seccomp-bpf allows filtering based on syscall arguments (e.g., allowing `openat` only with read-only flags, or `socket` only for specific network families/protocols). The generated profiles should leverage this for finer-grained control.

*   **Performance Considerations**:
    *   `Steps.md` (Item 6) notes a "< 5 µs switch" metric for sandboxing. While seccomp-bpf itself is generally very performant (often negligible overhead once the filter is loaded), this metric implies that any transitions into/out of more heavily sandboxed code segments (if such a dynamic model were used, e.g., for FFI) or the initial setup cost should be minimal.
    *   For the static, whole-program sandboxing approach, the main performance impact is typically at startup when loading the BPF program. Per-syscall overhead is very low.

*   **Error Handling and Reporting**:
    *   When a sandboxed program attempts a denied syscall, seccomp-bpf can be configured to:
        *   Kill the process/thread (`SECCOMP_RET_KILL_PROCESS`, `SECCOMP_RET_KILL_THREAD`). This is the safest default for unexpected violations.
        *   Return a specific error code (e.g., `EPERM` via `SECCOMP_RET_ERRNO`).
        *   Send a signal (e.g., `SIGSYS`) which can be caught for logging/debugging if not too risky.
        *   Trap (`SECCOMP_RET_TRAP`) allowing a tracer (like `ptrace`) to inspect and potentially allow/deny the syscall (more for debugging/development).
    *   The Ferra runtime should define a clear behavior for seccomp violations. For production, `SECCOMP_RET_KILL_PROCESS` is often preferred for security.
    *   Logging or reporting sandbox violations (e.g., via `syslog` or to stderr if possible before termination) is important for debugging profiles.

### 4.4. Interaction between Permissions and Sandboxing

The capability-based permissions (`manifest.perms`) and the runtime sandboxing mechanisms are designed to be complementary layers of security.

*   **Permissions as Declaration of Intent**:
    *   `manifest.perms` declare the *intended* capabilities of the package, as understood by the developer and verifiable by static analysis (to some extent).
    *   The Ferra compiler and standard library use these declarations to perform compile-time and in-process runtime checks (Section 3.4).

*   **Sandboxing as Runtime Enforcement Barrier**:
    *   Sandboxing (WASI, seccomp-bpf) provides a lower-level, often OS-enforced barrier that restricts what the process can actually do, regardless of its internal logic.
    *   It acts as a fail-safe if:
        *   Compile-time permission checks are insufficient (e.g., for dynamically determined resources).
        *   There are bugs in the Ferra runtime's permission checking.
        *   Unsafe code (e.g., FFI) attempts to bypass Ferra's permission model.
        *   A vulnerability in the application or a dependency is exploited.

*   **Informing Sandbox Profiles**:
    *   Ideally, the `manifest.perms` should directly inform the generation or configuration of the sandbox profile. For example, if no `net:*` permissions are declared, the seccomp profile should block all socket-related syscalls, and the WASI runtime should not grant any network capabilities.
    *   This alignment ensures that the runtime enforcement matches the declared intent.

*   **Discrepancies**:
    *   If a program tries an operation allowed by its `manifest.perms` but blocked by the sandbox, it typically indicates a mismatch in profile generation (the sandbox is too strict for the declared needs) or an overly broad permission declaration that couldn't be precisely mapped to a safe sandbox rule.
    *   If a program tries an operation *not* allowed by its `manifest.perms` but the sandbox *would* have allowed it (unlikely if profiles are derived from permissions), it indicates a failure of the Ferra-level permission checks.
    *   If an operation is denied by Ferra's runtime permission check *before* hitting the sandbox layer, this is the ideal scenario for denied intended operations.

### 4.5. Sandboxing FFI Calls

Foreign Function Interface (FFI) calls, especially to C/C++ libraries (see `FFI_C_CPP.md`), represent a significant security boundary and challenge for sandboxing.

*   **Challenges**:
    *   Native C/C++ code called via FFI operates outside Ferra's memory safety and type system.
    *   It can potentially execute arbitrary syscalls or access memory in ways not directly visible to Ferra's static analysis.
    *   A general `ffi:load:<library>` permission might grant access to a library with a vast API surface, some parts of which could be dangerous.

*   **Strategies and Considerations**:
    1.  **Granular FFI Permissions (Future)**:
        *   While v0.1 might start with `ffi:load:<library_pattern>`, a future enhancement could be `ffi:call:<library_pattern>:<symbol_pattern>` to restrict calls to specific functions within a native library. This would require more advanced static analysis or developer annotations.
    2.  **Seccomp-bpf for FFI**: The seccomp-bpf filter applied to the entire Ferra process will also restrict syscalls made by native code called via FFI. This is a key benefit. If the native library tries to `open` a file unexpectedly, seccomp can block it if `openat` isn't in the profile.
    3.  **Dedicated Sandboxed Processes/Actors for Risky FFI**:
        *   For particularly untrusted or powerful FFI libraries, a more robust approach (though complex) is to run the FFI calls in a separate, heavily sandboxed Ferra process or a dedicated actor with minimal, specific permissions.
        *   Communication with this sandboxed FFI worker would occur via IPC (e.g., a restricted form of channels as per `CONCURRENCY_MODEL.md`).
        *   This is an advanced pattern, likely beyond v0.1 scope, but important for long-term security.
    4.  **WASI and FFI**: When running as Wasm/WASI, FFI calls typically mean calling other Wasm modules or functions imported from the host. The security of these calls depends on the capabilities granted to those other modules or the nature of the host imports. WASI itself does not directly sandbox calls *between* Wasm modules in the same instance beyond interface types.
    5.  **Static Analysis of FFI Usage**: Tooling could attempt to analyze the symbols imported by a native library linked via FFI to infer potential capabilities it might need, cross-referencing against declared `ffi:` permissions.
    6.  **Developer Responsibility**: Ultimately, using FFI introduces a level of trust in the native library. Developers must be aware of the risks and use FFI permissions judiciously. `unsafe` blocks in Ferra for FFI calls highlight this.

For v0.1, the combination of `ffi:load` permissions and process-wide seccomp-bpf (for native) or WASI capabilities (for Wasm) will provide the initial layer of control over FFI. More granular FFI sandboxing is a significant area for future research and development.

## 5. Security Considerations for Other Ferra Features

Ferra's security model, particularly its capability-based permissions and sandboxing, must interact coherently with other key language and system features. This section explores these interactions.

### 5.1. Concurrency Model (`CONCURRENCY_MODEL.md`)

Ferra's deterministic actor model (detailed in `CONCURRENCY_MODEL.md`) introduces concurrent units of execution. The security model needs to consider how permissions and sandboxing apply in this context.

*   **Actor-Specific Permissions (TBD SEC-ACTOR-PERM-1)**:
    *   **Question**: Can individual actors within a Ferra application operate with different, more restricted sets of permissions than the application as a whole?
    *   **Potential Benefits**: This would allow for fine-grained privilege separation within an application. For example, an actor responsible for network communication could be granted `net:connect` permissions, while a computationally-focused actor has none, limiting the attack surface if the computation actor is compromised.
    *   **Implementation Complexity**:
        *   How would per-actor permissions be declared (e.g., in `Ferra.toml` alongside actor definitions, or via attributes on actor code)?
        *   How would the runtime enforce these distinct permission sets for different actors, especially if they share the same OS process? This might require the actor scheduler and message-passing system to be permission-aware.
    *   **v0.1 Scope**: For the initial v0.1 security model, per-actor permissions are likely an advanced feature. The primary focus will be on process-level permissions and sandboxing. However, the design should not preclude future extension to per-actor capabilities.

*   **Inter-Actor Communication and Capability Flow**:
    *   If actors can have different permission sets, how capabilities (or handles to resources obtained via capabilities) are passed between actors via messages needs careful consideration.
    *   The principle of least privilege should still apply: an actor should not be able to gain more permissions than it was initially granted simply by receiving a message from a more privileged actor, unless there's an explicit and secure delegation mechanism.
    *   Ferra's ownership and borrowing system, applied to message passing, can help manage resource handles, but permission to *use* the underlying resource might still need to be checked.

*   **Sandboxing Individual Actors**:
    *   **OS-Level**: Sandboxing individual actors within a single OS process using mechanisms like seccomp-bpf for each actor is highly complex and likely not feasible for v0.1. Process-level sandboxing is the primary target.
    *   **Wasm Actors**: If actors are compiled as separate Wasm modules (a more advanced architectural pattern), then each could potentially be instantiated in its own Wasm runtime with distinct WASI capabilities.
    *   For v0.1, actors within a native Ferra process will share the same OS-level sandbox. Actors within a Ferra Wasm module will share the same WASI sandbox capabilities.

The interaction between the concurrency model and the security model is a rich area, with initial v0.1 focusing on process-wide security and leaving per-actor granularity as a future TBD.

### 5.2. AI APIs (`AI_API_REFACTOR_VERIFY.md`, `AI_API_AST.md`)

The AI-assisted development features in Ferra also have security implications that need to be addressed by the overall security model.

*   **Permissions for AI API Usage**:
    *   Accessing code structure (`ai::ast()`) or invoking potentially resource-intensive AI operations (`ai::refactor`, `ai::verify`) might themselves be subject to permissions.
    *   **Conceptual Permissions**:
        *   `ai:access_ast`: Permission for a script or tool to call `ai::ast()` and retrieve detailed AST/semantic information. This is relevant because the AST can contain sensitive source code details. `AI_API_AST.md` already specifies an opt-in mechanism (`#![allow(ai_ast)]` or `--ai-ast` flag).
        *   `ai:invoke_refactor`: Permission to use `ai::refactor` capabilities.
        *   `ai:invoke_verify`: Permission to use `ai::verify` capabilities.
        *   `ai:access_model:<model_identifier_or_class>`: Permission to interact with a specific (potentially remote or resource-intensive) AI model.
    *   These permissions would be declared in `manifest.perms` for tools or build scripts that utilize these AI APIs.

*   **Sandboxing AI Model Execution**:
    *   If AI models (especially third-party or less trusted ones) are executed locally as part of the `ai::refactor` or `ai::verify` process:
        *   They should ideally run in a sandboxed environment (e.g., a separate process with strict seccomp rules, or as a Wasm module if the model can be compiled to Wasm).
        *   This sandbox would limit the model's access to the filesystem, network, and other system resources, protecting the developer's machine.
    *   This is particularly important if the AI models themselves are complex pieces of software that could have vulnerabilities.

*   **Security of Code Generated or Modified by AI**:
    *   **Code Quality and Vulnerabilities**: AI-generated code is not infallible and could potentially introduce bugs or security vulnerabilities.
        *   The `ai::verify()` API itself (e.g., fuzzing, static analysis checks) should be used to scrutinize AI-generated code.
        *   Developers must always review and test AI-suggested code changes.
    *   **`X-AI-Provenance`**: As detailed in Section 4 of `AI_API_REFACTOR_VERIFY.md`, provenance tracking is crucial for identifying AI-generated code and understanding its origin, which aids in security audits.
    *   **Least Privilege for AI Tools**: AI tools that modify code should themselves operate under the principle of least privilege, only requesting permissions necessary for their specific refactoring task and the files they need to modify.

*   **Data Privacy with Remote AI Services**:
    *   If `ai::refactor` or `ai::verify` rely on remote/cloud-based AI services, the security model must consider the privacy of source code and other context data sent to these services.
    *   Clear user consent and data handling policies would be needed.
    *   The permission model could potentially include permissions like `net:connect:ai.ferra_official_service.com:443` to make this explicit.

### 5.3. Standard Library Design and Permissions

The Ferra standard library is the primary interface through which Ferra programs interact with the system. Its design must be tightly integrated with the permission model.

*   **Annotating Stdlib Functions**:
    *   Functions in the standard library that perform operations requiring capabilities (e.g., opening a file, making a network connection, reading an environment variable) must be internally annotated or otherwise identifiable by the compiler as requiring specific permissions.
    *   Example: `std::fs::read_to_string(path: String)` would be associated with a need for an `fs:read:{path}`-compatible permission.

*   **Compile-Time Permission Checks for Stdlib Usage**:
    *   When the compiler encounters a call to such a stdlib function, it will check if the calling package has declared the necessary permission in its `manifest.perms`.
    *   If the permission is missing, a compile-time error will be issued (as per Section 3.4).

*   **Runtime Permission Checks within Stdlib Functions**:
    *   For operations where the specific resource is determined at runtime (e.g., `std::fs::read_to_string(user_provided_path)`), the stdlib function itself, before performing the OS call, must:
        1.  Determine the exact resource being accessed (e.g., the fully resolved `user_provided_path`).
        2.  Query an internal runtime permission manager with this specific resource and the required operation type (e.g., "can I read /actual/path/to/file?").
        3.  If the permission manager denies the request based on the `manifest.perms` loaded for the application, the stdlib function must return an appropriate error (e.g., `PermissionDeniedError`) instead of proceeding with the OS call.

*   **Error Types**: The standard library should define clear error types (e.g., within a common `std::io::Error` or a new `std::security::Error`) to represent permission denied errors, distinct from other operational errors (like "file not found").

*   **Designing for Least Privilege**:
    *   Stdlib APIs should be designed to encourage requesting minimal necessary permissions. For example, if a function only needs to read metadata, it shouldn't require full read permission for the file content.
    *   Avoid overly broad APIs that make it difficult to assign fine-grained permissions.

By deeply integrating permission checks into the standard library, Ferra can provide a strong first line of defense, ensuring that most common operations are subject to the declared capability model.

## 6. Developer Workflow and Tooling

For Ferra's security model to be effective and adopted, it must be supported by a developer-friendly workflow and appropriate tooling. The goal is to make security considerations a natural part of development, not an afterthought or an undue burden.

*   **Declaring Permissions in `Ferra.toml`**:
    *   As detailed in Section 3.2, developers will declare necessary permissions in the `[package.permissions]` section of their `Ferra.toml` manifest file.
    *   The syntax should be clear and well-documented.
    *   IDEs with Ferra support could offer auto-completion for permission strings or UI-based editors for this section of the manifest.

*   **Compiler Feedback on Permission Issues**:
    *   **Compile-Time Errors**: If the compiler detects (via static analysis of stdlib calls or FFI usage) that an operation requires a permission not declared in the manifest, it will issue a clear compile-time error.
        *   The error message should specify the missing permission string and point to the code location requiring it (aligning with `DESIGN_DIAGNOSTICS.md`).
        *   IDEs should surface these errors prominently.
    *   **Quick Fixes (IDE)**: IDEs might offer quick fixes to add the suggested missing permission to `Ferra.toml` (though developers should be encouraged to understand why it's needed).

*   **Runtime Feedback**:
    *   If a runtime permission check fails, a specific error type (e.g., `PermissionDeniedError`) should be raised, clearly indicating the denied operation and resource.
    *   Stack traces should help pinpoint where the unauthorized operation was attempted.

*   **Debugging Sandboxed Applications**:
    *   Debugging applications running under strict sandboxes (especially seccomp-bpf) can be challenging if the sandbox kills the process on violation.
    *   **Permissive/Logging Mode for Sandbox**: During development, it should be possible to run the sandbox in a more permissive or "logging-only" mode where violations are logged (e.g., to syslog or stderr) but don't necessarily terminate the application immediately. This helps in developing and refining sandbox profiles.
    *   **Tooling for Sandbox Profile Debugging**: Tools might be provided to analyze which syscall led to a seccomp violation if the program terminates abruptly.
    *   WASI runtimes often provide their own debugging mechanisms for capability issues.

*   **Tools for Auditing Permissions and Sandbox Profiles**:
    *   **`lang permissions audit` (or similar)**: As proposed in Section 3.6, a CLI command to display the full effective permission set of a project (including all dependencies) is essential.
        *   This tool could also highlight potentially dangerous or overly broad permissions.
    *   **`lang doctor security` (Conceptual)**: As proposed in Section 3.7, a specialized `lang doctor` subcommand could perform more advanced security analysis, check for common permission pitfalls, or analyze the tightness of generated sandbox profiles against actual code usage.
    *   **IDE Integration**: IDEs could visually indicate the permissions required by a function or module, or highlight code sections that trigger specific permission requirements.

*   **Documentation and Education**:
    *   Comprehensive documentation on all defined permission strings, their implications, and best practices for requesting minimal necessary permissions.
    *   Guides on secure coding practices in Ferra, including how to work with the permission and sandboxing model effectively.
    *   Examples of how to configure `manifest.perms` for common application types.

The overall aim is to empower developers to build secure applications by making security features understandable, manageable, and well-supported by the Ferra toolchain.

## 7. Limitations and Future Work

The v0.1 Ferra security model, while providing a strong foundation with capability-based permissions and initial sandboxing mechanisms, will have limitations. This section acknowledges these and outlines potential areas for future enhancements.

*   **Limitations of v0.1 Model**:
    *   **Static Permission Checking Granularity**: Compile-time permission checks might not always be able to verify permissions against dynamically constructed resource locators (e.g., file paths, network hosts from user input) with perfect precision. Runtime checks will cover some of this, but complex dynamic behavior can still be challenging.
    *   **Complexity of Managing Fine-Grained Permissions**: While desirable, an extremely large number of very fine-grained permission strings could become cumbersome for developers to manage. Finding the right balance is key.
    *   **Challenges in Perfect Sandboxing**:
        *   Creating truly minimal and correct seccomp-bpf profiles automatically for all possible programs is a hard problem. Initial profiles might be somewhat broader than strictly necessary or might miss edge cases.
        *   Sandboxing FFI calls to arbitrary native code remains a significant challenge; the primary protection comes from the OS-level sandbox applied to the whole process.
        *   WASI capabilities are still evolving, and not all desirable system interactions might be coverable by stable WASI in v0.1.
    *   **Performance Overhead**: While the target is minimal, both runtime permission checks and sandboxing mechanisms (especially transitions or complex filter evaluations) can introduce some performance overhead. This needs to be monitored.
    *   **Initial Scope of Permissions**: The initial set of defined permission strings in v0.1 will cover common cases but might not encompass all possible sensitive operations.

*   **Potential Future Work and Enhancements**:
    *   **More Sophisticated Permission Models**:
        *   **Runtime Prompts**: For certain less critical or user-driven permissions, explore a model where the application can request a permission at runtime, and the user (or a host environment policy) can grant or deny it interactively (common in mobile OSes).
        *   **Dynamic Permission Granting/Revocation**: Mechanisms for a program to temporarily acquire a permission for a specific operation and then drop it.
        *   **Conditional Permissions**: Permissions that depend on runtime conditions or configuration.
    *   **Enhanced Sandboxing Technologies**:
        *   Support for more OS-level sandboxing backends (e.g., refined AppArmor profiles, macOS sandbox, Windows AppContainers).
        *   Finer-grained sandboxing for concurrent units like actors, if feasible (see TBD SEC-ACTOR-PERM-1).
        *   Research into advanced techniques for sandboxing or instrumenting FFI calls more deeply.
    *   **Improved Tooling for Profile Generation**: More sophisticated tools to help developers automatically generate or refine minimal seccomp-bpf or other sandbox profiles based on static and potentially dynamic analysis of their application's behavior.
    *   **Formal Verification**: Explore the use of formal methods to verify aspects of the permission model or the correctness of sandbox profile generation.
    *   **Integration with Security Auditing Tools**: Ensure that Ferra's permission declarations and sandbox configurations can be easily consumed and analyzed by third-party security auditing tools.
    *   **Expanded Set of Permission Strings**: Continuously review and expand the defined permission categories and strings as the Ferra standard library and common use cases evolve.
    *   **Time-Bound or Resource-Limited Permissions**: Permissions that are only valid for a certain duration or for a limited quantity of resource usage.

The Ferra security model is expected to evolve iteratively, incorporating feedback, addressing new threats, and leveraging advances in security technologies.

## 8. Open Questions / TBD

This section consolidates the "To Be Determined" items identified throughout this document, which will require further investigation, prototyping, and decision-making during the implementation of Module 3.4.

*   **(SEC-PERM-SYNTAX-1)**: **Final Permission String Syntax**:
    *   The definitive, detailed syntax for all permission strings to be supported in `manifest.perms` for v0.1, including precise wildcard/pattern matching rules for resource specifiers (e.g., file paths, hostnames, command arguments).
    *   The exact structure within `Ferra.toml` for the `[package.permissions]` section (e.g., flat list vs. nested tables by category).
*   **(SEC-PERM-DEPS-1)**: **Dependency Permission Strategy**:
    *   The definitive strategy for how an application's effective permissions are determined considering the permissions declared by its direct and transitive dependencies (e.g., union, explicit re-declaration, override mechanisms).
*   **(SEC-PERM-ENFORCE-1)**: **Precise Enforcement Mechanisms**:
    *   For each type of permission (e.g., `fs:read`, `net:connect`, `ffi:load`), the precise mechanism for its compile-time check (if any) and its runtime check (if any), including which stdlib functions are responsible for initiating runtime checks.
    *   How the runtime permission manager accesses and interprets the embedded `manifest.perms`.
*   **(SEC-SANDBOX-SECCOMP-1)**: **seccomp-bpf Design Details**:
    *   The detailed design for seccomp-bpf profile generation (e.g., default generation strategy based on manifest, format for user-provided profiles).
    *   The primary mechanism for profile application (self-applied by program vs. external launcher) for v0.1.
    *   Default action for seccomp violations (e.g., `SECCOMP_RET_KILL_PROCESS`).
*   **(SEC-SANDBOX-FFI-1)**: **FFI Sandboxing Strategies (v0.1)**:
    *   What, if any, specific sandboxing strategies beyond process-wide seccomp/WASI capabilities can be realistically applied to FFI calls in v0.1?
    *   Will `ffi:call` permissions be pursued for v0.1, or will `ffi:load` be the primary FFI permission?
*   **(SEC-ACTOR-PERM-1)**: **Per-Actor Permissions**:
    *   Feasibility and high-level design sketch for per-actor permissions/sandboxing, to determine if any foundational aspects need to be considered in v0.1 even if full implementation is deferred.
*   **(SEC-STDIO-PERM-1) (Resolved)**: **Stdio Access Permission**. Decision: Stdio access (`stdin`, `stdout`, `stderr`) will require explicit permission (e.g., `stdio:access`) for v0.1 to maintain strict "deny by default". Common project templates generated by `lang new` may include this by default for convenience. (Covered in §3.3, §3.5).
*   **(SEC-TOOL-AUDIT-1)**: **`lang permissions audit` Details**:
    *   Specific output format and analysis capabilities of the initial version of a permissions auditing tool.

Resolving these TBDs will be crucial for delivering a robust and usable v0.1 Ferra security model.
---
This document will guide the design of Ferra's security model, focusing on capabilities and sandboxing. 