---
title: "Energy Profiling"
duration: "4h"
level: "advanced"
---

# Energy Profiling

> **Duration**: 4 hours
> **Goal**: Implement energy-aware applications and optimize energy consumption in Ferra

## Overview

This tutorial covers energy profiling, optimization, and monitoring in Ferra applications. You'll learn how to measure, analyze, and optimize energy consumption.

## 1. Energy Measurement (1 hour)

### 1.1. Basic Profiling

```ferra
// Energy profiler actor with security model and energy profiling
actor EnergyProfilerActor {
    data ProfilerState {
        measurements: Map<String, Measurement>,
        thresholds: Map<String, Float>,
        energy_budget: Float,  // Total energy budget for the profiler
        permissions: Set<Capability>,  // Security capabilities
        energy_metrics: EnergyMetrics,  // Track energy usage
        security_context: SecurityContext  // Security context for operations
    }

    fn init() -> ProfilerState {
        return ProfilerState {
            measurements: Map::new(),
            thresholds: Map::new(),
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
                scope: "energy_profiling",
                audit_log: []
            }
        }
    }

    async fn handle_measure(self_state: ProfilerState, operation: Operation) -> (ProfilerState, Result<Measurement, ProfilerError>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::Profiling]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                operation.principal,
                "measure",
                operation.name,
                false,
                "Missing required capabilities"
            );
            return (self_state, Err(ProfilerError::PermissionDenied));
        }

        // Check energy budget
        let measure_energy_cost = calculate_energy_cost(10.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < measure_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                operation.principal,
                "measure",
                operation.name,
                false,
                "Insufficient energy budget"
            );
            return (self_state, Err(ProfilerError::InsufficientEnergy));
        }

        // Execute operation and measure energy
        let result = await operation.execute();
        let end_ops = measure_ops();
        
        // Calculate energy usage based on µops and TDP factors
        let alu_cost = (end_ops.alu_ops - start_ops.alu_ops) * TDP_FACTOR_ALU;
        let mem_cost = (end_ops.mem_ops - start_ops.mem_ops) * TDP_FACTOR_MEM;
        let fp_cost = (end_ops.fp_ops - start_ops.fp_ops) * TDP_FACTOR_FP;
        let total_energy = alu_cost + mem_cost + fp_cost;

        // Create measurement
        let measurement = Measurement {
            operation: operation.name,
            energy_used: total_energy,
            timestamp: now(),
            deterministic: true,
            alu_ops: end_ops.alu_ops - start_ops.alu_ops,
            mem_ops: end_ops.mem_ops - start_ops.mem_ops,
            fp_ops: end_ops.fp_ops - start_ops.fp_ops,
            tdp_factors: TDPFactors {
                alu: TDP_FACTOR_ALU,
                mem: TDP_FACTOR_MEM,
                fp: TDP_FACTOR_FP
            }
        };

        // Update state
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            operation.principal,
            "measure",
            operation.name,
            true,
            null
        );

        let new_state = ProfilerState {
            measurements: self_state.measurements.insert(operation.name, measurement),
            thresholds: self_state.thresholds,
            energy_budget: self_state.energy_budget - measure_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        };

        return (new_state, Ok(measurement));
    }
}

// Energy measurement data structures
data EnergyMetrics {
    total_ops: Int,  // Total micro-operations
    alu_ops: Int,    // ALU operations
    mem_ops: Int,    // Memory operations
    fp_ops: Int,     // Floating point operations
    last_measurement: Time
}

data Measurement {
    operation: String
    energy_used: Float
    timestamp: Time
    deterministic: Bool
    alu_ops: Int
    mem_ops: Int
    fp_ops: Int
    tdp_factors: TDPFactors
}

data TDPFactors {
    alu: Float  // TDP factor for ALU operations
    mem: Float  // TDP factor for memory operations
    fp: Float   // TDP factor for floating point operations
}

// Constants for TDP factors (example values - would be calibrated)
const TDP_FACTOR_ALU: Float = 1.0;  // Base TDP factor for ALU
const TDP_FACTOR_MEM: Float = 5.0;  // Higher TDP factor for memory ops
const TDP_FACTOR_FP: Float = 3.0;   // Higher TDP factor for FP ops

// LLVM pass implementation for energy profiling
#[llvm_pass]
fn measure_ops() -> EnergyMetrics {
    // This would be implemented by the LLVM pass to count micro-operations
    // The pass would analyze the IR and count operations by type
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
    // Apply TDP factors based on operation types
    let alu_cost = metrics.alu_ops * TDP_FACTOR_ALU;
    let mem_cost = metrics.mem_ops * TDP_FACTOR_MEM;
    let fp_cost = metrics.fp_ops * TDP_FACTOR_FP;
    
    return base_cost + (alu_cost + mem_cost + fp_cost).joules;
}

// Error types
data ProfilerError {
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
    MeasurementFailed
}

// Capability types
data Capability {
    Profiling
    BudgetManagement
    Optimization
    Monitoring
    Analysis
    EnergyManagement
}

// Operation definition
type Operation = {
    name: String
    execute: Function
    principal: String
    capabilities: Set<Capability>
    scope: String
}

### 1.2. Energy Budgets

```ferra
// Budget manager actor with security model and energy profiling
actor BudgetManagerActor {
    data BudgetState {
        budgets: Map<String, Budget>,
        usage: Map<String, Float>,
        energy_budget: Float,
        permissions: Set<Capability>
    }
    fn init() -> BudgetState {
        return BudgetState {
            budgets: Map::new(),
            usage: Map::new(),
            energy_budget: 1000.0.joules,
            permissions: Set::new()
        }
    }
    async fn handle_check_budget(self_state: BudgetState, request: BudgetRequest) -> (BudgetState, Result<Unit, BudgetError>) {
        if !self_state.permissions.contains(Capability::BudgetManagement) {
            return (self_state, Err(BudgetError::PermissionDenied))
        }
        let budget_energy_cost = 5.0.joules
        if self_state.energy_budget < budget_energy_cost {
            return (self_state, Err(BudgetError::InsufficientEnergy))
        }
        let budget = match self_state.budgets.get(request.operation) {
            Some(b) => b,
            None => return (self_state, Err(BudgetError::BudgetNotFound))
        }
        let usage = self_state.usage.get(request.operation) || 0.0
        if usage + request.energy > budget.limit {
            return (self_state, Err(BudgetError::BudgetExceeded))
        }
        let new_state = BudgetState {
            budgets: self_state.budgets,
            usage: self_state.usage.insert(request.operation, usage + request.energy),
            energy_budget: self_state.energy_budget - budget_energy_cost,
            permissions: self_state.permissions
        }
        return (new_state, Ok(Unit))
    }
}
data BudgetRequest {
    operation: String
    energy: Float
}
data BudgetError {
    BudgetNotFound
    BudgetExceeded
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
}

