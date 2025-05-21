---
title: "Embedded Systems"
duration: "4h"
level: "advanced"
---

# Embedded Systems

> **Duration**: 4 hours
> **Goal**: Build efficient embedded systems using Ferra's embedded development framework

## Overview

This tutorial covers embedded system development, hardware interaction, and resource optimization in Ferra applications.

## 1. Embedded Basics (1 hour)

### 1.1. Hardware Abstraction

```ferra
// Hardware manager actor with security model and energy profiling
#[ai::tag(embedded_component)]
actor HardwareManagerActor {
    data ManagerState {
        devices: Map<String, Device>,
        drivers: Map<String, Driver>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>,  // Security capabilities
        energy_metrics: EnergyMetrics,  // Track energy usage
        security_context: SecurityContext  // Security context for operations
    }

    fn init() -> ManagerState {
        return ManagerState {
            devices: Map::new(),
            drivers: Map::new(),
            energy_budget: 1000.0.joules,  // Initial energy budget
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
                scope: "hardware_management",
                audit_log: []
            }
        }
    }

    async fn handle_device(self_state: ManagerState, request: DeviceRequest) -> (ManagerState, Result<Device, DeviceError>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::DeviceManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "device",
                request.id,
                false,
                "Missing required capabilities"
            );
            return (ManagerState {
                devices: self_state.devices,
                drivers: self_state.drivers,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(DeviceError::PermissionDenied))
        }

        // Check energy budget
        let device_energy_cost = 5.0.joules
        if self_state.energy_budget < device_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "device",
                request.id,
                false,
                "Insufficient energy budget"
            );
            return (ManagerState {
                devices: self_state.devices,
                drivers: self_state.drivers,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(DeviceError::InsufficientEnergy))
        }

        let device = match self_state.devices.get(request.id) {
            Some(d) => d,
            None => {
                let end_ops = measure_ops();
                let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                let new_context = audit_operation(
                    self_state.security_context,
                    request.principal,
                    "device",
                    request.id,
                    false,
                    "Device not found"
                );
                return (ManagerState {
                    devices: self_state.devices,
                    drivers: self_state.drivers,
                    energy_budget: self_state.energy_budget,
                    permissions: self_state.permissions,
                    energy_metrics: new_metrics,
                    security_context: new_context
                }, Err(DeviceError::DeviceNotFound))
            }
        }
        
        let driver = match self_state.drivers.get(device.driver) {
            Some(d) => d,
            None => {
                let end_ops = measure_ops();
                let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                let new_context = audit_operation(
                    self_state.security_context,
                    request.principal,
                    "device",
                    request.id,
                    false,
                    "Driver not found"
                );
                return (ManagerState {
                    devices: self_state.devices,
                    drivers: self_state.drivers,
                    energy_budget: self_state.energy_budget,
                    permissions: self_state.permissions,
                    energy_metrics: new_metrics,
                    security_context: new_context
                }, Err(DeviceError::DriverNotFound))
            }
        }
        
        let new_state = ManagerState {
            devices: self_state.devices,
            drivers: self_state.drivers,
            energy_budget: self_state.energy_budget - device_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        }
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "device",
            request.id,
            true,
            null
        );
        
        return (ManagerState {
            devices: new_state.devices,
            drivers: new_state.drivers,
            energy_budget: new_state.energy_budget,
            permissions: new_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Ok(device))
    }
}

// Message definitions
data DeviceRequest {
    id: String
    type: String
    energy_budget: Float
}

data DeviceError {
    DeviceNotFound
    DriverNotFound
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
}

// Security model
data Capability {
    DeviceManagement
    DriverManagement
    EnergyManagement
}

// Device definition
type Device = {
    id: String
    type: String
    driver: String
    energy_budget: Float
}
```

### 1.2. Resource Management

