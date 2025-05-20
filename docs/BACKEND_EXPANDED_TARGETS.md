# Ferra Expanded Backends Design v0.1

> **Status:** Initial Draft - Module 3.6 (Steps 3.6.1 - 3.6.3)

## 1. Introduction and Goals

This document specifies the design considerations for expanding Ferra's backend capabilities to significantly broaden its reach onto critical modern platforms. The primary focus of this module is to enable Ferra applications to target ARM-64 architectures, to be submittable to Apple's App Stores via Bitcode, and to be packaged as Android App Bundles (AABs) for distribution on Google Play.

These expanded backend targets are essential for Ferra to be a viable language for mobile application development (as envisioned in `UI_DSL_MOBILE.md`), server-side applications on ARM infrastructure, and generally to ensure wide applicability in today's diverse hardware ecosystem.

*   **Purpose**:
    *   To detail the technical requirements and design strategies for targeting the ARM-64 architecture.
    *   To specify the process for generating Apple Bitcode, enabling Ferra applications to be distributed on iOS, macOS, watchOS, and tvOS platforms through the App Store.
    *   To define how Ferra projects will be compiled and packaged into Android App Bundles (AABs) for efficient distribution via Google Play and other Android app stores.
    *   To ensure these new backend targets integrate smoothly with Ferra's existing IR, compiler infrastructure, and build tooling.

*   **Relationship to Existing Backends**:
    *   The design for ARM-64 and Apple Bitcode generation will heavily leverage Ferra's existing LLVM integration, as detailed in `BACKEND_LLVM_X86-64.md`. Many principles of IR-to-LLVM-IR conversion, optimization, and runtime interaction will be applicable.
    *   For Android AAB generation, packaging considerations will draw upon insights from `BACKEND_WASM_WASI.md` (regarding self-contained deployable units) and will interact closely with the `PACKAGE_MANAGER_SPEC.md` for managing build artifacts and potentially Android-specific dependencies.

*   **Core Requirements (from `Steps.md` & `comprehensive_plan.md`)**:
    *   **Step 3.6.1: Specify ARM-64 backend**: Enable compilation for ARM-64 CPUs.
    *   **Step 3.6.2: Specify Apple Bitcode backend**: Enable generation of LLVM Bitcode suitable for Apple's submission process.
    *   **Step 3.6.3: Specify Android AAB generation**: Enable packaging of Ferra Android applications into the AAB format.

*   **Goals for this Document**:
    1.  Outline ARM-64 specific code generation details, including ABI compliance and performance considerations.
    2.  Detail the process, LLVM requirements, and compatibility constraints for emitting Apple Bitcode.
    3.  Describe the structure of Android App Bundles and the steps Ferra tooling will take to produce them, including native library packaging for multiple ABIs.
    4.  Identify necessary toolchain dependencies (e.g., specific LLVM versions, Android SDK/NDK components) and integration points.
    5.  Address cross-cutting concerns like FFI compatibility, runtime adjustments, and debugging for these new targets.

This specification aims to pave the way for Ferra to become a truly cross-platform language, capable of targeting a wide array of important server, desktop, and mobile environments.

## 2. Common Considerations for Expanded Backends

While each new backend target (ARM-64, Apple Bitcode, Android AAB) has unique specifics, several common design considerations and infrastructure requirements apply across them. Addressing these cohesively will ensure a more robust and maintainable multi-target compiler.

*   **Conceptual EBNF for Relevant `lang build` Options**:
    ```ebnf
    BuildCmd         ::= "lang" "build" BuildOption*
    BuildOption      ::= TargetOption | EmitFormatOption | PackageFormatOption | OtherBuildFlags
    TargetOption     ::= "--target" "=" TARGET_TRIPLE
    EmitFormatOption ::= "--emit" "=" ("ir" | "bc" | "llvm-ir" | "asm" | "lib" | "bin") (* `bc` here could mean generic LLVM bitcode *)
    PackageFormatOption ::= "--package-format" "=" ("aab" | "ipa" (* conceptual *) | "native_executable" (* default *))
                        | "--emit-apple-bitcode" (* Boolean flag, implies specific output/linking for Apple targets *)
    TARGET_TRIPLE    ::= STRING_LITERAL (* e.g., "aarch64-unknown-linux-gnu", "aarch64-apple-ios15.0" *)
    OtherBuildFlags  ::= (* e.g., "--release", "--opt-level=s" *)
    ```
    *Note: This EBNF is illustrative. The actual CLI parsing will depend on the chosen CLI framework. `TARGET_TRIPLE` would be validated against a list of supported/known triples. The `--emit-apple-bitcode` flag is a more specific way to request Bitcode output for relevant Apple targets, possibly influencing linker behavior and object file sections, beyond just a generic `--emit=bc`.* 

