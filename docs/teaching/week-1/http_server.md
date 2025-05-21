---
title: "HTTP Server"
duration: "1h"
level: "intermediate"
---

# HTTP Server

> **Duration**: 1 hour
> **Goal**: Learn how to build and configure HTTP servers in Ferra

## Overview

This tutorial covers HTTP server setup, routing, middleware, and request handling in Ferra.

## 1. Server Setup (15 minutes)

```ferra
// HTTP server implementation
module http {
    data ServerConfig {
        host: String
        port: Int
        max_connections: Int
        timeout: Duration
        ssl: SSLConfig?
    }

    data SSLConfig {
        cert_path: String
        key_path: String
        protocols: List<String>
    }

    fn create_server(config: ServerConfig) -> Result<Server, ServerError> {
        // Validate configuration
        if !is_valid_config(config) {
            return Err(ServerError::InvalidConfig)
        }

        // Create server instance
        let server = Server {
            config: config,
            routes: Map::new(),
            middleware: List::new(),
            state: ServerState::new()
        }

        // Initialize server
        match server.initialize() {
            Ok(_) => return Ok(server),
            Err(e) => return Err(ServerError::InitializationFailed(e))
        }
    }

    fn start_server(server: Server) -> Result<Unit, ServerError> {
        // Start listening
        match server.listen() {
            Ok(_) => {
                // Handle incoming connections
                while true {
                    match server.accept() {
                        Ok(connection) => {
                            spawn handle_connection(connection)
                        }
                        Err(e) => {
                            log_error("Connection error: " + e.message)
                            continue
                        }
                    }
                }
            }
            Err(e) => return Err(ServerError::StartFailed(e))
        }
    }
}

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
}
```

## 2. Routing (15 minutes)

```ferra
// Routing implementation
module routing {
    data Route {
        path: String
        method: HTTPMethod
        handler: Function
        middleware: List<Function>
    }

    data HTTPMethod {
        GET
        POST
        PUT
        DELETE
        PATCH
    }

    fn add_route(server: Server, route: Route) -> Result<Unit, RouteError> {
        // Validate route
        if !is_valid_route(route) {
            return Err(RouteError::InvalidRoute)
        }

        // Add route to server
        let key = route.method.to_string() + ":" + route.path
        server.routes.insert(key, route)
        
        return Ok(Unit)
    }

    fn handle_request(server: Server, request: Request) -> Result<Response, RequestError> {
        // Find matching route
        let route = match find_route(server, request) {
            Some(r) => r,
            None => return Err(RequestError::RouteNotFound)
        }

        // Execute middleware
        for middleware in route.middleware {
            match middleware(request) {
                Ok(_) => continue,
                Err(e) => return Err(RequestError::MiddlewareFailed(e))
            }
        }

        // Execute handler
        return route.handler(request)
    }
}
```

## 3. Middleware (15 minutes)

```ferra
// Middleware implementation
module middleware {
    data MiddlewareContext {
        request: Request
        response: Response?
        state: Map<String, Any>
    }

    fn create_middleware(handler: Function) -> Function {
        return fn(context: MiddlewareContext) -> Result<MiddlewareContext, MiddlewareError> {
            // Pre-processing
            let context = match pre_process(context) {
                Ok(ctx) => ctx,
                Err(e) => return Err(MiddlewareError::PreProcessFailed(e))
            }

            // Execute handler
            let context = match handler(context) {
                Ok(ctx) => ctx,
                Err(e) => return Err(MiddlewareError::HandlerFailed(e))
            }

            // Post-processing
            return post_process(context)
        }
    }

    fn logging_middleware(context: MiddlewareContext) -> Result<MiddlewareContext, MiddlewareError> {
        // Log request
        log_info("Request: " + context.request.method + " " + context.request.path)
        
        // Process request
        let context = match process_request(context) {
            Ok(ctx) => ctx,
            Err(e) => return Err(MiddlewareError::ProcessFailed(e))
        }
        
        // Log response
        if context.response {
            log_info("Response: " + context.response.status.to_string())
        }
        
        return Ok(context)
    }
}
```

## 4. Request Handling (15 minutes)

```ferra
// Request handling implementation
module request {
    data Request {
        method: HTTPMethod
        path: String
        headers: Map<String, String>
        body: Any?
        params: Map<String, String>
        query: Map<String, String>
    }

    data Response {
        status: HTTPStatus
        headers: Map<String, String>
        body: Any?
    }

    fn handle_request(request: Request) -> Result<Response, HttpError> {
        // Energy profiling
        let start_energy = measure_energy()
        
        match request.method {
            "GET" => handle_get(request),
            "POST" => handle_post(request),
            "PUT" => handle_put(request),
            "DELETE" => handle_delete(request),
            _ => {
                let end_energy = measure_energy()
                log_energy_usage("http_error", end_energy - start_energy)
                Err(HttpError::MethodNotAllowed)
            }
        }
    }

    data HttpError {
        MethodNotAllowed
        InvalidRequest
        ServerError
    }

    fn parse_request(request: Request) -> Result<ParsedRequest, ParseError> {
        // Parse headers
        let headers = parse_headers(request.headers)
        
        // Parse body
        let body = match request.body {
            Some(b) => parse_body(b),
            None => null
        }
        
        // Parse query parameters
        let query = parse_query(request.query)
        
        return Ok(ParsedRequest {
            method: request.method,
            path: request.path,
            headers: headers,
            body: body,
            query: query
        })
    }
}
```

## Quiz

1. What's the first step in setting up an HTTP server?
   - A. Adding routes
   - B. Creating server configuration
   - C. Starting the server
   - D. Handling requests

2. How do you add middleware to a route?
   - A. Using the middleware property
   - B. Using the add_middleware function
   - C. Using the use_middleware function
   - D. Using the with_middleware function

3. What's the correct order of request processing?
   - A. Parse -> Validate -> Process -> Format
   - B. Validate -> Parse -> Process -> Format
   - C. Process -> Parse -> Validate -> Format
   - D. Format -> Parse -> Validate -> Process

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)
- [Design Diagnostics](../../reference/DESIGN_DIAGNOSTICS.md)

## Next Steps

- [JSON Processing](./json_processing.md)
- [Cross-Platform Development](./cross_platform.md)
- [Testing](./testing.md)

## Video Content

- **Duration**: 1 hour
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (5m)
  2. Server Setup (15m)
  3. Routing (15m)
  4. Middleware (15m)
  5. Request Handling (15m)
  6. Conclusion (5m)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration
- **Chapter Markers**: For easy navigation 