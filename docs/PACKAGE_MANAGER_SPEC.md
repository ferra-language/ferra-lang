# Ferra Package Manager (Beta) Specification v0.1

> **Status:** Initial Draft - Module 2.4 (Steps 2.4.1 - 2.4.3)
> **Last Reviewed:** 2024-07-29
> **Version:** 0.1

## 1. Introduction and Goals

*   To provide a secure, reliable, and developer-friendly system for managing Ferra project dependencies and distributing Ferra libraries and applications, primarily through the `lang` CLI tool.
*   Key design goals:
    *   Secure and verifiable package management.
    *   Content-addressed storage for de-duplication and integrity.
    *   Support for semantic versioning and precise dependency pinning (CID-based).
    *   Generation of Software Bill of Materials (SBOM) in SPDX format.
    *   Integration with Sigstore for package signing and verification.
    *   User-friendly CLI for common package operations.
*   Relationship to Ferra's build system, module system, and overall developer experience.

## 2. Content-Addressed Storage (Step 2.4.1)

Ferra's package manager will use content-addressed storage to ensure integrity, verifiability, and efficient de-duplication of package data across multiple projects. This approach aligns with modern secure software supply chain practices.

*   **2.1. Storage Location**:
    *   **Central Shared Cache**: As specified in `Steps.md` (Section 7), a global shared cache will be located at `~/.lang/pkg` (the exact path might be configurable, but this serves as the default convention).
    *   **Benefits**: This global cache allows different Ferra projects on the same system to share downloaded package dependencies, saving disk space and reducing redundant downloads.
*   **2.2. Package Identification via Content Identifiers (CIDs)**:
    *   **Primary Identifier**: Each version of a package and its constituent files will be identified by a Content Identifier (CID). A CID is derived from a cryptographic hash of the package's content.
    *   **Hash Algorithm**: A strong cryptographic hash algorithm will be used. **The default will be SHA-256** due to its wide adoption and security track record. Support for other algorithms like Blake3 may be considered in the future for performance benefits, potentially configurable via a setting like `lang config package.hash_algorithm=blake3` (details tracked in TBD TAG-PKG-4). The chosen algorithm(s) must prioritize collision resistance and integrity verification.
    *   **CID Generation**: CIDs are typically generated for normalized package archives (e.g., a gzipped tarball `.tar.gz` with consistent file ordering and metadata to ensure deterministic hashing). The CID string itself should be prefixed with the algorithm (e.g., `sha256-...`).
    *   **Verification**: When a package is retrieved, its CID can be re-calculated from its content and compared against the expected CID to verify integrity and ensure it hasn't been tampered with.
*   **2.3. Storage Layout within `~/.lang/pkg`**:
    *   **CID-Based Organization**: Packages will be stored in subdirectories derived from their CIDs to prevent naming collisions and allow direct lookup.
        *   Example: A package with CID `sha256-abcdef123...` might be stored in `~/.lang/pkg/sha256/ab/cdef123.../`.
        *   This sharding (e.g., using the first few characters of the hash for directory levels) helps manage a large number of stored packages efficiently.
        *   *Illustrative Directory Tree for a Cached Package:*
            ```text
            ~/.lang/pkg/
            └── sha256/                     # Hash algorithm directory
                └── ab/                     # First shard level (e.g., first 2 chars of hash)
                    └── cdef1234567890.../  # Second shard level (e.g., rest of hash)
                        ├── package.tar.gz  # The canonical compressed package archive
                        ├── manifest.ferra  # A copy of the package's manifest for quick inspection
                        └── metadata.json   # Additional cached metadata (e.g., source URL, download time)
            ```
    *   **Stored Content**: The cache will primarily store the canonical, compressed package archives (e.g., `package.tar.gz`). Optionally, an unpacked (but still immutable) version might be cached for faster access by the build system, though this increases disk usage and complexity.
    *   **Metadata**: Associated metadata for each cached package (e.g., its original source URL, a copy of its manifest) might be stored alongside the content or in a separate local database indexed by CID.
