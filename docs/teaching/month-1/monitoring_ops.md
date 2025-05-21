---
title: "Monitoring & Operations"
duration: "4h"
level: "advanced"
---

# Monitoring & Operations

> **Duration**: 4 hours
> **Goal**: Implement comprehensive monitoring and operational practices in Ferra applications

## Overview

This tutorial covers application monitoring, logging, metrics collection, deployment strategies, and incident response in Ferra applications.

## 1. Application Monitoring (1 hour)

### 1.1. Health Checks

```ferra
// Health check actor
#[ai::tag(monitoring_component)]
actor HealthCheckActor {
    data HealthState {
        checks: Map<String, Check>,
        status: Map<String, Status>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> HealthState {
        return HealthState {
            checks: Map::new(),
            status: Map::new(),
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
                scope: "health_check",
                audit_log: []
            }
        }
    }

    async fn handle_check(self_state: HealthState, request: CheckRequest) -> (HealthState, Response) {
        let start_ops = measure_ops();
        // Example: Add security/energy checks if needed
        let check = self_state.checks.get(request.name)
        let result = await check.run()
        let new_status = self_state.status.insert(request.name, result)
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            "system",
            "handle_check",
            request.name,
            true,
            null
        );
        let new_state = HealthState {
            checks: self_state.checks,
            status: new_status,
            energy_metrics: new_metrics,
            security_context: new_context
        };
        return (new_state, Response(200, result))
    }
}

// Health check definition
let health = HealthCheck {
    checks: {
        database: {
            type: .database,
            timeout: 5.seconds,
            energy_budget: 0.1.joules,
            deterministic: true
        },
        redis: {
            type: .redis,
            timeout: 3.seconds,
            energy_budget: 0.05.joules,
            deterministic: true
        },
        api: {
            type: .http,
            endpoint: "/api/health",
            timeout: 2.seconds,
            energy_budget: 0.02.joules,
            deterministic: true
        }
    },
    actor: true
}
```

### 1.2. Performance Monitoring

```ferra
// Performance monitor actor
actor PerformanceMonitorActor {
    data MonitorState {
        metrics: Map<String, Metric>,
        thresholds: Map<String, Threshold>
    }

    async fn handle_metric(self_state: MonitorState, metric: Metric) -> (MonitorState, Unit) {
        let threshold = self_state.thresholds.get(metric.name)
        
        if threshold && metric.value > threshold.value {
            await alert(metric)
        }
        
        let new_metrics = self_state.metrics.insert(metric.name, metric)
        
        let new_state = MonitorState {
            metrics: new_metrics,
            thresholds: self_state.thresholds
        }
        
        return (new_state, Unit)
    }
}

// Metric collection
actor MetricCollectorActor {
    data CollectorState {
        metrics: List<Metric>,
        energy_budget: Float
    }

    async fn handle_collect(self_state: CollectorState) -> (CollectorState, List<Metric>) {
        let metrics = [
            Metric {
                name: "cpu_usage",
                value: get_cpu_usage(),
                timestamp: now(),
                energy_budget: 0.01.joules
            },
            Metric {
                name: "memory_usage",
                value: get_memory_usage(),
                timestamp: now(),
                energy_budget: 0.01.joules
            },
            Metric {
                name: "response_time",
                value: get_avg_response_time(),
                timestamp: now(),
                energy_budget: 0.01.joules
            }
        ]
        
        let new_state = CollectorState {
            metrics: self_state.metrics.append(metrics),
            energy_budget: self_state.energy_budget
        }
        
        return (new_state, metrics)
    }
}
```

## 2. Logging and Metrics (1 hour)

### 2.1. Structured Logging

