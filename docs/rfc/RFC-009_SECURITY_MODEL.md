---
number: RFC-009
title: "Security Model Design"
status: Draft
version: v0.3
authors: ["Amrit Doll <amritdoll@example.com>"]
last_reviewed: 2025-05-21
last_updated: 2025-05-21
---

# RFC-009: Security Model Design

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
This RFC specifies the design for Ferra's security model, focusing on capability-based permissions and sandboxing mechanisms. The model aims to provide fine-grained control over program capabilities while maintaining a strong security posture through multiple layers of enforcement.

## 2. Motivation
Modern software development requires robust security mechanisms that protect both developers and users. Ferra's security model aims to:
- Provide fine-grained control over program capabilities through explicit permission declarations
- Implement strong sandboxing mechanisms for runtime enforcement
- Ensure security is a first-class concern throughout the language and ecosystem
- Enable secure-by-default development practices

## 3. Impact
### 3.1 Developer Experience
- Clear, explicit permission declarations in `Ferra.toml`
- Intuitive tooling for permission management and auditing
- Comprehensive error messages for security-related issues
- Seamless integration with IDE and build tools

### 3.2 Ecosystem
- Enhanced supply chain security through transparent permission requirements
- Standardized security practices across the ecosystem
- Improved trust in third-party packages
- Better security auditing capabilities

### 3.3 Performance
- **Sandboxing Overhead**: < 5 µs for sandbox transitions
- **Permission Checks**: Negligible impact on runtime performance
- **Compile-time Analysis**: Minimal impact on build times
- **Resource Usage**: Efficient memory and CPU utilization for security features

## 4. Design Decisions

### 4.1 Capability-Based Permissions
- **Core Concept**: Programs must explicitly declare required permissions in `Ferra.toml`
- **Permission String Grammar**:
  ```ebnf
  permission_string = category ":" operation [":" resource_specifier [":" options]] ;
  category = "net" | "fs" | "env" | "ffi" | "sys" ;
  operation = "fetch" | "connect" | "read" | "write" | "load" | "exec" ;
  resource_specifier = string | glob_pattern ;
  options = option {"," option} ;
  option = "readonly" | "temporary" | "isolated" ;
  ```
- **Syntax**:
  ```toml
  [package.permissions]
  # Network permissions
  net_fetch = ["https://api.example.com/data"]
  net_connect = ["localhost:8080"]
  
  # Filesystem permissions
  fs_read = ["./config.json", "/data/input/*.csv"]
  fs_write = ["./output/"]
  
  # Environment variables
  env_read = ["USER_CONFIG_PATH"]
  
  # FFI permissions
  ffi_load = ["libssl.so.*"]
  ```

- **Permission Resolution & Caching**:
  ```ferra
  struct PermissionCache {
      // In-memory cache of resolved permissions
      resolved_perms: Map<PackageId, Set<Permission>>,
      // Cache invalidation triggers
      invalidation_triggers: [
        "manifest_change",
        "dependency_update",
        "permission_override",
        "cache_ttl_expired"
      ],
      // Cache TTL: 5 minutes
      ttl_seconds: 300,
  }
  ```

### 4.2 Dependency Permissions
- **Transitive Analysis**: Permissions aggregate across dependency tree
- **Audit Process**:
  ```bash
  # Audit all dependencies
  lang permissions audit --recursive
  
  # Show permission summary
  lang permissions summary
  
  # Generate permission report
  lang permissions report --format=spdx
  ```
- **Permission Resolution**:
  1. Direct dependencies inherit declared permissions
  2. Transitive dependencies require explicit opt-in
  3. Conflicts resolved by most restrictive policy
  4. Overrides possible via `[package.permissions.override]`

### 4.3 Platform-Specific Sandboxing
Each platform implements sandboxing according to its native capabilities:

#### Linux (seccomp-bpf)
```toml
[package.permissions]
fs_read = ["./config.toml"]
net_connect = ["https://api.example.com"]

# Generated seccomp profile
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "syscalls": [
    {
      "name": "read",
      "action": "SCMP_ACT_ALLOW",
      "args": [
        {
          "index": 1,
          "op": "SCMP_CMP_EQ",
          "value": "./config.toml"
        }
      ]
    }
  ]
}
```

#### Windows (AppContainer)
```toml
[package.permissions]
fs_read = ["./config.toml"]
net_connect = ["https://api.example.com"]

# Generated AppContainer manifest
{
  "executable": "app.exe",
  "capabilities": [
    "internetClient",
    "documentsLibrary"
  ],
  "restrictedCapabilities": [
    "runFullTrust"
  ]
}
```

#### macOS (Sandbox)
```toml
[package.permissions]
fs_read = ["./config.toml"]
net_connect = ["https://api.example.com"]

# Generated sandbox profile
(version 1)
(allow file-read* (path "./config.toml"))
(allow network-outbound (host "api.example.com"))
```