*   **2.4. Immutability**: Packages stored in the content-addressed cache are strictly immutable. If a new version of a package is released or a package is rebuilt with changes, it will result in a new CID and be stored as a distinct entry. There is no in-place modification of cached package content.
    *   This immutability is fundamental to the integrity and reproducibility guarantees of the system.

## 3. CLI Commands (Step 2.4.2)

The `lang` command-line tool will be the primary interface for interacting with the Ferra package manager. The following commands are proposed for the beta version, drawing directly from `Steps.md` (Section 7) and `comprehensive_plan.md` (Module 2.4.2).

Conceptual EBNF for core `lang` subcommands related to package management:
```ebnf
Command         ::= "lang" Subcommand
Subcommand      ::= AddCommand
                  | VendorCommand
                  | InitCommand         (* Future Consideration *)
                  | RemoveCommand       (* Future Consideration *)
                  | BuildCommand
                  | RunCommand
                  | TestCommand
                  (* ... other future subcommands ... *)

AddCommand      ::= "add" PackageSpecifier Options*
PackageSpecifier::= IDENTIFIER ("@" (VersionSpec | CIDSpec))? (* e.g., mylib, mylib@^1.0, mylib@sha256-abc... *)
Options         ::= "--git" URL
                  | "--branch" IDENTIFIER
                  | "--tag" IDENTIFIER
                  | "--rev" IDENTIFIER
                  | "--path" FilePath
                  (* ... other options ... *)

VendorCommand   ::= "vendor" ( "--sbom" )?
InitCommand     ::= "init" ( ProjectName )?
RemoveCommand   ::= "remove" PackageName
BuildCommand    ::= "build" BuildOptions*
RunCommand      ::= "run" BuildOptions* (* Plus arguments for the program *)
TestCommand     ::= "test" BuildOptions* (* Plus arguments for the test runner *)
BuildOptions    ::= "--release" | "--target" "=" TargetTriple | "--package-format" "=" PackageFormat | ("--emit" "=" EmitType)
TargetTriple    ::= IDENTIFIER (* e.g., "aarch64-apple-ios", "wasm32-wasi" *)
PackageFormat   ::= IDENTIFIER (* e.g., "aab", "ipa" - conceptual *)
EmitType        ::= IDENTIFIER (* e.g., "lib", "bin", "ir", "asm" *)
```
*(Note: `PackageSpecifier`, `VersionSpec`, `CIDSpec`, `URL`, `FilePath`, `ProjectName`, `PackageName` are placeholders for more detailed token/rule definitions. Error reporting for CLI commands should adhere to `DESIGN_DIAGNOSTICS.md`.)*