```ferra
// Resource manager actor with security model and energy profiling
#[ai::tag(embedded_component)]
actor ResourceManagerActor {
    data ManagerState {
        resources: Map<String, Resource>,
        allocations: Map<String, Allocation>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>,  // Security capabilities
        energy_metrics: EnergyMetrics,  // Track energy usage
        security_context: SecurityContext  // Security context for operations
    }

    fn init() -> ManagerState {
        return ManagerState {
            resources: Map::new(),
            allocations: Map::new(),
            energy_budget: 2000.0.joules,  // Initial energy budget
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

    async fn handle_allocate(self_state: ManagerState, request: AllocationRequest) -> (ManagerState, Result<Resource, AllocationError>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::ResourceManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "allocate",
                request.id,
                false,
                "Missing required capabilities"
            );
            return (ManagerState {
                resources: self_state.resources,
                allocations: self_state.allocations,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(AllocationError::PermissionDenied))
        }

        // Check energy budget
        let allocation_energy_cost = 10.0.joules
        if self_state.energy_budget < allocation_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "allocate",
                request.id,
                false,
                "Insufficient energy budget"
            );
            return (ManagerState {
                resources: self_state.resources,
                allocations: self_state.allocations,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(AllocationError::InsufficientEnergy))
        }

        let resource = match select_resource(request) {
            Some(r) => r,
            None => {
                let end_ops = measure_ops();
                let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                let new_context = audit_operation(
                    self_state.security_context,
                    request.principal,
                    "allocate",
                    request.id,
                    false,
                    "No suitable resource"
                );
                return (ManagerState {
                    resources: self_state.resources,
                    allocations: self_state.allocations,
                    energy_budget: self_state.energy_budget,
                    permissions: self_state.permissions,
                    energy_metrics: new_metrics,
                    security_context: new_context
                }, Err(AllocationError::NoSuitableResource))
            }
        }
        
        let new_state = ManagerState {
            resources: self_state.resources.insert(resource.id, resource),
            allocations: self_state.allocations.insert(request.id, Allocation {
                id: request.id,
                resource: resource.id,
                energy_budget: request.energy_budget
            }),
            energy_budget: self_state.energy_budget - allocation_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        }
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "allocate",
            request.id,
            true,
            null
        );
        
        return (ManagerState {
            resources: new_state.resources,
            allocations: new_state.allocations,
            energy_budget: new_state.energy_budget,
            permissions: new_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Ok(resource))
    }
}

// Message definitions
data AllocationRequest {
    id: String
    type: String
    requirements: Map<String, Any>
    energy_budget: Float
}

data AllocationError {
    NoSuitableResource
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
}

// Security model
data Capability {
    ResourceManagement
    AllocationManagement
    EnergyManagement
}

// Resource definition
type Resource = {
    id: String
    type: String
    capacity: Int
    energy_budget: Float
}

// Allocation definition
type Allocation = {
    id: String
    resource: String
    energy_budget: Float
}
```

## 2. Hardware Interaction (1 hour)

### 2.1. I/O Management

```ferra
// I/O manager actor with security model and energy profiling
#[ai::tag(embedded_component)]
actor IOManagerActor {
    data ManagerState {
        ports: Map<String, Port>,
        operations: Map<String, Operation>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>,  // Security capabilities
        energy_metrics: EnergyMetrics,  // Track energy usage
        security_context: SecurityContext  // Security context for operations
    }

    fn init() -> ManagerState {
        return ManagerState {
            ports: Map::new(),
            operations: Map::new(),
            energy_budget: 3000.0.joules,  // Initial energy budget
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
                scope: "io_management",
                audit_log: []
            }
        }
    }

    async fn handle_io(self_state: ManagerState, request: IORequest) -> (ManagerState, Result<Unit, IOError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::IOManagement) {
            return (self_state, Err(IOError::PermissionDenied))
        }

        // Check energy budget
        let io_energy_cost = 15.0.joules
        if self_state.energy_budget < io_energy_cost {
            return (self_state, Err(IOError::InsufficientEnergy))
        }

        let port = match self_state.ports.get(request.port) {
            Some(p) => p,
            None => return (self_state, Err(IOError::PortNotFound))
        }
        
        let operation = match self_state.operations.get(request.operation) {
            Some(o) => o,
            None => return (self_state, Err(IOError::OperationNotFound))
        }
        
        match await port.execute(operation, request.data) {
            Ok(_) => {
                let new_state = ManagerState {
                    ports: self_state.ports,
                    operations: self_state.operations,
                    energy_budget: self_state.energy_budget - io_energy_cost,
                    permissions: self_state.permissions,
                    energy_metrics: self_state.energy_metrics,
                    security_context: self_state.security_context
                }
                return (new_state, Ok(Unit))
            }
            Err(e) => return (self_state, Err(IOError::ExecutionFailed))
        }
    }
}

// Message definitions
data IORequest {
    port: String
    operation: String
    data: Any
    energy_budget: Float
}

data IOError {
    PortNotFound
    OperationNotFound
    PermissionDenied
    InsufficientEnergy
    ExecutionFailed
    InvalidRequest
}

// Security model
data Capability {
    IOManagement
    PortManagement
    EnergyManagement
}

// Port definition
type Port = {
    id: String
    type: String
    execute: Function
    energy_budget: Float
}

// Operation definition
type Operation = {
    id: String
    type: String
    execute: Function
    energy_cost: Float
}
```

