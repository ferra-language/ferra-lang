---
title: "Monitoring"
duration: "1h"
level: "intermediate"
---

# Monitoring

> **Duration**: 1 hour
> **Goal**: Learn how to implement comprehensive monitoring and observability in Ferra applications

## Overview

This tutorial covers metrics collection, logging, tracing, and alerting in Ferra applications.

## 1. Metrics Collection (15 minutes)

```ferra
// Metrics collection implementation
module metrics {
    data Metric {
        name: String
        value: Float
        type: MetricType
        labels: Map<String, String>
        timestamp: Time
    }

    data MetricType {
        Counter
        Gauge
        Histogram
        Summary
    }

    fn collect_metrics() -> List<Metric> {
        let metrics = []
        
        // System metrics
        metrics.append(collect_system_metrics())
        
        // Application metrics
        metrics.append(collect_application_metrics())
        
        // Custom metrics
        metrics.append(collect_custom_metrics())
        
        return metrics
    }

    fn collect_system_metrics() -> List<Metric> {
        return [
            Metric {
                name: "cpu_usage",
                value: get_cpu_usage(),
                type: MetricType::Gauge,
                labels: Map::new(),
                timestamp: now()
            },
            Metric {
                name: "memory_usage",
                value: get_memory_usage(),
                type: MetricType::Gauge,
                labels: Map::new(),
                timestamp: now()
            },
            Metric {
                name: "energy_consumption",
                value: get_energy_usage(),
                type: MetricType::Counter,
                labels: Map::new(),
                timestamp: now()
            }
        ]
    }
}

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
}
```

## 2. Logging (15 minutes)

```ferra
// Logging implementation
module logging {
    data LogLevel {
        Debug
        Info
        Warning
        Error
        Critical
    }

    data LogEntry {
        level: LogLevel
        message: String
        timestamp: Time
        context: Map<String, Any>
        trace_id: String?
    }

    fn log(level: LogLevel, message: String, context: Map<String, Any>) {
        let entry = LogEntry {
            level: level,
            message: message,
            timestamp: now(),
            context: context,
            trace_id: get_current_trace_id()
        }
        
        // Write to appropriate sinks
        write_to_console(entry)
        write_to_file(entry)
        if should_send_to_remote(entry) {
            send_to_remote(entry)
        }
    }

    fn write_to_console(entry: LogEntry) {
        let formatted = format_log_entry(entry)
        print(formatted)
    }

    fn write_to_file(entry: LogEntry) {
        let path = get_log_file_path()
        append_to_file(path, format_log_entry(entry))
    }
}
```

## 3. Tracing (15 minutes)

```ferra
// Tracing implementation
module tracing {
    data Span {
        id: String
        parent_id: String?
        name: String
        start_time: Time
        end_time: Time?
        attributes: Map<String, Any>
        events: List<Event>
    }

    data Event {
        name: String
        timestamp: Time
        attributes: Map<String, Any>
    }

    fn start_span(name: String) -> Span {
        return Span {
            id: generate_span_id(),
            parent_id: get_current_span_id(),
            name: name,
            start_time: now(),
            end_time: null,
            attributes: Map::new(),
            events: []
        }
    }

    fn end_span(span: Span) -> Span {
        return Span {
            id: span.id,
            parent_id: span.parent_id,
            name: span.name,
            start_time: span.start_time,
            end_time: now(),
            attributes: span.attributes,
            events: span.events
        }
    }

    fn add_event(span: Span, name: String, attributes: Map<String, Any>) -> Span {
        let event = Event {
            name: name,
            timestamp: now(),
            attributes: attributes
        }
        
        return Span {
            id: span.id,
            parent_id: span.parent_id,
            name: span.name,
            start_time: span.start_time,
            end_time: span.end_time,
            attributes: span.attributes,
            events: span.events.append(event)
        }
    }
}
```

## 4. Alerting (15 minutes)

```ferra
// Alerting implementation
module alerting {
    data Alert {
        name: String
        severity: AlertSeverity
        condition: AlertCondition
        message: String
        timestamp: Time
        status: AlertStatus
    }

    data AlertSeverity {
        Info
        Warning
        Critical
    }

    data AlertCondition {
        metric: String
        operator: String
        threshold: Float
        duration: Duration
    }

    fn check_alerts(metrics: List<Metric>) -> List<Alert> {
        let alerts = []
        
        for condition in get_alert_conditions() {
            if is_condition_triggered(condition, metrics) {
                alerts.append(create_alert(condition))
            }
        }
        
        return alerts
    }

    fn create_alert(condition: AlertCondition) -> Alert {
        return Alert {
            name: format_alert_name(condition),
            severity: determine_severity(condition),
            condition: condition,
            message: format_alert_message(condition),
            timestamp: now(),
            status: AlertStatus::Firing
        }
    }

    fn notify_alert(alert: Alert) {
        // Send notifications through configured channels
        for channel in get_notification_channels() {
            send_notification(channel, alert)
        }
    }
}
```

## Quiz

1. What's the best way to collect application metrics?
   - A. Using print statements
   - B. Using a metrics collection framework
   - C. Using debug logs
   - D. Using system commands

2. How should you implement distributed tracing?
   - A. Using local logs only
   - B. Using spans and trace IDs
   - C. Using debug prints
   - D. Using system metrics

3. What's the recommended approach for alerting?
   - A. Alert on every error
   - B. Use severity levels and conditions
   - C. Ignore non-critical issues
   - D. Alert only on system failures

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Energy Profiler](../../reference/ENERGY_PROFILER.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [Testing](./testing.md)
- [Deployment](./deployment.md)
- [Security](./security.md)

## Video Content

- **Duration**: 1 hour
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (5m)
  2. Metrics Collection (15m)
  3. Logging (15m)
  4. Tracing (15m)
  5. Alerting (15m)
  6. Conclusion (5m)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration
- **Chapter Markers**: For easy navigation 