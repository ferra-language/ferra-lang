---
title: "Serverless Backend Development"
duration: "4h"
level: "advanced"
---

# Serverless Backend Development

> **Duration**: 4 hours
> **Goal**: Build and deploy a serverless backend using Ferra's OCI container support

## Overview

This tutorial covers building a serverless backend using Ferra's OCI container support. You'll learn how to create, deploy, and optimize serverless functions for production use.

## 1. OCI Container Basics (1 hour)

### 1.1. Container Definition

```ferra
// Basic serverless function
fn handler(req: Request) -> Response {
    Response {
        status: 200,
        body: "Hello, World!"
    }
}

// Container configuration
container {
    name: "hello-world",
    handler: handler,
    memory: 128,
    timeout: 30,
    environment: {
        "ENV": "production"
    }
}
```

### 1.2. Resource Management

```ferra
// Resource-constrained function
fn process_data(req: Request) -> Response {
    // Set memory limit
    memory_limit(256)
    
    // Set CPU limit
    cpu_limit(0.5)
    
    // Process data
    let result = heavy_computation()
    
    Response {
        status: 200,
        body: result
    }
}
```

## 2. Function Design (1 hour)

### 2.1. Stateless Functions

```ferra
// Stateless function with external storage
fn process_order(req: Request) -> Response {
    let order = req.body.parse<Order>()
    
    // Store in database
    db.orders.insert(order)
    
    // Publish event
    event_bus.publish("order.created", order)
    
    Response {
        status: 201,
        body: order
    }
}
```

### 2.2. Cold Start Optimization

```ferra
// Optimized cold start
fn optimized_handler(req: Request) -> Response {
    // Lazy initialization
    static DB = Lazy(|| Database.connect())
    
    // Warm-up connection
    if DB.value.is_none() {
        DB.value = Some(Database.connect())
    }
    
    Response {
        status: 200,
        body: "Ready"
    }
}
```

## 3. Cost Optimization (1 hour)

### 3.1. Resource Scaling

```ferra
// Auto-scaling configuration
container {
    name: "api",
    handler: api_handler,
    scaling: {
        min_instances: 1,
        max_instances: 10,
        target_cpu: 70,
        cooldown: 300
    }
}
```

### 3.2. Caching Strategy

```ferra
// Cached function
fn cached_handler(req: Request) -> Response {
    let cache_key = req.path
    
    // Check cache
    if let Some(cached) = cache.get(cache_key) {
        return Response {
            status: 200,
            body: cached
        }
    }
    
    // Compute and cache
    let result = compute_result()
    cache.set(cache_key, result, 3600)
    
    Response {
        status: 200,
        body: result
    }
}
```

## 4. Deployment (1 hour)

### 4.1. CI/CD Pipeline

```ferra
// Deployment configuration
deploy {
    name: "api",
    container: "api",
    triggers: {
        on_push: "main",
        schedule: "0 0 * * *"
    },
    environment: {
        "ENV": "production",
        "DB_URL": "secret://db-url"
    }
}
```

### 4.2. Monitoring

```ferra
// Instrumented function
fn monitored_handler(req: Request) -> Response {
    // Start timer
    let timer = Timer.start()
    
    // Process request
    let result = process_request(req)
    
    // Record metrics
    metrics.record {
        duration: timer.elapsed(),
        memory: memory_used(),
        status: result.status
    }
    
    result
}
```

## Quiz

1. What is the main benefit of using OCI containers for serverless functions?
   - A. Better performance
   - B. Portability
   - C. Lower cost
   - D. Easier debugging

2. How do you optimize cold starts in serverless functions?
   - A. Using static variables
   - B. Increasing memory
   - C. Using warm-up functions
   - D. All of the above

