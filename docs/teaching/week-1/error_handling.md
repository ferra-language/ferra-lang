---
title: "Error Handling"
duration: "1h"
level: "intermediate"
---

# Error Handling

> **Duration**: 1 hour
> **Goal**: Learn how to handle errors effectively in Ferra applications

## Overview

This tutorial covers error handling patterns, custom error types, and error propagation in Ferra.

## 1. Basic Error Handling (15 minutes)

```ferra
// Basic error handling with Result type
fn divide(a: Float, b: Float) -> Result<Float, DivisionError> {
    if b == 0.0 {
        return Err(DivisionError::DivisionByZero)
    }
    Ok(a / b)
}

// Custom error type
data DivisionError {
    DivisionByZero
    Overflow
    Underflow
}

// Error handling in practice
fn calculate_average(numbers: List<Float>) -> Result<Float, DivisionError> {
    if numbers.is_empty() {
        return Err(DivisionError::DivisionByZero)
    }
    
    let sum = numbers.fold(0.0, |acc, x| acc + x)
    divide(sum, numbers.len() as Float)
}

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
}
```

## 2. Error Propagation (15 minutes)

```ferra
// Error propagation with ? operator
fn process_data(data: Json) -> Result<ProcessedData, ProcessingError> {
    let value = data.get("value")?.as_number()?
    let name = data.get("name")?.as_string()?
    
    Ok(ProcessedData {
        value: value,
        name: name
    })
}

// Custom error type with context
data ProcessingError {
    InvalidJson(JsonError)
    MissingField(String)
    InvalidType(String)
    ValidationError(ValidationError)
}

// Error conversion
impl From<JsonError> for ProcessingError {
    fn from(err: JsonError) -> ProcessingError {
        ProcessingError::InvalidJson(err)
    }
}

impl From<ValidationError> for ProcessingError {
    fn from(err: ValidationError) -> ProcessingError {
        ProcessingError::ValidationError(err)
    }
}
```

## 3. Error Recovery (15 minutes)

```ferra
// Error recovery patterns
fn fetch_data(url: String) -> Result<Data, FetchError> {
    match http::get(url) {
        Ok(response) => {
            match response.status {
                200 => Ok(response.body().parse_json()?),
                404 => Err(FetchError::NotFound),
                500 => Err(FetchError::ServerError),
                _ => Err(FetchError::Unknown)
            }
        }
        Err(e) => Err(FetchError::NetworkError(e))
    }
}

// Retry mechanism
fn fetch_with_retry(url: String, max_retries: Int) -> Result<Data, FetchError> {
    let mut retries = 0
    while retries < max_retries {
        match fetch_data(url) {
            Ok(data) => return Ok(data),
            Err(FetchError::NetworkError(_)) => {
                retries += 1
                if retries == max_retries {
                    return Err(FetchError::MaxRetriesExceeded)
                }
                sleep(1000) // Wait 1 second before retry
            }
            Err(e) => return Err(e)
        }
    }
    Err(FetchError::MaxRetriesExceeded)
}
```

## 4. Error Logging and Monitoring (15 minutes)

```ferra
// Error logging implementation
impl FetchError {
    fn log(self) {
        match self {
            FetchError::NotFound => {
                logger::warn("Resource not found")
            }
            FetchError::ServerError => {
                logger::error("Server error occurred")
            }
            FetchError::NetworkError(e) => {
                logger::error("Network error: " + e.message)
            }
            FetchError::MaxRetriesExceeded => {
                logger::error("Max retries exceeded")
            }
            FetchError::Unknown => {
                logger::error("Unknown error occurred")
            }
        }
    }
}

// Error monitoring
fn monitor_errors(errors: List<Error>) -> ErrorReport {
    let mut report = ErrorReport::new()
    
    for error in errors {
        match error {
            FetchError e => report.add_fetch_error(e),
            ProcessingError e => report.add_processing_error(e),
            ValidationError e => report.add_validation_error(e),
            _ => report.add_unknown_error(error)
        }
    }
    
    report
}

// Error report type
data ErrorReport {
    fetch_errors: Int
    processing_errors: Int
    validation_errors: Int
    unknown_errors: Int
    total_errors: Int
}
```

## Quiz

1. What's the correct way to propagate errors in Ferra?
   - A. Using try-catch blocks
   - B. Using the ? operator
   - C. Using throw statements
   - D. Using error codes

2. How do you define a custom error type?
   - A. Using `class Error`
   - B. Using `data Error`
   - C. Using `type Error`
   - D. Using `struct Error`

3. What's the best way to handle recoverable errors?
   - A. Using panic
   - B. Using Result type
   - C. Using exceptions
   - D. Using error codes

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Error Handling Guide](../../reference/ERROR_HANDLING.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [Testing](./testing.md)
- [API Design](./api_design.md)
- [Logging](./logging.md)

## Video Content

- **Duration**: 1 hour
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (5m)
  2. Basic Error Handling (15m)
  3. Error Propagation (15m)
  4. Error Recovery (15m)
  5. Error Logging and Monitoring (15m)
  6. Conclusion (5m)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration
- **Chapter Markers**: For easy navigation 