*   **3.1. `lang add <package_specifier>`**:
    *   **Purpose**: Adds a dependency to the current Ferra project. This command will resolve the specified package, download it if necessary, store it in the content-addressed cache (`~/.lang/pkg`), and update the project's manifest file (e.g., `Ferra.toml`) and lockfile (e.g., `Ferra.lock`) with the resolved dependency information (including its CID).
    *   **Package Specifiers**: The `<package_specifier>` can take several forms:
        *   **CID-Pinned Version**: `lang add <package_name>@<cid>` (e.g., `lang add http@sha256-abc...def` or `lang add http@ipfs_cid_v1...`). This provides exact, verifiable pinning to a specific package content. `Steps.md` shows `lang add http@1.2.0` with a comment `# CID-pinned`. This implies that version numbers like `1.2.0` might resolve to a CID through a registry, or the syntax might directly support a version that is also a CID or has an associated CID in its metadata.
        *   **Semantic Versioning (SemVer)**: `lang add <package_name>@"<semver_req>"` (e.g., `lang add serde@"^1.0.4"`, `lang add utils@">=0.5, <0.7"`). The `lang add serde::*` example from `Steps.md` likely implies resolving to the latest compatible version based on semver rules (e.g., latest `1.x.x` for `^1.0`). The exact syntax for semver constraints (e.g., direct, with `*`, `^`, `~`) needs to be finalized, aligning with common conventions.
        *   **Git Dependencies (Future Consideration for Beta)**: `lang add <package_name> --git <url> [--branch <branch> | --tag <tag> | --rev <commit_hash>]` (e.g., `lang add my_lib --git https://github.com/user/repo.git`). This would fetch the package directly from a Git repository.
        *   **Local Path Dependencies (Future Consideration for Beta)**: `lang add <package_name> --path <local_path>` (e.g., `lang add helper_lib --path ../helper_lib`). This links a local package for development purposes.
    *   **Resolution Process**: 
        1.  Parse the `<package_specifier>`.
        2.  If a registry is involved (future), query the registry for metadata, versions, and CIDs.
        3.  For Git/path, fetch/access the source directly.
        4.  Resolve the specific version and its CID based on constraints (semver) or direct specification (CID, Git rev).
        5.  Check the local cache (`~/.lang/pkg`) for the package by its CID. If present and verified, use it.
        6.  If not cached, download the package archive from its source (registry, Git, etc.).
        7.  Verify the downloaded package against its expected CID (if known beforehand, e.g., from a lockfile or registry metadata). If the CID is being established for the first time (e.g., for a new semver resolution), calculate and store it.
        8.  Store the verified package (archive and possibly unpacked form) in `~/.lang/pkg` indexed by its CID.
        9.  Update the project manifest (e.g., `Ferra.toml`) to include the dependency with its resolved version specifier.
        10. Update the project lockfile (e.g., `Ferra.lock`) with the exact CID and version of this package and all its transitive dependencies.
*   **3.2. `lang vendor [--sbom]`**:
    *   **Purpose**: As specified in `Steps.md` (Section 7), this command vendors all project dependencies (direct and transitive, based on the lockfile) into a local directory within the project (e.g., `vendor/` or `.ferra/vendor/`). This allows for offline builds and easier auditing of dependency code.
    *   **Operation**: 
        1.  Reads the project's lockfile to get the exact list of all required package CIDs and versions.
        2.  For each dependency, copies the package content (e.g., the unpacked source, or the archive) from the global cache (`~/.lang/pkg`) into the specified local vendor directory.
        3.  The structure within the vendor directory should allow the build system to locate these vendored packages.
    *   **`--sbom` Flag**: If the `--sbom` flag is provided, the command will also generate a Software Bill of Materials (SBOM) in SPDX format for the vendored dependencies. This SBOM will be placed in a well-known location (e.g., project root or the vendor directory). Details of SBOM generation are in Section 4.
    *   **Scope for Native Dependencies**: For v0.1, `lang vendor` will primarily focus on vendoring Ferra package dependencies. Handling of native C/C++ dependencies declared in `Ferra.toml` during vendoring needs further specification:
        *   If a native dependency is defined by a local path within the project or a Git URL that can be fetched, `lang vendor` might copy these sources into the vendor directory.
        *   System-provided libraries or those found via `pkg-config` are typically not vendored; the build process (even with vendored Ferra dependencies) would still rely on these being present on the build machine or resolved by the build script.
        *   Clear documentation will be provided on how vendoring interacts with different types of native dependencies.
*   **3.3. Other Potential Commands (Future Considerations for Beta Release)**:
    *   **`lang init`**: Initializes a new Ferra project in the current directory, creating a `Ferra.toml` manifest. (This might be an alternative or complement to `lang new myapp` which creates a directory).
    *   **`lang build [--sbom]`**: Compiles the current project. The optional `--sbom` flag would generate an SBOM for the project and its dependencies, similar to `lang vendor --sbom` but potentially focused on the built artifacts.
    *   **`lang test`**: Runs project tests.
    *   **`lang run`**: Builds and runs the project's main executable.
    *   **`lang clean`**: Removes build artifacts.
    *   **`lang remove <package_name>`**: Removes a dependency from the project manifest and updates the lockfile.
    *   **`lang update [<package_name>]`**: Updates the specified package (or all packages if none specified) to the latest compatible versions allowed by the manifest, then updates the lockfile.
    *   **`lang list` or `lang tree`**: Displays the project's dependencies, possibly as a tree showing transitive dependencies.
    *   **`lang publish`**: (Post-Beta) Publishes a package to a central or specified registry. This would involve packaging, CID generation, signing with Sigstore, and uploading.
    *   **`lang search <query>`**: (Post-Beta) Searches for packages in a configured registry.
    *   **`lang permissions show`**: (Post-Beta, see `SECURITY_MODEL.md` §3.6) Lists all permissions declared by the current package in its `Ferra.toml`.
    *   **`lang permissions audit`**: (Post-Beta, see `SECURITY_MODEL.md` §3.6) Lists all unique permissions requested by the current package and all its direct and transitive dependencies, providing a full permission footprint.

