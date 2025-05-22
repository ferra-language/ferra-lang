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
    *   The design for ARM-64 and Apple Bitcode generation will heavily leverage Ferra's existing LLVM integration, as detailed in `BACKEND_LLVM_X86-64.md` (see §3.1). Many principles of IR-to-LLVM-IR conversion, optimization, and runtime interaction will be applicable.
    *   For Android AAB generation, packaging considerations will draw upon insights from `BACKEND_WASM_WASI.md` (see §2.3) and will interact closely with the `PACKAGE_MANAGER_SPEC.md` (see §3.2) for managing build artifacts and potentially Android-specific dependencies.

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

### 2.1 CLI Grammar

The `lang build` command supports various options for targeting different platforms and output formats. Here is the formal grammar:

```ebnf
BuildCmd         ::= "lang" "build" Option*
Option           ::= TargetOption | EmitOption | PackageOption | BuildFlag
TargetOption     ::= "--target" "=" TARGET_TRIPLE
EmitOption       ::= "--emit" "=" ("ir"|"bc"|"llvm-ir"|"asm"|"lib"|"bin")
                 | "--emit-apple-bitcode"
PackageOption    ::= "--package-format" "=" ("native_executable"|"ipa"|"aab")
TARGET_TRIPLE    ::= STRING_LITERAL
BuildFlag        ::= "--release" | "--opt-level" "=" ("s"|"z"|"2"|"3")
```

### 2.2 Toolchain Detection and Management

The Ferra build system follows this process to locate and validate required toolchains:

```ascii
lang build --target <triple>
    │
    ├─► Check Ferra.toml for target-specific settings
    │   (see §4.6 for Apple, §5.5 for Android)
    │
    ├─► Detect Required Toolchains:
    │   ├─► LLVM/Clang: Check PATH, LLVM_HOME, or Ferra.toml
    │   ├─► Xcode Tools: Check xcode-select path for Apple targets
    │   └─► Android NDK: Check ANDROID_NDK_HOME or Ferra.toml
    │
    ├─► Validate Toolchain Versions:
    │   ├─► LLVM ≥ 17.0 (see BACKEND_LLVM_X86-64.md §2.1)
    │   ├─► Xcode ≥ 14.0 for Bitcode (see §4.3)
    │   └─► NDK ≥ r25c for Android (see §5.4)
    │
    └─► Report Errors via DESIGN_DIAGNOSTICS.md schema
        if any toolchain is missing or incompatible
```

### 2.3 CI Integration

The Ferra build system can be integrated into CI pipelines to verify multi-target builds. Here's an example GitHub Actions workflow:

```yaml
# Example: .github/workflows/multi-target.yml
name: Multi-Target Build

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - aarch64-unknown-linux-gnu
          - aarch64-apple-darwin
          - aarch64-apple-ios
          - aarch64-linux-android

    steps:
      - uses: actions/checkout@v4
      
      - name: Setup LLVM
        uses: llvm/llvm-action@v1
        with:
          version: '17.0'
          
      - name: Setup Android NDK
        if: contains(matrix.target, 'android')
        uses: android-actions/setup-android@v3
        with:
          ndk-version: '25.2.9519653'
          
      - name: Setup Xcode
        if: contains(matrix.target, 'apple')
        uses: maxim-lobanov/setup-xcode@v1
        with:
          xcode-version: '14.3.1'
          
      - name: Build
        run: |
          lang build --target ${{ matrix.target }} --release
          
      - name: Package
        if: contains(matrix.target, 'apple') || contains(matrix.target, 'android')
        run: |
          if [[ "${{ matrix.target }}" == *"apple"* ]]; then
            lang build --target ${{ matrix.target }} --emit-apple-bitcode
          elif [[ "${{ matrix.target }}" == *"android"* ]]; then
            lang build --target ${{ matrix.target }} --package-format=aab
          fi
```

Add this badge to your README.md:
```markdown
[![Multi-Target Build](https://github.com/your-org/ferra-lang/actions/workflows/multi-target.yml/badge.svg)](https://github.com/your-org/ferra-lang/actions/workflows/multi-target.yml)
```

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

#### Target Triple Reference Table

