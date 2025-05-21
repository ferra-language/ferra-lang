---
title: "Distributed Systems"
duration: "4h"
level: "advanced"
---

# Distributed Systems

> **Duration**: 4 hours
> **Goal**: Build reliable distributed systems using Ferra's distributed computing framework

## Overview

This tutorial covers distributed system design, consensus algorithms, and fault tolerance in Ferra applications.

## 1. Distributed Basics (1 hour)

### 1.1. Node Management

```ferra
// Node manager actor with energy budget and security model
#[ai::tag(distributed_component)]
actor NodeManagerActor {
    data ManagerState {
        nodes: Map<String, Node>,
        connections: Map<String, Connection>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>,  // Security capabilities
        energy_metrics: EnergyMetrics,  // Track energy usage
        security_context: SecurityContext  // Security context for operations
    }

    data EnergyMetrics {
        total_ops: Int,  // Total micro-operations
        alu_ops: Int,    // ALU operations
        mem_ops: Int,    // Memory operations
        fp_ops: Int,     // Floating point operations
        last_measurement: Time
    }

    fn init() -> ManagerState {
        return ManagerState {
            nodes: Map::new(),
            connections: Map::new(),
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
                scope: "node_management",
                audit_log: []
            }
        }
    }

    async fn handle_join(self_state: ManagerState, request: JoinRequest, replier: ActorRef<JoinResponse>) -> ManagerState {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::NodeManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "join",
                request.id,
                false,
                "Missing required capabilities"
            );
            replier ! JoinResponse { success: false, error: JoinError::PermissionDenied };
            return ManagerState {
                nodes: self_state.nodes,
                connections: self_state.connections,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            };
        }

        // Check energy budget
        let join_energy_cost = calculate_energy_cost(10.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < join_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "join",
                request.id,
                false,
                "Insufficient energy budget"
            );
            replier ! JoinResponse { success: false, error: JoinError::InsufficientEnergy };
            return ManagerState {
                nodes: self_state.nodes,
                connections: self_state.connections,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            };
        }

        // Validate request
        if !is_valid_join_request(request) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "join",
                request.id,
                false,
                "Invalid request"
            );
            replier ! JoinResponse { success: false, error: JoinError::InvalidRequest };
            return ManagerState {
                nodes: self_state.nodes,
                connections: self_state.connections,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            };
        }

        let node = Node {
            id: request.id,
            type: request.type,
            status: "active",
            energy_budget: request.energy_budget
        };
        
        let new_state = ManagerState {
            nodes: self_state.nodes.insert(node.id, node),
            connections: self_state.connections,
            energy_budget: self_state.energy_budget - join_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        // Create a channel for join notification
        let (tx, rx) = Channel::new(capacity: 1);
        
        // Broadcast join notification
        match await broadcast_join(node, tx) {
            Ok(()) => {
                // Wait for confirmation
                match await rx.receive() {
                    Ok(JoinConfirmation { success: true }) => {
                        let end_ops = measure_ops();
                        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
                        let new_context = audit_operation(
                            new_state.security_context,
                            request.principal,
                            "join",
                            request.id,
                            true,
                            null
                        );
                        replier ! JoinResponse { success: true, error: null };
                        return ManagerState {
                            nodes: new_state.nodes,
                            connections: new_state.connections,
                            energy_budget: new_state.energy_budget,
                            permissions: new_state.permissions,
                            energy_metrics: new_metrics,
                            security_context: new_context
                        };
                    }
                    Ok(JoinConfirmation { success: false }) => {
                        let end_ops = measure_ops();
                        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                        let new_context = audit_operation(
                            self_state.security_context,
                            request.principal,
                            "join",
                            request.id,
                            false,
                            "Broadcast failed"
                        );
                        replier ! JoinResponse { success: false, error: JoinError::BroadcastFailed };
                        return ManagerState {
                            nodes: self_state.nodes,
                            connections: self_state.connections,
                            energy_budget: self_state.energy_budget,
                            permissions: self_state.permissions,
                            energy_metrics: new_metrics,
                            security_context: new_context
                        };
                    }
                    Err(_) => {
                        let end_ops = measure_ops();
                        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                        let new_context = audit_operation(
                            self_state.security_context,
                            request.principal,
                            "join",
                            request.id,
                            false,
                            "Channel closed"
                        );
                        replier ! JoinResponse { success: false, error: JoinError::ChannelClosed };
                        return ManagerState {
                            nodes: self_state.nodes,
                            connections: self_state.connections,
                            energy_budget: self_state.energy_budget,
                            permissions: self_state.permissions,
                            energy_metrics: new_metrics,
                            security_context: new_context
                        };
                    }
                }
            }
            Err(e) => {
                let end_ops = measure_ops();
                let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                let new_context = audit_operation(
                    self_state.security_context,
                    request.principal,
                    "join",
                    request.id,
                    false,
                    "Broadcast failed"
                );
                replier ! JoinResponse { success: false, error: JoinError::BroadcastFailed };
                return ManagerState {
                    nodes: self_state.nodes,
                    connections: self_state.connections,
                    energy_budget: self_state.energy_budget,
                    permissions: self_state.permissions,
                    energy_metrics: new_metrics,
                    security_context: new_context
                };
            }
        }
    }
}

// Energy profiling implementation
data EnergyMetrics {
    total_ops: Int,  // Total micro-operations
    alu_ops: Int,    // ALU operations
    mem_ops: Int,    // Memory operations
    fp_ops: Int,     // Floating point operations
    last_measurement: Time
}

data EnergyProfile {
    metrics: EnergyMetrics
    tdp_factor: Float  // Thermal Design Power factor
    energy_cost: Float // Total energy cost in joules
}

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
    // These factors would be calibrated against real hardware measurements
    let alu_cost = metrics.alu_ops * 1.0;  // Base TDP factor for ALU
    let mem_cost = metrics.mem_ops * 5.0;  // Higher TDP factor for memory ops
    let fp_cost = metrics.fp_ops * 3.0;    // Higher TDP factor for FP ops
    
    return base_cost + (alu_cost + mem_cost + fp_cost).joules;
}

fn create_energy_profile(metrics: EnergyMetrics) -> EnergyProfile {
    // Calculate TDP factor based on operation mix
    let total_ops = metrics.total_ops;
    let alu_ratio = metrics.alu_ops / total_ops;
    let mem_ratio = metrics.mem_ops / total_ops;
    let fp_ratio = metrics.fp_ops / total_ops;
    
    // Weighted TDP factor based on operation mix
    let tdp_factor = (alu_ratio * 1.0) + (mem_ratio * 5.0) + (fp_ratio * 3.0);
    
    // Calculate total energy cost
    let energy_cost = calculate_energy_cost(0.0.joules, metrics);
    
    return EnergyProfile {
        metrics: metrics,
        tdp_factor: tdp_factor,
        energy_cost: energy_cost
    };
}

// Energy budget management
data EnergyBudget {
    total: Float
    used: Float
    remaining: Float
    last_update: Time
}

fn update_energy_budget(budget: EnergyBudget, cost: Float) -> EnergyBudget {
    let new_used = budget.used + cost;
    let new_remaining = budget.total - new_used;
    
    return EnergyBudget {
        total: budget.total,
        used: new_used,
        remaining: new_remaining,
        last_update: now()
    };
}

fn check_energy_budget(budget: EnergyBudget, required: Float) -> Bool {
    return budget.remaining >= required;
}

// Energy monitoring
data EnergyMonitor {
    profiles: List<EnergyProfile>
    budgets: List<EnergyBudget>
    alerts: List<EnergyAlert>
}

data EnergyAlert {
    timestamp: Time
    level: AlertLevel
    message: String
    profile: EnergyProfile
    budget: EnergyBudget
}

data AlertLevel {
    Info
    Warning
    Critical
}

fn monitor_energy_usage(monitor: EnergyMonitor, profile: EnergyProfile, budget: EnergyBudget) -> EnergyMonitor {
    let alerts = monitor.alerts;
    
    // Check for high energy usage
    if profile.energy_cost > budget.total * 0.8 {
        alerts = alerts.append(EnergyAlert {
            timestamp: now(),
            level: AlertLevel::Warning,
            message: "High energy usage detected",
            profile: profile,
            budget: budget
        });
    }
    
    // Check for critical energy usage
    if profile.energy_cost > budget.total * 0.95 {
        alerts = alerts.append(EnergyAlert {
            timestamp: now(),
            level: AlertLevel::Critical,
            message: "Critical energy usage detected",
            profile: profile,
            budget: budget
        });
    }
    
    return EnergyMonitor {
        profiles: monitor.profiles.append(profile),
        budgets: monitor.budgets.append(budget),
        alerts: alerts
    };
}

// Message definitions
data JoinRequest {
    id: String
    type: String
    energy_budget: Float
    principal: String  // Identity making the request
    capabilities: Set<Capability>  // Requested capabilities
    scope: String  // Requested scope of operations
}

data JoinResponse {
    success: Bool
    error: JoinError?
    granted_capabilities: Set<Capability>?  // Granted capabilities if successful
    scope: String?  // Granted scope if successful
}

data JoinConfirmation {
    success: Bool
    principal: String  // Identity confirming the join
    capabilities: Set<Capability>  // Capabilities used for confirmation
}

data JoinError {
    InvalidRequest
    BroadcastFailed
    ChannelClosed
    PermissionDenied
    InsufficientEnergy
    InvalidCapabilities
    InvalidScope
}

// Security model implementation
data Capability {
    NodeManagement
    ConnectionManagement
    BroadcastManagement
    EnergyManagement
    SecurityManagement
}

data SecurityContext {
    principal: String  // The identity making the request
    granted_capabilities: Set<Capability>  // Capabilities granted to this context
    scope: String  // Scope of operations (e.g., "node_management", "connection_management")
    audit_log: List<AuditEntry>  // Log of security-relevant operations
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

data SecurityError {
    InsufficientCapabilities
    InvalidScope
    ExcessiveEnergyCost
    UnknownOperation
    InvalidPrincipal
    InvalidResource
}

// Security context propagation
fn propagate_security_context(
    context: SecurityContext,
    operation: String,
    resource: String
) -> SecurityContext {
    // Create a new context with the same capabilities but updated scope
    return SecurityContext {
        principal: context.principal,
        granted_capabilities: context.granted_capabilities,
        scope: format("{}.{}", context.scope, operation),
        audit_log: context.audit_log
    };
}

// Security audit trail
data SecurityAudit {
    entries: List<AuditEntry>
    violations: List<SecurityViolation>
    energy_usage: List<EnergyUsage>
}

data SecurityViolation {
    timestamp: Time
    principal: String
    operation: String
    resource: String
    error: SecurityError
    context: SecurityContext
}

data EnergyUsage {
    timestamp: Time
    principal: String
    operation: String
    resource: String
    cost: Float
    context: SecurityContext
}

fn record_security_violation(
    audit: SecurityAudit,
    principal: String,
    operation: String,
    resource: String,
    error: SecurityError,
    context: SecurityContext
) -> SecurityAudit {
    let violation = SecurityViolation {
        timestamp: now(),
        principal: principal,
        operation: operation,
        resource: resource,
        error: error,
        context: context
    };
    
    return SecurityAudit {
        entries: audit.entries,
        violations: audit.violations.append(violation),
        energy_usage: audit.energy_usage
    };
}

fn record_energy_usage(
    audit: SecurityAudit,
    principal: String,
    operation: String,
    resource: String,
    cost: Float,
    context: SecurityContext
) -> SecurityAudit {
    let usage = EnergyUsage {
        timestamp: now(),
        principal: principal,
        operation: operation,
        resource: resource,
        cost: cost,
        context: context
    };
    
    return SecurityAudit {
        entries: audit.entries,
        violations: audit.violations,
        energy_usage: audit.energy_usage.append(usage)
    };
}

// Node definition
type Node = {
    id: String
    type: String
    status: String
    energy_budget: Float
}
```

