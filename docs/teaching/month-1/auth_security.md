---
title: "Authentication & Security"
duration: "4h"
level: "advanced"
---

# Authentication & Security

> **Duration**: 4 hours
> **Goal**: Implement secure authentication and authorization in Ferra applications

## Overview

This tutorial covers authentication, authorization, and security best practices in Ferra applications, focusing on the capability-based security model and semantic tags.

## 1. Authentication (1 hour)

### 1.1. User Authentication

```ferra
// Authentication actor with semantic tags
#[ai::tag(auth_component)]
actor AuthActor {
    data AuthState {
        tokens: Map<String, Token>,
        users: Map<String, User>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> AuthState {
        return AuthState {
            tokens: Map::new(),
            users: Map::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "auth_management"
            }
        }
    }

    async fn handle_login(self_state: AuthState, request: LoginRequest) -> (AuthState, Result<Token>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["auth:login"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "login",
                request.email,
                false,
                "Missing required capabilities"
            );
            return (AuthState {
                tokens: self_state.tokens,
                users: self_state.users,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let login_energy_cost = calculate_energy_cost(0.05.joules, self_state.energy_metrics);
        if self_state.energy_budget < login_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "login",
                request.email,
                false,
                "Insufficient energy budget"
            );
            return (AuthState {
                tokens: self_state.tokens,
                users: self_state.users,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let user = self_state.users.get(request.email);
        
        if !user || !verify_password(request.password, user.password_hash) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "login",
                request.email,
                false,
                "Invalid credentials"
            );
            return (AuthState {
                tokens: self_state.tokens,
                users: self_state.users,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Invalid credentials"));
        }
        
        let token = generate_token {
            sub: user.id,
            capabilities: user.capabilities,
            scope: user.scope,
            energy_budget: login_energy_cost,
            deterministic: true
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "login",
            request.email,
            true,
            null
        );
        
        return (AuthState {
            tokens: self_state.tokens.insert(token.id, token),
            users: self_state.users,
            energy_metrics: new_metrics,
            security_context: new_context
        }, token);
    }
}

// Password hashing with energy budget
#[ai::tag(auth_operation)]
fn hash_password(password: String) -> String {
    return bcrypt.hash(password, {
        rounds: 12,
        salt: generate_salt(),
        energy_budget: 0.1.joules
    });
}
```

### 1.2. OAuth Integration

```ferra
// OAuth actor with semantic tags
#[ai::tag(auth_component)]
actor OAuthActor {
    data OAuthState {
        providers: Map<String, OAuthProvider>,
        sessions: Map<String, OAuthSession>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init(providers: Map<String, OAuthProvider>) -> OAuthState {
        return OAuthState {
            providers: providers,
            sessions: Map::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "oauth_management"
            }
        }
    }

    async fn handle_oauth_login(self_state: OAuthState, request: OAuthRequest) -> (OAuthState, Result<User>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["auth:oauth"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "oauth_login",
                request.provider,
                false,
                "Missing required capabilities"
            );
            return (OAuthState {
                providers: self_state.providers,
                sessions: self_state.sessions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let oauth_energy_cost = calculate_energy_cost(0.05.joules, self_state.energy_metrics);
        if self_state.energy_budget < oauth_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "oauth_login",
                request.provider,
                false,
                "Insufficient energy budget"
            );
            return (OAuthState {
                providers: self_state.providers,
                sessions: self_state.sessions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let provider = self_state.providers.get(request.provider);
        
        if !provider {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "oauth_login",
                request.provider,
                false,
                "Provider not configured"
            );
            return (OAuthState {
                providers: self_state.providers,
                sessions: self_state.sessions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Provider not configured"));
        }
        
        let token = await provider.exchange_code(request.code);
        let profile = await provider.get_profile(token);
        
        let user = User {
            email: profile.email,
            name: profile.name,
            provider: request.provider,
            provider_id: profile.id,
            capabilities: profile.capabilities,
            scope: profile.scope,
            energy_budget: oauth_energy_cost
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "oauth_login",
            request.provider,
            true,
            null
        );
        
        return (OAuthState {
            providers: self_state.providers,
            sessions: self_state.sessions.insert(profile.id, OAuthSession { token, profile }),
            energy_metrics: new_metrics,
            security_context: new_context
        }, user);
    }
}
```

## 2. Authorization (1 hour)

### 2.1. Capability-Based Security