| Platform Description         | Target Triple         | Primary Use Cases & Notes                                  | Status for v0.1 |
|------------------------------|-----------------------|------------------------------------------------------------|-----------------|
| Linux (GNU) ARM-64           | `aarch64-unknown-linux-gnu`   | Servers, embedded, general Linux on ARM-64.                | [RESOLVED] Primary target |
| Linux (MUSL) ARM-64          | `aarch64-unknown-linux-musl`  | Lightweight/static-linked Linux on ARM-64.                 | [RESOLVED] Secondary target |
| macOS (Apple Silicon) ARM-64 | `aarch64-apple-darwin`        | Native macOS applications on M-series chips.               | [RESOLVED] Primary target |
| Windows on ARM-64            | `aarch64-pc-windows-msvc`     | Native Windows applications on ARM-64.                     | [RESOLVED] Secondary target |
| iOS ARM-64                   | `aarch64-apple-ios<version>`  | iOS applications (often involves Bitcode). Example: `aarch64-apple-ios15.0`. | [RESOLVED] Via Bitcode |
| Android ARM-64 (arm64-v8a)   | `aarch64-linux-android`       | Native libraries for Android `arm64-v8a` ABI.              | [RESOLVED] Via AAB |
| Android ARM-32 (armeabi-v7a) | `armv7a-linux-androideabi`    | Native libraries for older Android `armeabi-v7a` ABI.      | [RESOLVED] Deferred to v0.2 |

#### Quick Start Example

Here's a minimal example of building and running a Ferra program for ARM-64 Linux:

```bash
# Create a new Ferra project
lang new hello_arm64
cd hello_arm64

# Write a simple program
cat > src/main.lang << 'EOF'
fn main() {
    println("Hello from ARM-64!")
}
EOF

# Build for ARM-64 Linux
lang build --target aarch64-unknown-linux-gnu --release

# Run the program (on ARM-64 Linux)
./hello_arm64
```

For cross-compilation from x86-64 to ARM-64, you'll need the appropriate toolchain installed (see §2.2 for toolchain detection).

### 3.2. LLVM Integration

The ARM-64 backend will primarily leverage LLVM, building upon the existing infrastructure outlined in `BACKEND_LLVM_X86-64.md` (see §3.1). Many principles of IR-to-LLVM-IR conversion, optimization, and runtime interaction will be applicable.

*   **LLVM Backend Re-use**: The core logic for Ferra IR to LLVM IR translation (see `BACKEND_LLVM_X86-64.md` §3.2) will be largely reused. LLVM itself handles the target-specific instruction selection and code generation for ARM-64 once it receives generic LLVM IR.
*   **Target-Specific LLVM Features**:
    *   The Ferra compiler will instruct LLVM to target the appropriate ARM-64 sub-architecture and CPU features (e.g., via `-mcpu` and `-mattr` LLVM flags if exposed, or by configuring the LLVM TargetMachine).
    *   **NEON SIMD**: LLVM's auto-vectorizer should be able to target ARM NEON SIMD instructions for data-parallel `for_each` constructs (as discussed in `DATA_PARALLEL_GPU.md` §3.1). Specific NEON intrinsics might be exposed in Ferra's standard library in the future if high-level auto-vectorization is insufficient for critical performance needs (TBD ARM-SPECIFIC-1).
*   **Calling Conventions**:
    *   Adherence to the **AAPCS64 (Procedure Call Standard for the ARM 64-bit Architecture)** is critical for C FFI compatibility and general OS interaction. LLVM handles this when configured for an ARM-64 target. See `FFI_C_CPP.md` §2.1 for detailed ABI requirements.
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

*   **DWARF Generation**: Generation of DWARF debug information for ARM-64 targets, compatible with debuggers like LLDB and GDB on ARM-64 platforms. This follows the principles in `BACKEND_LLVM_X86-64.md` (see §6.2).
    *   Ensuring accurate source mapping, variable tracking, and call stack information.

### 3.6. Open Questions/TBD (ARM-64)