### 1.2. Communication

```ferra
// Communication manager actor with energy budget and security model
#[ai::tag(distributed_component)]
actor CommunicationManagerActor {
    data ManagerState {
        channels: Map<String, Channel<Message>>,
        subscribers: Map<String, Set<ActorRef<Message>>>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ManagerState {
        return ManagerState {
            channels: Map::new(),
            subscribers: Map::new(),
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
                scope: "communication_management",
                audit_log: []
            }
        }
    }

    async fn handle_message(self_state: ManagerState, request: Message, replier: ActorRef<MessageResponse>) -> ManagerState {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::BroadcastManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "broadcast",
                request.channel_id,
                false,
                "Missing required capabilities"
            );
            replier ! MessageResponse { success: false, error: CommunicationError::PermissionDenied };
            return ManagerState {
                channels: self_state.channels,
                subscribers: self_state.subscribers,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            };
        }

        // Check energy budget
        let message_energy_cost = calculate_energy_cost(5.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < message_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "broadcast",
                request.channel_id,
                false,
                "Insufficient energy budget"
            );
            replier ! MessageResponse { success: false, error: CommunicationError::InsufficientEnergy };
            return ManagerState {
                channels: self_state.channels,
                subscribers: self_state.subscribers,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            };
        }

        // Get or create channel
        let channel = match self_state.channels.get(request.channel_id) {
            Some(ch) => ch,
            None => {
                let (tx, rx) = Channel::new(capacity: 100);
                self_state.channels.insert(request.channel_id, tx);
                tx
            }
        };

        // Create a channel for broadcast confirmation
        let (confirm_tx, confirm_rx) = Channel::new(capacity: 1);
        
        // Broadcast message
        match await broadcast_message(channel, request, confirm_tx) {
            Ok(()) => {
                // Wait for confirmation
                match await confirm_rx.receive() {
                    Ok(BroadcastConfirmation { success: true }) => {
                        let end_ops = measure_ops();
                        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                        let new_context = audit_operation(
                            self_state.security_context,
                            request.principal,
                            "broadcast",
                            request.channel_id,
                            true,
                            null
                        );
                        replier ! MessageResponse { success: true, error: null };
                        return ManagerState {
            channels: self_state.channels,
                            subscribers: self_state.subscribers,
                            energy_budget: self_state.energy_budget - message_energy_cost,
                            permissions: self_state.permissions,
                            energy_metrics: new_metrics,
                            security_context: new_context
                        };
                    }
                    Ok(BroadcastConfirmation { success: false }) => {
                        let end_ops = measure_ops();
                        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                        let new_context = audit_operation(
                            self_state.security_context,
                            request.principal,
                            "broadcast",
                            request.channel_id,
                            false,
                            "Broadcast failed"
                        );
                        replier ! MessageResponse { success: false, error: CommunicationError::BroadcastFailed };
                        return ManagerState {
                            channels: self_state.channels,
                            subscribers: self_state.subscribers,
                            energy_budget: self_state.energy_budget,
                            permissions: self_state.permissions,
                            energy_metrics: new_metrics,
                            security_context: new_context
                        };
                    }
                    Err(_) => {
                        let end_ops = measure_ops();
                        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                        let new_context = audit_operation(
                            self_state.security_context,
                            request.principal,
                            "broadcast",
                            request.channel_id,
                            false,
                            "Channel closed"
                        );
                        replier ! MessageResponse { success: false, error: CommunicationError::ChannelClosed };
                        return ManagerState {
                            channels: self_state.channels,
                            subscribers: self_state.subscribers,
                            energy_budget: self_state.energy_budget,
                            permissions: self_state.permissions,
                            energy_metrics: new_metrics,
                            security_context: new_context
                        };
                    }
                }
            }
            Err(e) => {
                let end_ops = measure_ops();
                let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
                let new_context = audit_operation(
                    self_state.security_context,
                    request.principal,
                    "broadcast",
                    request.channel_id,
                    false,
                    "Broadcast failed"
                );
                replier ! MessageResponse { success: false, error: CommunicationError::BroadcastFailed };
                return ManagerState {
                    channels: self_state.channels,
                    subscribers: self_state.subscribers,
                    energy_budget: self_state.energy_budget,
                    permissions: self_state.permissions,
                    energy_metrics: new_metrics,
                    security_context: new_context
                };
            }
        }
    }

    async fn handle_subscribe(self_state: ManagerState, request: SubscribeRequest, replier: ActorRef<SubscribeResponse>) -> ManagerState {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, [Capability::BroadcastManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "subscribe",
                request.channel_id,
                false,
                "Missing required capabilities"
            );
            replier ! SubscribeResponse { success: false, error: CommunicationError::PermissionDenied };
            return ManagerState {
                channels: self_state.channels,
                subscribers: self_state.subscribers,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            };
        }

        // Get or create subscribers set
        let subscribers = match self_state.subscribers.get(request.channel_id) {
            Some(subs) => subs,
            None => Set::new()
        };

        // Add subscriber
        let new_subscribers = subscribers.insert(request.subscriber);
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "subscribe",
            request.channel_id,
            true,
            null
        );
        
        replier ! SubscribeResponse { success: true, error: null };
        return ManagerState {
            channels: self_state.channels,
            subscribers: self_state.subscribers.insert(request.channel_id, new_subscribers),
            energy_budget: self_state.energy_budget,
            permissions: self_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        };
    }
}

// Message definitions
data Message {
    channel_id: String
    content: String
    principal: String
    capabilities: Set<Capability>
    scope: String
}

data MessageResponse {
    success: Bool
    error: CommunicationError?
}

data BroadcastConfirmation {
    success: Bool
    principal: String
    capabilities: Set<Capability>
}

data CommunicationError {
    InvalidRequest
    BroadcastFailed
    ChannelClosed
    PermissionDenied
    InsufficientEnergy
    InvalidCapabilities
    InvalidScope
}

data SubscribeRequest {
    channel_id: String
    subscriber: ActorRef<Message>
    principal: String
    capabilities: Set<Capability>
    scope: String
}

data SubscribeResponse {
    success: Bool
    error: CommunicationError?
}

// Helper functions
async fn broadcast_message(channel: Channel<Message>, message: Message, confirm_tx: Channel<BroadcastConfirmation>) -> Result<(), Error> {
    match await channel.send(message) {
        Ok(()) => {
            confirm_tx.send(BroadcastConfirmation {
                success: true,
                principal: message.principal,
                capabilities: message.capabilities
            });
            Ok(())
        }
        Err(e) => {
            confirm_tx.send(BroadcastConfirmation {
                success: false,
                principal: message.principal,
                capabilities: message.capabilities
            });
            Err(e)
        }
    }
}
```

