---
title: "REST API Basics"
duration: "1h"
level: "beginner"
---

# REST API Basics

> **Duration**: 1 hour
> **Goal**: Learn the fundamentals of building REST APIs with Ferra

## Overview

This tutorial covers the basics of REST API development, including HTTP methods, request handling, and response formatting.

## 1. HTTP Server Setup (15 minutes)

```ferra
// Basic HTTP server with energy profiling
module server {
    data ServerConfig {
        host: String
        port: Int
        energy_budget: Float
    }

    fn start_server(config: ServerConfig) -> Result<Unit, ServerError> {
        // Energy profiling
        let start_energy = measure_energy()
        
        // Initialize server
        match init_server(config) {
            Ok(server) => {
                // Start server
                match server.start() {
                    Ok(_) => {
                        let end_energy = measure_energy()
                        log_energy_usage("server_start", end_energy - start_energy)
                        Ok(Unit)
                    }
                    Err(e) => {
                        let end_energy = measure_energy()
                        log_energy_usage("server_start_error", end_energy - start_energy)
                        Err(ServerError::StartFailed(e))
                    }
                }
            }
            Err(e) => {
                let end_energy = measure_energy()
                log_energy_usage("server_init_error", end_energy - start_energy)
                Err(ServerError::InitFailed(e))
            }
        }
    }
}

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
}
```

## 2. Request Handling (15 minutes)

```ferra
// Request handler with energy profiling
module handler {
    data Request {
        method: String
        path: String
        headers: Map<String, String>
        body: String?
        energy_budget: Float
    }

    fn handle_request(request: Request) -> Result<Response, HandlerError> {
        // Energy profiling
        let start_energy = measure_energy()
        
        // Parse request
        match parse_request(request) {
            Ok(parsed) => {
                // Process request
                match process_request(parsed) {
                    Ok(response) => {
                        let end_energy = measure_energy()
                        log_energy_usage("request_handling", end_energy - start_energy)
                        Ok(response)
                    }
                    Err(e) => {
                        let end_energy = measure_energy()
                        log_energy_usage("request_handling_error", end_energy - start_energy)
                        Err(HandlerError::ProcessFailed(e))
                    }
                }
            }
            Err(e) => {
                let end_energy = measure_energy()
                log_energy_usage("request_parsing_error", end_energy - start_energy)
                Err(HandlerError::ParseFailed(e))
            }
        }
    }
}
```

## 3. Response Formatting (15 minutes)

```ferra
// Response formatter with energy profiling
module response {
    data Response {
        status: Int
        headers: Map<String, String>
        body: String?
        energy_budget: Float
    }

    fn format_response(response: Response) -> Result<String, FormatError> {
        // Energy profiling
        let start_energy = measure_energy()
        
        // Format response
        match format_http_response(response) {
            Ok(formatted) => {
                let end_energy = measure_energy()
                log_energy_usage("response_formatting", end_energy - start_energy)
                Ok(formatted)
            }
            Err(e) => {
                let end_energy = measure_energy()
                log_energy_usage("response_formatting_error", end_energy - start_energy)
                Err(FormatError::FormatFailed(e))
            }
        }
    }
}
```

## 4. Error Handling (15 minutes)

```ferra
// Error handler with energy profiling
module error {
    data ErrorResponse {
        code: Int
        message: String
        details: String?
        energy_budget: Float
    }

    fn handle_error(error: Error) -> Result<ErrorResponse, Error> {
        // Energy profiling
        let start_energy = measure_energy()
        
        // Format error
        match format_error(error) {
            Ok(formatted) => {
                let end_energy = measure_energy()
                log_energy_usage("error_handling", end_energy - start_energy)
                Ok(formatted)
            }
            Err(e) => {
                let end_energy = measure_energy()
                log_energy_usage("error_handling_error", end_energy - start_energy)
                Err(e)
            }
        }
    }
}
```

## Quiz

1. What is the purpose of HTTP methods in a REST API?
   - A. To define data types
   - B. To specify operations
   - C. To handle errors
   - D. To format responses

2. How do you handle different types of requests?
   - A. Using different functions
   - B. Using request parsing
   - C. Using middleware
   - D. All of the above

3. What is the purpose of error handling in a REST API?
   - A. To improve performance
   - B. To provide better feedback
   - C. To reduce code size
   - D. To simplify testing

## Resources

- [HTTP Server](./http_server.md)
- [Error Handling](./error_handling.md)
- [Testing](./testing.md)
- [API Design](./api_design.md)

## Next Steps

- [REST API Implementation](./rest_api.md)
- [Security](./security.md)
- [Performance](./performance.md)

## Video Content

- **Duration**: 1 hour
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (5m)
  2. HTTP Server Setup (15m)
  3. Request Handling (15m)
  4. Response Formatting (15m)
  5. Error Handling (15m)
  6. Conclusion (5m)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration
- **Chapter Markers**: For easy navigation 