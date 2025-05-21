---
title: "FFI Integration"
duration: "4h"
level: "advanced"
---

# FFI Integration

> **Duration**: 4 hours
> **Goal**: Integrate native code and external libraries using Ferra's Foreign Function Interface

## Overview

This tutorial covers FFI integration, native code binding, and performance optimization in Ferra applications.

## 1. FFI Basics (1 hour)

### 1.1. Function Binding

```ferra
// FFI manager actor with security model and energy profiling
#[ai::tag(ffi_component)]
actor FFIManagerActor {
    data ManagerState {
        bindings: Map<String, Binding>,
        libraries: Map<String, Library>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ManagerState {
        return ManagerState {
            bindings: Map::new(),
            libraries: Map::new(),
            energy_budget: 1000.0.joules,
            permissions: Set::new(),
            energy_metrics: EnergyMetrics {
                total_ops: 0,
                alu_ops: 0,
                mem_ops: 0,
                fp_ops: 0,
                last_measurement: now()
            },
            security_context: SecurityContext {
                principal: "system",
                granted_capabilities: Set::new(),
                scope: "ffi_management",
                audit_log: []
            }
        }
    }

    async fn handle_bind(self_state: ManagerState, request: BindRequest) -> (ManagerState, Result<Unit, BindingError>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::BindingManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "bind",
                request.binding.id,
                false,
                "Missing required capabilities"
            );
            return (self_state, Err(BindingError::PermissionDenied));
        }

        // Check energy budget
        let binding_energy_cost = calculate_energy_cost(10.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < binding_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "bind",
                request.binding.id,
                false,
                "Insufficient energy budget"
            );
            return (self_state, Err(BindingError::InsufficientEnergy));
        }

        // Get library
        let library = match self_state.libraries.get(request.binding.library) {
            Some(l) => l,
            None => {
                let end_ops = measure_ops();
                let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                let new_context = audit_operation(
                    self_state.security_context,
                    request.principal,
                    "bind",
                    request.binding.id,
                    false,
                    "Library not found"
                );
                return (self_state, Err(BindingError::LibraryNotFound));
            }
        };

        // Register binding
        match await register_binding(request.binding, library) {
            Ok(_) => {
                let end_ops = measure_ops();
                let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                let new_context = audit_operation(
                    self_state.security_context,
                    request.principal,
                    "bind",
                    request.binding.id,
                    true,
                    null
                );
                let new_state = ManagerState {
                    bindings: self_state.bindings.insert(request.binding.id, request.binding),
                    libraries: self_state.libraries,
                    energy_budget: self_state.energy_budget - binding_energy_cost,
                    permissions: self_state.permissions,
                    energy_metrics: new_metrics,
                    security_context: new_context
                };
                return (new_state, Ok(Unit));
            }
            Err(e) => {
                let end_ops = measure_ops();
                let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                let new_context = audit_operation(
                    self_state.security_context,
                    request.principal,
                    "bind",
                    request.binding.id,
                    false,
                    "Registration failed"
                );
                return (self_state, Err(BindingError::RegistrationFailed));
            }
        }
    }
}

// Message definitions
data BindRequest {
    binding: Binding
    principal: String
    capabilities: Set<Capability>
    scope: String
}

data BindingError {
    LibraryNotFound
    PermissionDenied
    InsufficientEnergy
    RegistrationFailed
    InvalidRequest
}

// Security model
data Capability {
    BindingManagement
    LibraryManagement
    EnergyManagement
}

// Binding definition
type Binding = {
    id: String
    library: String
    function: String
    signature: Signature
    energy_budget: Float
    tdp_factors: TDPFactors
}

// Type mapping
data TypeMapping {
    c_type: String
    ferra_type: Type
    size: Int
    alignment: Int
    energy_budget: Float
}

// Example of C type mapping
const TYPE_MAPPINGS: Map<String, TypeMapping> = {
    "char": TypeMapping {
        c_type: "char",
        ferra_type: "c_char",
        size: 1,
        alignment: 1,
        energy_budget: 0.1.joules
    },
    "int": TypeMapping {
        c_type: "int",
        ferra_type: "c_int",
        size: 4,
        alignment: 4,
        energy_budget: 0.2.joules
    },
    "float": TypeMapping {
        c_type: "float",
        ferra_type: "c_float",
        size: 4,
        alignment: 4,
        energy_budget: 0.3.joules
    },
    "double": TypeMapping {
        c_type: "double",
        ferra_type: "c_double",
        size: 8,
        alignment: 8,
        energy_budget: 0.4.joules
    }
}

// Example of C struct mapping
#[repr(C)]
data CPoint {
    x: c_int
    y: c_int
    energy_budget: Float
}

// Example of C function declaration
extern "C" {
    fn process_point(p: CPoint) -> c_int;
    fn get_point() -> CPoint;
    fn process_point_ptr(p_ptr: *const CPoint) -> c_int;
}

// Example of C callback type
type CCallback = extern "C" fn(c_int, c_int) -> c_int;

extern "C" {
    fn register_callback(cb: CCallback) -> c_int;
}

// Example of opaque handle
#[repr(transparent)]
data OpaqueHandle {
    ptr: *mut c_void
    energy_budget: Float
}

extern "C" {
    fn create_handle() -> OpaqueHandle;
    fn process_with_handle(h: OpaqueHandle) -> c_int;
    fn destroy_handle(h: OpaqueHandle) -> Unit;
}

// Example of safe wrapper
fn safe_process_point(p: CPoint) -> Result<c_int, FFIError> {
    unsafe {
        let result = process_point(p);
        if result < 0 {
            return Err(FFIError::ProcessingFailed);
        }
        return Ok(result);
    }
}

// Example of string conversion
fn c_string_to_ferra(c_str: *const c_char) -> Result<String, FFIError> {
    unsafe {
        if c_str.is_null() {
            return Err(FFIError::NullPointer);
        }
        let len = strlen(c_str);
        let mut result = String::with_capacity(len);
        for i in 0..len {
            result.push(*c_str.add(i) as char);
        }
        return Ok(result);
    }
}

fn ferra_to_c_string(s: String) -> Result<*mut c_char, FFIError> {
    unsafe {
        let len = s.len();
        let c_str = ferra_alloc((len + 1) * size_of::<c_char>()) as *mut c_char;
        if c_str.is_null() {
            return Err(FFIError::AllocationFailed);
        }
        for (i, c) in s.chars().enumerate() {
            *c_str.add(i) = c as c_char;
        }
        *c_str.add(len) = 0;
        return Ok(c_str);
    }
}

// Error types
data FFIError {
    NullPointer
    AllocationFailed
    ProcessingFailed
    InvalidType
    ConversionFailed
}

// Energy profiling
data EnergyMetrics {
    total_ops: Int
    alu_ops: Int
    mem_ops: Int
    fp_ops: Int
    last_measurement: Time
}

data TDPFactors {
    alu: Float
    mem: Float
    fp: Float
}

// Constants for TDP factors
const TDP_FACTOR_ALU: Float = 1.0;
const TDP_FACTOR_MEM: Float = 5.0;
const TDP_FACTOR_FP: Float = 3.0;

// LLVM pass implementation
#[llvm_pass]
fn measure_ops() -> EnergyMetrics {
    return EnergyMetrics {
        total_ops: 0,
        alu_ops: 0,
        mem_ops: 0,
        fp_ops: 0,
        last_measurement: now()
    };
}

fn update_energy_metrics(current: EnergyMetrics, start: EnergyMetrics, end: EnergyMetrics) -> EnergyMetrics {
    return EnergyMetrics {
        total_ops: current.total_ops + (end.total_ops - start.total_ops),
        alu_ops: current.alu_ops + (end.alu_ops - start.alu_ops),
        mem_ops: current.mem_ops + (end.mem_ops - start.mem_ops),
        fp_ops: current.fp_ops + (end.fp_ops - start.fp_ops),
        last_measurement: now()
    };
}

fn calculate_energy_cost(base_cost: Float, metrics: EnergyMetrics) -> Float {
    let alu_cost = metrics.alu_ops * TDP_FACTOR_ALU;
    let mem_cost = metrics.mem_ops * TDP_FACTOR_MEM;
    let fp_cost = metrics.fp_ops * TDP_FACTOR_FP;
    
    return base_cost + (alu_cost + mem_cost + fp_cost).joules;
}

// Example: Safe FFI call with type mapping and error handling
fn safe_ffi_call(binding: Binding, args: List<Any>) -> Result<Any, FFIError> {
    let mapping = TYPE_MAPPINGS.get(binding.signature.return_type)
        || return Err(FFIError::TypeMappingNotFound);
    // ... perform type-safe call ...
    match ffi_invoke(binding, args) {
        Ok(result) => Ok(mapping.cast(result)),
        Err(e) => Err(FFIError::InvocationFailed)
    }
}
```