## 2. Consensus Algorithms (1 hour)

### 2.1. Leader Election

```ferra
// Leader election actor with security model and energy profiling
#[ai::tag(distributed_component)]
actor LeaderElectionActor {
    data ElectionState {
        nodes: Map<String, Node>,
        leader: String?,
        term: Int,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> ElectionState {
        return ElectionState {
            nodes: Map::new(),
            leader: null,
            term: 0,
            energy_budget: 3000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_election(self_state: ElectionState, request: ElectionRequest) -> (ElectionState, Result<Unit, ElectionError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::ElectionManagement) {
            return (self_state, Err(ElectionError::PermissionDenied))
        }

        // Check energy budget
        let election_energy_cost = 20.0.joules
        if self_state.energy_budget < election_energy_cost {
            return (self_state, Err(ElectionError::InsufficientEnergy))
        }

        let votes = []
        
        for node in self_state.nodes {
            let vote = await node.vote(request.candidate)
            votes.append(vote)
        }
        
        if count_votes(votes) > self_state.nodes.size() / 2 {
            let new_state = ElectionState {
                nodes: self_state.nodes,
                leader: request.candidate,
                term: self_state.term + 1,
                energy_budget: self_state.energy_budget - election_energy_cost,
                permissions: self_state.permissions
            }
            
            match await broadcast_leader(request.candidate) {
                Ok(_) => return (new_state, Ok(Unit)),
                Err(e) => return (self_state, Err(ElectionError::BroadcastFailed))
            }
        }
        
        return (self_state, Err(ElectionError::ElectionFailed))
    }
}

// Message definitions
data ElectionRequest {
    candidate: String
    term: Int
    energy_budget: Float
}

data ElectionError {
    ElectionFailed
    BroadcastFailed
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
}

// Security model
data Capability {
    ElectionManagement
    VoteManagement
    BroadcastManagement
}

// Vote definition
type Vote = {
    id: String
    candidate: String
    term: Int
    energy_budget: Float
}
```