### 2.2. Sensor Integration

```ferra
// Sensor manager actor with security model and energy profiling
actor SensorManagerActor {
    data ManagerState {
        sensors: Map<String, Sensor>,
        readings: Map<String, List<Reading>>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> ManagerState {
        return ManagerState {
            sensors: Map::new(),
            readings: Map::new(),
            energy_budget: 4000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_reading(self_state: ManagerState, request: ReadingRequest) -> (ManagerState, Result<Reading, SensorError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::SensorManagement) {
            return (self_state, Err(SensorError::PermissionDenied))
        }

        // Check energy budget
        let reading_energy_cost = 20.0.joules
        if self_state.energy_budget < reading_energy_cost {
            return (self_state, Err(SensorError::InsufficientEnergy))
        }

        let sensor = match self_state.sensors.get(request.sensor) {
            Some(s) => s,
            None => return (self_state, Err(SensorError::SensorNotFound))
        }
        
        match await sensor.read() {
            Ok(reading) => {
                let new_state = ManagerState {
                    sensors: self_state.sensors,
                    readings: self_state.readings.insert(
                        request.sensor, 
                        self_state.readings.get(request.sensor).unwrap_or([]).append(reading)
                    ),
                    energy_budget: self_state.energy_budget - reading_energy_cost,
                    permissions: self_state.permissions
                }
                return (new_state, Ok(reading))
            }
            Err(e) => return (self_state, Err(SensorError::ReadingFailed))
        }
    }
}

// Message definitions
data ReadingRequest {
    sensor: String
    type: String
    energy_budget: Float
}

data SensorError {
    SensorNotFound
    PermissionDenied
    InsufficientEnergy
    ReadingFailed
    InvalidRequest
}

// Security model
data Capability {
    SensorManagement
    ReadingManagement
    EnergyManagement
}

// Sensor definition
type Sensor = {
    id: String
    type: String
    read: Function
    energy_budget: Float
}

// Reading definition
type Reading = {
    timestamp: DateTime
    value: Float
    unit: String
    energy_cost: Float
}
```

## 3. Power Management (1 hour)

### 3.1. Energy Optimization

```ferra
// Energy manager actor with security model and energy profiling
actor EnergyManagerActor {
    data ManagerState {
        budgets: Map<String, Budget>,
        optimizations: Map<String, Optimization>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> ManagerState {
        return ManagerState {
            budgets: Map::new(),
            optimizations: Map::new(),
            energy_budget: 5000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_optimize(self_state: ManagerState, request: OptimizationRequest) -> (ManagerState, Result<Unit, OptimizationError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::EnergyManagement) {
            return (self_state, Err(OptimizationError::PermissionDenied))
        }

        // Check energy budget
        let optimization_energy_cost = 25.0.joules
        if self_state.energy_budget < optimization_energy_cost {
            return (self_state, Err(OptimizationError::InsufficientEnergy))
        }

        let budget = match self_state.budgets.get(request.budget) {
            Some(b) => b,
            None => return (self_state, Err(OptimizationError::BudgetNotFound))
        }
        
        let optimizations = [
            reduce_cpu_usage,
            optimize_memory,
            minimize_io,
            power_down_unused
        ]
        
        for opt in optimizations {
            match await opt(budget) {
                Ok(_) => continue,
                Err(e) => return (self_state, Err(OptimizationError::OptimizationFailed))
            }
        }
        
        let new_state = ManagerState {
            budgets: self_state.budgets,
            optimizations: self_state.optimizations,
            energy_budget: self_state.energy_budget - optimization_energy_cost,
            permissions: self_state.permissions
        }
        
        return (new_state, Ok(Unit))
    }
}

// Message definitions
data OptimizationRequest {
    budget: String
    type: String
    energy_budget: Float
}

data OptimizationError {
    BudgetNotFound
    PermissionDenied
    InsufficientEnergy
    OptimizationFailed
    InvalidRequest
}

// Security model
data Capability {
    EnergyManagement
    BudgetManagement
    OptimizationManagement
}

// Budget definition
type Budget = {
    id: String
    type: String
    limit: Float
    energy_budget: Float
}

// Optimization definition
type Optimization = {
    id: String
    type: String
    apply: Function
    energy_cost: Float
}
```