### 1.2. Type Mapping

```ferra
// Type mapper actor with security model and energy profiling
actor TypeMapperActor {
    data MapperState {
        mappings: Map<Type, Type>,
        converters: Map<Type, Converter>,
        energy_budget: Float,  // Total energy budget for the mapper
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> MapperState {
        return MapperState {
            mappings: Map::new(),
            converters: Map::new(),
            energy_budget: 2000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_map(self_state: MapperState, request: MapRequest) -> (MapperState, Result<Type, MappingError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::TypeMapping) {
            return (self_state, Err(MappingError::PermissionDenied))
        }

        // Check energy budget
        let mapping_energy_cost = 20.0.joules
        if self_state.energy_budget < mapping_energy_cost {
            return (self_state, Err(MappingError::InsufficientEnergy))
        }

        let converter = match self_state.converters.get(request.source_type) {
            Some(c) => c,
            None => return (self_state, Err(MappingError::ConverterNotFound))
        }

        let new_state = MapperState {
            mappings: self_state.mappings.insert(request.source_type, request.target_type),
            converters: self_state.converters,
            energy_budget: self_state.energy_budget - mapping_energy_cost,
            permissions: self_state.permissions
        }

        match await convert_type(request.source_type, request.target_type, converter) {
            Ok(t) => return (new_state, Ok(t)),
            Err(e) => return (self_state, Err(MappingError::ConversionFailed))
        }
    }
}

// Message definitions
data MapRequest {
    source_type: Type
    target_type: Type
    energy_budget: Float
}

data MappingError {
    ConverterNotFound
    PermissionDenied
    InsufficientEnergy
    ConversionFailed
    InvalidType
}

// Security model
data Capability {
    TypeMapping
    ConverterManagement
    EnergyManagement
}

// Type definitions
type Type = {
    name: String
    size: Int
    alignment: Int
    energy_budget: Float
}
```