### 2.2. State Replication

```ferra
// State replicator actor with security model and energy profiling
#[ai::tag(distributed_component)]
actor StateReplicatorActor {
    data ReplicatorState {
        state: State,
        replicas: Map<String, Replica>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> ReplicatorState {
        return ReplicatorState {
            state: State::new(),
            replicas: Map::new(),
            energy_budget: 4000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_replicate(self_state: ReplicatorState, update: StateUpdate) -> (ReplicatorState, Result<Unit, ReplicationError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::ReplicationManagement) {
            return (self_state, Err(ReplicationError::PermissionDenied))
        }

        // Check energy budget
        let replication_energy_cost = 25.0.joules
        if self_state.energy_budget < replication_energy_cost {
            return (self_state, Err(ReplicationError::InsufficientEnergy))
        }

        let new_state = State {
            id: self_state.state.id,
            data: update.data,
            version: self_state.state.version + 1,
            energy_budget: self_state.state.energy_budget
        }
        
        for replica in self_state.replicas {
            match await replica.update(new_state) {
                Ok(_) => continue,
                Err(e) => return (self_state, Err(ReplicationError::ReplicaUpdateFailed))
            }
        }
        
        let new_replicator_state = ReplicatorState {
            state: new_state,
            replicas: self_state.replicas,
            energy_budget: self_state.energy_budget - replication_energy_cost,
            permissions: self_state.permissions
        }
        
        return (new_replicator_state, Ok(Unit))
    }
}

// Message definitions
data StateUpdate {
    data: Any
    version: Int
    energy_budget: Float
}

data ReplicationError {
    ReplicaUpdateFailed
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
}

// Security model
data Capability {
    ReplicationManagement
    StateManagement
    ReplicaManagement
}

// Replica definition
type Replica = {
    id: String
    state: State
    update: Function
    energy_budget: Float
}
```

