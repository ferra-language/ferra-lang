---
title: "Concurrency & Actors"
duration: "4h"
level: "advanced"
---

# Concurrency & Actors

> **Duration**: 4 hours
> **Goal**: Implement concurrent and distributed systems using Ferra's actor model

## Overview

This tutorial covers the actor model, message passing, deterministic scheduling, and distributed systems in Ferra applications.

## 1. Actor Model (1 hour)

### 1.1. Basic Actors

```ferra
// Basic actor
#[ai::tag(concurrency_component)]
actor CounterActor {
    data CounterState {
        count: Int,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> CounterState {
        return CounterState {
            count: 0,
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
                scope: "counter",
                audit_log: []
            }
        }
    }

    async fn handle_increment(self_state: CounterState) -> (CounterState, Int) {
        // Capability and energy checks
        if !self_state.permissions.contains(Capability::Increment) {
            return (self_state, -1)
        }
        let increment_energy_cost = 1.0.joules;
        if self_state.energy_budget < increment_energy_cost {
            return (self_state, -1)
        }
        let new_state = CounterState {
            count: self_state.count + 1,
            energy_budget: self_state.energy_budget - increment_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        return (new_state, new_state.count)
    }

    async fn handle_get(self_state: CounterState) -> (CounterState, Int) {
        return (self_state, self_state.count)
    }
}

// Actor usage
async fn main() {
    let counter = spawn CounterActor()
    let (_, count) = await counter.ask(Increment())
    println("Count:", count)
}
```

### 1.2. Message Passing

```ferra
// Message types
type Message = {
    Increment
    Decrement
    Get
    Set(Int)
}

// Message handler actor
#[ai::tag(concurrency_component)]
actor MessageHandlerActor {
    data HandlerState {
        messages: List<Message>,
        handlers: Map<String, Handler>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> HandlerState {
        return HandlerState {
            messages: List::new(),
            handlers: Map::new(),
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
                scope: "message_handler",
                audit_log: []
            }
        }
    }

    async fn handle_message(self_state: HandlerState, message: Message) -> (HandlerState, Result<Any>) {
        // Capability and energy checks
        if !self_state.permissions.contains(Capability::MessageHandling) {
            return (self_state, Error("Permission denied"))
        }
        let message_energy_cost = 1.0.joules;
        if self_state.energy_budget < message_energy_cost {
            return (self_state, Error("Insufficient energy"))
        }
        let handler = self_state.handlers.get(message.type)
        if !handler {
            return (self_state, Error("No handler for message type"))
        }
        let (new_state, result) = await handler(self_state, message)
        return (new_state, result)
    }
}
```

## 2. Deterministic Scheduling (1 hour)

### 2.1. Actor Scheduling

```ferra
// Scheduler actor
actor SchedulerActor {
    data SchedulerState {
        actors: Map<String, ActorRef>,
        schedule: List<ScheduleEntry>
    }

    async fn handle_schedule(self_state: SchedulerState, entry: ScheduleEntry) -> (SchedulerState, Unit) {
        let new_schedule = self_state.schedule.append(entry)
        
        let new_state = SchedulerState {
            actors: self_state.actors,
            schedule: new_schedule
        }
        
        return (new_state, Unit)
    }
}

// Schedule entry
type ScheduleEntry = {
    actor_id: String
    message: Message
    timestamp: DateTime
    energy_budget: Float
    deterministic: Bool
}
```

### 2.2. Message Ordering

```ferra
// Message queue actor
actor MessageQueueActor {
    data QueueState {
        queue: List<Message>,
        order: Map<String, Int>
    }

    async fn handle_enqueue(self_state: QueueState, message: Message) -> (QueueState, Unit) {
        let order = self_state.order.get(message.type) || 0
        
        let new_state = QueueState {
            queue: self_state.queue.append(message),
            order: self_state.order.insert(message.type, order + 1)
        }
        
        return (new_state, Unit)
    }

    async fn handle_dequeue(self_state: QueueState) -> (QueueState, Message?) {
        if self_state.queue.is_empty() {
            return (self_state, null)
        }
        
        let message = self_state.queue.first()
        let new_state = QueueState {
            queue: self_state.queue.drop(1),
            order: self_state.order
        }
        
        return (new_state, message)
    }
}
```

## 3. Distributed Systems (1 hour)

### 3.1. Actor Clustering

```ferra
// Cluster actor
actor ClusterActor {
    data ClusterState {
        nodes: Map<String, Node>,
        partitions: Map<String, Partition>
    }

    async fn handle_join(self_state: ClusterState, node: Node) -> (ClusterState, Result<Unit>) {
        let new_state = ClusterState {
            nodes: self_state.nodes.insert(node.id, node),
            partitions: self_state.partitions
        }
        
        return (new_state, Unit)
    }

    async fn handle_partition(self_state: ClusterState, partition: Partition) -> (ClusterState, Result<Unit>) {
        let new_state = ClusterState {
            nodes: self_state.nodes,
            partitions: self_state.partitions.insert(partition.id, partition)
        }
        
        return (new_state, Unit)
    }
}

// Node definition
type Node = {
    id: String
    address: String
    capabilities: Set<String>
    energy_budget: Float
    deterministic: Bool
}
```

### 3.2. Message Routing

```ferra
// Router actor
actor RouterActor {
    data RouterState {
        routes: Map<String, Route>,
        load_balancer: LoadBalancer
    }

    async fn handle_route(self_state: RouterState, message: Message) -> (RouterState, Result<Unit>) {
        let route = self_state.routes.get(message.type)
        
        if !route {
            return (self_state, Error("No route for message type"))
        }
        
        let target = await self_state.load_balancer.select(route.targets)
        await target.ask(message)
        
        return (self_state, Unit)
    }
}

// Route definition
type Route = {
    pattern: String
    targets: List<ActorRef>
    strategy: RoutingStrategy
    energy_budget: Float
}
```