## 2. Native Code Integration (1 hour)

### 2.1. Library Loading

```ferra
// Library loader actor
actor LibraryLoaderActor {
    data LoaderState {
        libraries: Map<String, Library>,
        dependencies: Map<String, List<String>>
    }

    fn init() -> LoaderState {
        return LoaderState {
            libraries: Map::new(),
            dependencies: Map::new()
        }
    }

    async fn handle_load(self_state: LoaderState, library: Library) -> (LoaderState, Result<Unit>) {
        let dependencies = resolve_dependencies(library)
        
        for dep in dependencies {
            await load_dependency(dep)
        }
        
        let new_state = LoaderState {
            libraries: self_state.libraries.insert(library.id, library),
            dependencies: self_state.dependencies.insert(library.id, dependencies)
        }
        
        await load_library(library)
        
        return (new_state, Unit)
    }
}

// Library definition
type Library = {
    id: String
    path: String
    symbols: List<Symbol>
    energy_budget: Float
}
```

### 2.2. Symbol Resolution

```ferra
// Symbol resolver actor
actor SymbolResolverActor {
    data ResolverState {
        symbols: Map<String, Symbol>,
        bindings: Map<String, Binding>
    }

    fn init() -> ResolverState {
        return ResolverState {
            symbols: Map::new(),
            bindings: Map::new()
        }
    }

    async fn handle_resolve(self_state: ResolverState, name: String) -> (ResolverState, Result<Symbol>) {
        let symbol = self_state.symbols.get(name)
        
        if !symbol {
            return (self_state, Error("Symbol not found"))
        }
        
        let new_state = ResolverState {
            symbols: self_state.symbols,
            bindings: self_state.bindings
        }
        
        return (new_state, symbol)
    }
}

// Symbol definition
type Symbol = {
    name: String
    type: String
    address: Int
    energy_budget: Float
}
```

## 3. Memory Management (1 hour)

### 3.1. Memory Allocation