## 4. SPDX SBOM Generation and Sigstore Integration (Step 2.4.3)

Ensuring software supply chain security is a critical objective for the Ferra package manager. This involves generating comprehensive Software Bills of Materials (SBOMs) and integrating with projects like Sigstore for cryptographic signing and verification of packages. These features align with the requirements in `Steps.md` (Section 7 and Section 11).

*   **4.1. SPDX SBOM Generation**:
    *   **Purpose**: To provide a detailed inventory of all software components (including transitive dependencies) within a Ferra project. This aids in vulnerability management, license compliance, and understanding software provenance.
    *   **Trigger**: Primarily via the `lang vendor --sbom` command. It could also be triggered by `lang build --sbom` (if implemented) or as part of a `lang publish` workflow to generate an SBOM for the package being published.
    *   **Format**: SPDX (Software Package Data Exchange). The specific version targeted will be a current, widely adopted one (e.g., SPDX 2.2, 2.3, or later as appropriate at the time of implementation). Output will typically be in a machine-readable format like SPDX JSON or Tag-Value.
    *   **Content Requirements**: The generated SBOM must include, at a minimum:
        *   **Document Creation Information**: SPDX version, data license (e.g., CC0-1.0), SPDX ID, document name, creator information (tool: `lang vendor`, timestamp).
        *   **Package Information (for the main project and each dependency)**:
            *   Name
            *   Version
            *   SPDXID (unique identifier within the document)
            *   Supplier & Originator (if known/applicable)
            *   Download Location (URL from which the package was obtained, or local path if applicable)
            *   Checksums (e.g., SHA256 of the package archive, matching its CID if possible)
            *   LicenseConcluded & LicenseDeclared (SPDX license expressions)
            *   Copyright text (if available from package metadata)
            *   Primary Package Purpose (e.g., LIBRARY, APPLICATION)
        *   **Relationships**: Accurate depiction of dependencies between packages using SPDX relationship types (e.g., `DEPENDS_ON`, `CONTAINS`, `DESCRIBES`).
        *   **File Information (Optional but Recommended for Deeper Audits)**: For each package, potentially list key files, their types, checksums, and license/copyright info if distinct from the package level.
    *   **Output**: The SBOM will be generated as a standalone file in a machine-readable format (e.g., SPDX JSON). 
        *   For `lang vendor --sbom`, the default filename could be `vendor.spdx.json` located within the vendor directory (e.g., `vendor/vendor.spdx.json`).
        *   For `lang build --sbom` (a future consideration mentioned in Section 3.3), the output might be `target/<build_profile>/<project_name>.sbom.spdx.json`.
        *   The exact naming and location should be consistent and predictable.
