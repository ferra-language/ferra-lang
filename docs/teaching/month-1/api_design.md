---
title: "API Design & Integration"
duration: "4h"
level: "advanced"
---

# API Design & Integration

> **Duration**: 4 hours
> **Goal**: Design and implement robust APIs in Ferra applications

## Overview

This tutorial covers API design principles, REST and GraphQL implementation, versioning strategies, and documentation best practices in Ferra applications.

## 1. REST API Design (1 hour)

### 1.1. Resource Definition

```ferra
// API actor
#[ai::tag(api_component)]
actor APIActor {
    data APIState {
        resources: Map<String, Resource>,
        handlers: Map<String, Handler>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> APIState {
        return APIState {
            resources: Map::new(),
            handlers: Map::new(),
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
                scope: "api",
                audit_log: []
            }
        }
    }

    async fn handle_request(self_state: APIState, request: Request) -> (APIState, Response) {
        let start_ops = measure_ops();
        // Example: Add security/energy checks if needed
        let resource = self_state.resources.get(request.path)
        let handler = self_state.handlers.get(request.method + ":" + request.path)
        if !resource || !handler {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                "system",
                "handle_request",
                request.path,
                false,
                "Not Found"
            );
            return (APIState {
                resources: self_state.resources,
                handlers: self_state.handlers,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Response(404, "Not Found"))
        }
        let (new_state, response) = await handler(self_state, request)
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            "system",
            "handle_request",
            request.path,
            true,
            null
        );
        let updated_state = APIState {
            resources: new_state.resources,
            handlers: new_state.handlers,
            energy_metrics: new_metrics,
            security_context: new_context
        };
        return (updated_state, response)
    }
}

// Resource definition
let api = API {
    prefix: "/api/v1",
    resources: {
        users: User @resource @deterministic,
        posts: Post @resource @deterministic,
        comments: Comment @resource @deterministic
    },
    energy_budget: 0.1.joules
}

// Resource handler
actor UserResourceActor {
    data UserState {
        users: Map<String, User>,
        db: Database
    }

    async fn handle_get(self_state: UserState, request: Request) -> (UserState, Response) {
        let user = self_state.users.get(request.params.id)
        
        if !user {
            return (self_state, Response(404, "User not found"))
        }
        
        return (self_state, Response(200, user))
    }

    async fn handle_post(self_state: UserState, request: Request) -> (UserState, Response) {
        let user = User {
            id: generate_id(),
            name: request.body.name,
            email: request.body.email,
            energy_budget: 0.05.joules
        }
        
        let new_state = UserState {
            users: self_state.users.insert(user.id, user),
            db: self_state.db
        }
        
        return (new_state, Response(201, user))
    }
}
```

### 1.2. Error Handling

```ferra
// Error handling actor
actor ErrorHandlerActor {
    data ErrorState {
        errors: Map<String, ErrorHandler>
    }

    async fn handle_error(self_state: ErrorState, error: Error) -> (ErrorState, Response) {
        let handler = self_state.errors.get(error.type)
        
        let response = Response {
            status: handler.status,
            body: {
                error: error.message,
                code: error.code,
                details: error.details
            },
            energy_budget: 0.01.joules
        }
        
        return (self_state, response)
    }
}

// Error types
enum APIError {
    NotFound(String)
    ValidationError(String)
    AuthError(String)
    ServerError(String)
}

// Error handling middleware
actor ErrorMiddlewareActor {
    data MiddlewareState {
        handler: ActorRef<ErrorHandlerActor>
    }

    async fn handle_request(self_state: MiddlewareState, request: Request) -> (MiddlewareState, Response) {
        try {
            return (self_state, await handle_route(request))
        } catch error {
            let (_, response) = await self_state.handler.ask(HandleError(error))
            return (self_state, response)
        }
    }
}
```

## 2. GraphQL Integration (1 hour)

### 2.1. Schema Definition

```ferra
// GraphQL actor
actor GraphQLActor {
    data GraphQLState {
        schema: Schema,
        resolvers: Map<String, Resolver>
    }

    async fn handle_query(self_state: GraphQLState, request: Request) -> (GraphQLState, Response) {
        let query = request.body.query
        let variables = request.body.variables
        
        let result = await execute_query(self_state.schema, query, variables)
        
        return (self_state, Response(200, result))
    }
}

// Schema definition
let schema = Schema {
    types: {
        User: {
            id: ID!,
            name: String!,
            email: String!,
            posts: [Post!]!
        },
        Post: {
            id: ID!,
            title: String!,
            content: String!,
            author: User!
        }
    },
    queries: {
        user: (id: ID!): User
        post: (id: ID!): Post
    },
    mutations: {
        createUser: (input: UserInput!): User!
        createPost: (input: PostInput!): Post!
    },
    energy_budget: 0.1.joules
}

// Resolver implementation
actor UserResolverActor {
    data ResolverState {
        users: Map<String, User>
    }

    async fn resolve_user(self_state: ResolverState, args: Map<String, Any>) -> (ResolverState, User) {
        let user = self_state.users.get(args.get("id"))
        return (self_state, user)
    }

    async fn resolve_posts(self_state: ResolverState, user: User) -> (ResolverState, List<Post>) {
        let posts = user.posts
        return (self_state, posts)
    }
}
```

### 2.2. Data Loaders