```ferra
// Memory manager actor with security model and energy profiling
actor MemoryManagerActor {
    data ManagerState {
        allocations: Map<Ptr, Allocation>,
        pools: Map<String, MemoryPool>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> ManagerState {
        return ManagerState {
            allocations: Map::new(),
            pools: Map::new(),
            energy_budget: 3000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_allocate(self_state: ManagerState, request: AllocateRequest) -> (ManagerState, Result<Ptr, AllocationError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::MemoryManagement) {
            return (self_state, Err(AllocationError::PermissionDenied))
        }

        // Check energy budget
        let allocation_energy_cost = 30.0.joules
        if self_state.energy_budget < allocation_energy_cost {
            return (self_state, Err(AllocationError::InsufficientEnergy))
        }

        let pool = match self_state.pools.get(request.pool_name) {
            Some(p) => p,
            None => return (self_state, Err(AllocationError::PoolNotFound))
        }

        let new_state = ManagerState {
            allocations: self_state.allocations,
            pools: self_state.pools,
            energy_budget: self_state.energy_budget - allocation_energy_cost,
            permissions: self_state.permissions
        }

        match await allocate_memory(request.size, request.alignment, pool) {
            Ok(ptr) => {
                let allocation = Allocation {
                    ptr: ptr,
                    size: request.size,
                    alignment: request.alignment,
                    pool: request.pool_name
                }
                return (new_state.allocations.insert(ptr, allocation), Ok(ptr))
            },
            Err(e) => return (self_state, Err(AllocationError::AllocationFailed))
        }
    }
}

// Message definitions
data AllocateRequest {
    size: Int
    alignment: Int
    pool_name: String
    energy_budget: Float
}

data AllocationError {
    PoolNotFound
    PermissionDenied
    InsufficientEnergy
    AllocationFailed
    InvalidRequest
}

// Security model
data Capability {
    MemoryManagement
    PoolManagement
    EnergyManagement
}

// Memory allocation
type Allocation = {
    ptr: Ptr
    size: Int
    alignment: Int
    pool: String
    energy_budget: Float
}
```

### 3.2. Memory Safety

```ferra
// Memory safety actor
actor MemorySafetyActor {
    data SafetyState {
        checks: Map<String, Check>,
        violations: List<Violation>
    }

    fn init() -> SafetyState {
        return SafetyState {
            checks: Map::new(),
            violations: []
        }
    }

    async fn handle_check(self_state: SafetyState, allocation: Allocation) -> (SafetyState, Result<Unit>) {
        let violations = []
        
        for check in self_state.checks {
            if !check.verify(allocation) {
                violations.append(Violation {
                    check: check.id,
                    allocation: allocation.id,
                    reason: check.reason,
                    energy_budget: 0.01.joules
                })
            }
        }
        
        if !violations.is_empty() {
            return (self_state, Error("Memory safety violation"))
        }
        
        let new_state = SafetyState {
            checks: self_state.checks,
            violations: self_state.violations.append(violations)
        }
        
        return (new_state, Unit)
    }
}

// Safety check
type Check = {
    id: String
    type: String
    verify: Function
    energy_budget: Float
}
```

## 4. Performance Optimization (1 hour)

### 4.1. Call Optimization

```ferra
// Call optimizer actor
actor CallOptimizerActor {
    data OptimizerState {
        optimizations: Map<String, Optimization>,
        results: Map<String, Result>
    }

    fn init() -> OptimizerState {
        return OptimizerState {
            optimizations: Map::new(),
            results: Map::new()
        }
    }

    async fn handle_optimize(self_state: OptimizerState, call: Call) -> (OptimizerState, Result<Call>) {
        let optimizations = [
            inline_small_calls,
            batch_similar_calls,
            cache_frequent_calls,
            optimize_parameters
        ]
        
        let optimized = call
        for opt in optimizations {
            optimized = await opt(optimized)
        }
        
        let new_state = OptimizerState {
            optimizations: self_state.optimizations,
            results: self_state.results.insert(call.id, Result {
                performance_improvement: measure_performance(call, optimized),
                energy_saved: measure_energy_saved(call, optimized)
            })
        }
        
        return (new_state, optimized)
    }
}

// Call definition
type Call = {
    id: String
    function: String
    arguments: List<Any>
    energy_budget: Float
}
```