*   **TBD ARM-SPECIFIC-1**: Should Ferra's standard library expose specific ARM NEON SIMD intrinsics for high-performance data-parallel operations, or rely solely on LLVM's auto-vectorizer for `for_each` constructs? This is a performance optimization question that might be addressed in future versions. [RESOLVED for v0.1: Rely on LLVM auto-vectorizer]
*   **TBD ARM-TARGET-TRIPLES-1**: Final list of officially supported ARM-64 target triples for Linux, macOS, and Windows for v0.1, and the default selection logic. [RESOLVED for v0.1: See Target Triple Reference Table in §3.1]
*   **TBD ARM-SPECIFIC-INTRINSICS-1**: Strategy for exposing specific ARM-64 intrinsics (e.g., advanced NEON, cryptography extensions) to Ferra code if high-level abstractions or LLVM auto-vectorization are insufficient for certain performance-critical tasks. This is likely future work beyond v0.1.
*   **TBD ARM-ABI-VARIANTS-1**: Investigation and handling of any subtle C ABI differences or calling convention nuances between the various ARM-64 OS targets (Linux variants, macOS, Windows) that might affect FFI. LLVM typically manages this, but verification is needed.
*   **TBD ARM-TESTING-HW-1**: Availability of diverse ARM-64 hardware (or accurate emulators) for comprehensive CI testing across different OSes and CPU vendors (e.g., Apple Silicon, Qualcomm Snapdragon, AWS Graviton).

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
*   **Manifest Configuration (`Ferra.toml`) for Apple Targets**:
    *   Specific settings for Apple targets, including Bitcode generation, are configurable in `Ferra.toml` under a target-specific section (see `PACKAGE_MANAGER_SPEC.md` §3.2 for the full manifest schema):
        ```toml
        # Example Ferra.toml (Apple target settings)
        [package]
        name = "my_ios_app_lib"
        version = "0.1.0"
        authors = ["Your Name <your.email@example.com>"]

        [target.aarch64-apple-ios]
        sdk_path = "/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk"
        min_ios_version = "15.0"
        enable_bitcode = true  # Required for App Store submission

        [target.aarch64-apple-darwin]
        enable_bitcode = false  # Default to false for macOS native unless specified
        ```

### 4.7. Interaction with UI-DSL for Mobile (`UI_DSL_MOBILE.md`)

*   For Ferra applications developed using the UI-DSL and targeting iOS, watchOS, or tvOS, this Bitcode generation backend is essential for App Store distribution.
*   The Ferra native library component (containing application logic and UI state management) would be compiled with Bitcode enabled, and then linked into the standard Xcode project structure which also includes the Swift UI code generated from the Ferra UI-DSL (as per `UI_DSL_MOBILE.md` §3.3).

### 4.8. Open Questions/TBD (Apple Bitcode)

*   **TBD BITCODE-LLVM-VER-1**: Determine the specific range of Apple Clang/LLVM versions that Ferra should target for generating compatible Bitcode for current App Store submission policies. How to handle changes in Apple's required versions.
*   **TBD BITCODE-MARKERS-1**: Investigate if any special metadata, markers, or specific LLVM IR annotations (beyond standard `-fembed-bitcode`) are needed or beneficial for Apple's Bitcode processing or re-optimization.
*   **TBD BITCODE-VALIDATION-1**: Tools or processes for validating Ferra-generated Bitcode for App Store compatibility before submission (e.g., using `otool` or other Apple-provided utilities to inspect embedded Bitcode sections).
*   **TBD BITCODE-FAT-BINARIES-1**: How Bitcode generation interacts with the creation of "fat binaries" or XCFrameworks that might contain slices for multiple Apple platforms (e.g., iOS device, iOS simulator) or architectures.

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

An AAB (`.aab` file) is a publishing format that includes all of an app's compiled code and resources. Key components relevant to Ferra are detailed in `UI_DSL_MOBILE.md` §4.2.

### 5.4. Ferra Integration for AAB Generation

The Ferra build system (`lang build`) will be responsible for orchestrating the creation of an AAB file.

*   **Input to AAB Generation Process**:
    1.  **Compiled Ferra Native Libraries**: Ferra code (application logic, potentially parts of the UI-DSL implementation if native) compiled into shared object (`.so`) files for all targeted Android ABIs (e.g., `arm64-v8a`, `armeabi-v7a`). This relies on the ARM-64 backend (Section 3) and potentially an ARM-32 backend if supported.
    2.  **Android Application Manifest (`AndroidManifest.xml`)**: Either provided by the developer or generated by Ferra tooling based on `Ferra.toml` settings.
    3.  **Java/Kotlin Code (if any)**: For Android apps using Ferra UI-DSL, this includes the Kotlin/Jetpack Compose UI code generated by the Ferra compiler (see `UI_DSL_MOBILE.md` §4). For purely native Ferra apps, this might be minimal (e.g., a basic Activity to load the Ferra library).
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
    *   A new section, e.g., `[target.android]`, in `Ferra.toml` specifies AAB-related metadata and build configurations (see `PACKAGE_MANAGER_SPEC.md` §3.2 for the full manifest schema):
        ```toml
        # Example Ferra.toml (Android target settings)
        [package]
        name = "my_ferra_android_app"
        version = "0.1.0"
        authors = ["Your Name <your.email@example.com>"]

        [target.android]
        application_id = "com.example.myferraapp"
        version_code = 1
        version_name = "1.0.0"
        min_sdk_version = 21
        target_sdk_version = 33
        # Permissions used by Ferra native code would still be in [package.permissions]
        # but these might inform parts of AndroidManifest.xml
        ```