*   **4.2. Sigstore Integration for Signing & Verification**:
    *   **Purpose**: To provide cryptographic verification of package integrity and authenticity, leveraging Sigstore's tooling (like Cosign for signing/verification and Rekor for transparency log entries).
    *   **Package Signing (Primarily for `lang publish` - Future)**:
        *   When a Ferra developer publishes a package to a registry, the `lang publish` command (or an associated tool/workflow) would facilitate signing the package artifact (e.g., the CID-identified archive) and its associated attestations (such as its SPDX SBOM) using Cosign.
        *   This involves creating a signature using the developer's identity (e.g., OIDC token) and optionally uploading the signature, the SBOM attestation, and other metadata to the Rekor transparency log.
    *   **Dependency Verification (During `lang add`, `lang build`, or explicit `lang verify-deps`)**:
        *   When a dependency is fetched, the `lang` tool (or build process) can attempt to verify its signature and associated attestations (like its SBOM) against information from Sigstore (Rekor log, Fulcio root certificates).
        *   Verification steps would include:
            1.  Retrieving the package artifact (identified by its CID).
            2.  Fetching its signature and associated attestations (e.g., SBOM) from the package metadata, a registry, or by querying Rekor.
            3.  Using Cosign (or equivalent logic) to verify the signature against the artifact and the publisher's identity (via Fulcio code-signing certificates).
            4.  Verifying the inclusion of the signature/attestation in the Rekor transparency log.
        *   **Policy for Unverified Packages**: The package manager will need a configurable policy for handling packages that fail verification or lack signatures (e.g., `strict` mode to deny, `audit` mode to warn, `allow` mode for compatibility with unsigned packages, possibly with user confirmation).
*   **4.3. Manifest and Lockfile Role in Security**: 
    *   **Manifest (`Ferra.toml`)**: Declares intended dependencies and acceptable version ranges. For security-critical dependencies, it might allow specifying an expected publisher identity or key.
    *   **Lockfile (`Ferra.lock`)**: Records the exact resolved version, its verified CID, and potentially a digest of its verified signature/attestations. This ensures that subsequent builds use precisely the same, verified bits. Changes to the lockfile should be auditable (e.g., in version control).
    *   The lockfile itself could be a candidate for being included in a project-level signature or attestation, indicating a fully verified set of dependencies.

By integrating these features, the Ferra package manager aims to provide a robust defense against common supply chain attacks, such as tampering and unauthorized modification of packages.

## 5. Package Structure and Manifest File (Conceptual)

*   **5.1. Package Contents**: Defines what constitutes a Ferra package that can be published and consumed. This typically includes:
    *   Ferra source files (`*.ferra`) within a defined directory structure (e.g., `src/`).
    *   A project manifest file (e.g., `Ferra.toml`) at the root.
    *   Optionally, other assets like documentation, examples, test files, build scripts for FFI.