### 4.2. Resource Management

```ferra
// Resource manager actor
actor ResourceManagerActor {
    data ResourceState {
        resources: Map<String, Resource>,
        allocations: Map<String, Allocation>
    }

    fn init() -> ResourceState {
        return ResourceState {
            resources: Map::new(),
            allocations: Map::new()
        }
    }

    async fn handle_allocate(self_state: ResourceState, request: AllocationRequest) -> (ResourceState, Result<Resource>) {
        let resource = select_resource(request)
        
        if !resource {
            return (self_state, Error("No suitable resource found"))
        }
        
        let new_state = ResourceState {
            resources: self_state.resources.insert(resource.id, resource),
            allocations: self_state.allocations.insert(request.id, Allocation {
                id: request.id,
                resource: resource.id,
                energy_budget: request.energy_budget
            })
        }
        
        return (new_state, resource)
    }
}

// Resource definition
type Resource = {
    id: String
    type: String
    capacity: Int
    energy_budget: Float
}
```

### 4.3. Performance Optimization

```ferra
// Performance optimizer actor with security model and energy profiling
actor PerformanceOptimizerActor {
    data OptimizerState {
        optimizations: Map<String, Optimization>,
        metrics: Map<String, Metric>,
        energy_budget: Float,  // Total energy budget for the optimizer
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> OptimizerState {
        return OptimizerState {
            optimizations: Map::new(),
            metrics: Map::new(),
            energy_budget: 4000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_optimize(self_state: OptimizerState, request: OptimizeRequest) -> (OptimizerState, Result<Optimization, OptimizationError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::PerformanceOptimization) {
            return (self_state, Err(OptimizationError::PermissionDenied))
        }

        // Check energy budget
        let optimization_energy_cost = 40.0.joules
        if self_state.energy_budget < optimization_energy_cost {
            return (self_state, Err(OptimizationError::InsufficientEnergy))
        }

        let metric = match self_state.metrics.get(request.metric_name) {
            Some(m) => m,
            None => return (self_state, Err(OptimizationError::MetricNotFound))
        }

        let new_state = OptimizerState {
            optimizations: self_state.optimizations,
            metrics: self_state.metrics,
            energy_budget: self_state.energy_budget - optimization_energy_cost,
            permissions: self_state.permissions
        }

        match await apply_optimization(request.target, request.strategy, metric) {
            Ok(opt) => {
                let optimization = Optimization {
                    id: generate_id(),
                    target: request.target,
                    strategy: request.strategy,
                    metric: request.metric_name,
                    energy_budget: request.energy_budget
                }
                return (new_state.optimizations.insert(optimization.id, optimization), Ok(optimization))
            },
            Err(e) => return (self_state, Err(OptimizationError::OptimizationFailed))
        }
    }
}

// Message definitions
data OptimizeRequest {
    target: String
    strategy: String
    metric_name: String
    energy_budget: Float
}

data OptimizationError {
    MetricNotFound
    PermissionDenied
    InsufficientEnergy
    OptimizationFailed
    InvalidRequest
}

// Security model
data Capability {
    PerformanceOptimization
    MetricManagement
    EnergyManagement
}

// Optimization definition
type Optimization = {
    id: String
    target: String
    strategy: String
    metric: String
    energy_budget: Float
}
```

## Quiz

1. What is the main benefit of FFI integration?
   - A. Better performance
   - B. Native code access
   - C. Simpler implementation
   - D. Faster response times

2. How do you handle memory management in FFI?
   - A. Using allocations
   - B. Using pools
   - C. Both A and B
   - D. Neither A nor B

3. Which system is used for performance optimization?
   - A. Call optimization
   - B. Resource management
   - C. Both A and B
   - D. Neither A nor B

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [FFI Guide](../../reference/FFI_C_CPP.md)
- [Memory Guide](../../reference/MEMORY_GUIDE.md)
- [Performance Guide](../../reference/PERFORMANCE_GUIDE.md)
- [Safety Guide](../../reference/SAFETY_GUIDE.md)

## Next Steps

- [Mobile Development](./mobile_development.md)
- [WebAssembly](./webassembly.md)
- [Embedded Systems](./embedded_systems.md) 
- [Embedded Systems](./embedded_systems.md) 