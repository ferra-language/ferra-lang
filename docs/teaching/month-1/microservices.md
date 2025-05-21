---
title: "Microservices"
duration: "4h"
level: "advanced"
---

# Microservices

> **Duration**: 4 hours
> **Goal**: Build scalable microservices using Ferra's actor model and security features

## Overview

This tutorial covers microservices architecture, service communication, and deployment using Ferra's actor model, focusing on concurrency, security, and energy profiling.

## 1. Service Architecture (1 hour)

### 1.1. Service Definition

```ferra
// Service manager actor with capability-based security
#[ai::tag(service_component)]
actor ServiceManagerActor {
    data ManagerState {
        services: Map<String, Service>,
        endpoints: Map<String, Endpoint>,
        energy_budget: Float,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ManagerState {
        return ManagerState {
            services: Map::new(),
            endpoints: Map::new(),
            energy_budget: 1000.0.joules,
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "service_management"
            }
        }
    }

    async fn handle_register(self_state: ManagerState, request: RegisterRequest) -> (ManagerState, Result<Unit>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["svc:register"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "register",
                request.service_id,
                false,
                "Missing required capabilities"
            );
            return (ManagerState {
                services: self_state.services,
                endpoints: self_state.endpoints,
                energy_budget: self_state.energy_budget,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let register_energy_cost = calculate_energy_cost(0.1.joules, self_state.energy_metrics);
        if self_state.energy_budget < register_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "register",
                request.service_id,
                false,
                "Insufficient energy budget"
            );
            return (ManagerState {
                services: self_state.services,
                endpoints: self_state.endpoints,
                energy_budget: self_state.energy_budget,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let new_state = ManagerState {
            services: self_state.services.insert(request.service_id, request.service),
            endpoints: self_state.endpoints.insert(request.service_id, request.endpoint),
            energy_budget: self_state.energy_budget,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        await register_service(request.service, request.endpoint);
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "register",
            request.service_id,
            true,
            null
        );
        
        return (ManagerState {
            services: new_state.services,
            endpoints: new_state.endpoints,
            energy_budget: self_state.energy_budget,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Unit);
    }
}

// Service definition with capabilities
type Service = {
    id: String
    name: String
    version: String
    capabilities: Set<Capability>
    energy_budget: Float
}

// Endpoint definition with security context
type Endpoint = {
    url: String
    method: String
    capabilities: Set<Capability>
    security_context: SecurityContext
    energy_budget: Float
}

// Register request
data RegisterRequest {
    service_id: String
    service: Service
    endpoint: Endpoint
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

### 1.2. Service Discovery

```ferra
// Service discovery actor with capability-based security
#[ai::tag(service_component)]
actor DiscoveryActor {
    data DiscoveryState {
        registry: Map<String, Service>,
        health: Map<String, Health>,
        energy_budget: Float,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> DiscoveryState {
        return DiscoveryState {
            registry: Map::new(),
            health: Map::new(),
            energy_budget: 1000.0.joules,
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "service_discovery"
            }
        }
    }

    async fn handle_discover(self_state: DiscoveryState, request: DiscoverRequest) -> (DiscoveryState, Result<List<Service>>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["svc:discover"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "discover",
                request.service_id,
                false,
                "Missing required capabilities"
            );
            return (DiscoveryState {
                registry: self_state.registry,
                health: self_state.health,
                energy_budget: self_state.energy_budget,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let discover_energy_cost = calculate_energy_cost(0.05.joules, self_state.energy_metrics);
        if self_state.energy_budget < discover_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "discover",
                request.service_id,
                false,
                "Insufficient energy budget"
            );
            return (DiscoveryState {
                registry: self_state.registry,
                health: self_state.health,
                energy_budget: self_state.energy_budget,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let services = self_state.registry.filter { s => 
            s.capabilities.contains_all(request.required_capabilities)
        };
        
        let new_state = DiscoveryState {
            registry: self_state.registry,
            health: self_state.health,
            energy_budget: self_state.energy_budget,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "discover",
            request.service_id,
            true,
            null
        );
        
        return (DiscoveryState {
            registry: new_state.registry,
            health: new_state.health,
            energy_budget: self_state.energy_budget,
            energy_metrics: new_metrics,
            security_context: new_context
        }, services);
    }
}

// Health check with energy metrics
type Health = {
    status: String
    last_check: Time
    energy_usage: Float
    security_context: SecurityContext
}

// Discover request
data DiscoverRequest {
    service_id: String
    required_capabilities: Set<Capability>
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

## 2. Service Communication (1 hour)

### 2.1. Message Broker

```ferra
// Message broker actor with capability-based security
#[ai::tag(service_component)]
actor MessageBrokerActor {
    data BrokerState {
        queues: Map<String, Queue>,
        subscriptions: Map<String, List<Subscription>>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> BrokerState {
        return BrokerState {
            queues: Map::new(),
            subscriptions: Map::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "message_broker"
            }
        }
    }

    async fn handle_publish(self_state: BrokerState, request: PublishRequest) -> (BrokerState, Result<Unit>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["msg:publish"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "publish",
                request.topic,
                false,
                "Missing required capabilities"
            );
            return (BrokerState {
                queues: self_state.queues,
                subscriptions: self_state.subscriptions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let publish_energy_cost = calculate_energy_cost(0.1.joules, self_state.energy_metrics);
        if self_state.energy_budget < publish_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "publish",
                request.topic,
                false,
                "Insufficient energy budget"
            );
            return (BrokerState {
                queues: self_state.queues,
                subscriptions: self_state.subscriptions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let queue = self_state.queues.get(request.topic);
        
        if !queue {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "publish",
                request.topic,
                false,
                "Queue not found"
            );
            return (BrokerState {
                queues: self_state.queues,
                subscriptions: self_state.subscriptions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Queue not found"));
        }
        
        await queue.publish(request.message);
        
        let new_state = BrokerState {
            queues: self_state.queues,
            subscriptions: self_state.subscriptions,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "publish",
            request.topic,
            true,
            null
        );
        
        return (BrokerState {
            queues: new_state.queues,
            subscriptions: new_state.subscriptions,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Unit);
    }
}

// Queue definition with security context
type Queue = {
    topic: String
    messages: List<Message>
    capabilities: Set<Capability>
    security_context: SecurityContext
    energy_budget: Float
}

// Message definition with security context
type Message = {
    id: String
    content: Any
    security_context: SecurityContext
    energy_budget: Float
}

// Publish request
data PublishRequest {
    topic: String
    message: Message
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

### 2.2. Service Gateway

```ferra
// Service gateway actor with capability-based security
#[ai::tag(service_component)]
actor GatewayActor {
    data GatewayState {
        routes: Map<String, Route>,
        middleware: List<Middleware>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> GatewayState {
        return GatewayState {
            routes: Map::new(),
            middleware: [],
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "service_gateway"
            }
        }
    }

    async fn handle_request(self_state: GatewayState, request: GatewayRequest) -> (GatewayState, Result<Response>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["svc:request"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "request",
                request.path,
                false,
                "Missing required capabilities"
            );
            return (GatewayState {
                routes: self_state.routes,
                middleware: self_state.middleware,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let request_energy_cost = calculate_energy_cost(0.2.joules, self_state.energy_metrics);
        if self_state.energy_budget < request_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "request",
                request.path,
                false,
                "Insufficient energy budget"
            );
            return (GatewayState {
                routes: self_state.routes,
                middleware: self_state.middleware,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let route = self_state.routes.get(request.path);
        
        if !route {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "request",
                request.path,
                false,
                "Route not found"
            );
            return (GatewayState {
                routes: self_state.routes,
                middleware: self_state.middleware,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Route not found"));
        }
        
        let response = await route.handle(request);
        
        let new_state = GatewayState {
            routes: self_state.routes,
            middleware: self_state.middleware,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "request",
            request.path,
            true,
            null
        );
        
        return (GatewayState {
            routes: new_state.routes,
            middleware: new_state.middleware,
            energy_metrics: new_metrics,
            security_context: new_context
        }, response);
    }
}

// Route definition with security context
type Route = {
    path: String
    service: Service
    capabilities: Set<Capability>
    security_context: SecurityContext
    energy_budget: Float
}

// Gateway request with security context
data GatewayRequest {
    path: String
    method: String
    headers: Map<String, String>
    body: Any
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

## 3. Service Deployment (1 hour)

### 3.1. Container Management

```ferra
// Container manager actor with capability-based security
#[ai::tag(service_component)]
actor ContainerManagerActor {
    data ManagerState {
        containers: Map<String, Container>,
        images: Map<String, Image>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ManagerState {
        return ManagerState {
            containers: Map::new(),
            images: Map::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "container_management"
            }
        }
    }

    async fn handle_deploy(self_state: ManagerState, request: DeployRequest) -> (ManagerState, Result<Unit>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["cnt:deploy"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "deploy",
                request.container_id,
                false,
                "Missing required capabilities"
            );
            return (ManagerState {
                containers: self_state.containers,
                images: self_state.images,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let deploy_energy_cost = calculate_energy_cost(0.3.joules, self_state.energy_metrics);
        if self_state.energy_budget < deploy_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "deploy",
                request.container_id,
                false,
                "Insufficient energy budget"
            );
            return (ManagerState {
                containers: self_state.containers,
                images: self_state.images,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let image = self_state.images.get(request.image_id);
        
        if !image {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "deploy",
                request.container_id,
                false,
                "Image not found"
            );
            return (ManagerState {
                containers: self_state.containers,
                images: self_state.images,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Image not found"));
        }
        
        let container = await create_container(image, request.config);
        
        let new_state = ManagerState {
            containers: self_state.containers.insert(request.container_id, container),
            images: self_state.images,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "deploy",
            request.container_id,
            true,
            null
        );
        
        return (ManagerState {
            containers: new_state.containers,
            images: new_state.images,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Unit);
    }
}

// Container definition with security context
type Container = {
    id: String
    image: Image
    config: Config
    capabilities: Set<Capability>
    security_context: SecurityContext
    energy_budget: Float
}

// Image definition with security context
type Image = {
    id: String
    name: String
    tag: String
    capabilities: Set<Capability>
    security_context: SecurityContext
    energy_budget: Float
}

// Deploy request
data DeployRequest {
    container_id: String
    image_id: String
    config: Config
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

### 3.2. Service Scaling

```ferra
// Service scaler actor with capability-based security
#[ai::tag(service_component)]
actor ScalerActor {
    data ScalerState {
        services: Map<String, Service>,
        metrics: Map<String, Metrics>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ScalerState {
        return ScalerState {
            services: Map::new(),
            metrics: Map::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "service_scaling"
            }
        }
    }

    async fn handle_scale(self_state: ScalerState, request: ScaleRequest) -> (ScalerState, Result<Unit>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["svc:scale"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "scale",
                request.service_id,
                false,
                "Missing required capabilities"
            );
            return (ScalerState {
                services: self_state.services,
                metrics: self_state.metrics,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let scale_energy_cost = calculate_energy_cost(0.2.joules, self_state.energy_metrics);
        if self_state.energy_budget < scale_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "scale",
                request.service_id,
                false,
                "Insufficient energy budget"
            );
            return (ScalerState {
                services: self_state.services,
                metrics: self_state.metrics,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let service = self_state.services.get(request.service_id);
        
        if !service {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "scale",
                request.service_id,
                false,
                "Service not found"
            );
            return (ScalerState {
                services: self_state.services,
                metrics: self_state.metrics,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Service not found"));
        }
        
        let metrics = self_state.metrics.get(request.service_id);
        
        if !metrics {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "scale",
                request.service_id,
                false,
                "Metrics not found"
            );
            return (ScalerState {
                services: self_state.services,
                metrics: self_state.metrics,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Metrics not found"));
        }
        
        let new_replicas = calculate_replicas(metrics, request.target);
        
        await scale_service(service, new_replicas);
        
        let new_state = ScalerState {
            services: self_state.services,
            metrics: self_state.metrics,
            energy_metrics: self_state.energy_metrics,
            security_context: self_state.security_context
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(new_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            new_state.security_context,
            request.principal,
            "scale",
            request.service_id,
            true,
            null
        );
        
        return (ScalerState {
            services: new_state.services,
            metrics: new_state.metrics,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Unit);
    }
}

// Metrics definition with energy metrics
type Metrics = {
    cpu: Float
    memory: Float
    requests: Int
    energy_usage: Float
    security_context: SecurityContext
}

// Scale request
data ScaleRequest {
    service_id: String
    target: Target
    principal: String
    capabilities: Set<Capability>
    scope: String
}
```

## Key Concepts

1. **Actor Model**:
   - Message-based communication
   - Isolated state
   - Concurrent execution
   - Energy profiling

2. **Service Architecture**:
   - Service registration
   - Service discovery
   - Health monitoring
   - Security context

3. **Service Communication**:
   - Message broker
   - Service gateway
   - Capability-based security
   - Energy metrics

4. **Service Deployment**:
   - Container management
   - Service scaling
   - Security context
   - Energy profiling

5. **Semantic Tags**:
   - `#[ai::tag(service_component)]` for service components
   - `#[ai::tag(service_operation)]` for service operations
   - Security context integration
   - Energy metrics tracking

## Best Practices

1. **Service Architecture**:
   - Use actor model
   - Implement service discovery
   - Monitor health
   - Track energy usage

2. **Service Communication**:
   - Use message broker
   - Implement gateway
   - Check capabilities
   - Monitor energy

3. **Service Deployment**:
   - Use containers
   - Implement scaling
   - Check security
   - Track metrics

4. **Security Context**:
   - Propagate security context
   - Check required capabilities
   - Monitor energy usage
   - Maintain audit trail

5. **Energy Profiling**:
   - Track operation energy
   - Monitor service energy
   - Optimize energy usage
   - Log energy metrics

## Quiz

1. What is the main benefit of the actor model?
   - A. Better performance
   - B. Message-based communication
   - C. Simpler implementation
   - D. Faster response times

2. How do you handle service security?
   - A. Using capabilities
   - B. Using security context
   - C. Both A and B
   - D. Neither A nor B

3. Which feature prevents service tampering?
   - A. Security context
   - B. Capability checks
   - C. Energy monitoring
   - D. All of the above

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Concurrency Model](../../reference/CONCURRENCY_MODEL.md)
- [Security Model](../../reference/SECURITY_MODEL.md)
- [Energy Profiler](../../reference/ENERGY_PROFILER.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)

## Next Steps

- [API Design & Integration](./api_design.md)
- [Monitoring & Operations](./monitoring_ops.md)
- [Advanced Topics](./advanced_topics.md) 