## 3. Fault Tolerance (1 hour)

### 3.1. Failure Detection

```ferra
// Failure detector actor with security model and energy profiling
#[ai::tag(distributed_component)]
actor FailureDetectorActor {
    data DetectorState {
        nodes: Map<String, Node>,
        failures: List<Failure>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> DetectorState {
        return DetectorState {
            nodes: Map::new(),
            failures: [],
            energy_budget: 5000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_check(self_state: DetectorState, node: Node) -> (DetectorState, Result<Unit, DetectionError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::FailureDetection) {
            return (self_state, Err(DetectionError::PermissionDenied))
        }

        // Check energy budget
        let check_energy_cost = 15.0.joules
        if self_state.energy_budget < check_energy_cost {
            return (self_state, Err(DetectionError::InsufficientEnergy))
        }

        let status = await check_node(node)
        
        if status == "failed" {
            let failure = Failure {
                node: node.id,
                time: now(),
                reason: "timeout",
                energy_budget: 0.01.joules
            }
            
            let new_state = DetectorState {
                nodes: self_state.nodes,
                failures: self_state.failures.append(failure),
                energy_budget: self_state.energy_budget - check_energy_cost,
                permissions: self_state.permissions
            }
            
            match await handle_failure(failure) {
                Ok(_) => return (new_state, Ok(Unit)),
                Err(e) => return (self_state, Err(DetectionError::FailureHandlingFailed))
            }
        }
        
        return (self_state, Ok(Unit))
    }
}

// Message definitions
data DetectionError {
    FailureHandlingFailed
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
}

// Security model
data Capability {
    FailureDetection
    NodeManagement
    FailureHandling
}

// Failure definition
type Failure = {
    node: String
    time: Time
    reason: String
    energy_budget: Float
}
```