*   **5.2. Project Manifest File (e.g., `Ferra.toml`)**: This file, located at the root of a Ferra project/package, defines metadata and dependencies. Inspired by Cargo.toml (Rust) or pyproject.toml (Python).
    *   **Core Metadata**:
        *   `name`: String (Package name, adhering to naming conventions - TBD TAG-PKG-6).
        *   `version`: String (Semantic Version, e.g., "0.1.0").
        *   `description`: String (Optional, brief description).
        *   `authors`: Array of String (Optional, e.g., ["Name <email@example.com>"]).
        *   `license`: String (Optional, SPDX license identifier, e.g., "Apache-2.0").
        *   `repository`: String (Optional, URL to the source code repository).
    *   **`[package.permissions]` Section (Conceptual for v0.1 Security Model)**:
        *   This optional section declares the capabilities required by the package, as detailed in `docs/SECURITY_MODEL.md` (Section 3).
        *   Example:
            ```toml
            [package.permissions]
            net_fetch = ["https://api.example.com"]
            fs_read = ["./config.toml"]
            ```
        *   The exact syntax and available permission strings are defined in `SECURITY_MODEL.md`.
    *   **Dependencies Section**: Specifies direct dependencies.
        *   Simple version: `http = "1.2.0"` (implies exact version, potentially CID-backed if registry supports it).
        *   Semantic versioning: `serde = "^1.0"` or `utils = ">=0.5, <0.7"`.
        *   Git dependencies: `my_lib = { git = "https://github.com/user/repo.git", branch = "main" }`.
        *   Local path dependencies: `helper_lib = { path = "../helper_lib" }`.
        *   Registry source (Future): `database_driver = { version = "0.3.0", registry = "ferra-central" }`.
    *   **Native (C/C++) Dependencies Section (Conceptual for v0.1)**:
        To support FFI as detailed in `docs/FFI_C_CPP.md`, the manifest needs a way to declare dependencies on external C/C++ libraries and specify how to build/link local C/C++ sources.
        ```toml
        # Example: In Ferra.toml
        [native-dependencies] # Or a more generic [external-dependencies]
        # For linking against system or pre-built libraries
        zlib = { system = true, version = ">=1.2" } # Tries to find via pkg-config or standard paths
        sdl2 = { pkg_config_name = "sdl2", version = "2.0" }
        my_custom_c_lib = { path = "libs/my_custom_c_lib.a", static = true }

        # For compiling and linking C/C++ sources included in the project/package
        [native-build.my_c_helpers] # Arbitrary name for the build target
        sources = ["src_c/helper1.c", "src_c/helper2.c"]
        include_dirs = ["src_c/include"]
        cflags = ["-O2", "-Wall"]
        # Output will be a static library linked into the Ferra build.
        ```
        The exact structure and keys (`system`, `pkg_config_name`, `path`, `static`, `sources`, `include_dirs`, `cflags`) are illustrative and need finalization. The build system would interpret these to locate or build and then link these native dependencies.
    *   **Build Script (`build.ferra`)**:
        Projects can include an optional `build.ferra` script (e.g., specified via `build_script = "build.ferra"` in `Ferra.toml` under a `[package]` or `[build]` section). The package manager/build system will execute this script before compiling the Ferra crate.
        This script can perform tasks like:
        *   Probing for C/C++ libraries using `pkg-config` or other system tools.
        *   Compiling bundled C/C++ code.
        *   Generating code that will be included in the Ferra compilation.
        *   Outputting directives to the build system for linking, such as:
            *   `ferra:link-lib=<kind>:<name>` (e.g., `ferra:link-lib=dylib:ssl`, `ferra:link-lib=static:myhelper`)
            *   `ferra:link-search=<kind>:<path>` (e.g., `ferra:link-search=native:/opt/local/lib`)
            *   `ferra:include-path=<path>` (for C headers if compiling C code)
        The exact set of directives and their format is TBD (FFI-C-10).
    *   **Build Configuration (Optional, Future)**:
        *   Instructions for building FFI components or other custom build steps (now largely covered by build scripts and native dependency declarations).
    *   **Target-Specific Configuration (Conceptual)**:
        *   The manifest may also include target-specific configuration sections, for example, to provide metadata needed for packaging or compilation for a particular target platform.
        *   Example for Android (see `BACKEND_EXPANDED_TARGETS.md` Section 5.5):
            ```toml
            [target.android]
            application_id = "com.example.myferraapp"
            version_code = 1
            # ... other Android-specific settings ...
            ```
        *   The structure and keys for such sections would be defined by the relevant backend or packaging specifications.
    *   *(Cross-reference: `VSCODE_PLUGIN_ALPHA_SPEC.md` details that `lang new myapp` generates a basic `Ferra.toml`.)*

## 6. Dependency Resolution Strategy (Conceptual)

A robust dependency resolution strategy is essential for a package manager to handle complex project dependencies, ensure compatibility, and provide reproducible builds. For the Ferra package manager beta, the following conceptual approach is proposed:

*   **Algorithm Choice**: To ensure correctness and provide clear diagnostics in case of conflicts, Ferra's package manager should aim to implement or adapt a modern version resolution algorithm. A strong candidate is one similar to **PubGrub**, which is known for its: 
    *   Correctness in finding a valid set of package versions that satisfy all constraints.
    *   Ability to provide human-understandable explanations when a conflict is unsolvable.
    *   Efficiency in many common cases.
*   **Input to Resolver**:
    *   The primary input is the list of direct dependencies and their version constraints specified in the project's manifest file (e.g., `Ferra.toml`).
    *   If a `Ferra.lock` file exists and is up-to-date with the manifest, the resolver may first attempt to use the exact versions specified in the lockfile for faster, reproducible resolution. If the manifest has changed, or if explicitly requested (e.g., `lang update`), a full resolution will be performed.