```ferra
// Logger actor
actor LoggerActor {
    data LoggerState {
        logs: List<Log>,
        storage: Storage
    }

    async fn handle_log(self_state: LoggerState, log: Log) -> (LoggerState, Unit) {
        let structured_log = Log {
            timestamp: now(),
            level: log.level,
            message: log.message,
            context: log.context,
            energy_budget: 0.02.joules,
            deterministic: true
        }
        
        let new_state = LoggerState {
            logs: self_state.logs.append(structured_log),
            storage: self_state.storage
        }
        
        await new_state.storage.store(structured_log)
        return (new_state, Unit)
    }
}

// Log levels
enum LogLevel {
    Debug
    Info
    Warning
    Error
    Critical
}

// Log context
type LogContext = {
    request_id: String
    user_id: String?
    action: String
    duration: Duration
    energy_used: Float
}
```

### 2.2. Metrics Collection

```ferra
// Metrics actor
actor MetricsActor {
    data MetricsState {
        counters: Map<String, Int>,
        gauges: Map<String, Float>,
        histograms: Map<String, Histogram>
    }

    async fn handle_increment(self_state: MetricsState, name: String) -> (MetricsState, Unit) {
        let current = self_state.counters.get(name) || 0
        let new_count = current + 1
        
        let new_state = MetricsState {
            counters: self_state.counters.insert(name, new_count),
            gauges: self_state.gauges,
            histograms: self_state.histograms
        }
        
        return (new_state, Unit)
    }

    async fn handle_gauge(self_state: MetricsState, name: String, value: Float) -> (MetricsState, Unit) {
        let new_state = MetricsState {
            counters: self_state.counters,
            gauges: self_state.gauges.insert(name, value),
            histograms: self_state.histograms
        }
        
        return (new_state, Unit)
    }
}

// Metric types
type Metric = {
    name: String
    value: Float
    timestamp: DateTime
    labels: Map<String, String>
    energy_budget: Float
}

// Histogram
type Histogram = {
    buckets: Map<Float, Int>
    sum: Float
    count: Int
    energy_budget: Float
}
```

## 3. Deployment Strategies (1 hour)

### 3.1. Blue-Green Deployment

```ferra
// Deployment actor
actor DeploymentActor {
    data DeploymentState {
        versions: Map<String, Version>,
        current: String,
        next: String?
    }

    async fn handle_deploy(self_state: DeploymentState, version: Version) -> (DeploymentState, Result<Unit>) {
        let new_state = DeploymentState {
            versions: self_state.versions.insert(version.id, version),
            current: self_state.current,
            next: version.id
        }
        
        let result = await deploy_version(version)
        
        if result.is_success() {
            return (new_state, Unit)
        } else {
            return (self_state, Error("Deployment failed"))
        }
    }

    async fn handle_switch(self_state: DeploymentState) -> (DeploymentState, Result<Unit>) {
        if !self_state.next {
            return (self_state, Error("No next version to switch to"))
        }
        
        let result = await switch_traffic(self_state.next)
        
        if result.is_success() {
            return (DeploymentState {
                versions: self_state.versions,
                current: self_state.next,
                next: null
            }, Unit)
        } else {
            return (self_state, Error("Switch failed"))
        }
    }
}

// Version definition
type Version = {
    id: String
    image: String
    config: Map<String, Any>
    energy_budget: Float
    deterministic: Bool
}
```

### 3.2. Canary Deployment

```ferra
// Canary actor
actor CanaryActor {
    data CanaryState {
        version: Version
        percentage: Float
        metrics: Map<String, Metric>
    }

    async fn handle_rollout(self_state: CanaryState, percentage: Float) -> (CanaryState, Result<Unit>) {
        let new_state = CanaryState {
            version: self_state.version,
            percentage: percentage,
            metrics: self_state.metrics
        }
        
        let result = await update_traffic_split(percentage)
        
        if result.is_success() {
            return (new_state, Unit)
        } else {
            return (self_state, Error("Rollout failed"))
        }
    }

    async fn handle_metrics(self_state: CanaryState, metrics: Map<String, Metric>) -> (CanaryState, Bool) {
        let new_state = CanaryState {
            version: self_state.version,
            percentage: self_state.percentage,
            metrics: metrics
        }
        
        let is_healthy = check_metrics_health(metrics)
        return (new_state, is_healthy)
    }
}

// Traffic split
fn update_traffic_split(percentage: Float) -> Result<Unit> {
    return update_load_balancer {
        version: canary.version,
        weight: percentage,
        energy_budget: 0.05.joules
    }
}
```