// Budget definition
type Budget = {
    operation: String
    limit: Float
    period: Duration
    deterministic: Bool
}
```

### 1.3. Static Energy Estimation and CI Integration

```ferra
// Static energy estimation function (matches ENERGY_PROFILER.md)
fn estimate_static_energy(ir: IRModule, tdp_factors: TDPFactors) -> Float {
    let mut total_alu = 0;
    let mut total_mem = 0;
    let mut total_fp = 0;
    for instr in ir.instructions {
        match instr.kind {
            .ALU => total_alu += instr.count,
            .MEM => total_mem += instr.count,
            .FP => total_fp += instr.count,
            _ => {}
        }
    }
    return (total_alu * tdp_factors.alu) + (total_mem * tdp_factors.mem) + (total_fp * tdp_factors.fp);
}

// Example CI check for energy budget
fn ci_energy_check(module: IRModule, tdp_factors: TDPFactors, budget: Float) -> Bool {
    let energy = estimate_static_energy(module, tdp_factors);
    if energy > budget {
        println("CI FAIL: Energy budget exceeded: {} J", energy);
        return false;
    }
    println("CI PASS: Energy usage: {} J", energy);
    return true;
}
```

## 2. Energy Optimization (1 hour)

### 2.1. Code Optimization

```ferra
// Optimizer actor with security model and energy profiling
actor OptimizerActor {
    data OptimizerState {
        optimizations: Map<String, Optimization>,
        results: Map<String, Result>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> OptimizerState {
        return OptimizerState {
            optimizations: Map::new(),
            results: Map::new(),
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
                scope: "optimization",
                audit_log: []
            }
        }
    }

    async fn handle_optimize(self_state: OptimizerState, code: Code) -> (OptimizerState, Result<Code, OptimizerError>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::Optimization]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                code.principal,
                "optimize",
                code.id,
                false,
                "Missing required capabilities"
            );
            return (self_state, Err(OptimizerError::PermissionDenied));
        }

        // Check energy budget
        let optimize_energy_cost = calculate_energy_cost(15.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < optimize_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                code.principal,
                "optimize",
                code.id,
                false,
                "Insufficient energy budget"
            );
            return (self_state, Err(OptimizerError::InsufficientEnergy));
        }

        // Apply optimizations
        let optimizations = [
            remove_unused_code,
            inline_small_functions,
            optimize_loops,
            reduce_memory_usage
        ];

        let optimized = code;
        let optimization_metrics = EnergyMetrics {
            total_ops: 0,
            alu_ops: 0,
            mem_ops: 0,
            fp_ops: 0,
            last_measurement: now()
        };

        for opt in optimizations {
            let opt_start = measure_ops();
            optimized = await opt(optimized);
            let opt_end = measure_ops();
            optimization_metrics = update_energy_metrics(optimization_metrics, opt_start, opt_end);
        }

        // Calculate energy savings
        let original_energy = calculate_energy_cost(0.0.joules, code.energy_metrics);
        let optimized_energy = calculate_energy_cost(0.0.joules, optimization_metrics);
        let energy_saved = original_energy - optimized_energy;

        // Create result
        let result = Result {
            energy_saved: energy_saved,
            deterministic: true,
            alu_ops_saved: code.energy_metrics.alu_ops - optimization_metrics.alu_ops,
            mem_ops_saved: code.energy_metrics.mem_ops - optimization_metrics.mem_ops,
            fp_ops_saved: code.energy_metrics.fp_ops - optimization_metrics.fp_ops,
            tdp_factors: TDPFactors {
                alu: TDP_FACTOR_ALU,
                mem: TDP_FACTOR_MEM,
                fp: TDP_FACTOR_FP
            }
        };

        // Update state
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            code.principal,
            "optimize",
            code.id,
            true,
            null
        );

        let new_state = OptimizerState {
            optimizations: self_state.optimizations,
            results: self_state.results.insert(code.id, result),
            energy_budget: self_state.energy_budget - optimize_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        };

        return (new_state, Ok(optimized));
    }
}