## 4. Error Handling & Recovery (1 hour)

### 4.1. Actor Supervision

```ferra
// Supervisor actor
actor SupervisorActor {
    data SupervisorState {
        children: Map<String, ActorRef>,
        strategies: Map<String, Strategy>
    }

    async fn handle_failure(self_state: SupervisorState, failure: Failure) -> (SupervisorState, Unit) {
        let strategy = self_state.strategies.get(failure.type)
        
        match strategy {
            Restart => {
                let child = self_state.children.get(failure.actor_id)
                await child.restart()
            }
            Stop => {
                let child = self_state.children.get(failure.actor_id)
                await child.stop()
            }
            Escalate => {
                await self.ask(Escalate(failure))
            }
        }
        
        return (self_state, Unit)
    }
}

// Failure types
type Failure = {
    actor_id: String
    type: String
    error: Error
    timestamp: DateTime
    energy_budget: Float
}
```

### 4.2. Circuit Breaking

```ferra
// Circuit breaker actor
actor CircuitBreakerActor {
    data BreakerState {
        failures: Int
        threshold: Int
        timeout: Duration
        last_failure: DateTime?
    }

    async fn handle_request(self_state: BreakerState) -> (BreakerState, Result<Unit>) {
        if self_state.is_open() {
            return (self_state, Error("Circuit breaker is open"))
        }
        
        try {
            let result = await handle_operation()
            return (self_state, result)
        } catch error {
            let new_state = BreakerState {
                failures: self_state.failures + 1,
                threshold: self_state.threshold,
                timeout: self_state.timeout,
                last_failure: now()
            }
            
            return (new_state, Error("Operation failed"))
        }
    }
}
```

## Quiz

1. What is the main benefit of using actors for concurrency?
   - A. Better performance
   - B. Deterministic execution
   - C. Simpler implementation
   - D. Faster response times

2. How do you handle message ordering in Ferra?
   - A. Using timestamps
   - B. Using sequence numbers
   - C. Both A and B
   - D. Neither A nor B

3. Which strategy is used for actor supervision?
   - A. Restart
   - B. Stop
   - C. Escalate
   - D. All of the above

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Concurrency Model](../../reference/CONCURRENCY_MODEL.md)
- [Actor Model](../../reference/ACTOR_MODEL.md)
- [Error Handling](../../reference/ERROR_HANDLING.md)
- [Energy Profiler](../../reference/ENERGY_PROFILER.md)

## Next Steps

- [Energy Profiling](./energy_profiling.md)
- [Data-Parallel Processing](./data_parallel.md)
- [GPU Acceleration](./gpu_acceleration.md)

## Actor-based concurrency with message passing
actor MessageActor {
    data ActorState {
        messages: List<Message>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ActorState {
        return ActorState {
            messages: List::new(),
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
                scope: "actor",
                audit_log: []
            }
        }
    }

    async fn handle_message(self_state: ActorState, msg: Message) -> (ActorState, Response) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::MessageHandling]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                "system",
                "handle_message",
                "message",
                false,
                "Missing required capabilities"
            );
            return (self_state, Response::Error("Permission denied"));
        }

        // Check energy budget
        let message_energy_cost = calculate_energy_cost(10.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < message_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                "system",
                "handle_message",
                "message",
                false,
                "Insufficient energy budget"
            );
            return (self_state, Response::Error("Insufficient energy"));
        }

        // Process message
        let response = match msg {
            Message::Text(text) => {
                self_state.messages.append(text);
                Response::Success("Message received")
            },
            Message::Command(cmd) => {
                match cmd {
                    Command::Clear => {
                        self_state.messages.clear();
                        Response::Success("Messages cleared")
                    },
                    Command::Count => {
                        Response::Success(format!("Message count: {}", self_state.messages.len()))
                    }
                }
            }
        };

        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            "system",
            "handle_message",
            "message",
            true,
            null
        );

        let new_state = ActorState {
            messages: self_state.messages,
            energy_budget: self_state.energy_budget - message_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        };

        return (new_state, response);
    }
}

// Message types
data Message {
    Text(String)
    Command(Command)
}

data Command {
    Clear
    Count
}

data Response {
    Success(String)
    Error(String)
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

// Security model
data Capability {
    MessageHandling
    DataAccess
    EnergyManagement
}

data SecurityContext {
    principal: String
    granted_capabilities: Set<Capability>
    scope: String
    audit_log: List<AuditEntry>
}

data AuditEntry {
    timestamp: Time
    principal: String
    operation: String
    resource: String
    success: Bool
    reason: String?
}

fn has_required_capabilities(context: SecurityContext, required: List<Capability>) -> Bool {
    for capability in required {
        if !context.granted_capabilities.contains(capability) {
            return false;
        }
    }
    return true;
}

fn audit_operation(
    context: SecurityContext,
    principal: String,
    operation: String,
    resource: String,
    success: Bool,
    reason: String?
) -> SecurityContext {
    let entry = AuditEntry {
        timestamp: now(),
        principal: principal,
        operation: operation,
        resource: resource,
        success: success,
        reason: reason
    };
    
    return SecurityContext {
        principal: context.principal,
        granted_capabilities: context.granted_capabilities,
        scope: context.scope,
        audit_log: context.audit_log.append(entry)
    };
} 