### 3.2. Recovery

```ferra
// Recovery manager actor with security model and energy profiling
#[ai::tag(distributed_component)]
actor RecoveryManagerActor {
    data ManagerState {
        failures: List<Failure>,
        strategies: Map<String, Strategy>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> ManagerState {
        return ManagerState {
            failures: [],
            strategies: Map::new(),
            energy_budget: 6000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_recover(self_state: ManagerState, failure: Failure) -> (ManagerState, Result<Unit, RecoveryError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::RecoveryManagement) {
            return (self_state, Err(RecoveryError::PermissionDenied))
        }

        // Check energy budget
        let recovery_energy_cost = 35.0.joules
        if self_state.energy_budget < recovery_energy_cost {
            return (self_state, Err(RecoveryError::InsufficientEnergy))
        }

        let strategy = match self_state.strategies.get(failure.type) {
            Some(s) => s,
            None => return (self_state, Err(RecoveryError::StrategyNotFound))
        }
        
        match await strategy.recover(failure) {
            Ok(_) => {
        let new_state = ManagerState {
            failures: self_state.failures.filter { f => f.node != failure.node },
                    strategies: self_state.strategies,
                    energy_budget: self_state.energy_budget - recovery_energy_cost,
                    permissions: self_state.permissions
                }
                return (new_state, Ok(Unit))
            }
            Err(e) => return (self_state, Err(RecoveryError::RecoveryFailed))
        }
    }
}

// Message definitions
data RecoveryError {
    StrategyNotFound
    RecoveryFailed
    PermissionDenied
    InsufficientEnergy
    InvalidRequest
}

// Security model
data Capability {
    RecoveryManagement
    StrategyManagement
    FailureHandling
}

// Strategy definition
type Strategy = {
    id: String
    type: String
    recover: Function
    energy_budget: Float
}
```