// Message definitions
data OptimizerError {
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
    OptimizationFailed
}

// Optimization result
type Result = {
    energy_saved: Float
    deterministic: Bool
    alu_ops_saved: Int
    mem_ops_saved: Int
    fp_ops_saved: Int
    tdp_factors: TDPFactors
}

// Code definition
type Code = {
    id: String
    content: String
    principal: String
    capabilities: Set<Capability>
    scope: String
    energy_metrics: EnergyMetrics
}
```

### 2.2. Resource Management

```ferra
// Resource manager actor with security model and energy profiling
actor ResourceManagerActor {
    data ResourceState {
        resources: Map<String, Resource>,
        usage: Map<String, Float>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ResourceState {
        return ResourceState {
            resources: Map::new(),
            usage: Map::new(),
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
                scope: "resource_management",
                audit_log: []
            }
        }
    }

    async fn handle_allocate(self_state: ResourceState, request: AllocationRequest) -> (ResourceState, Result<Resource, ResourceError>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::EnergyManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "allocate",
                request.type,
                false,
                "Missing required capabilities"
            );
            return (self_state, Err(ResourceError::PermissionDenied));
        }

        // Check energy budget
        let allocation_energy_cost = calculate_energy_cost(10.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < allocation_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "allocate",
                request.type,
                false,
                "Insufficient energy budget"
            );
            return (self_state, Err(ResourceError::InsufficientEnergy));
        }

        // Get resource
        let resource = match self_state.resources.get(request.type) {
            Some(r) => r,
            None => {
                let end_ops = measure_ops();
                let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                let new_context = audit_operation(
                    self_state.security_context,
                    request.principal,
                    "allocate",
                    request.type,
                    false,
                    "Resource not found"
                );
                return (self_state, Err(ResourceError::ResourceNotFound));
            }
        };

        // Calculate resource energy cost
        let resource_energy_cost = calculate_energy_cost(request.energy_budget, self_state.energy_metrics);
        
        // Check if resource has enough energy budget
        if resource.energy_budget < resource_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "allocate",
                request.type,
                false,
                "Resource has insufficient energy budget"
            );
            return (self_state, Err(ResourceError::InsufficientResourceEnergy));
        }

        // Update resource
        let updated_resource = Resource {
            id: resource.id,
            type: resource.type,
            energy_budget: resource.energy_budget - resource_energy_cost,
            tdp_factors: resource.tdp_factors
        };

        // Update state
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "allocate",
            request.type,
            true,
            null
        );

        let new_state = ResourceState {
            resources: self_state.resources.insert(request.type, updated_resource),
            usage: self_state.usage.insert(request.type, resource_energy_cost),
            energy_budget: self_state.energy_budget - allocation_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        };

        return (new_state, Ok(updated_resource));
    }
}