3. Which strategy is best for cost optimization?
   - A. Always using maximum resources
   - B. Implementing auto-scaling
   - C. Using fixed resources
   - D. Manual scaling

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Backend Targets](../../reference/BACKEND_EXPANDED_TARGETS.md)
- [Security Model](../../reference/SECURITY_MODEL.md)
- [Energy Profiler](../../reference/ENERGY_PROFILER.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [Database & Storage](./database_storage.md)
- [Authentication & Security](./auth_security.md)
- [API Design & Integration](./api_design.md)
- [Monitoring & Operations](./monitoring_ops.md)

#[ai::tag(serverless_component)]
actor FunctionManagerActor {
    data ManagerState {
        functions: Map<String, Function>,
        executions: Map<String, Execution>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ManagerState {
        return ManagerState {
            functions: Map::new(),
            executions: Map::new(),
            energy_budget: 4000.0.joules,
            permissions: Set::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext::new()
        }
    }

    async fn handle_function(self_state: ManagerState, request: FunctionRequest) -> (ManagerState, Result<FunctionResult, FunctionError>) {
        let start_ops = measure_ops();
        
        // Check permissions
        if !has_required_capabilities(self_state.security_context, [Capability::FunctionManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "function",
                request.id,
                false,
                "Missing required capabilities"
            );
            return (ManagerState {
                functions: self_state.functions,
                executions: self_state.executions,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(FunctionError::PermissionDenied))
        }

        // Check energy budget
        let function_energy_cost = calculate_energy_cost(40.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < function_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "function",
                request.id,
                false,
                "Insufficient energy budget"
            );
            return (ManagerState {
                functions: self_state.functions,
                executions: self_state.executions,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(FunctionError::InsufficientEnergy))
        }

        // Handle function
        let function = match self_state.functions.get(request.id) {
            Some(f) => f,
            None => return (self_state, Err(FunctionError::FunctionNotFound))
        }

        let result = await function.execute(request)
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "function",
            request.id,
            true,
            null
        );

        return (ManagerState {
            functions: self_state.functions,
            executions: self_state.executions.insert(request.id, result.execution),
            energy_budget: self_state.energy_budget - function_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Ok(result))
    }
}

#[ai::tag(container_component)]
actor ContainerManagerActor {
    data ContainerState {
        containers: Map<String, Container>,
        resources: Map<String, Resource>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ContainerState {
        return ContainerState {
            containers: Map::new(),
            resources: Map::new(),
            energy_budget: 5000.0.joules,
            permissions: Set::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext::new()
        }
    }

    async fn handle_container(self_state: ContainerState, request: ContainerRequest) -> (ContainerState, Result<ContainerResult, ContainerError>) {
        let start_ops = measure_ops();
        
        // Check permissions
        if !has_required_capabilities(self_state.security_context, [Capability::ContainerManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "container",
                request.id,
                false,
                "Missing required capabilities"
            );
            return (ContainerState {
                containers: self_state.containers,
                resources: self_state.resources,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(ContainerError::PermissionDenied))
        }

        // Check energy budget
        let container_energy_cost = calculate_energy_cost(50.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < container_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "container",
                request.id,
                false,
                "Insufficient energy budget"
            );
            return (ContainerState {
                containers: self_state.containers,
                resources: self_state.resources,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(ContainerError::InsufficientEnergy))
        }

        // Handle container
        let container = match self_state.containers.get(request.id) {
            Some(c) => c,
            None => return (self_state, Err(ContainerError::ContainerNotFound))
        }

        let result = await container.handle(request)
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "container",
            request.id,
            true,
            null
        );

        return (ContainerState {
            containers: self_state.containers.insert(request.id, result.container),
            resources: self_state.resources.insert(request.id, result.resources),
            energy_budget: self_state.energy_budget - container_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Ok(result))
    }
}

// Message definitions
data FunctionRequest {
    id: String
    principal: String
    capabilities: Set<Capability>
    scope: String
}

data FunctionResult {
    id: String
    execution: Execution
    duration: Duration
    energy_used: Float
}

data FunctionError {
    FunctionNotFound
    PermissionDenied
    InsufficientEnergy
    ExecutionError
    InvalidRequest
}

data ContainerRequest {
    id: String
    principal: String
    capabilities: Set<Capability>
    scope: String
}

data ContainerResult {
    id: String
    container: Container
    resources: Resource
    duration: Duration
    energy_used: Float
}

data ContainerError {
    ContainerNotFound
    PermissionDenied
    InsufficientEnergy
    ContainerError
    InvalidRequest
}

// Security model
data Capability {
    FunctionManagement
    FunctionExecution
    ContainerManagement
    ContainerExecution
    EnergyManagement
} 