## 4. Scalability (1 hour)

### 4.1. Load Distribution

```ferra
// Load distributor actor with security model and energy profiling
#[ai::tag(distributed_component)]
actor LoadDistributorActor {
    data DistributorState {
        nodes: Map<String, Node>,
        distribution: Map<String, Distribution>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> DistributorState {
        return DistributorState {
            nodes: Map::new(),
            distribution: Map::new(),
            energy_budget: 7000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_distribute(self_state: DistributorState, request: DistributionRequest) -> (DistributorState, Result<Unit, DistributionError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::DistributionManagement) {
            return (self_state, Err(DistributionError::PermissionDenied))
        }

        // Check energy budget
        let distribution_energy_cost = 40.0.joules
        if self_state.energy_budget < distribution_energy_cost {
            return (self_state, Err(DistributionError::InsufficientEnergy))
        }

        let nodes = self_state.nodes.values()
        
        let distribution = calculate_distribution(nodes, request.load)
        
        for node in nodes {
            match await node.adjust(distribution.get(node.id)) {
                Ok(_) => continue,
                Err(e) => return (self_state, Err(DistributionError::AdjustmentFailed))
            }
        }
        
        let new_state = DistributorState {
            nodes: self_state.nodes,
            distribution: self_state.distribution.insert(request.id, distribution),
            energy_budget: self_state.energy_budget - distribution_energy_cost,
            permissions: self_state.permissions
        }
        
        return (new_state, Ok(Unit))
    }
}

// Message definitions
data DistributionRequest {
    id: String
    load: Float
    energy_budget: Float
}

data DistributionError {
    PermissionDenied
    InsufficientEnergy
    AdjustmentFailed
    InvalidRequest
}

// Security model
data Capability {
    DistributionManagement
    NodeManagement
    EnergyManagement
}

// Distribution definition
type Distribution = {
    id: String
    load: Float
    nodes: Map<String, Float>
    energy_budget: Float
}
```