```ferra
// Data loader actor
actor DataLoaderActor {
    data LoaderState {
        cache: Map<String, Any>,
        batch_size: Int
    }

    async fn handle_load(self_state: LoaderState, keys: List<String>) -> (LoaderState, Map<String, Any>) {
        let uncached = keys.filter { k => !self_state.cache.contains(k) }
        
        if uncached.length > 0 {
            let batch = await load_batch(uncached)
            let new_cache = self_state.cache.merge(batch)
            
            let new_state = LoaderState {
                cache: new_cache,
                batch_size: self_state.batch_size
            }
            
            return (new_state, batch)
        }
        
        return (self_state, self_state.cache)
    }
}

// Batch loading
async fn load_batch(keys: List<String>) -> Map<String, Any> {
    return await db.query("""
        SELECT * FROM users 
        WHERE id IN ($1)
    """, [keys])
}
```

## 3. API Versioning (1 hour)

### 3.1. Versioning Strategies

```ferra
// Version manager actor
actor VersionManagerActor {
    data VersionState {
        versions: Map<String, Version>,
        current: String
    }

    async fn handle_request(self_state: VersionState, request: Request) -> (VersionState, Response) {
        let version = request.headers.get("API-Version") || self_state.current
        let handler = self_state.versions.get(version)
        
        if !handler {
            return (self_state, Response(400, "Unsupported API version"))
        }
        
        return (self_state, await handler(request))
    }
}

// Version definition
let versions = {
    v1: {
        prefix: "/api/v1",
        resources: {
            users: User @resource @deterministic,
            posts: Post @resource @deterministic
        }
    },
    v2: {
        prefix: "/api/v2",
        resources: {
            users: UserV2 @resource @deterministic,
            posts: PostV2 @resource @deterministic
        }
    }
}

// Version migration
actor MigrationActor {
    data MigrationState {
        migrations: Map<String, Migration>
    }

    async fn handle_migrate(self_state: MigrationState, request: MigrationRequest) -> (MigrationState, Response) {
        let migration = self_state.migrations.get(request.version)
        
        let result = await migration.apply(request.data)
        
        return (self_state, Response(200, result))
    }
}
```

### 3.2. Backward Compatibility

```ferra
// Compatibility layer actor
actor CompatibilityActor {
    data CompatibilityState {
        adapters: Map<String, Adapter>
    }

    async fn handle_adapt(self_state: CompatibilityState, request: Request) -> (CompatibilityState, Response) {
        let adapter = self_state.adapters.get(request.version)
        
        let adapted = await adapter.adapt(request.data)
        
        return (self_state, Response(200, adapted))
    }
}

// Data adapter
fn adapt_user_v1_to_v2(user: UserV1) -> UserV2 {
    return UserV2 {
        id: user.id,
        name: user.name,
        email: user.email,
        profile: {
            bio: user.bio,
            avatar: user.avatar
        },
        energy_budget: 0.05.joules
    }
}
```

## 4. API Documentation (1 hour)

### 4.1. OpenAPI/Swagger

```ferra
// Documentation actor
actor DocumentationActor {
    data DocState {
        spec: OpenAPISpec,
        examples: Map<String, Example>
    }

    async fn handle_docs(self_state: DocState, request: Request) -> (DocState, Response) {
        let path = request.path
        let method = request.method
        
        let endpoint = self_state.spec.paths.get(path).get(method)
        let example = self_state.examples.get(path + ":" + method)
        
        let docs = {
            endpoint: endpoint,
            example: example,
            energy_budget: 0.01.joules
        }
        
        return (self_state, Response(200, docs))
    }
}

// OpenAPI specification
let spec = OpenAPI {
    info: {
        title: "My API",
        version: "1.0.0",
        description: "API documentation"
    },
    paths: {
        "/users": {
            get: {
                summary: "List users",
                responses: {
                    "200": {
                        description: "List of users",
                        content: {
                            "application/json": {
                                schema: User
                            }
                        }
                    }
                }
            }
        }
    },
    energy_budget: 0.1.joules
}
```

### 4.2. Interactive Documentation

```ferra
// Interactive docs actor
actor InteractiveDocsActor {
    data InteractiveState {
        ui: UI,
        spec: OpenAPISpec
    }

    async fn handle_ui(self_state: InteractiveState, request: Request) -> (InteractiveState, Response) {
        let ui = self_state.ui.render(self_state.spec)
        
        return (self_state, Response(200, ui))
    }
}

// UI components
let components = {
    endpoint: {
        method: String,
        path: String,
        description: String,
        parameters: List<Parameter>,
        responses: Map<String, Response>
    },
    example: {
        request: String,
        response: String
    },
    try_it: {
        url: String,
        method: String,
        headers: Map<String, String>,
        body: String
    }
}
```

## Quiz

1. What is the main benefit of using actors for API design?
   - A. Better performance
   - B. Deterministic execution
   - C. Simpler implementation
   - D. Faster response times

2. How do you handle API versioning in Ferra?
   - A. URL versioning
   - B. Header versioning
   - C. Both A and B
   - D. Neither A nor B

3. Which documentation format is used for Ferra APIs?
   - A. OpenAPI
   - B. GraphQL Schema
   - C. Both A and B
   - D. Neither A nor B

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [API Design Guidelines](../../reference/API_DESIGN_GUIDELINES.md)
- [GraphQL Specification](../../reference/GRAPHQL_SPEC.md)
- [OpenAPI Specification](../../reference/OPENAPI_SPEC.md)
- [Energy Profiler](../../reference/ENERGY_PROFILER.md)

## Next Steps

- [Monitoring & Operations](./monitoring_ops.md)
- [Advanced Topics](./advanced_topics.md)
- [Project Structure](./project_structure.md) 