```ferra
// Capability actor with semantic tags
#[ai::tag(auth_component)]
actor CapabilityActor {
    data CapabilityState {
        capabilities: Map<String, Set<Capability>>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> CapabilityState {
        return CapabilityState {
            capabilities: Map::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext {
                principal: "system",
                capabilities: Set::new(),
                scope: "capability_management"
            }
        }
    }

    async fn handle_check_capability(self_state: CapabilityState, request: CapabilityRequest) -> (CapabilityState, Bool) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["auth:check_capability"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "check_capability",
                request.capability,
                false,
                "Missing required capabilities"
            );
            return (CapabilityState {
                capabilities: self_state.capabilities,
                energy_metrics: new_metrics,
                security_context: new_context
            }, false);
        }

        // Check energy budget
        let check_energy_cost = calculate_energy_cost(0.01.joules, self_state.energy_metrics);
        if self_state.energy_budget < check_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "check_capability",
                request.capability,
                false,
                "Insufficient energy budget"
            );
            return (CapabilityState {
                capabilities: self_state.capabilities,
                energy_metrics: new_metrics,
                security_context: new_context
            }, false);
        }

        let user_caps = self_state.capabilities.get(request.user_id);
        
        let has_capability = user_caps && user_caps.contains(request.capability);
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "check_capability",
            request.capability,
            has_capability,
            null
        );
        
        return (CapabilityState {
            capabilities: self_state.capabilities,
            energy_metrics: new_metrics,
            security_context: new_context
        }, has_capability);
    }
}

// Secure file operation with actor
#[ai::tag(auth_component)]
actor FileAccessActor {
    data FileState {
        capabilities: ActorRef<CapabilityActor>,
        files: Map<String, File>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    async fn handle_read_file(self_state: FileState, request: ReadRequest) -> (FileState, Result<File>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["fs:read"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "read_file",
                request.path,
                false,
                "Missing required capabilities"
            );
            return (FileState {
                capabilities: self_state.capabilities,
                files: self_state.files,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let read_energy_cost = calculate_energy_cost(0.02.joules, self_state.energy_metrics);
        if self_state.energy_budget < read_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "read_file",
                request.path,
                false,
                "Insufficient energy budget"
            );
            return (FileState {
                capabilities: self_state.capabilities,
                files: self_state.files,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let (_, has_access) = await self_state.capabilities.ask(CheckCapability {
            user_id: request.user_id,
            capability: FileAccess {
                read: true,
                write: false,
                path: request.path
            }
        });
        
        if !has_access {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "read_file",
                request.path,
                false,
                "Access denied"
            );
            return (FileState {
                capabilities: self_state.capabilities,
                files: self_state.files,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Access denied"));
        }
        
        let file = self_state.files.get(request.path);
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "read_file",
            request.path,
            true,
            null
        );
        
        return (FileState {
            capabilities: self_state.capabilities,
            files: self_state.files,
            energy_metrics: new_metrics,
            security_context: new_context
        }, file);
    }
}
```

## 3. Security Best Practices (1 hour)

### 3.1. Input Validation

```ferra
// Input validation actor with semantic tags
#[ai::tag(auth_component)]
actor ValidationActor {
    data ValidationState {
        rules: Map<String, ValidationRule>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    async fn handle_validate(self_state: ValidationState, request: ValidationRequest) -> (ValidationState, Result<ValidatedInput>) {
        // Start energy measurement
        let start_ops = measure_ops();
        
        // Check security context
        if !has_required_capabilities(self_state.security_context, ["auth:validate"]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "validate",
                request.type,
                false,
                "Missing required capabilities"
            );
            return (ValidationState {
                rules: self_state.rules,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Permission denied"));
        }

        // Check energy budget
        let validate_energy_cost = calculate_energy_cost(0.01.joules, self_state.energy_metrics);
        if self_state.energy_budget < validate_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "validate",
                request.type,
                false,
                "Insufficient energy budget"
            );
            return (ValidationState {
                rules: self_state.rules,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Insufficient energy"));
        }

        let rule = self_state.rules.get(request.type);
        
        if !rule {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "validate",
                request.type,
                false,
                "Rule not found"
            );
            return (ValidationState {
                rules: self_state.rules,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Error("Rule not found"));
        }
        
        let validated = ValidatedInput {
            sanitized: request.input
                .trim()
                .escape_html()
                .validate_length(1, 100),
            original: request.input,
            energy_budget: validate_energy_cost
        };
        
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "validate",
            request.type,
            true,
            null
        );
        
        return (ValidationState {
            rules: self_state.rules,
            energy_metrics: new_metrics,
            security_context: new_context
        }, validated);
    }
}
```

## Key Concepts

1. **Capability-Based Security**:
   - Fine-grained access control
   - Explicit permission declarations
   - Deny by default principle
   - Security context propagation

2. **Authentication**:
   - User authentication with energy budget
   - OAuth integration with security context
   - Token-based session management
   - Password hashing with energy profiling

3. **Authorization**:
   - Capability checking
   - Resource access control
   - Security context validation
   - Energy budget enforcement

4. **Security Best Practices**:
   - Input validation
   - Energy profiling
   - Security context tracking
   - Audit logging

5. **Semantic Tags**:
   - `#[ai::tag(auth_component)]` for auth components
   - `#[ai::tag(auth_operation)]` for auth operations
   - Security context integration
   - Energy metrics tracking

## Best Practices

1. **Authentication**:
   - Use capability-based security
   - Implement proper password hashing
   - Track energy usage
   - Maintain audit logs

2. **Authorization**:
   - Check capabilities before operations
   - Validate security context
   - Monitor energy budget
   - Log security events

3. **Input Validation**:
   - Sanitize all inputs
   - Use validation rules
   - Track validation energy
   - Log validation failures

4. **Security Context**:
   - Propagate security context
   - Check required capabilities
   - Monitor energy usage
   - Maintain audit trail

5. **Energy Profiling**:
   - Track operation energy
   - Monitor component energy
   - Optimize energy usage
   - Log energy metrics

## Quiz

1. What is the main benefit of using actors for authentication?
   - A. Better performance
   - B. Deterministic execution
   - C. Simpler implementation
   - D. Faster response times

2. How do you prevent SQL injection in Ferra?
   - A. Using prepared statements
   - B. Input validation
   - C. Both A and B
   - D. Neither A nor B

3. Which security header prevents clickjacking?
   - A. CSP
   - B. X-Frame-Options
   - C. HSTS
   - D. XSS-Protection

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Security Model](../../reference/SECURITY_MODEL.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)
- [Concurrency Model](../../reference/CONCURRENCY_MODEL.md)
- [Energy Profiler](../../reference/ENERGY_PROFILER.md)

## Next Steps

- [API Design & Integration](./api_design.md)
- [Monitoring & Operations](./monitoring_ops.md)
- [Advanced Topics](./advanced_topics.md) 