// Message definitions
data AllocationRequest {
    type: String
    energy_budget: Float
    principal: String
    capabilities: Set<Capability>
    scope: String
}

data ResourceError {
    ResourceNotFound
    PermissionDenied
    InsufficientEnergy
    InsufficientResourceEnergy
    InvalidRequest
}

// Resource definition
type Resource = {
    id: String
    type: String
    energy_budget: Float
    tdp_factors: TDPFactors
}
```

## 3. Energy Monitoring (1 hour)

### 3.1. Real-time Monitoring

```ferra
// Monitor actor with security model and energy profiling
actor MonitorActor {
    data MonitorState {
        metrics: Map<String, Metric>,
        alerts: List<Alert>,
        energy_budget: Float,
        permissions: Set<Capability>
    }
    fn init() -> MonitorState {
        return MonitorState {
            metrics: Map::new(),
            alerts: [],
            energy_budget: 1000.0.joules,
            permissions: Set::new()
        }
    }
    async fn handle_metric(self_state: MonitorState, metric: Metric) -> (MonitorState, Result<Unit, MonitorError>) {
        if !self_state.permissions.contains(Capability::Monitoring) {
            return (self_state, Err(MonitorError::PermissionDenied))
        }
        let monitor_energy_cost = 5.0.joules
        if self_state.energy_budget < monitor_energy_cost {
            return (self_state, Err(MonitorError::InsufficientEnergy))
        }
        let threshold = get_threshold(metric.name)
        if metric.value > threshold {
            let alert = Alert {
                metric: metric.name,
                value: metric.value,
                threshold: threshold,
                timestamp: now(),
                energy_budget: 0.01.joules
            }
            let new_state = MonitorState {
                metrics: self_state.metrics.insert(metric.name, metric),
                alerts: self_state.alerts.append(alert),
                energy_budget: self_state.energy_budget - monitor_energy_cost,
                permissions: self_state.permissions
            }
            await send_alert(alert)
            return (new_state, Ok(Unit))
        }
        let new_state = MonitorState {
            metrics: self_state.metrics.insert(metric.name, metric),
            alerts: self_state.alerts,
            energy_budget: self_state.energy_budget - monitor_energy_cost,
            permissions: self_state.permissions
        }
        return (new_state, Ok(Unit))
    }
}
data MonitorError {
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
}