## 4. Incident Response (1 hour)

### 4.1. Alerting

```ferra
// Alert actor
actor AlertActor {
    data AlertState {
        rules: Map<String, AlertRule>,
        notifications: List<Notification>
    }

    async fn handle_alert(self_state: AlertState, alert: Alert) -> (AlertState, Unit) {
        let rule = self_state.rules.get(alert.type)
        
        if rule && rule.should_trigger(alert) {
            let notification = Notification {
                alert: alert,
                channels: rule.channels,
                energy_budget: 0.02.joules
            }
            
            let new_state = AlertState {
                rules: self_state.rules,
                notifications: self_state.notifications.append(notification)
            }
            
            await send_notification(notification)
            return (new_state, Unit)
        }
        
        return (self_state, Unit)
    }
}

// Alert types
enum AlertSeverity {
    Info
    Warning
    Critical
}

type Alert = {
    type: String
    severity: AlertSeverity
    message: String
    context: Map<String, Any>
    timestamp: DateTime
    energy_budget: Float
}
```

### 4.2. Incident Management

```ferra
// Incident actor
actor IncidentActor {
    data IncidentState {
        incidents: Map<String, Incident>,
        responders: Map<String, Responder>
    }

    async fn handle_incident(self_state: IncidentState, incident: Incident) -> (IncidentState, Unit) {
        let responder = self_state.responders.get(incident.type)
        
        if responder {
            await responder.notify(incident)
        }
        
        let new_state = IncidentState {
            incidents: self_state.incidents.insert(incident.id, incident),
            responders: self_state.responders
        }
        
        return (new_state, Unit)
    }

    async fn handle_resolve(self_state: IncidentState, incident_id: String) -> (IncidentState, Unit) {
        let incident = self_state.incidents.get(incident_id)
        
        if incident {
            let resolved = Incident {
                id: incident.id,
                type: incident.type,
                status: .resolved,
                resolution: incident.resolution,
                energy_budget: 0.05.joules
            }
            
            let new_state = IncidentState {
                incidents: self_state.incidents.insert(incident_id, resolved),
                responders: self_state.responders
            }
            
            return (new_state, Unit)
        }
        
        return (self_state, Unit)
    }
}

// Incident types
enum IncidentStatus {
    Open
    Investigating
    Resolved
}

type Incident = {
    id: String
    type: String
    status: IncidentStatus
    description: String
    impact: String
    resolution: String?
    energy_budget: Float
}
```

## Quiz

1. What is the main benefit of using actors for monitoring?
   - A. Better performance
   - B. Deterministic execution
   - C. Simpler implementation
   - D. Faster response times

2. How do you handle metrics collection in Ferra?
   - A. Using actors
   - B. Using callbacks
   - C. Both A and B
   - D. Neither A nor B

3. Which deployment strategy is best for zero-downtime updates?
   - A. Blue-Green
   - B. Canary
   - C. Both A and B
   - D. Neither A nor B

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Monitoring Guidelines](../../reference/MONITORING_GUIDELINES.md)
- [Deployment Strategies](../../reference/DEPLOYMENT_STRATEGIES.md)
- [Incident Response](../../reference/INCIDENT_RESPONSE.md)
- [Energy Profiler](../../reference/ENERGY_PROFILER.md)

## Next Steps

- [Advanced Topics](./advanced_topics.md)
- [Project Structure](./project_structure.md)
- [Best Practices](./best_practices.md) 