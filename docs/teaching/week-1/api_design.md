---
title: "API Design"
duration: "1h"
level: "intermediate"
---

# API Design

> **Duration**: 1 hour
> **Goal**: Learn how to design and implement RESTful APIs in Ferra

## Overview

This tutorial covers REST API design principles, endpoint implementation, and best practices in Ferra.

## 1. REST API Basics (15 minutes)

```ferra
// Basic HTTP server setup
module api {
    fn start_server(port: Int) -> Result<Server, ServerError> {
        let server = Server::new(port)
        
        // Register routes
        server.get("/health", health_check)
        server.get("/api/v1/users", get_users)
        server.post("/api/v1/users", create_user)
        server.get("/api/v1/users/:id", get_user)
        server.put("/api/v1/users/:id", update_user)
        server.delete("/api/v1/users/:id", delete_user)
        
        server.start()
    }

    // Health check endpoint
    fn health_check(req: Request) -> Response {
        Response {
            status: 200,
            body: Json::object([
                ("status", Json::string("healthy")),
                ("timestamp", Json::string(now().to_string()))
            ])
        }
    }
}

// Request and Response types
data Request {
    method: String
    path: String
    params: Map<String, String>
    body: Json?
    headers: Map<String, String>
}

data Response {
    status: Int
    body: Json
    headers: Map<String, String>
}

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
}
```

## 2. Resource Endpoints (15 minutes)

```ferra
// User resource endpoints
module user_api {
    fn get_users(req: Request) -> Response {
        match db::get_all("users") {
            Ok(users) => Response {
                status: 200,
                body: Json::array(users.map(|u| u.to_json())),
                headers: Map::new()
            },
            Err(e) => Response {
                status: 500,
                body: Json::object([
                    ("error", Json::string("Failed to fetch users"))
                ]),
                headers: Map::new()
            }
        }
    }

    fn create_user(req: Request) -> Response {
        let user_data = match req.body {
            Some(body) => body,
            None => return Response {
                status: 400,
                body: Json::object([
                    ("error", Json::string("Missing request body"))
                ]),
                headers: Map::new()
            }
        }
        
        match User::from_json(user_data) {
            Ok(user) => {
                match db::insert("users", user) {
                    Ok(_) => Response {
                        status: 201,
                        body: user.to_json(),
                        headers: Map::new()
                    },
                    Err(e) => Response {
                        status: 500,
                        body: Json::object([
                            ("error", Json::string("Failed to create user"))
                        ]),
                        headers: Map::new()
                    }
                }
            },
            Err(e) => Response {
                status: 400,
                body: Json::object([
                    ("error", Json::string("Invalid user data"))
                ]),
                headers: Map::new()
            }
        }
    }
}
```

## 3. Error Handling and Validation (15 minutes)

```ferra
// API error handling
module api_errors {
    data ApiError {
        NotFound(String)
        ValidationError(String)
        DatabaseError(DatabaseError)
        AuthenticationError(String)
        AuthorizationError(String)
    }

    fn handle_error(error: ApiError) -> Response {
        match error {
            ApiError::NotFound(msg) => Response {
                status: 404,
                body: Json::object([
                    ("error", Json::string(msg))
                ]),
                headers: Map::new()
            },
            ApiError::ValidationError(msg) => Response {
                status: 400,
                body: Json::object([
                    ("error", Json::string(msg))
                ]),
                headers: Map::new()
            },
            ApiError::DatabaseError(e) => Response {
                status: 500,
                body: Json::object([
                    ("error", Json::string("Database error occurred"))
                ]),
                headers: Map::new()
            },
            ApiError::AuthenticationError(msg) => Response {
                status: 401,
                body: Json::object([
                    ("error", Json::string(msg))
                ]),
                headers: Map::new()
            },
            ApiError::AuthorizationError(msg) => Response {
                status: 403,
                body: Json::object([
                    ("error", Json::string(msg))
                ]),
                headers: Map::new()
            }
        }
    }
}

// Input validation
module validation {
    fn validate_user(user: User) -> Result<User, ValidationError> {
        if user.name.is_empty() {
            return Err(ValidationError::InvalidName)
        }
        
        if !is_valid_email(user.email) {
            return Err(ValidationError::InvalidEmail)
        }
        
        Ok(user)
    }
}
```

## 4. API Documentation and Testing (15 minutes)

```ferra
// API documentation using OpenAPI/Swagger
module api_docs {
    fn generate_openapi_spec() -> Json {
        Json::object([
            ("openapi", Json::string("3.0.0")),
            ("info", Json::object([
                ("title", Json::string("User API")),
                ("version", Json::string("1.0.0"))
            ])),
            ("paths", Json::object([
                ("/api/v1/users", Json::object([
                    ("get", Json::object([
                        ("summary", Json::string("Get all users")),
                        ("responses", Json::object([
                            ("200", Json::object([
                                ("description", Json::string("List of users"))
                            ]))
                        ]))
                    ]))
                ]))
            ]))
        ])
    }
}

// API testing utilities
module api_testing {
    fn test_endpoint(endpoint: String, method: String, body: Json?) -> Response {
        let req = Request {
            method: method,
            path: endpoint,
            params: Map::new(),
            body: body,
            headers: Map::new()
        }
        
        match method {
            "GET" => user_api::get_users(req),
            "POST" => user_api::create_user(req),
            _ => Response {
                status: 405,
                body: Json::object([
                    ("error", Json::string("Method not allowed"))
                ]),
                headers: Map::new()
            }
        }
    }
}
```

## Quiz

1. What's the correct way to handle HTTP methods in Ferra?
   - A. Using if-else statements
   - B. Using pattern matching
   - C. Using switch statements
   - D. Using method handlers

2. How do you validate API input?
   - A. Using assertions
   - B. Using Result type
   - C. Using exceptions
   - D. Using null checks

3. What's the best practice for API error handling?
   - A. Return error codes
   - B. Throw exceptions
   - C. Use Result type with custom errors
   - D. Log errors only

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [API Design Guide](../../reference/API_DESIGN_GUIDE.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [Logging](./logging.md)
- [Performance](./performance.md)
- [Security](./security.md)

## Video Content

- **Duration**: 1 hour
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (5m)
  2. REST API Basics (15m)
  3. Resource Endpoints (15m)
  4. Error Handling and Validation (15m)
  5. API Documentation and Testing (15m)
  6. Conclusion (5m)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration
- **Chapter Markers**: For easy navigation 