### 4.2. Resource Scaling

```ferra
// Resource scaler actor with security model and energy profiling
#[ai::tag(distributed_component)]
actor ResourceScalerActor {
    data ScalerState {
        resources: Map<String, Resource>,
        metrics: Map<String, Metric>,
        energy_budget: Float,  // Total energy budget for the manager
        permissions: Set<Capability>  // Security capabilities
    }

    fn init() -> ScalerState {
        return ScalerState {
            resources: Map::new(),
            metrics: Map::new(),
            energy_budget: 8000.0.joules,  // Initial energy budget
            permissions: Set::new()
        }
    }

    async fn handle_scale(self_state: ScalerState, request: ScaleRequest) -> (ScalerState, Result<Unit, ScalingError>) {
        // Check permissions
        if !self_state.permissions.contains(Capability::ScalingManagement) {
            return (self_state, Err(ScalingError::PermissionDenied))
        }

        // Check energy budget
        let scaling_energy_cost = 45.0.joules
        if self_state.energy_budget < scaling_energy_cost {
            return (self_state, Err(ScalingError::InsufficientEnergy))
        }

        let resource = match self_state.resources.get(request.resource) {
            Some(r) => r,
            None => return (self_state, Err(ScalingError::ResourceNotFound))
        }
        
        let metric = match self_state.metrics.get(resource.id) {
            Some(m) => m,
            None => return (self_state, Err(ScalingError::MetricNotFound))
        }
        
        if metric.value > request.threshold {
            match await resource.scale(request.factor) {
                Ok(_) => {
        let new_state = ScalerState {
            resources: self_state.resources,
                        metrics: self_state.metrics,
                        energy_budget: self_state.energy_budget - scaling_energy_cost,
                        permissions: self_state.permissions
                    }
                    return (new_state, Ok(Unit))
                }
                Err(e) => return (self_state, Err(ScalingError::ScalingFailed))
            }
        }
        
        return (self_state, Ok(Unit))
    }
}

// Message definitions
data ScaleRequest {
    resource: String
    threshold: Float
    factor: Float
    energy_budget: Float
}

data ScalingError {
    ResourceNotFound
    MetricNotFound
    PermissionDenied
    InsufficientEnergy
    ScalingFailed
    InvalidRequest
}

// Security model
data Capability {
    ScalingManagement
    ResourceManagement
    EnergyManagement
}

// Resource definition
type Resource = {
    id: String
    type: String
    scale: Function
    energy_budget: Float
}
```

## Quiz

1. What is the main benefit of distributed systems?
   - A. Better performance
   - B. Fault tolerance
   - C. Simpler implementation
   - D. Faster response times

2. How do you handle consensus in distributed systems?
   - A. Using leader election
   - B. Using state replication
   - C. Both A and B
   - D. Neither A nor B

3. Which system is used for fault tolerance?
   - A. Failure detection
   - B. Recovery
   - C. Both A and B
   - D. Neither A nor B

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Distributed Guide](../../reference/DISTRIBUTED_GUIDE.md)
- [Consensus Guide](../../reference/CONSENSUS_GUIDE.md)
- [Fault Guide](../../reference/FAULT_GUIDE.md)
- [Scale Guide](../../reference/SCALE_GUIDE.md)

## Next Steps

- [IoT Development](./iot_development.md)
- [Microservices](./microservices.md)
- [Edge Computing](./edge_computing.md) 
- [Edge Computing](./edge_computing.md) 
- [Edge Computing](./edge_computing.md) 


