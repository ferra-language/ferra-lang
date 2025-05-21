---
title: "Logging"
duration: "1h"
level: "intermediate"
---

# Logging

> **Duration**: 1 hour
> **Goal**: Learn how to implement effective logging in Ferra applications

## Overview

This tutorial covers logging fundamentals, structured logging, log levels, and best practices in Ferra.

## 1. Basic Logging (15 minutes)

```ferra
// Basic logging setup
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
    }

    fn log(level: LogLevel, message: String, context: Map<String, Any>?) {
        let entry = LogEntry {
            level: level,
            message: message,
            timestamp: now(),
            context: context.unwrap_or(Map::new())
        }
        
        match level {
            LogLevel::Debug => debug(entry),
            LogLevel::Info => info(entry),
            LogLevel::Warning => warning(entry),
            LogLevel::Error => error(entry),
            LogLevel::Critical => critical(entry)
        }
    }

    // Convenience functions
    fn debug(message: String, context: Map<String, Any>?) {
        log(LogLevel::Debug, message, context)
    }

    fn info(message: String, context: Map<String, Any>?) {
        log(LogLevel::Info, message, context)
    }

    fn warning(message: String, context: Map<String, Any>?) {
        log(LogLevel::Warning, message, context)
    }

    fn error(message: String, context: Map<String, Any>?) {
        log(LogLevel::Error, message, context)
    }

    fn critical(message: String, context: Map<String, Any>?) {
        log(LogLevel::Critical, message, context)
    }
}

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
}
```

## 2. Structured Logging (15 minutes)

```ferra
// Structured logging implementation
module structured_logging {
    data StructuredLog {
        level: LogLevel
        message: String
        timestamp: Time
        trace_id: String?
        span_id: String?
        context: Map<String, Any>
        metadata: Map<String, Any>
    }

    fn log_structured(entry: StructuredLog) {
        let json = Json::object([
            ("level", Json::string(entry.level.to_string())),
            ("message", Json::string(entry.message)),
            ("timestamp", Json::string(entry.timestamp.to_iso8601())),
            ("trace_id", entry.trace_id.map(|id| Json::string(id))),
            ("span_id", entry.span_id.map(|id| Json::string(id))),
            ("context", Json::object(entry.context.map(|k, v| (k, v.to_json())))),
            ("metadata", Json::object(entry.metadata.map(|k, v| (k, v.to_json()))))
        ])
        
        write_log(json)
    }

    fn with_context(context: Map<String, Any>) -> Function {
        return |message: String, level: LogLevel| {
            log_structured(StructuredLog {
                level: level,
                message: message,
                timestamp: now(),
                trace_id: get_current_trace_id(),
                span_id: get_current_span_id(),
                context: context,
                metadata: Map::new()
            })
        }
    }
}
```

## 3. Log Handlers and Formatters (15 minutes)

```ferra
// Log handlers and formatters
module log_handlers {
    data LogHandler {
        format: Function
        write: Function
        filter: Function?
    }

    // Console handler
    fn create_console_handler() -> LogHandler {
        return LogHandler {
            format: format_console,
            write: write_console,
            filter: null
        }
    }

    // File handler
    fn create_file_handler(path: String) -> LogHandler {
        return LogHandler {
            format: format_json,
            write: |entry| write_file(path, entry),
            filter: null
        }
    }

    // JSON formatter
    fn format_json(entry: LogEntry) -> String {
        return Json::stringify(entry.to_json())
    }

    // Console formatter
    fn format_console(entry: LogEntry) -> String {
        return format(
            "[{}] {} - {} - {}",
            entry.timestamp.to_string(),
            entry.level.to_string(),
            entry.message,
            entry.context.to_string()
        )
    }
}
```

## 4. Log Management and Best Practices (15 minutes)

```ferra
// Log management and configuration
module log_management {
    data LogConfig {
        level: LogLevel
        handlers: List<LogHandler>
        format: String
        rotation: RotationConfig?
    }

    data RotationConfig {
        max_size: Int
        max_files: Int
        compress: Bool
    }

    fn configure_logging(config: LogConfig) {
        set_log_level(config.level)
        set_handlers(config.handlers)
        set_format(config.format)
        
        if let Some(rotation) = config.rotation {
            configure_rotation(rotation)
        }
    }

    // Log cleanup
    fn cleanup_logs(retention_days: Int) {
        let cutoff = now() - Duration::days(retention_days)
        
        for file in list_log_files() {
            if file.modified_time < cutoff {
                delete_log_file(file.path)
            }
        }
    }

    // Log analysis
    fn analyze_logs(start_time: Time, end_time: Time) -> LogAnalysis {
        let entries = read_logs(start_time, end_time)
        
        return LogAnalysis {
            total_entries: entries.size(),
            error_count: entries.filter(|e| e.level == LogLevel::Error).size(),
            warning_count: entries.filter(|e| e.level == LogLevel::Warning).size(),
            average_response_time: calculate_avg_response_time(entries),
            top_errors: find_top_errors(entries)
        }
    }
}
```

## Quiz

1. What's the recommended way to handle log levels in Ferra?
   - A. Using integers
   - B. Using enums
   - C. Using strings
   - D. Using constants

2. How should you structure log entries?
   - A. Plain text
   - B. JSON format
   - C. CSV format
   - D. Binary format

3. What's the best practice for log rotation?
   - A. Never rotate logs
   - B. Rotate based on time only
   - C. Rotate based on size and time
   - D. Rotate based on log level

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Logging Guide](../../reference/LOGGING_GUIDE.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [Performance](./performance.md)
- [Security](./security.md)
- [Monitoring](./monitoring.md)

## Video Content

- **Duration**: 1 hour
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (5m)
  2. Basic Logging (15m)
  3. Structured Logging (15m)
  4. Log Handlers and Formatters (15m)
  5. Log Management and Best Practices (15m)
  6. Conclusion (5m)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration
- **Chapter Markers**: For easy navigation 