### 5.8. Signing AABs

*   AABs must be cryptographically signed with an upload key before they can be uploaded to Google Play. Google Play then re-signs the derived APKs with an app signing key.
*   **Ferra Tooling Support**:
    *   The `lang build ... --package-format=aab` command produces an unsigned AAB (e.g., `myapp-unsigned.aab`).
    *   Developers then use standard Android tooling to sign the AAB. Here's a concrete example using `bundletool` (the recommended approach):
        ```bash
        # Example: Signing an AAB with bundletool
        # 1. First, build the AAB
        lang build --target android --package-format=aab

        # 2. Sign the AAB using bundletool
        bundletool build-bundle \
            --modules=myapp-unsigned.aab \
            --output=myapp-signed.aab \
            --ks=my-release-key.keystore \
            --ks-pass=pass:your_keystore_password \
            --ks-key-alias=your_key_alias \
            --key-pass=pass:your_key_password

        # 3. Verify the signed AAB
        bundletool validate --bundle=myapp-signed.aab
        ```
    *   Alternatively, if using Gradle (see §5.4), signing is configured in `build.gradle`:
        ```groovy
        android {
            signingConfigs {
                release {
                    storeFile file("my-release-key.keystore")
                    storePassword "your_keystore_password"
                    keyAlias "your_key_alias"
                    keyPassword "your_key_password"
                }
            }
            buildTypes {
                release {
                    signingConfig signingConfigs.release
                }
            }
        }
        ```
    *   Ferra documentation will guide users through the standard Android signing process, which usually involves configuring signing details in a Gradle build if a Gradle project is used or generated by Ferra for AAB packaging.

## 7. Open Questions and TBDs

### ARM-64 TBDs
*   **TBD ARM-SPECIFIC-1**: Should Ferra's standard library expose specific ARM NEON SIMD intrinsics for high-performance data-parallel operations, or rely solely on LLVM's auto-vectorizer for `for_each` constructs? This is a performance optimization question that might be addressed in future versions. [RESOLVED for v0.1: Rely on LLVM auto-vectorizer]
*   **TBD ARM-TARGET-TRIPLES-1**: Final list of officially supported ARM-64 target triples for Linux, macOS, and Windows for v0.1, and the default selection logic. [RESOLVED for v0.1: See Target Triple Reference Table in §3.1]
*   **TBD ARM-SPECIFIC-INTRINSICS-1**: Strategy for exposing specific ARM-64 intrinsics (e.g., advanced NEON, cryptography extensions) to Ferra code if high-level abstractions or LLVM auto-vectorization are insufficient for certain performance-critical tasks. This is likely future work beyond v0.1.
*   **TBD ARM-ABI-VARIANTS-1**: Investigation and handling of any subtle C ABI differences or calling convention nuances between the various ARM-64 OS targets (Linux variants, macOS, Windows) that might affect FFI. LLVM typically manages this, but verification is needed.
*   **TBD ARM-TESTING-HW-1**: Availability of diverse ARM-64 hardware (or accurate emulators) for comprehensive CI testing across different OSes and CPU vendors (e.g., Apple Silicon, Qualcomm Snapdragon, AWS Graviton).

### Apple Bitcode TBDs
*   **TBD BITCODE-LLVM-VER-1**: Determine the specific range of Apple Clang/LLVM versions that Ferra should target for generating compatible Bitcode for current App Store submission policies. How to handle changes in Apple's required versions.
*   **TBD BITCODE-MARKERS-1**: Investigate if any special metadata, markers, or specific LLVM IR annotations (beyond standard `-fembed-bitcode`) are needed or beneficial for Apple's Bitcode processing or re-optimization.
*   **TBD BITCODE-VALIDATION-1**: Tools or processes for validating Ferra-generated Bitcode for App Store compatibility before submission (e.g., using `otool` or other Apple-provided utilities to inspect embedded Bitcode sections).
*   **TBD BITCODE-FAT-BINARIES-1**: How Bitcode generation interacts with the creation of "fat binaries" or XCFrameworks that might contain slices for multiple Apple platforms (e.g., iOS device, iOS simulator) or architectures.