*   **Toolchain Requirements and Management (TBD EXP-BACKEND-TOOLCHAIN-1)**:
    *   **LLVM Version**: Confirm if the LLVM version used for x86-64 (currently LLVM 17+ as per `BACKEND_LLVM_X86-64.md`) is suitable and sufficient for robust ARM-64 code generation and Apple Bitcode emission, or if specific versions or forks (e.g., Apple's Clang/LLVM for Bitcode) are necessary.
    *   **Platform SDKs**:
        *   For Apple Bitcode: Access to macOS SDKs, Xcode command-line tools.
        *   For Android AAB: Access to Android SDK (build-tools, platform-tools) and NDK (for native libraries).
    *   **Linkers**: Appropriate linkers for each target platform and object format (e.g., `lld` for ARM-64 Linux, Apple's linker for Bitcode-enabled targets, NDK linkers for Android `.so` files).
    *   **Strategy for Acquisition/Use (TBD EXP-BACKEND-TOOLCHAIN-1)**: How will the Ferra build system locate or guide users to install these external toolchain dependencies? This involves:
        *   **Detection Logic**: Attempting to find tools (LLVM/Clang, Xcode tools, Android NDK/SDK) in standard system paths, via environment variables (e.g., `ANDROID_NDK_HOME`, `XCODE_SELECT_PATH`), or potentially via configuration in `Ferra.toml` or a global Ferra config file.
        *   **Error Reporting**: Clear errors if required toolchains/SDKs are not found for a given target.
        *   **Documentation**: Guidance for developers on installing and configuring these prerequisites.
        *   *(Consideration for future: A `lang doctor --check-toolchains` command?)*

*   **Target Triples and Cross-Compilation**:
    *   **Defining Target Triples**: Accurately define the LLVM target triples for each supported platform variant (e.g., `aarch64-unknown-linux-gnu`, `aarch64-apple-ios`, `armv7a-linux-androideabi`, `aarch64-linux-android`).
    *   **Cross-Compilation Support**: The Ferra compiler and build system must be designed to support cross-compilation from a host development machine (e.g., x86-64 Linux/macOS) to these new target architectures and OSes. This involves using the correct sysroot, linker, and target-specific LLVM settings.

*   **Runtime Support Adjustments (TBD EXP-BACKEND-RUNTIME-1)**:
    *   Ferra's core runtime functions (e.g., for memory allocation `ferra_alloc`, panic handling `ferra_panic`, as defined in `BACKEND_LLVM_X86-64.md` Section 4.1) must be compiled and function correctly for ARM-64.
    *   Any platform-specific OS interactions within the runtime (if any) need to be ported or adapted for ARM-64 Linux, macOS/iOS, and Android.
    *   Panic unwinding mechanisms might need target-specific implementations.

*   **FFI Compatibility (`FFI_C_CPP.md`)**:
    *   **Calling Conventions**: Ensure the backend generates code adhering to the correct C ABI calling conventions for each target (e.g., AAPCS64 for ARM-64).
    *   **Data Layout**: `#[repr(C)]` structs and other FFI-safe types must have layouts consistent with the target platform's C ABI.
    *   **Testing**: FFI functionality must be thoroughly tested on each new target platform.

*   **Build System Integration (`lang build`)**:
    *   The `lang build` command (and underlying build system) will need to:
        *   Accept target specifiers (e.g., `--target aarch64-apple-ios`, `--target android-arm64-v8a`).
        *   Invoke the compiler with the correct LLVM backend settings for the specified target.
        *   Handle linking against target-specific system libraries or SDKs.
        *   Produce the correct output format (e.g., executable, static/dynamic library, Apple Bitcode embedded object, AAB).
    *   This likely requires extending the build configuration options in `Ferra.toml` (see `PACKAGE_MANAGER_SPEC.md`) to specify target platforms and any associated settings.

*   **Testing Strategy (TBD EXP-BACKEND-CI-1)**:
    *   Establishing CI infrastructure to build and run tests for these new targets is crucial.
    *   This may involve using emulators (e.g., QEMU for ARM-64, Android emulators) or physical hardware for testing.
    *   Specific test suites might be needed to verify backend correctness and performance on each target.

*   **Code Generation Consistency**:
    *   While output is target-specific, the process of lowering Ferra IR to LLVM IR should remain as common as possible, with target-specific deviations handled primarily by LLVM's backend capabilities and appropriate target feature flags. This maximizes code reuse in the Ferra compiler's mid-end.

Addressing these common considerations upfront will streamline the specification and implementation of each individual expanded backend.

## 3. ARM-64 Backend Specification (Step 3.6.1)

This section details the specifics of targeting the ARM-64 (AArch64) architecture. Given ARM-64's prevalence in mobile devices, servers, and increasingly in desktop/laptop computers (e.g., Apple Silicon), robust support is crucial for Ferra's cross-platform ambitions.

### 3.1. Target Platforms

Ferra's ARM-64 backend will initially aim to support the following common platforms:

*   **Linux on ARM-64**:
    *   Target Triple (example): `aarch64-unknown-linux-gnu` (for glibc-based systems) or `aarch64-unknown-linux-musl` (for musl-based systems).
    *   Use Cases: Servers, embedded systems, Linux-based mobile/IoT devices.
*   **macOS on ARM-64 (Apple Silicon)**:
    *   Target Triple: `aarch64-apple-darwin`.
    *   Use Cases: macOS desktop and laptop applications.
*   **Windows on ARM-64**:
    *   Target Triple (example): `aarch64-pc-windows-msvc`.
    *   Use Cases: Windows laptops and devices running ARM processors.
*   **Mobile Platforms (via specific packaging formats)**:
    *   **iOS (ARM-64)**: While iOS apps are ultimately packaged with Bitcode (see Section 4), the underlying native code generation will target ARM-64 (e.g., `aarch64-apple-ios`).
    *   **Android (ARM-64)**: Android App Bundles (AABs, see Section 5) will include native libraries for `arm64-v8a` and potentially `armeabi-v7a` (though Module 3.6 explicitly focuses on ARM-64, v7a might be a related consideration for broader Android support). Target triples like `aarch64-linux-android`.

The initial focus for v0.1 of this backend will be `aarch64-unknown-linux-gnu` and `aarch64-apple-darwin` due to toolchain availability and common development environments, with iOS/Android ARM-64 support being primarily through their respective packaging/Bitcode requirements.

*   **Summary Table of Key ARM-64 Target Triples (Illustrative)**:

    | Platform Description         | Example Target Triple         | Primary Use Cases & Notes                                  |
    |------------------------------|-------------------------------|------------------------------------------------------------|
    | Linux (GNU) ARM-64           | `aarch64-unknown-linux-gnu`   | Servers, embedded, general Linux on ARM-64.                |
    | Linux (MUSL) ARM-64          | `aarch64-unknown-linux-musl`  | Lightweight/static-linked Linux on ARM-64.                 |
    | macOS (Apple Silicon) ARM-64 | `aarch64-apple-darwin`        | Native macOS applications on M-series chips.               |
    | Windows on ARM-64            | `aarch64-pc-windows-msvc`     | Native Windows applications on ARM-64.                     |
    | iOS ARM-64                   | `aarch64-apple-ios<version>`  | iOS applications (often involves Bitcode). Example: `aarch64-apple-ios15.0`. |
    | Android ARM-64 (arm64-v8a)   | `aarch64-linux-android`       | Native libraries for Android `arm64-v8a` ABI.              |
    | Android ARM-32 (armeabi-v7a) | `armv7a-linux-androideabi`    | Native libraries for older Android `armeabi-v7a` ABI (Future consideration). |

### 3.2. LLVM Integration

The ARM-64 backend will primarily leverage LLVM, building upon the existing infrastructure outlined in `BACKEND_LLVM_X86-64.md`.

*   **LLVM Backend Re-use**: The core logic for Ferra IR to LLVM IR translation (Section 3 of `BACKEND_LLVM_X86-64.md`) will be largely reused. LLVM itself handles the target-specific instruction selection and code generation for ARM-64 once it receives generic LLVM IR.
*   **Target-Specific LLVM Features**:
    *   The Ferra compiler will instruct LLVM to target the appropriate ARM-64 sub-architecture and CPU features (e.g., via `-mcpu` and `-mattr` LLVM flags if exposed, or by configuring the LLVM TargetMachine).
    *   **NEON SIMD**: LLVM's auto-vectorizer should be able to target ARM NEON SIMD instructions for data-parallel `for_each` constructs (as discussed in `DATA_PARALLEL_GPU.md` Section 3). Specific NEON intrinsics might be exposed in Ferra's standard library in the future if high-level auto-vectorization is insufficient for critical performance needs (TBD ARM-SPECIFIC-1).
*   **Calling Conventions**:
    *   Adherence to the **AAPCS64 (Procedure Call Standard for the ARM 64-bit Architecture)** is critical for C FFI compatibility and general OS interaction. LLVM handles this when configured for an ARM-64 target.
*   **LLVM Version**: Ensure the chosen LLVM version (17+ per existing docs) has mature and robust support for the targeted ARM-64 platforms and features.

### 3.3. Ferra IR to LLVM IR for ARM-64

Most of the Ferra IR to LLVM IR mapping defined in `BACKEND_LLVM_X86-64.md` (Section 3) remains applicable. Key points for ARM-64:

*   **Type Mapping**:
    *   Pointer sizes will be 64-bit. Ferra types like `usize` and `isize` will map to `i64` and `ptrtoint`/`inttoptr` correctly.
    *   Other primitive types (`Int`, `Float`, `Bool`, `Char`) map as previously defined.
*   **Instruction Lowering**: Standard IR instructions (`add`, `load`, `store`, `call`, etc.) are lowered to their generic LLVM IR counterparts. LLVM's ARM-64 backend then selects appropriate machine instructions.
*   **Atomics**: If Ferra supports atomic operations, ensure they are correctly lowered to LLVM atomic instructions, which will then map to ARM-64's load-linked/store-conditional (LL/SC) or other atomic primitives.
*   **Memory Model**: LLVM's memory model and alignment requirements for ARM-64 will be respected.

### 3.4. Performance Optimizations

*   **Leveraging LLVM**: Primary reliance will be on LLVM's extensive optimization passes, which are generally effective for ARM-64.
*   **ARM-64 Specific Optimizations**:
    *   LLVM's backend includes many optimizations specific to ARM-64 microarchitectures (e.g., instruction scheduling, peephole optimizations).
    *   The Ferra compiler should pass appropriate CPU type/feature flags to LLVM to enable these (e.g. for Apple M-series, specific Cortex cores).
*   **Mobile Considerations**: For ARM-64 targets on mobile devices (iOS, Android), there's often a greater emphasis on balancing performance with power efficiency and code size.
    *   Optimization levels (`-Osize`, `-Os` in addition to `-O2`/`-O3`) will be important.
    *   Techniques like Link-Time Optimization (LTO) can be particularly beneficial.
*   **NEON SIMD**: As mentioned, ensuring effective auto-vectorization for NEON is a key performance goal for data-parallel code.

### 3.5. Debug Information

*   **DWARF Generation**: Generation of DWARF debug information for ARM-64 targets, compatible with debuggers like LLDB and GDB on ARM-64 platforms. This follows the principles in `BACKEND_LLVM_X86-64.md` (Section 6).
*   Ensuring accurate source mapping, variable tracking, and call stack information.

### 3.6. Open Questions/TBD (ARM-64)

*   **(ARM-TARGET-TRIPLES-1)**: Final list of officially supported ARM-64 target triples for Linux, macOS, and Windows for v0.1, and the default selection logic.
*   **(ARM-SPECIFIC-INTRINSICS-1)**: Strategy for exposing specific ARM-64 intrinsics (e.g., advanced NEON, cryptography extensions) to Ferra code if high-level abstractions or LLVM auto-vectorization are insufficient for certain performance-critical tasks. This is likely future work beyond v0.1.
*   **(ARM-ABI-VARIANTS-1)**: Investigation and handling of any subtle C ABI differences or calling convention nuances between the various ARM-64 OS targets (Linux variants, macOS, Windows) that might affect FFI. LLVM typically manages this, but verification is needed.
*   **(ARM-TESTING-HW-1)**: Availability of diverse ARM-64 hardware (or accurate emulators) for comprehensive CI testing across different OSes and CPU vendors (e.g., Apple Silicon, Qualcomm Snapdragon, AWS Graviton).

## 4. Apple Bitcode Backend Specification (Step 3.6.2)

This section details Ferra's approach to generating Apple Bitcode. Bitcode is an intermediate representation of a compiled program, used by Apple for apps submitted to the App Store for iOS, watchOS, tvOS, and (optionally, historically) macOS. Submitting Bitcode allows Apple to re-optimize the application binary for specific device architectures and future processor improvements without requiring a new submission from the developer.

### 4.1. Target Platforms

*   **Primary Targets**: iOS, watchOS, tvOS.
*   **Secondary Target**: macOS (where Bitcode is supported and makes sense for distribution, e.g., Mac App Store).

The primary driver for Bitcode support is enabling Ferra applications, especially those built with the `UI_DSL_MOBILE.md`, to be distributed through Apple's App Stores.

### 4.2. Rationale

*   **App Store Submission**: Historically, Bitcode was required for watchOS and tvOS apps, and encouraged for iOS. While policies evolve (e.g., Bitcode is no longer strictly required for iOS as of Xcode 14), providing it can still offer benefits.
*   **Future-Proofing**: Allows Apple to recompile and optimize the app for new device instruction sets or architectures without developer intervention.
*   **App Thinning (Indirectly)**: While Bitcode itself isn't app thinning, it enables Apple to generate optimized executables for different devices, contributing to smaller app sizes for end-users.

Ferra will aim to provide an option to embed Bitcode in its compiled artifacts for Apple platforms.

### 4.3. LLVM Integration

Apple Bitcode is a specific variant of LLVM Intermediate Representation (IR). Therefore, generating Bitcode naturally fits within Ferra's LLVM-based backend strategy.

*   **LLVM IR as a Prerequisite**: Ferra code is first compiled to standard LLVM IR, as detailed in `BACKEND_LLVM_X86-64.md` (which applies conceptually to ARM-64 targets as well).
*   **Apple's LLVM Toolchain**:
    *   Generation of Apple-compatible Bitcode typically requires using **Apple's version of Clang and LLVM**, which are distributed with Xcode. Standard open-source LLVM might not produce Bitcode in the exact format or with the metadata Apple expects. (TBD BITCODE-LLVM-VER-1)
    *   The Ferra build system will need to be able to detect and use the LLVM toolchain provided by Xcode when targeting Bitcode emission.
*   **Compiler Flags**:
    *   Specific compiler flags are passed to Clang/LLVM to instruct it to embed Bitcode in the output object files. A common flag is `-fembed-bitcode`.
    *   For libraries, `-fembed-bitcode-marker` might be used for placeholder Bitcode, with full Bitcode embedded in executables or final linked products. Ferra will aim for full Bitcode embedding where appropriate.

### 4.4. Process

The generation of Apple Bitcode within the Ferra compilation flow would generally involve:

1.  **Ferra Source -> Ferra IR**: Standard frontend and mid-end compilation.
2.  **Ferra IR -> LLVM IR**: Using the existing Ferra-to-LLVM-IR lowering passes (as used for x86-64 and ARM-64 native targets).
3.  **LLVM IR -> Object File with Embedded Bitcode**:
    *   The Ferra compiler (or its build system integration) invokes Apple's Clang (or an LLVM static compiler tool like `llc` configured appropriately) with the generated LLVM IR.
    *   The crucial step is passing the `-fembed-bitcode` flag (or equivalent). This tells the LLVM backend to compile the LLVM IR to machine code for the target architecture (e.g., ARM-64 for iOS) AND to also embed the LLVM Bitcode itself (in Apple's specific format) within a dedicated section (e.g., `__LLVM,__bitcode`) of the output object files (`.o`).
4.  **Linking**:
    *   When these object files are linked together (e.g., to create a static library, dynamic library, or executable), the linker (Apple's `ld`) consolidates the Bitcode sections.
    *   The final linked product (e.g., an app binary within an `.ipa`) will contain this embedded Bitcode.

### 4.5. Compatibility and Restrictions

*   **Adherence to Apple's Requirements**:
    *   The generated Bitcode must be compatible with the versions and formats expected by Apple's App Store processing. This often means aligning with the LLVM version used by the current stable Xcode release.
    *   Apple may impose restrictions on the LLVM IR constructs or features that are permissible in Bitcode submitted to the store. The Ferra compiler must ensure its generated LLVM IR is compatible.
*   **Limitations on Low-Level Features**:
    *   Using inline assembly or other very low-level, non-portable constructs in Ferra code might prevent those code sections from being represented in Bitcode, or might cause Bitcode generation to fail.
    *   Ferra code intended for Bitcode-enabled targets should generally avoid such constructs or provide alternative, higher-level implementations.
*   **Build Flags and Configurations**: The Ferra build system must correctly set all necessary SDK paths, target triples (e.g., `arm64-apple-ios15.0`), and compiler/linker flags required by Apple's toolchain for Bitcode-compatible builds.

### 4.6. Toolchain Management (TBD EXP-BACKEND-TOOLCHAIN-1)

*   As mentioned, generating valid Apple Bitcode typically requires Xcode and its embedded Apple Clang/LLVM.
*   The Ferra build system (`lang build --target aarch64-apple-ios --emit-bitcode`) needs to:
    *   Detect if Xcode command-line tools are installed and accessible.
    *   Invoke the correct Clang/LLVM executables from the Xcode toolchain with the appropriate flags.
    *   This is part of the broader TBD EXP-BACKEND-TOOLCHAIN-1 concerning management of external SDKs and toolchains.
*   **Manifest Configuration (`Ferra.toml`) for Apple Targets (Conceptual)**:
    *   Specific settings for Apple targets, including Bitcode generation, might be configurable in `Ferra.toml` under a target-specific section.
        ```toml
        [package]
        name = "my_ios_app_lib"
        # ...

        [target.aarch64-apple-ios]
        # sdk_path = "/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk"
        # min_ios_version = "15.0"
        # enable_bitcode = true # Could be a build flag or a manifest setting

        [target.aarch64-apple-darwin]
        # enable_bitcode = false # e.g., default to false for macOS native unless specified
        ```
    *   This allows project-specific overrides or configurations for Apple platform builds. The Ferra build system would use these to set appropriate compiler and linker flags.

### 4.7. Interaction with UI-DSL for Mobile (`UI_DSL_MOBILE.md`)

*   For Ferra applications developed using the UI-DSL and targeting iOS, watchOS, or tvOS, this Bitcode generation backend is essential for App Store distribution.
*   The Ferra native library component (containing application logic and UI state management) would be compiled with Bitcode enabled, and then linked into the standard Xcode project structure which also includes the Swift UI code generated from the Ferra UI-DSL (as per `UI_DSL_MOBILE.md` Section 3.3).

### 4.8. Open Questions/TBD (Apple Bitcode)

*   **(BITCODE-LLVM-VER-1)**: Determine the specific range of Apple Clang/LLVM versions that Ferra should target for generating compatible Bitcode for current App Store submission policies. How to handle changes in Apple's required versions.
*   **(BITCODE-MARKERS-1)**: Investigate if any special metadata, markers, or specific LLVM IR annotations (beyond standard `-fembed-bitcode`) are needed or beneficial for Apple's Bitcode processing or re-optimization.
*   **(BITCODE-VALIDATION-1)**: Tools or processes for validating Ferra-generated Bitcode for App Store compatibility before submission (e.g., using `otool` or other Apple-provided utilities to inspect embedded Bitcode sections).
*   **(BITCODE-FAT-BINARIES-1)**: How Bitcode generation interacts with the creation of "fat binaries" or XCFrameworks that might contain slices for multiple Apple platforms (e.g., iOS device, iOS simulator) or architectures.

Successful Apple Bitcode generation will enable Ferra developers to reach the broad audience on Apple's platforms using the App Store.

## 5. Android App Bundle (AAB) Generation (Step 3.6.3)

This section details Ferra's strategy for packaging applications as Android App Bundles (AABs). AAB is Google Play's standard publishing format, offering optimized delivery and smaller app sizes for users.

### 5.1. Target Platform

*   **Android**: All versions of Android supported by the AAB format and targeted by Ferra applications.

### 5.2. Rationale

*   **Google Play Requirement/Recommendation**: AAB is the preferred (and for new apps, often required) publishing format for Google Play.
*   **Optimized Delivery (Dynamic Delivery)**: Google Play uses the AAB to generate and serve optimized APKs tailored to each user's device configuration (e.g., CPU architecture, screen density, language), reducing download size.
*   **Support for Multiple ABIs**: AABs naturally support including native libraries (`.so` files) for various Android ABIs (e.g., `arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86`), and Play handles delivering the correct ones.
*   **Future Features**: Enables use of Play Feature Delivery (dynamic feature modules) and Play Asset Delivery, though these are more advanced features beyond initial v0.1 AAB support.

### 5.3. AAB Structure Overview

An AAB (`.aab` file) is a publishing format that includes all of an app's compiled code and resources. Key components relevant to Ferra:

*   **Base Module**: Contains the core application code and resources, including:
    *   `AndroidManifest.xml`
    *   Compiled DEX files (from Java/Kotlin code, e.g., UI code generated from Ferra UI-DSL).
    *   Resources (`res/` directory).
    *   Assets (`assets/` directory).
    *   Native libraries (`lib/<ABI>/lib<ferra_app_name>.so`).
*   **Configuration APKs**: Generated by Google Play from the AAB to target specific device configurations. Not directly created by Ferra, but the AAB must contain the necessary information for their generation.
*   **Dynamic Feature Modules (Future)**: Optional modules that can be downloaded on demand. Not a focus for Ferra's initial AAB support.

Ferra's AAB generation will focus on correctly packaging the base module, especially the Ferra-compiled native libraries.

### 5.4. Ferra Integration for AAB Generation

The Ferra build system (`lang build`) will be responsible for orchestrating the creation of an AAB file.

*   **Input to AAB Generation Process**:
    1.  **Compiled Ferra Native Libraries**: Ferra code (application logic, potentially parts of the UI-DSL implementation if native) compiled into shared object (`.so`) files for all targeted Android ABIs (e.g., `arm64-v8a`, `armeabi-v7a`). This relies on the ARM-64 backend (Section 3) and potentially an ARM-32 backend if supported.
    2.  **Android Application Manifest (`AndroidManifest.xml`)**: Either provided by the developer or generated by Ferra tooling based on `Ferra.toml` settings.
    3.  **Java/Kotlin Code (if any)**: For Android apps using Ferra UI-DSL, this includes the Kotlin/Jetpack Compose UI code generated by the Ferra compiler (see `UI_DSL_MOBILE.md` Section 4). For purely native Ferra apps, this might be minimal (e.g., a basic Activity to load the Ferra library).
    4.  **Resources and Assets**: Standard Android `res/` and `assets/` directories provided by the developer.

*   **Tooling**:
    *   **Ferra Build System (`lang build`)**:
        *   A command like `lang build --target android --package-format=aab` would initiate the process.
        *   It will compile Ferra code for the necessary Android ABIs.
        *   It will assemble all components into the structure required by `bundletool`.
    *   **Android SDK Tools**:
        *   **`bundletool`**: This is Google's command-line tool for building AABs from compiled code and resources, and also for generating APK sets from AABs for testing. Ferra's build process will invoke `bundletool build-bundle`.
        *   **Android NDK**: Required for compiling Ferra code to native `.so` libraries for Android ABIs. The build system needs to integrate with the NDK's C/C++ toolchains (even if Ferra is generating LLVM IR, LLVM needs to be configured for Android targets using NDK headers/libraries).
    *   **Gradle Integration (TBD AAB-GRADLE-1)**:
        *   **Option 1 (Direct `bundletool` Invocation)**: Ferra's build system could directly prepare all inputs for `bundletool` and invoke it. This gives Ferra more control but means reimplementing some aspects of Android project assembly.
        *   **Option 2 (Generate a Gradle Project)**: Ferra could generate a standard Android Gradle project structure, place its compiled `.so` files and generated Kotlin code into it, and then let Gradle handle the AAB building (as Gradle ultimately calls `bundletool`). This leverages Android's standard build system but adds a dependency on Gradle understanding.
        *   **v0.1 Leaning**: Generating a minimal Gradle project structure that primarily serves to package pre-compiled Ferra native libraries and any generated Kotlin UI code might be the most pragmatic approach, letting Gradle handle the complexities of resource processing and AAB packaging via `bundletool`.

*   **Process**:
    1.  **Compile Ferra Code**: The `lang build` process compiles the Ferra application logic into `.so` files for each targeted Android ABI (e.g., `arm64-v8a`, `armeabi-v7a`). This uses the ARM-64 (and potentially ARM-32) backend targeting Android.
    2.  **Generate/Prepare Android Manifest**: Ensure a valid `AndroidManifest.xml` is available.
    3.  **Compile/Include Java/Kotlin Code**: If using Ferra UI-DSL, the generated Kotlin/Compose code is compiled. Any other Java/Kotlin shim code is also compiled.
    4.  **Assemble Inputs**: Organize the `.so` files (in `lib/<ABI>/`), DEX files, manifest, resources, and assets into the directory structure expected by `bundletool` (or place them correctly in a generated Gradle project).
    5.  **Invoke `bundletool`**: Use `bundletool build-bundle --modules base.zip --output myapp.aab --config BuildProto.pb` (or equivalent Gradle task) to create the final `.aab` file. The `base.zip` would contain the module contents, and `BuildProto.pb` might specify configurations like ABI filters if not handled by directory structure alone.

### 5.5. Manifest and Configuration (`Ferra.toml` and `AndroidManifest.xml`)

*   **`Ferra.toml` Android Settings**:
    *   A new section, e.g., `[target.android]`, in `Ferra.toml` could specify AAB-related metadata and build configurations:
        ```toml
        [package]
        name = "my_ferra_android_app"
        # ...

        [target.android]
        application_id = "com.example.myferraapp"
        version_code = 1
        version_name = "1.0.0"
        min_sdk_version = 21
        target_sdk_version = 33
        # Permissions used by Ferra native code would still be in [package.permissions]
        # but these might inform parts of AndroidManifest.xml
        ```
*   **`AndroidManifest.xml` (TBD AAB-MANIFEST-1)**:
    *   **Generation vs. User-Provided**:
        *   Option A: Ferra tooling generates a basic `AndroidManifest.xml` based on `Ferra.toml` settings and conventions (e.g., for the main activity that loads the Ferra library).
        *   Option B: The developer provides a full `AndroidManifest.xml` in their project.
        *   Option C (Hybrid): Ferra generates a manifest but allows the developer to provide overrides or additions.
    *   **Content**: Must declare application components (activities, services), permissions (Android permissions, distinct from Ferra's `manifest.perms` but potentially informed by them), hardware features, etc.
    *   If Ferra code is primarily native logic loaded by a standard Android Activity (Java/Kotlin), the manifest might be quite simple. If using the Ferra UI-DSL, the generated Kotlin activities would need to be declared.

### 5.6. Interaction with UI-DSL for Mobile (`UI_DSL_MOBILE.md`)

*   Android AAB generation is the primary delivery mechanism for Ferra applications built using the UI-DSL for Android.
*   The process involves:
    1.  Ferra UI-DSL code compiled to Kotlin/Jetpack Compose UI code (as per `UI_DSL_MOBILE.md` Section 4).
    2.  Ferra application logic compiled to native `.so` libraries.
    3.  Both the generated Kotlin and the Ferra `.so` libraries are packaged into the AAB.
    4.  The `AndroidManifest.xml` will need to declare the main Activity that hosts the Compose UI.

### 5.7. Native Library (`.so`) Considerations

*   **ABI Support**: The build system must correctly compile Ferra code for each required Android ABI (e.g., `arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86`). The `lang build --target android` command might default to a common set (e.g., `arm64-v8a`, `armeabi-v7a`) or allow explicit ABI specification.
*   **Naming**: Native libraries for Ferra code should follow standard Android naming conventions (e.g., `lib<module_name>.so`).
*   **Stripping**: Native libraries should be stripped of unnecessary debug symbols for release AABs to reduce size (standard practice with NDK builds).
*   **Dependencies**: If Ferra native code itself has FFI dependencies on other pre-compiled `.so` files, these must also be correctly packaged into the AAB under the appropriate ABI directories. `PACKAGE_MANAGER_SPEC.md`'s handling of native dependencies is relevant here.

### 5.8. Signing AABs

*   AABs must be cryptographically signed with an upload key before they can be uploaded to Google Play. Google Play then re-signs the derived APKs with an app signing key.
*   **Ferra Tooling Support**:
    *   The `lang build ... --package-format=aab` command itself might not perform the signing directly.
    *   Instead, it will produce an unsigned AAB (e.g., `myapp-unsigned.aab`).
    *   Developers would then use standard Android tooling to sign the AAB with their upload key. For example, using `jarsigner` (though `apksigner` is more modern for APKs, `bundletool` handles AAB signing implicitly if configured, or a separate signing step might be needed for the raw AAB if not using Gradle for this part):
        ```bash
        # Example using jarsigner (less common for AABs directly, usually for APKs within bundle if self-generating)
        # More typically, if using Gradle, signing is configured in build.gradle.
        # If bundletool is used and it doesn't sign, one might sign the generated APKs from the AAB.
        # For direct AAB signing if needed and supported by a tool:
        # hypothetical_aab_signer --keystore my-release-key.keystore --ks-key-alias my-alias \
        #                          --ks-pass pass:password --key-pass pass:password \
        #                          myapp-unsigned.aab myapp-signed.aab 
        # NOTE: The standard Android workflow is to configure signing in Gradle, which then correctly signs the AAB.
        # Ferra documentation will point to standard Android procedures for AAB signing.
        ```
    *   Ferra documentation must guide users through the standard Android signing process, which usually involves configuring signing details in a Gradle build if a Gradle project is used or generated by Ferra for AAB packaging.

### 5.9. Open Questions/TBD (Android AAB)

*   **(AAB-GRADLE-1)**: **Gradle Integration Strategy**: Final decision on the level of Gradle integration:
    *   Does Ferra generate a full Gradle project that developers then build?
    *   Does Ferra tooling invoke `bundletool` directly, requiring less reliance on Gradle from the Ferra user's perspective for pure Ferra apps?
    *   How does this integrate if the user *already* has an Android Gradle project and wants to add a Ferra native library component?
*   **(AAB-MANIFEST-1)**: **`AndroidManifest.xml` Management**: Definitive strategy for `AndroidManifest.xml` - fully generated, user-provided, or merged/templated? How are Android permissions (distinct from Ferra `manifest.perms`) handled?
*   **(AAB-SPLIT-1)**: **Dynamic Feature Module Support**: Initial v0.1 support will focus on the base module. When and how to support packaging Ferra code into dynamic feature modules is future work.
*   **(AAB-RESOURCES-1)**: Handling of standard Android resources (`res/` drawable, layouts, strings, etc.) if a Ferra project is not primarily using the Ferra UI-DSL (e.g., a Ferra native library for an existing Android app). How are these incorporated if Ferra tooling is driving the AAB creation?
*   **(AAB-NDK-VER-1)**: Specifying recommended or required Android NDK versions for compatibility.

Generating AABs involves interfacing significantly with the Android build ecosystem. The goal for Ferra is to make this as streamlined as possible for developers, especially when also using the Ferra UI-DSL.

## 6. Cross-Cutting Concerns

While each expanded backend target (ARM-64, Apple Bitcode, Android AAB) has its unique considerations, several cross-cutting concerns apply to the overall effort of broadening Ferra's platform support.

*   **Error Reporting and Diagnostics**:
    *   The Ferra compiler must provide clear, target-specific diagnostic messages when errors occur during compilation or packaging for these new backends (e.g., issues with LLVM's ARM-64 codegen, Bitcode incompatibility, AAB packaging failures).
    *   These diagnostics should adhere to the principles in `DESIGN_DIAGNOSTICS.md`.
    *   New diagnostic codes might be needed in `diagnostic_codes.md` for target-specific issues.

*   **Debugging Experience**:
    *   **ARM-64 Native**: Debugging Ferra code on ARM-64 (Linux, macOS, Windows) should be supported via DWARF and standard debuggers (LLDB, GDB), similar to x86-64.
    *   **Apple Bitcode**: When Bitcode is recompiled by Apple, the debugging experience relies on Apple's toolchain (Xcode, LLDB) and the dSYMs generated. Ferra's initial DWARF generation must be compatible with this process.
    *   **Android AAB (Native Libraries)**: Debugging Ferra native code (`.so` files) within an Android app typically involves using Android Studio's native debugging capabilities, which interface with LLDB. Ferra must generate appropriate debug symbols for NDK compatibility.
    *   Consistent source mapping across all targets is crucial for a good debugging UX.

*   **Performance Profiling and Optimization**:
    *   Once code is running on these new targets, developers will need ways to profile its performance.
    *   Ferra's own Energy Profiler (`ENERGY_PROFILER.md`) might need target-specific models or considerations for ARM-64.
    *   Integration with platform-specific profiling tools (e.g., Xcode Instruments for Apple platforms, Android Studio Profiler, Linux `perf` for ARM-64) should be considered and documented.
    *   Optimization strategies in the Ferra compiler (especially within LLVM IR generation) should be evaluated for their effectiveness on ARM-64, considering differences in CPU architecture (e.g., register counts, instruction pipelines) compared to x86-64.

*   **Build Time**: Supporting multiple complex backends can increase overall compiler build times and testing matrix complexity. Strategies for efficient compilation and testing (e.g., CI caching, selective testing) will be important.

*   **Documentation**: Comprehensive documentation will be required for developers targeting these platforms, covering:
    *   Target-specific build configurations in `Ferra.toml`.
    *   Toolchain setup (Xcode, Android NDK/SDK, ARM-64 cross-compilers).
    *   Debugging guides.
    *   Platform-specific FFI or runtime considerations.
    *   App store submission guidelines (for Bitcode and AABs).

## 7. Future Work

The v0.1 specification for these expanded backends lays the groundwork. Future enhancements could include:

*   **Broader ARM Architecture Support**:
    *   Explicit support and optimization for more specific ARM microarchitectures (e.g., different Cortex series, server-grade Neoverse).
    *   Support for 32-bit ARM targets (`armeabi-v7a` for Android) if strong demand exists, though the industry is heavily shifting to 64-bit.
*   **Other Native Architectures**: Consideration for other native CPU targets beyond x86-64 and ARM-64 (e.g., RISC-V) as Ferra matures and ecosystem demand grows.
*   **More Sophisticated Mobile Packaging**:
    *   Support for Android Dynamic Feature Modules and Play Asset Delivery.
    *   Deeper integration with Xcode build settings for Apple platforms (e.g., managing entitlements, code signing identities more directly from Ferra tooling if feasible).
    *   Support for other Apple platforms more explicitly if Bitcode generation proves insufficient (e.g., direct native compilation for macOS ARM-64 without mandatory Bitcode for non-App Store distribution).
*   **Enhanced Cross-Compilation Support**: More streamlined and automated setup for cross-compilation toolchains and sysroots.
*   **Performance Parity and Optimization**: Continuous work to ensure Ferra applications perform competitively on these expanded targets compared to other native languages.
*   **Specialized Standard Library Features**: Platform-specific modules in the standard library that expose unique capabilities of ARM, iOS, or Android in a safe, idiomatic Ferra way.

## 8. Open Questions / TBD (Overall for Expanded Backends)

This section consolidates the key "To Be Determined" items that apply across the design of these expanded backends. Specific TBDs for each backend are listed in their respective sections (3.6, 4.8, 5.9).

*   **(EXP-BACKEND-RUNTIME-1)**: **Target-Specific Runtime Adjustments**:
    *   Finalizing any necessary modifications or conditional compilation within Ferra's core runtime (e.g., memory allocator, panic handler, thread primitives if used by stdlib) to ensure correct and optimal behavior on ARM-64 (Linux, macOS, Windows), in Apple Bitcode environments, and within Android AAB/NDK contexts.
*   **(EXP-BACKEND-CI-1)**: **Comprehensive CI Setup**:
    *   Detailed plan for CI infrastructure to build, run tests (potentially on emulators or target hardware), and validate artifacts (Bitcode, AABs) for all supported variants of these expanded targets.
*   **(EXP-BACKEND-TOOLCHAIN-1)**: **Developer Toolchain Management Strategy**:
    *   How will Ferra guide developers in installing and configuring necessary external toolchains (specific LLVM versions for Bitcode, correct Android NDK/SDK versions, ARM-64 cross-compilers)?
    *   Will Ferra's build tools attempt to auto-detect or manage any part of these external toolchains?
*   **(EXP-BACKEND-FFI-TEST-1)**: **FFI Testing on New Targets**:
    *   A comprehensive FFI testing strategy to ensure C FFI as defined in `FFI_C_CPP.md` works reliably across all new target architecture and OS combinations, particularly regarding calling conventions and data layout.
*   **(EXP-BACKEND-PERF-BENCH-1)**: **Performance Benchmarking Strategy**:
    *   Establishing a set of relevant benchmarks and a methodology for comparing Ferra's performance on ARM-64 against x86-64 and potentially other languages on ARM.
*   **(EXP-BACKEND-DEFAULT-TARGETS-1)**: **Default Build Targets**:
    *   Defining which of these expanded targets Ferra will build for by default if a developer simply types `lang build` on a given host (e.g., on an ARM-64 macOS host, does it default to native ARM-64, or still x86-64 for wider initial compatibility unless specified?).
    *   Policy for `lang new` templates regarding default target configurations.

Addressing these overall TBDs, along with the specific ones for each backend, will be crucial for a successful and maintainable implementation of Ferra's expanded platform support.
---
This document will specify Ferra's approach to supporting ARM-64, Apple Bitcode, and Android App Bundles.