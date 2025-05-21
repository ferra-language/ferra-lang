---
title: "Package Management"
duration: "4h"
level: "advanced"
---

# Package Management

> **Duration**: 4 hours
> **Goal**: Manage dependencies and packages using Ferra's package management system

## Overview

This tutorial covers package management, dependency resolution, and version control in Ferra applications, focusing on content-addressed storage, CLI commands, and SBOM generation.

## 1. Package Basics (1 hour)

### 1.1. Package Definition

```ferra
// Package manager actor with content-addressed storage
#[ai::tag(package_component)]
actor PackageManagerActor {
    data ManagerState {
        packages: Map<String, Package>,
        dependencies: Map<String, List<Dependency>>,
        energy_budget: Float,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ManagerState {
        return ManagerState {
            packages: Map::new(),
            dependencies: Map::new(),
            energy_budget: 0.0,
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "package_management"
            }
        }
    }

    async fn handle_install(self_state: ManagerState, request: InstallRequest) -> (ManagerState, Result<Unit>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["pkg:install"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "install",
                request.package_id,
                false,
                "Missing required capabilities"
            );
            return (ManagerState {
                packages: self_state.packages,
                dependencies: self_state.dependencies,
                energy_budget: self_state.energy_budget,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let install_energy_cost = calculate_energy_cost(0.1.joules, self_state.energy_metrics);
        if self_state.energy_budget < install_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "install",
                request.package_id,
                false,
                "Insufficient energy budget"
            );
            return (ManagerState {
                packages: self_state.packages,
                dependencies: self_state.dependencies,
                energy_budget: self_state.energy_budget,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        // Generate CID for package
        let cid = generate_cid(request.package);
        
        // Store package in content-addressed cache
        let cache_path = format("~/.lang/pkg/{}/{}", cid.algorithm, cid.hash);
        await store_package(request.package, cache_path);
        
        // Update manifest and lockfile
        await update_manifest(request.package, cid);
        await update_lockfile(request.package, cid);
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "install",
            request.package_id,
            true,
            null
        );
        
        return (ManagerState {
            packages: self_state.packages.insert(request.package_id, request.package),
            dependencies: self_state.dependencies.insert(request.package_id, request.dependencies),
            energy_budget: self_state.energy_budget,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Unit);
    }
}

// Package definition with CID
type Package = {
    id: String
    name: String
    version: Version
    cid: CID
    dependencies: List<Dependency>
    energy_budget: Float
}

// Content Identifier
type CID = {
    algorithm: String  // e.g., "sha256"
    hash: String      // The actual hash value
}

// Install request
data InstallRequest {
    package_id: String
    package: Package
    dependencies: List<Dependency>
    principal: String
    capabilities: Set<Capability>
    scope: String
}

### 1.2. Version Management

```ferra
// Version manager actor with semantic versioning
#[ai::tag(package_component)]
actor VersionManagerActor {
    data VersionState {
        versions: Map<String, List<Version>>,
        current: Map<String, Version>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> VersionState {
        return VersionState {
            versions: Map::new(),
            current: Map::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "version_management"
            }
        }
    }

    async fn handle_update(self_state: VersionState, request: UpdateRequest) -> (VersionState, Result<Unit>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["pkg:update"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "update",
                request.package_id,
                false,
                "Missing required capabilities"
            );
            return (VersionState {
                versions: self_state.versions,
                current: self_state.current,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let update_energy_cost = calculate_energy_cost(0.05.joules, self_state.energy_metrics);
        if self_state.energy_budget < update_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "update",
                request.package_id,
                false,
                "Insufficient energy budget"
            );
            return (VersionState {
                versions: self_state.versions,
                current: self_state.current,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let versions = self_state.versions.get(request.package_id) || [];
        
        if !versions.contains(request.version) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "update",
                request.package_id,
                false,
                "Version not found"
            );
            return (VersionState {
                versions: self_state.versions,
                current: self_state.current,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Version not found"));
        }
        
        let new_state = VersionState {
            versions: self_state.versions,
            current: self_state.current.insert(request.package_id, request.version),
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        await update_package(request.package_id, request.version);
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "update",
            request.package_id,
            true,
            null
        );
        
        return (VersionState {
            versions: new_state.versions,
            current: new_state.current,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Unit);
    }
}

// Version definition with semantic versioning
type Version = {
    major: Int
    minor: Int
    patch: Int
    pre_release: String?
    build: String?
    cid: CID
    energy_budget: Float
}

// Update request
data UpdateRequest {
    package_id: String
    version: Version
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

## 2. Dependency Resolution (1 hour)

### 2.1. Dependency Graph

```ferra
// Dependency resolver actor with CID-based resolution
#[ai::tag(package_component)]
actor DependencyResolverActor {
    data ResolverState {
        graph: Graph<Package>,
        conflicts: List<Conflict>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ResolverState {
        return ResolverState {
            graph: Graph::new(),
            conflicts: [],
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "dependency_resolution"
            }
        }
    }

    async fn handle_resolve(self_state: ResolverState, request: ResolveRequest) -> (ResolverState, Result<List<Package>>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["pkg:resolve"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "resolve",
                request.package_id,
                false,
                "Missing required capabilities"
            );
            return (ResolverState {
                graph: self_state.graph,
                conflicts: self_state.conflicts,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let resolve_energy_cost = calculate_energy_cost(0.2.joules, self_state.energy_metrics);
        if self_state.energy_budget < resolve_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "resolve",
                request.package_id,
                false,
                "Insufficient energy budget"
            );
            return (ResolverState {
                graph: self_state.graph,
                conflicts: self_state.conflicts,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let resolved = resolve_dependencies(request.package);
        
        for dep in resolved {
            self_state.graph.add_edge(request.package, dep);
        }
        
        let conflicts = detect_conflicts(self_state.graph);
        
        let new_state = ResolverState {
            graph: self_state.graph,
            conflicts: self_state.conflicts.append(conflicts),
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "resolve",
            request.package_id,
            true,
            null
        );
        
        return (ResolverState {
            graph: new_state.graph,
            conflicts: new_state.conflicts,
            energy_metrics: new_metrics,
            security_context: new_context
        }, resolved);
    }
}

// Dependency definition with CID
type Dependency = {
    package: String
    version: Version
    cid: CID
    type: String
    energy_budget: Float
}

// Resolve request
data ResolveRequest {
    package_id: String
    package: Package
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

### 2.2. Conflict Resolution

```ferra
// Conflict resolver actor with CID-based resolution
#[ai::tag(package_component)]
actor ConflictResolverActor {
    data ResolverState {
        conflicts: List<Conflict>,
        strategies: Map<String, Strategy>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ResolverState {
        return ResolverState {
            conflicts: [],
            strategies: Map::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "conflict_resolution"
            }
        }
    }

    async fn handle_resolve(self_state: ResolverState, request: ConflictRequest) -> (ResolverState, Result<Resolution>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["pkg:resolve_conflict"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "resolve_conflict",
                request.conflict_id,
                false,
                "Missing required capabilities"
            );
            return (ResolverState {
                conflicts: self_state.conflicts,
                strategies: self_state.strategies,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let resolve_energy_cost = calculate_energy_cost(0.15.joules, self_state.energy_metrics);
        if self_state.energy_budget < resolve_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "resolve_conflict",
                request.conflict_id,
                false,
                "Insufficient energy budget"
            );
            return (ResolverState {
                conflicts: self_state.conflicts,
                strategies: self_state.strategies,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let strategy = self_state.strategies.get(request.conflict.type);
        
        if !strategy {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "resolve_conflict",
                request.conflict_id,
                false,
                "Strategy not found"
            );
            return (ResolverState {
                conflicts: self_state.conflicts,
                strategies: self_state.strategies,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Strategy not found"));
        }
        
        let resolution = await strategy.resolve(request.conflict);
        
        let new_state = ResolverState {
            conflicts: self_state.conflicts.filter { c => c.id != request.conflict.id },
            strategies: self_state.strategies,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "resolve_conflict",
            request.conflict_id,
            true,
            null
        );
        
        return (ResolverState {
            conflicts: new_state.conflicts,
            strategies: new_state.strategies,
            energy_metrics: new_metrics,
            security_context: new_context
        }, resolution);
    }
}

// Conflict definition with CID
type Conflict = {
    id: String
    type: String
    packages: List<Package>
    cids: List<CID>
    reason: String
    energy_budget: Float
}

// Conflict request
data ConflictRequest {
    conflict_id: String
    conflict: Conflict
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

## 3. Package Publishing (1 hour)

### 3.1. Package Registry

```ferra
// Registry manager actor with CID-based storage
#[ai::tag(package_component)]
actor RegistryManagerActor {
    data RegistryState {
        packages: Map<String, Package>,
        metadata: Map<String, Metadata>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> RegistryState {
        return RegistryState {
            packages: Map::new(),
            metadata: Map::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "registry_management"
            }
        }
    }

    async fn handle_publish(self_state: RegistryState, request: PublishRequest) -> (RegistryState, Result<Unit>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["pkg:publish"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "publish",
                request.package_id,
                false,
                "Missing required capabilities"
            );
            return (RegistryState {
                packages: self_state.packages,
                metadata: self_state.metadata,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let publish_energy_cost = calculate_energy_cost(0.3.joules, self_state.energy_metrics);
        if self_state.energy_budget < publish_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "publish",
                request.package_id,
                false,
                "Insufficient energy budget"
            );
            return (RegistryState {
                packages: self_state.packages,
                metadata: self_state.metadata,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        // Generate CID for package
        let cid = generate_cid(request.package);
        
        // Generate SBOM
        let sbom = generate_sbom(request.package);
        
        // Sign package and SBOM
        let signature = sign_package(request.package, sbom);
        
        // Store in registry
        await store_in_registry(request.package, cid, sbom, signature);
        
        let metadata = generate_metadata(request.package);
        
        let new_state = RegistryState {
            packages: self_state.packages.insert(request.package_id, request.package),
            metadata: self_state.metadata.insert(request.package_id, metadata),
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "publish",
            request.package_id,
            true,
            null
        );
        
        return (RegistryState {
            packages: new_state.packages,
            metadata: new_state.metadata,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Unit);
    }
}

// Metadata definition with SBOM
type Metadata = {
    id: String
    description: String
    authors: List<String>
    license: String
    cid: CID
    sbom: SBOM
    signature: Signature
    energy_budget: Float
}

// Publish request
data PublishRequest {
    package_id: String
    package: Package
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

## Key Concepts

1. **Content-Addressed Storage**:
   - CID-based package identification
   - Immutable package storage
   - Efficient de-duplication
   - Integrity verification

2. **Package Management**:
   - Semantic versioning
   - Dependency resolution
   - Conflict handling
   - Energy profiling

3. **Security**:
   - Package signing
   - SBOM generation
   - Capability-based access
   - Audit logging

4. **CLI Commands**:
   - `lang add` for adding dependencies
   - `lang vendor` for vendoring packages
   - `lang build` for building projects
   - `lang publish` for publishing packages

5. **Semantic Tags**:
   - `#[ai::tag(package_component)]` for package components
   - `#[ai::tag(package_operation)]` for package operations
   - Security context integration
   - Energy metrics tracking

## Best Practices

1. **Package Management**:
   - Use content-addressed storage
   - Implement semantic versioning
   - Generate SBOMs
   - Sign packages

2. **Dependency Resolution**:
   - Use CID-based resolution
   - Handle conflicts gracefully
   - Track energy usage
   - Maintain audit logs

3. **Package Publishing**:
   - Generate CIDs
   - Create SBOMs
   - Sign packages
   - Update registry

4. **Security Context**:
   - Propagate security context
   - Check required capabilities
   - Monitor energy usage
   - Maintain audit trail

5. **Energy Profiling**:
   - Track operation energy
   - Monitor component energy
   - Optimize energy usage
   - Log energy metrics

## Quiz

1. What is the main benefit of content-addressed storage?
   - A. Better performance
   - B. Integrity verification
   - C. Simpler implementation
   - D. Faster response times

2. How do you handle package conflicts?
   - A. Using resolution strategies
   - B. Using CID-based resolution
   - C. Both A and B
   - D. Neither A nor B

3. Which security feature prevents package tampering?
   - A. SBOM generation
   - B. Package signing
   - C. CID verification
   - D. All of the above

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Package Manager Spec](../../reference/PACKAGE_MANAGER_SPEC.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)
- [Security Model](../../reference/SECURITY_MODEL.md)
- [Energy Profiler](../../reference/ENERGY_PROFILER.md)

## Next Steps

- [API Design & Integration](./api_design.md)
- [Monitoring & Operations](./monitoring_ops.md)
- [Advanced Topics](./advanced_topics.md)

## Additional Code

### Example: Generate SBOM
fn generate_sbom(package: Package) -> SBOM {
    // ... SBOM generation logic ...
    return SBOM::from_package(package);
}

### Example: Validate package structure
fn validate_package_structure(package: Package) -> Bool {
    // ... validation logic ...
    return true;
} 