### Android AAB TBDs
*   **TBD AAB-GRADLE-1**: Should Ferra's AAB generation use direct `bundletool` invocation or generate a Gradle project? [RESOLVED for v0.1: Generate minimal Gradle project for v0.1, with direct `bundletool` support planned for v0.2]
*   **TBD AAB-TESTING-1**: Strategy for testing AAB generation and validation in CI, including handling of signing keys and test certificates.
*   **TBD AAB-FEATURES-1**: Support for advanced AAB features like dynamic feature modules and Play Asset Delivery in future versions.

### Toolchain Management TBDs
*   **TBD EXP-BACKEND-TOOLCHAIN-1**: How should the Ferra build system manage and detect external toolchains (LLVM, Xcode, Android NDK) for these expanded targets? This includes version requirements, fallback mechanisms, and user configuration options. [RESOLVED for v0.1: See §2.2 for toolchain detection flowchart]
*   **TBD EXP-BACKEND-TOOLCHAIN-2**: Should Ferra provide built-in support for cross-compilation toolchains (e.g., building for Android ARM-64 from a Linux x86-64 host), or leave this to the user to set up? [RESOLVED for v0.1: Leave to user setup]
*   **TBD EXP-BACKEND-TOOLCHAIN-3**: How should Ferra handle platform-specific build flags and configurations (e.g., iOS deployment target, Android SDK version) in a unified way across different targets? [RESOLVED for v0.1: Via Ferra.toml target sections]
*   **TBD EXP-BACKEND-TOOLCHAIN-4**: What is the best way to integrate with platform-specific build systems (e.g., Xcode projects, Gradle) for these targets? [RESOLVED for v0.1: See §4.4 and §5.4 for integration approaches]

## 8. References

### Front-End
*   `DESIGN_LEXER.md`: Lexer design and implementation
*   `DESIGN_PARSER.md`: Parser design and implementation
*   `DESIGN_TYPE_INFERENCE.md`: Type inference system
*   `DESIGN_DIAGNOSTICS.md`: Diagnostic system design
*   `FRONTEND_ENHANCEMENTS.md`: Frontend improvements
*   `AST_SPECIFICATION.md`: Abstract Syntax Tree structure
*   `SYNTAX_GRAMMAR_V0.1.md`: Language grammar specification

### Mid-End
*   `IR_SPECIFICATION.md`: Intermediate representation specification
*   `AST_TO_IR_CONVERSION.md`: AST to IR conversion process
*   `IR_SEMANTIC_TAGS.md`: IR semantic tags
*   `OWNERSHIP_BORROW_CHECKER.md`: Ownership and borrowing rules
*   `CONCURRENCY_MODEL.md`: Concurrency and threading model

### Back-End
*   `BACKEND_LLVM_X86-64.md`: Core LLVM backend design and IR-to-LLVM conversion (see §3.1)
*   `BACKEND_WASM_WASI.md`: WebAssembly and WASI backend
*   `DATA_PARALLEL_GPU.md`: Data-parallel programming model and SIMD considerations
*   `FFI_C_CPP.md`: C ABI and calling conventions (see §2.1)

### Mobile & UI
*   `UI_DSL_MOBILE.md`: Mobile UI code generation and integration
*   `UI_DSL_ROADMAP.md`: UI DSL development roadmap

### Package Management & Security
*   `PACKAGE_MANAGER_SPEC.md`: Detailed Ferra.toml schema (see §3.2)
*   `SECURITY_MODEL.md`: Security considerations for mobile platforms
*   `ENERGY_PROFILER.md`: Energy consumption profiling and optimization

### Project Infrastructure
*   `PROJECT_OVERVIEW.md`: Project structure and manifest schema
*   `PROJECT_DOCS_MAP.md`: Overall documentation structure
*   `SPEC_OVERVIEW.md`: Specification overview and goals
*   `CODING_STANDARDS.md`: Code style and quality guidelines
*   `SOLIDIFICATION_CHECKLIST.md`: Implementation and testing checklist
*   `SELF_HOSTING_SUBSET.md`: Self-hosting compiler subset
*   `VSCODE_PLUGIN_ALPHA_SPEC.md`: IDE integration specifications

### AI Integration
*   `AI_API_AST.md`: AI API AST integration
*   `AI_API_REFACTOR_VERIFY.md`: AI API refactoring and verification

### Project Planning
*   `Steps.md`: Project implementation steps and timeline
*   `comprehensive_plan.md`: Comprehensive project plan