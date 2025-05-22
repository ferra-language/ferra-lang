---
number: RFC-006
title: "Package Manager Design"
status: Draft
version: v0.4
authors: ["Amrit Doll <amritdoll@example.com>"]
last_reviewed: 2025-05-21
last_updated: 2025-05-21
---

# RFC-006: Package Manager Design

## Table of Contents
1. [Summary](#1-summary)
2. [Motivation](#2-motivation)
3. [Impact](#3-impact)
   1. [Security](#31-security)
   2. [Developer Experience](#32-developer-experience)
   3. [Ecosystem](#33-ecosystem)
4. [Design Decisions](#4-design-decisions)
   1. [Content-Addressed Storage](#41-content-addressed-storage)
   2. [CLI Commands](#42-cli-commands)
   3. [SBOM & Sigstore Integration](#43-sbom--sigstore-integration)
   4. [Package Structure](#44-package-structure)
   5. [Dependency Resolution](#45-dependency-resolution)
5. [Drawbacks](#5-drawbacks)
   1. [Complexity](#51-complexity)
   2. [Performance](#52-performance)
   3. [Usability](#53-usability)
6. [Security & Privacy](#6-security--privacy)
   1. [Threat Model](#61-threat-model)
   2. [Permissions Impact](#62-permissions-impact)
   3. [Audit Requirements](#63-audit-requirements)
   4. [Data Handling](#64-data-handling)
   5. [Error Reporting](#65-error-reporting)
7. [Implementation Plan](#7-implementation-plan)
   1. [Phase 1: Core Package Management (Q4 2025)](#71-phase-1-core-package-management)
   2. [Phase 2: Security Features (Q4 2025)](#72-phase-2-security-features)
   3. [Phase 3: Build Integration (Q4 2025)](#73-phase-3-build-integration)
   4. [Issue Mapping](#74-issue-mapping)
8. [Migration Strategy](#8-migration-strategy)
9. [Unresolved Questions](#9-unresolved-questions)
10. [Future Possibilities](#10-future-possibilities)
11. [References](#11-references)

## 1. Summary
This RFC specifies the design for Ferra's package manager, focusing on secure, content-addressed package management and CLI tooling. The design prioritizes supply chain security, developer experience, and ecosystem compatibility, with features like CID-based storage, SPDX SBOM generation, and Sigstore integration.

## 2. Motivation
Modern software development requires robust package management that ensures security, reproducibility, and ease of use. Existing solutions often lack:
- Strong supply chain security guarantees
- Clear content addressing and verification
- Comprehensive SBOM generation
- Developer-friendly CLI tooling

Ferra's package manager aims to address these gaps while providing a seamless developer experience.

## 3. Impact
### 3.1 Security
- Content-addressed storage with cryptographic verification
- SPDX SBOM generation for supply chain transparency
- Sigstore integration for package signing and verification
- **Takeaway**: Strong security guarantees from package source to build
- **Metrics**:
  - CID calculation: ~5ms per package on x86_64
  - SBOM generation: ~100ms for typical package
  - Signature verification: ~10ms per package
  - Audit log overhead: <1ms per operation

### 3.2 Developer Experience
- Intuitive CLI commands (`lang add`, `lang vendor`)
- Clear dependency resolution and conflict handling
- Comprehensive manifest format (`Ferra.toml`)
- **Takeaway**: Package management that's both powerful and user-friendly
- **Metrics**:
  - Command latency: <50ms for common operations
  - Dependency resolution: <100ms for typical project
  - Build script execution: <200ms for simple scripts

### 3.3 Ecosystem
- Support for multiple package sources (CID, Git, local)
- Native dependency handling for FFI
- Build system integration
- **Takeaway**: Flexible and extensible package ecosystem
- **Metrics**:
  - Registry API response: <100ms for metadata, <500ms for package download
  - Package discovery: <200ms for typical search
  - Cross-platform compatibility: 100% for core platforms

## 4. Design Decisions
### 4.1 Content-Addressed Storage
- **Storage Location**: Global shared cache at `~/.lang/pkg`
- **CID Generation**: SHA-256 by default, with algorithm prefix
- **Storage Layout**: Sharded by CID for efficient lookup
- **Immutability**: Strictly immutable package storage

Example directory structure:
```text
~/.lang/pkg/
└── sha256/                     # Hash algorithm directory
    └── ab/                     # First shard level
        └── cdef1234567890.../  # Second shard level
            ├── package.tar.gz  # Canonical package archive
            ├── manifest.ferra  # Package manifest
            └── metadata.json   # Additional metadata
```

### 4.2 CLI Commands
Core commands for the beta version:

```ferra
# Add a dependency
lang add http@1.2.0           # CID-pinned
lang add serde::*             # semver caret
lang add mylib --git https://github.com/user/repo.git

# Vendor dependencies
lang vendor --sbom            # Vendors with SBOM generation

# Future commands (post-beta)
lang init                     # Initialize new project
lang build --sbom            # Build with SBOM
lang test                    # Run tests
lang run                     # Build and run
lang clean                   # Remove build artifacts
lang remove <package>        # Remove dependency
lang update [<package>]      # Update dependencies
lang list                    # Show dependencies
lang tree                    # Show dependency tree
```

CLI Grammar (simplified):
```ebnf
command     = "lang" (add | vendor | init | build | test | run | clean | remove | update | list | tree)
add         = "add" package-spec [git-spec]
package-spec = name version-spec
version-spec = "@" version | "::" semver
git-spec    = "--git" url
vendor      = "vendor" ["--sbom"]
```

> Note: See [PACKAGE_MANAGER_SPEC.md](../PACKAGE_MANAGER_SPEC.md#cli-grammar) for complete grammar specification.

### 4.3 SBOM & Sigstore Integration
- **SBOM Generation**:
  - SPDX format (JSON/Tag-Value)
  - Comprehensive package metadata
  - Relationship tracking
  - File-level information

- **Sigstore Integration**:
  - Package signing with Cosign
  - Rekor transparency log
  - Verification during dependency resolution
  - Configurable verification policies

### 4.4 Package Structure
Project manifest (`Ferra.toml`):
```toml
[package]
name = "my_project"
version = "0.1.0"
description = "A Ferra project"
authors = ["Name <email@example.com>"]
license = "Apache-2.0"

[package.permissions]
net_fetch = ["https://api.example.com"]
fs_read = ["./config.toml"]

[dependencies]
http = "1.2.0"
serde = "^1.0"
my_lib = { git = "https://github.com/user/repo.git" }

# Field name pending resolution in PKG-1
[native-dependencies]  # Alternative: [external-dependencies]
zlib = { system = true, version = ">=1.2" }
sdl2 = { pkg_config_name = "sdl2", version = "2.0" }

[native-build.my_c_helpers]
sources = ["src_c/helper1.c"]
include_dirs = ["src_c/include"]
cflags = ["-O2", "-Wall"]

[build]
# Build script directives
ferra:link-lib = ["z", "sdl2"]
ferra:rerun-if-changed = ["src_c/*.c", "src_c/include/*.h"]
ferra:test-deps = ["gtest", "benchmark"]

# CI/CD integration
ferra:ci = { 
  platforms = ["linux-x86_64", "macos-arm64"],
  test-coverage = 0.8,
  energy-budget = 70
}
```

### 4.5 Dependency Resolution
The package manager uses a PubGrub-inspired algorithm for dependency resolution:

1. **Version Selection**
```ferra
// Example manifest
[dependencies]
http = "1.2.0"      // Exact version
serde = "^1.0.0"    // Semver range
crypto = "=2.1.0"   // Exact version
```

2. **Conflict Resolution**
```ferra
// Example conflict and resolution
[dependencies]
lib-a = "1.0.0"     // Requires crypto ^2.0.0
lib-b = "2.0.0"     // Requires crypto =1.0.0

// Resolution error
error: Version conflict detected
  lib-a@1.0.0 requires crypto ^2.0.0
  lib-b@2.0.0 requires crypto =1.0.0
  No compatible version exists

// Resolution with override
[dependencies]
lib-a = "1.0.0"
lib-b = "2.0.0"
[overrides]
crypto = "2.0.0"    // Force specific version
```

3. **SBOM Generation**
```json
{
  "SPDXID": "SPDXRef-DOCUMENT",
  "spdxVersion": "SPDX-2.3",
  "name": "my-project-1.0.0",
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-http-1.2.0",
      "name": "http",
      "versionInfo": "1.2.0",
      "downloadLocation": "https://registry.ferra-lang.org/packages/http/1.2.0",
      "checksums": [
        {
          "algorithm": "SHA256",
          "checksumValue": "a1b2c3..."
        }
      ],
      "licenseConcluded": "MIT",
      "licenseDeclared": "MIT",
      "copyrightText": "Copyright (c) 2025"
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-Package-http-1.2.0",
      "relatedSpdxElement": "SPDXRef-Package-crypto-2.0.0",
      "relationshipType": "DEPENDS_ON"
    }
  ]
}
```

```ferra
// Example of dependency override scenario
$ ferra update --verbose
[INFO] Resolving dependencies...
[INFO] Found version conflict:
  - Package A v1.2.0 requires Package B v2.0.0
  - Package C v3.1.0 requires Package B v2.1.0
[INFO] Applying override from manifest:
  [dependencies]
  B = { version = "2.1.0", override = true }
[INFO] Resolution complete:
  A v1.2.0
  B v2.1.0 (overridden)
  C v3.1.0
```

## 5. Drawbacks
### 5.1 Complexity
- Content addressing adds complexity
- SBOM generation overhead
- Sigstore integration complexity

### 5.2 Performance
- CID calculation overhead
- SBOM generation time
- Verification latency

### 5.3 Usability
- Learning curve for security features
- Complex manifest format
- Native dependency complexity

## 6. Security & Privacy
### 6.1 Threat Model
- Package tampering
- Supply chain attacks
- Dependency confusion
- Malicious native code

### 6.2 Permissions Impact
- Package permissions in manifest
- Native dependency permissions
- Build script permissions
- **Permission Model**:
  ```toml
  [package.permissions]
  # Network access
  net_fetch = ["https://api.example.com"]
  net_connect = ["wss://stream.example.com"]
  
  # File system access
  fs_read = ["./config.toml", "./data/*"]
  fs_write = ["./build/*", "./logs/*"]
  
  # System access
  sys_env = ["PATH", "HOME"]
  sys_exec = ["/usr/bin/gcc"]
  
  # Build script permissions
  build_net = ["https://download.example.com"]
  build_fs = ["./src/*"]
  ```

### 6.3 Audit Requirements
- SBOM generation and verification
- Package signature verification
- Build artifact verification
- **Audit Log Format**:
  ```json
  {
    "timestamp": "2025-05-21T10:00:00Z",
    "event": "package_install",
    "package": {
      "name": "http",
      "version": "1.2.0",
      "cid": "sha256:abc123...",
      "source": "registry.example.com"
    },
    "verification": {
      "sbom": "spdx:2.3",
      "signature": "sigstore:cosign",
      "attestations": ["build", "test"]
    },
    "permissions": {
      "granted": ["net_fetch", "fs_read"],
      "denied": ["sys_exec"]
    }
  }
  ```

### 6.4 Data Handling
- Package metadata privacy
- SBOM data sensitivity
- Build artifact security
- **Memory Safety Checks**:
  - Static analysis of native code
  - Runtime bounds checking
  - Memory leak detection
  - Resource cleanup verification

### 6.5 Error Reporting
- Clear security warnings
- Verification failure messages
- Permission violation reports

## 7. Implementation Plan
### 7.1 Phase 1: Core Package Management (Q4 2025)
- Content-addressed storage
- Basic CLI commands
- Manifest format
- Dependency resolution
- **Test Harness**: Comprehensive test suite for package operations
  - Package download and verification
  - Dependency resolution edge cases
  - Build script execution
  - Native dependency integration
  - Performance benchmarks

### 7.2 Phase 2: Security Features (Q4 2025)
- SBOM generation
- Sigstore integration
- Verification policies
- Security auditing
- **Test Harness**: Security verification suite
  - Package signature verification
  - SBOM generation and validation
  - Permission enforcement
  - Audit log verification
  - Supply chain attack detection

### 7.3 Phase 3: Build Integration (Q4 2025)
- Build script support
- Native dependency handling
- Target-specific configuration
- CI/CD integration
- **Test Harness**: Build system integration tests
  - Cross-platform build verification
  - Native dependency compilation
  - Build script execution
  - CI/CD pipeline integration

### 7.4 Issue Mapping
| Phase   | Issue           | URL                                    | Priority | Target Date |
| ------- | --------------- | -------------------------------------- | -------- | ----------- |
| Phase 1 | #132 (PKG-1)    | https://github.com/org/repo/issues/132 | High     | Q4 2025     |
| Phase 1 | #133 (PKG-2)    | https://github.com/org/repo/issues/133 | High     | Q4 2025     |
| Phase 1 | #134 (PKG-3)    | https://github.com/org/repo/issues/134 | High     | Q4 2025     |
| Phase 2 | #135 (PKG-4)    | https://github.com/org/repo/issues/135 | Medium   | Q4 2025     |
| Phase 2 | #136 (PKG-5)    | https://github.com/org/repo/issues/136 | Medium   | Q4 2025     |
| Phase 2 | #137 (PKG-6)    | https://github.com/org/repo/issues/137 | Low      | Q4 2025     |
| Phase 3 | #138 (PKG-7)    | https://github.com/org/repo/issues/138 | Medium   | Q4 2025     |
| Phase 3 | #139 (PKG-8)    | https://github.com/org/repo/issues/139 | Medium   | Q4 2025     |
| Phase 3 | #140 (PKG-9)    | https://github.com/org/repo/issues/140 | Low      | Q4 2025     |

## 8. Migration Strategy
The migration from CID-based to registry-based package management will be handled in phases:

1. **Phase 1: Registry Integration**
   - Add registry support while maintaining CID compatibility
   - Update CLI to support both formats
   - Example manifest diff:
   ```diff
   [package]
   name = "my-package"
   version = "1.0.0"
   - cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
   + registry = "https://registry.ferra.dev"
   + package_id = "my-package@1.0.0"
   ```

2. **Phase 2: Migration Tools**
   - Provide `ferra migrate` command to convert CIDs to registry IDs
   - Example usage:
   ```bash
   $ ferra migrate --from-cid bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi
   [INFO] Converting CID to registry ID...
   [INFO] Generated manifest:
     registry = "https://registry.ferra.dev"
     package_id = "my-package@1.0.0"
   ```

3. **Phase 3: Deprecation**
   - Mark CID-based packages as deprecated
   - Provide warnings for CID usage
   - Set timeline for CID removal

## 9. Unresolved Questions
1. **Registry v1.0 Transition** (PKG-REG-1)
   - How to handle migration from CID-only to registry metadata
   - Tooling for bulk updates
   - Fallback mechanisms for offline use

## 10. Future Possibilities
- Central package registry
- Automated vulnerability scanning
- Advanced build customization
- Cross-platform binary distribution
- Package metrics and analytics

## 11. References
1. [PACKAGE_MANAGER_SPEC.md](../PACKAGE_MANAGER_SPEC.md)
2. [SECURITY_MODEL.md](../SECURITY_MODEL.md#package-permissions)
3. [FFI_C_CPP.md](../FFI_C_CPP.md#native-dependencies)
4. [Steps.md Section 7: Package Manager](../Steps.md#7--package-manager-content-addressed)
5. [diagnostic_codes.md](../diagnostic_codes.md#package-errors)
6. [VSCODE_PLUGIN_ALPHA_SPEC.md](../VSCODE_PLUGIN_ALPHA_SPEC.md#package-support)
7. [RFC-005: FFI Design](../rfc/RFC-005_FFI_DESIGN.md#native-dependencies)
8. [RFC-009: Security Model](../rfc/RFC-009_SECURITY_MODEL.md#package-permissions)
9. [RFC-010: Registry Design](../rfc/RFC-010_REGISTRY_DESIGN.md) (Placeholder)
10. [CLI Grammar Specification](../PACKAGE_MANAGER_SPEC.md#cli-grammar) 