*   **Conflict Handling and Diagnostics**:
    *   When version conflicts occur (e.g., Package A requires LibX v1.0 and Package B requires LibX v2.0), the resolver must detect this.
    *   Crucially, it should not just fail silently but provide a clear diagnostic message explaining which packages and versions are in conflict and what constraints led to the conflict. PubGrub-style algorithms excel at this by tracing the chain of decisions that led to the incompatibility.
    *   Where possible, the package manager might suggest potential remediations, such as updating a specific direct dependency or indicating which package introduced an incompatible transitive dependency.
*   **The `Ferra.lock` Lockfile**: 
    *   **Purpose**: After a successful dependency resolution, the package manager will generate or update a `Ferra.lock` file in the project root.
    *   **Content**: This file will precisely record:
        *   Every package in the complete dependency graph (direct and transitive).
        *   The exact resolved version for each package.
        *   The Content Identifier (CID) for each resolved package version, ensuring that the exact same bits are used in subsequent builds or by other developers.
        *   The source from which the package was resolved (e.g., registry URL, Git commit hash).
        *   Optionally, a hash of the package manifest to ensure the lockfile corresponds to the current manifest.
    *   **Reproducibility**: The lockfile is the key to reproducible builds. When `Ferra.lock` is present and consistent with `Ferra.toml`, subsequent `lang build` or `lang add` (without explicit update flags) operations should use the exact versions and CIDs from the lockfile, bypassing full resolution.
    *   **Version Control**: The `Ferra.lock` file **should be committed** to the project's version control system (e.g., Git) to ensure all collaborators and CI systems use the same set of resolved dependencies.
*   **Registry Interaction (Future)**:
    *   For the beta, dependency resolution might initially focus on specified CIDs, local paths, and Git URLs where version metadata is directly accessible or explicitly pinned.
    *   When a central package registry (or multiple registries) is introduced, the dependency resolver will interact with these registries via a defined API to:
        *   Fetch lists of available versions for a given package.
        *   Retrieve metadata for specific package versions (including their dependencies and CIDs).
*   **Semantic Versioning (SemVer) Adherence**: The resolver will strictly adhere to Semantic Versioning 2.0.0 rules when interpreting version constraints (e.g., `^1.0.4`, `>=0.5.0, <0.7.0`).

This strategy aims to provide Ferra developers with a reliable, understandable, and reproducible dependency management experience, drawing lessons from established package managers in other ecosystems.

## 7. Open Questions / TBD

*   (TAG-PKG-1) Final exact format, name, and comprehensive field list for the project manifest file (e.g., `Ferra.toml`).
*   (TAG-PKG-2) Detailed design of the package registry (architecture, API, policies) - this is a major item deferred beyond the beta package manager scope.
*   (TAG-PKG-3) Policy for handling name conflicts, typosquatting, and ownership of package names in a future central registry.
*   (TAG-PKG-4) Specific cryptographic hash algorithm(s) to be definitively used for CIDs (e.g., SHA2-256 default, support for others?).
*   (TAG-PKG-5) Detailed schema and workflow for package attestations beyond SBOMs (e.g., SLSA compliance levels, test results).
*   (TAG-PKG-6) Namespace/organization support for package names to avoid global conflicts and group related packages (e.g., `org::package_name`).
*   (TAG-PKG-7) Strategy for handling binary dependencies or pre-compiled artifacts within packages, especially for FFI.
*   (TAG-PKG-8) Details on how `lang add` with semver resolves to a CID-pinned version initially.
*   (TAG-PKG-9) Define the exact syntax and semantics of directives output by `build.ferra` scripts for FFI linking (e.g., `ferra:link-lib=...`, `ferra:link-search=...`), addressing TBD FFI-C-10 from `FFI_C_CPP.md`.

---
This document outlines the design for a beta version of the Ferra Package Manager, focusing on secure, content-addressed package management and CLI tooling.