### 4.4 Permission Override Semantics
Permission overrides follow these rules:

1. **Direct Override**: Explicit override takes precedence
```toml
# Original permissions
[package.permissions]
fs_read = ["./config.toml"]
net_connect = ["https://api.example.com"]

# Override in child package
[package.permissions.override]
fs_read = ["./data/*"]  # Replaces parent's fs_read
net_connect = []        # Removes all network access
```

2. **Transitive Override**: Overrides propagate to dependencies
```toml
# Parent package
[package.permissions]
fs_read = ["./config.toml"]

# Child package
[package.permissions.override]
fs_read = ["./data/*"]

# Grandchild package inherits child's override
# No need to specify override again
```

3. **Conflict Resolution**: Most restrictive override wins
```toml
# Package A
[package.permissions]
fs_read = ["./config.toml", "./data/*"]

# Package B (depends on A)
[package.permissions.override]
fs_read = ["./config.toml"]  # More restrictive

# Package C (depends on A)
[package.permissions.override]
fs_read = ["./data/*"]  # More restrictive

# Final effective permissions
# fs_read = []  # Intersection is empty
```

### 4.5 Enforcement Layers
1. **Compile-time Checks**:
   - Static analysis of stdlib calls
   - FFI usage verification
   - Permission requirement validation
   - Memory safety analysis
   - Resource usage estimation

2. **Runtime Checks**:
   - Resource access validation
   - Dynamic permission verification
   - Sandbox enforcement
   - Memory bounds checking
   - Stack overflow detection

3. **OS-level Enforcement**:
   - WASI capabilities
   - seccomp-bpf filters
   - Process isolation
   - Memory protection
   - Resource limits

### 4.6 Error Handling
- **Error Types**:
  ```ferra
  struct PermissionDeniedError {
      code: String,      // e.g., "SEC-PERM-001"
      message: String,   // Human-readable description
      details: Map,      // Additional context
      suggestion: String // Optional fix suggestion
  }

  struct SecurityViolationError {
      code: String,      // e.g., "SEC-VIOL-001"
      message: String,   // Human-readable description
      details: Map,      // Additional context
      stack_trace: String // Security-relevant stack trace
  }
  ```
- **Error Codes**:
  | Code | Description | Severity | Spec Reference |
  |------|-------------|-----------|----------------|
  | SEC-PERM-001 | Missing required permission | Error | SECURITY_MODEL.md#4.1-permission-syntax |
  | SEC-PERM-002 | Invalid permission string | Error | SECURITY_MODEL.md#4.1-permission-grammar |
  | SEC-PERM-003 | Resource access denied | Error | SECURITY_MODEL.md#4.3-enforcement-layers |
  | SEC-SANDBOX-001 | Sandbox violation | Error | SECURITY_MODEL.md#4.3-sandboxing-mechanisms |
  | SEC-SANDBOX-002 | Profile generation failed | Error | SECURITY_MODEL.md#4.3-profile-generation |
  | SEC-MEM-001 | Memory safety violation | Error | SECURITY_MODEL.md#4.3-memory-protection |
  | SEC-RES-001 | Resource limit exceeded | Error | SECURITY_MODEL.md#4.3-resource-limits |

### 4.7 Build System Integration
- **Build Script Directives**:
  ```toml
  [build]
  security_level = "strict"  # strict, standard, permissive
  sandbox_type = "wasi"      # wasi, seccomp, none
  memory_safety = true       # enable memory safety checks
  resource_limits = true     # enable resource limits
  
  [build.security]
  max_memory_mb = 512
  max_cpu_percent = 50
  max_file_descriptors = 100
  ```

- **CI/CD Integration**:
  ```yaml
  security_checks:
    - name: permission_audit
      command: lang doctor security --audit
      threshold: 0.95  # 95% of permissions must be valid
    
    - name: sandbox_verification
      command: lang doctor security --verify-sandbox
      timeout: 30s
    
    - name: memory_safety
      command: lang doctor security --memory-safety
      fail_on_warning: true
  ```

## 5. Drawbacks
- **Complexity**: Adds new abstraction layers and configuration requirements
- **Learning Curve**: Developers must understand permission model
- **Performance**: Small overhead from sandboxing and checks
- **Maintenance**: Permission strings must be kept up-to-date

## 6. Security & Privacy
- **Deny by Default**: All capabilities require explicit permission
- **Least Privilege**: Programs only get declared permissions
- **Transparency**: Clear permission requirements and audit trail
- **Isolation**: Strong sandboxing prevents privilege escalation

## 7. Implementation Plan
- **Phase 1a (Q1 2026)**: Core Permission System
  - Permission string syntax and validation
  - Manifest integration
  - Basic compile-time checks
  - Test harness:
    - Unit tests for permission parsing
    - Integration tests for manifest loading
    - Property-based tests for permission validation
    - Fuzzing for permission string parsing
    - Performance benchmarks for permission checks