// Metric definition
type Metric = {
    name: String
    value: Float
    timestamp: DateTime
    energy_budget: Float
}
```

### 3.2. Historical Analysis

```ferra
// Analyzer actor with security model and energy profiling
actor AnalyzerActor {
    data AnalyzerState {
        history: List<Measurement>,
        patterns: Map<String, Pattern>,
        energy_budget: Float,
        permissions: Set<Capability>
    }
    fn init() -> AnalyzerState {
        return AnalyzerState {
            history: [],
            patterns: Map::new(),
            energy_budget: 1000.0.joules,
            permissions: Set::new()
        }
    }
    async fn handle_analyze(self_state: AnalyzerState, period: Duration) -> (AnalyzerState, Result<Analysis, AnalyzerError>) {
        if !self_state.permissions.contains(Capability::Analysis) {
            return (self_state, Err(AnalyzerError::PermissionDenied))
        }
        let analyze_energy_cost = 10.0.joules
        if self_state.energy_budget < analyze_energy_cost {
            return (self_state, Err(AnalyzerError::InsufficientEnergy))
        }
        let measurements = self_state.history.filter { m => m.timestamp > now() - period }
        let patterns = detect_patterns(measurements)
        let new_state = AnalyzerState {
            history: self_state.history,
            patterns: self_state.patterns.merge(patterns),
            energy_budget: self_state.energy_budget - analyze_energy_cost,
            permissions: self_state.permissions
        }
        return (new_state, Ok(Analysis {
            patterns: patterns,
            period: period
        }))
    }
}
data AnalyzerError {
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
}

// Analysis result
type Analysis = {
    patterns: Map<String, Pattern>
    recommendations: List<Recommendation>
    energy_budget: Float
}
```

## 4. Energy-aware Development (1 hour)

### 4.1. Development Practices

```ferra
// Development guidelines
actor DevGuidelinesActor {
    data GuidelinesState {
        rules: Map<String, Rule>,
        violations: List<Violation>
    }

    async fn handle_check(self_state: GuidelinesState, code: Code) -> (GuidelinesState, List<Violation>) {
        let violations = []
        
        for rule in self_state.rules {
            if !rule.check(code) {
                violations.append(Violation {
                    rule: rule.name,
                    location: rule.location,
                    energy_impact: rule.energy_impact,
                    deterministic: true
                })
            }
        }
        
        let new_state = GuidelinesState {
            rules: self_state.rules,
            violations: self_state.violations.append(violations)
        }
        
        return (new_state, violations)
    }
}

// Violation definition
type Violation = {
    rule: String
    location: Location
    energy_impact: Float
    deterministic: Bool
}
```

### 4.2. Testing & Validation

```ferra
// Energy test actor
actor EnergyTestActor {
    data TestState {
        tests: Map<String, Test>,
        results: Map<String, Result>
    }

    async fn handle_test(self_state: TestState, test: Test) -> (TestState, Result) {
        let profiler = spawn EnergyProfilerActor()
        let (_, measurement) = await profiler.ask(Measure(test.operation))
        
        let result = Result {
            passed: measurement.energy_used <= test.budget,
            energy_used: measurement.energy_used,
            budget: test.budget,
            deterministic: true
        }
        
        let new_state = TestState {
            tests: self_state.tests,
            results: self_state.results.insert(test.id, result)
        }
        
        return (new_state, result)
    }
}

// Test definition
type Test = {
    id: String
    operation: Operation
    budget: Float
    deterministic: Bool
}
```

## Quiz

1. What is the main benefit of energy profiling?
   - A. Better performance
   - B. Lower energy consumption
   - C. Simpler implementation
   - D. Faster response times

2. How do you handle energy budgets in Ferra?
   - A. Using thresholds
   - B. Using limits
   - C. Both A and B
   - D. Neither A nor B

3. Which tool is used for energy optimization?
   - A. Profiler
   - B. Optimizer
   - C. Both A and B
   - D. Neither A nor B

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Energy Profiler](../../reference/ENERGY_PROFILER.md)
- [Optimization Guide](../../reference/OPTIMIZATION_GUIDE.md)
- [Testing Guide](../../reference/TESTING_GUIDE.md)
- [Development Guidelines](../../reference/DEV_GUIDELINES.md)

## Next Steps

- [Data-Parallel Processing](./data_parallel.md)
- [GPU Acceleration](./gpu_acceleration.md)
- [UI Development](./ui_development.md) 
- [UI Development](./ui_development.md) 
- [UI Development](./ui_development.md) 