### 3.2. Power States

```ferra
// Power manager actor with security model and energy profiling
actor PowerManagerActor {
    data ManagerState {
        states: Map<String, PowerState>,
        transitions: Map<String, Transition>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> ManagerState {
        return ManagerState {
            states: Map::new(),
            transitions: Map::new(),
            energy_budget: 6000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_transition(self_state: ManagerState, request: TransitionRequest) -> (ManagerState, Result<Unit, TransitionError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::PowerManagement) {
            return (self_state, Err(TransitionError::PermissionDenied))
        }

        // Check energy budget
        let transition_energy_cost = 30.0.joules
        if self_state.energy_budget < transition_energy_cost {
            return (self_state, Err(TransitionError::InsufficientEnergy))
        }

        let state = match self_state.states.get(request.state) {
            Some(s) => s,
            None => return (self_state, Err(TransitionError::StateNotFound))
        }
        
        let transition = match self_state.transitions.get(request.transition) {
            Some(t) => t,
            None => return (self_state, Err(TransitionError::TransitionNotFound))
        }
        
        match await transition.execute(state) {
            Ok(_) => {
                let new_state = ManagerState {
                    states: self_state.states,
                    transitions: self_state.transitions,
                    energy_budget: self_state.energy_budget - transition_energy_cost,
                    permissions: self_state.permissions
                }
                return (new_state, Ok(Unit))
            },
            Err(e) => return (self_state, Err(TransitionError::ExecutionFailed))
        }
    }
}

// Message definitions
data TransitionRequest {
    state: String
    transition: String
    energy_budget: Float
}

data TransitionError {
    StateNotFound
    TransitionNotFound
    PermissionDenied
    InsufficientEnergy
    ExecutionFailed
    InvalidRequest
}

// Security model
data Capability {
    PowerManagement
    StateManagement
    EnergyManagement
}

// Power state definition
type PowerState = {
    id: String
    type: String
    power: Float
    energy_budget: Float
}

// Transition definition
type Transition = {
    id: String
    from: String
    to: String
    execute: Function
    energy_cost: Float
}
```

## 4. Real-time Systems (1 hour)

### 4.1. Task Scheduling

```ferra
// Scheduler actor with security model and energy profiling
actor SchedulerActor {
    data SchedulerState {
        tasks: Map<String, Task>,
        schedule: Schedule,
        energy_budget: Float,  // Total energy budget for the scheduler
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> SchedulerState {
        return SchedulerState {
            tasks: Map::new(),
            schedule: Schedule::new(),
            energy_budget: 7000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_schedule(self_state: SchedulerState, request: ScheduleRequest) -> (SchedulerState, Result<Unit, ScheduleError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::TaskScheduling) {
            return (self_state, Err(ScheduleError::PermissionDenied))
        }

        // Check energy budget
        let scheduling_energy_cost = 35.0.joules
        if self_state.energy_budget < scheduling_energy_cost {
            return (self_state, Err(ScheduleError::InsufficientEnergy))
        }

        let task = match self_state.tasks.get(request.task) {
            Some(t) => t,
            None => return (self_state, Err(ScheduleError::TaskNotFound))
        }
        
        let new_schedule = self_state.schedule.add(task, request.priority)
        
        match await execute_schedule(new_schedule) {
            Ok(_) => {
                let new_state = SchedulerState {
                    tasks: self_state.tasks,
                    schedule: new_schedule,
                    energy_budget: self_state.energy_budget - scheduling_energy_cost,
                    permissions: self_state.permissions
                }
                return (new_state, Ok(Unit))
            },
            Err(e) => return (self_state, Err(ScheduleError::ExecutionFailed))
        }
    }
}

// Message definitions
data ScheduleRequest {
    task: String
    priority: Int
    energy_budget: Float
}

data ScheduleError {
    TaskNotFound
    PermissionDenied
    InsufficientEnergy
    ExecutionFailed
    InvalidRequest
}

// Security model
data Capability {
    TaskScheduling
    ScheduleManagement
    EnergyManagement
}

// Task definition
type Task = {
    id: String
    type: String
    priority: Int
    energy_budget: Float
}

// Schedule definition
type Schedule = {
    tasks: List<Task>
    priorities: Map<String, Int>
    energy_budget: Float
}
```