- **Phase 1b (Q1 2026)**: Sandboxing Implementation
  - WASI integration
  - seccomp-bpf profile generation
  - Runtime enforcement
  - CLI tools: `lang permissions`, `lang doctor security`
  - Test harness:
    - Sandbox isolation tests
    - Resource limit tests
    - Memory safety tests
    - Performance benchmarks
  - Metrics:
    - Sandbox transition latency: < 5 µs
    - Permission check overhead: < 1 µs
    - Profile generation time: < 100 ms
    - Memory safety check overhead: < 2 µs
    - False positive rate: < 0.1%

- **Phase 2 (Q2 2026)**: Enhanced Security Features
  - Advanced permission patterns
  - Improved sandboxing
  - Security auditing tools
  - Metrics: False positive rate, audit coverage

- **Phase 3 (Q3 2026)**: Ecosystem Integration
  - Package manager integration
  - IDE support
  - Documentation and examples
  - Metrics: Developer adoption, security incident rate

## 8. Migration Strategy
The security model supports gradual adoption:

1. **Phase 1: Strict-Deny Mode (v0.1)**
   - All manifests without permissions default to strict-deny
   - Basic permission categories: `fs`, `net`, `sys`
   - Simple override syntax
   ```toml
   # Default: no permissions
   [package]
   name = "myapp"
   
   # Opt-in to specific permissions
   [package.permissions]
   fs_read = ["./config.toml"]
   ```

2. **Phase 2: Granular Permissions (v0.2)**
   - Expanded permission categories
   - Transitive permission handling
   - Platform-specific sandboxing
   ```toml
   [package.permissions]
   fs_read = ["./config.toml"]
   net_connect = ["https://api.example.com"]
   sys_env = ["PATH", "HOME"]
   ```

3. **Phase 3: Advanced Security (v1.0)**
   - Full platform support
   - Advanced sandboxing
   - Audit and compliance features
   ```toml
   [package.permissions]
   fs_read = ["./config.toml"]
   net_connect = ["https://api.example.com"]
   sys_env = ["PATH", "HOME"]
   
   [package.security]
   audit = true
   compliance = ["SOC2", "ISO27001"]
   ```

## 9. Unresolved Questions
### High Priority
- SEC-PERM-SYNTAX-1: Final permission string syntax
- SEC-PERM-DEPS-1: Dependency permission strategy
- SEC-SANDBOX-SECCOMP-1: seccomp-bpf design details
- SEC-PERM-CACHE-1: Permission caching strategy
- SEC-MEM-SAFETY-1: Memory safety verification
- SEC-WIN-SANDBOX-1: Windows sandboxing implementation
- SEC-MAC-SANDBOX-1: macOS sandboxing implementation

### Medium Priority
- SEC-ACTOR-PERM-1: Per-actor permissions
- SEC-TOOL-AUDIT-1: Permission audit tool details
- SEC-PERM-ENFORCE-1: Enforcement mechanisms
- SEC-PROFILE-GEN-1: Profile generation optimization
- SEC-DIAG-INTEG-1: Diagnostic integration strategy

### Low Priority
- SEC-PERM-PROMPT-1: Runtime permission prompts
- SEC-PERM-DYNAMIC-1: Dynamic permission management
- SEC-PERF-OPT-1: Performance optimization strategy
- SEC-TOOL-UI-1: Tooling UI/UX design
- SEC-CROSS-PLAT-1: Cross-platform sandboxing strategy

## 10. Future Possibilities
- Per-actor permission isolation
- Runtime permission prompts
- Advanced sandboxing technologies
- Formal verification of security properties
- Integration with security auditing tools
- Windows sandboxing via Job Objects/AppContainer
- macOS sandboxing via Seatbelt/App Sandbox
- Cross-platform sandboxing unification
- Permission caching optimization
- Dynamic permission management
- Security policy templates
- Platform-specific sandbox profiles

## 11. References
1. [SECURITY_MODEL.md](../SECURITY_MODEL.md)
2. [FFI_C_CPP.md](../FFI_C_CPP.md)
3. [CONCURRENCY_MODEL.md](../CONCURRENCY_MODEL.md)
4. [AI_API_REFACTOR_VERIFY.md](../AI_API_REFACTOR_VERIFY.md)
5. [BACKEND_WASM_WASI.md](../BACKEND_WASM_WASI.md)
6. [diagnostic_codes.md](../diagnostic_codes.md)
7. [DESIGN_DIAGNOSTICS.md](../DESIGN_DIAGNOSTICS.md)
8. [Steps.md](../Steps.md)
9. [RFC-005_FFI.md](./RFC-005_FFI.md)
10. [RFC-006_PACKAGE_MANAGER.md](./RFC-006_PACKAGE_MANAGER.md) 