### 4.2. Interrupt Handling

```ferra
// Interrupt manager actor with security model and energy profiling
actor InterruptManagerActor {
    data ManagerState {
        handlers: Map<String, Handler>,
        priorities: Map<String, Int>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> ManagerState {
        return ManagerState {
            handlers: Map::new(),
            priorities: Map::new(),
            energy_budget: 8000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_interrupt(self_state: ManagerState, request: InterruptRequest) -> (ManagerState, Result<Unit, InterruptError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::InterruptManagement) {
            return (self_state, Err(InterruptError::PermissionDenied))
        }

        // Check energy budget
        let interrupt_energy_cost = 40.0.joules
        if self_state.energy_budget < interrupt_energy_cost {
            return (self_state, Err(InterruptError::InsufficientEnergy))
        }

        let handler = match self_state.handlers.get(request.type) {
            Some(h) => h,
            None => return (self_state, Err(InterruptError::HandlerNotFound))
        }
        
        let priority = match self_state.priorities.get(request.type) {
            Some(p) => p,
            None => return (self_state, Err(InterruptError::PriorityNotFound))
        }
        
        match await handler.handle(request.interrupt, priority) {
            Ok(_) => {
                let new_state = ManagerState {
                    handlers: self_state.handlers,
                    priorities: self_state.priorities,
                    energy_budget: self_state.energy_budget - interrupt_energy_cost,
                    permissions: self_state.permissions
                }
                return (new_state, Ok(Unit))
            },
            Err(e) => return (self_state, Err(InterruptError::HandlingFailed))
        }
    }
}

// Message definitions
data InterruptRequest {
    type: String
    interrupt: Interrupt
    energy_budget: Float
}

data InterruptError {
    HandlerNotFound
    PriorityNotFound
    PermissionDenied
    InsufficientEnergy
    HandlingFailed
    InvalidRequest
}

// Security model
data Capability {
    InterruptManagement
    HandlerManagement
    EnergyManagement
}

// Handler definition
type Handler = {
    id: String
    type: String
    handle: Function
    energy_budget: Float
}

// Interrupt definition
type Interrupt = {
    id: String
    type: String
    data: Any
    energy_cost: Float
}
```

## Quiz

1. What is the main benefit of embedded systems?
   - A. Better performance
   - B. Resource efficiency
   - C. Simpler implementation
   - D. Faster response times

2. How do you handle power management in embedded systems?
   - A. Using energy optimization
   - B. Using power states
   - C. Both A and B
   - D. Neither A nor B

3. Which system is used for real-time operations?
   - A. Task scheduling
   - B. Interrupt handling
   - C. Both A and B
   - D. Neither A nor B

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Embedded Guide](../../reference/EMBEDDED_GUIDE.md)
- [Hardware Guide](../../reference/HARDWARE_GUIDE.md)
- [Power Guide](../../reference/POWER_GUIDE.md)
- [Real-time Guide](../../reference/REALTIME_GUIDE.md)

## Next Steps

- [Cloud Integration](./cloud_integration.md)
- [Distributed Systems](./distributed_systems.md)
- [IoT Development](./iot_development.md)
- [IoT Development](./iot_development.md)
- [IoT Development](./iot_development.md)