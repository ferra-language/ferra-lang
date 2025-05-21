---
title: "UI-DSL Fundamentals"
duration: "4h"
level: "advanced"
---

# UI-DSL Fundamentals

> **Duration**: 4 hours
> **Goal**: Master Ferra's UI-DSL for building responsive, cross-platform interfaces

## Overview

This tutorial covers the fundamentals of Ferra's UI-DSL (Domain-Specific Language for User Interfaces). You'll learn how to create beautiful, responsive interfaces that work seamlessly across desktop and mobile platforms.

## 1. Basic Components (1 hour)

### 1.1. Layout Components

```ferra
// Basic layout with flexbox
fn main() {
    VStack {
        HStack {
            Text("Hello")
            Text("World")
        }
        .padding(16)
        .background(Color.blue)
        
        Spacer()
        
        Text("Centered")
            .center()
    }
    .fill()
}
```

### 1.2. Input Components

```ferra
// Form with validation
fn login_form() {
    let email = State("")
    let password = State("")
    
    VStack {
        TextField(email)
            .placeholder("Email")
            .keyboard(.email)
            .validate(|v| v.contains("@"))
        
        SecureField(password)
            .placeholder("Password")
            .validate(|v| v.len() >= 8)
        
        Button("Login") {
            if validate_form(email.value, password.value) {
                login(email.value, password.value)
            }
        }
    }
}
```

### 1.3. Navigation

```ferra
// Tab-based navigation
fn main() {
    TabView {
        Tab("Home") {
            HomeScreen()
        }
        Tab("Profile") {
            ProfileScreen()
        }
        Tab("Settings") {
            SettingsScreen()
        }
    }
}
```

## 2. Responsive Design (1 hour)

### 2.1. Adaptive Layouts

```ferra
// Responsive grid
fn product_grid() {
    Grid {
        columns: [
            Column(.flex(1)),
            Column(.flex(1)),
            Column(.flex(1))
        ],
        items: products.map(|p| ProductCard(p))
    }
    .adaptive {
        tablet: {
            columns: [.flex(1), .flex(1)]
        },
        mobile: {
            columns: [.flex(1)]
        }
    }
}
```

### 2.2. Platform-Specific Styling

```ferra
// Platform-aware styling
fn custom_button() {
    Button("Submit") {
        submit()
    }
    .style {
        desktop: {
            padding: 16,
            corner_radius: 8
        },
        mobile: {
            padding: 12,
            corner_radius: 4
        }
    }
}
```

## 3. State Management (1 hour)

### 3.1. Local State

```ferra
// Counter with local state
fn counter() {
    let count = State(0)
    
    VStack {
        Text("Count: \(count.value)")
        Button("Increment") {
            count.value += 1
        }
    }
}
```

### 3.2. Global State

```ferra
// Global state management
data AppState {
    user: Option<User>
    theme: Theme
    settings: Settings
}

fn main() {
    let state = State(AppState {
        user: None,
        theme: Theme.light,
        settings: Settings.default()
    })
    
    App {
        state: state,
        content: {
            if state.value.user.is_some() {
                MainScreen()
            } else {
                LoginScreen()
            }
        }
    }
}
```

## 4. Event Handling (1 hour)

### 4.1. User Events

```ferra
// Gesture handling
fn draggable_card() {
    let offset = State(Point(0, 0))
    
    Card {
        Text("Drag me!")
    }
    .gesture {
        drag: |e| {
            offset.value = e.translation
        },
        tap: |e| {
            println("Tapped at \(e.location)")
        }
    }
    .offset(offset.value)
}
```

### 4.2. System Events

```ferra
// Lifecycle events
fn app_lifecycle() {
    App {
        on_launch: {
            init_analytics()
        },
        on_background: {
            save_state()
        },
        on_foreground: {
            refresh_data()
        }
    }
}
```

## Quiz

1. What is the purpose of the `adaptive` modifier in UI-DSL?
   - A. To handle touch events
   - B. To create responsive layouts
   - C. To manage state
   - D. To handle navigation

2. Which component is best for creating a form?
   - A. `HStack`
   - B. `VStack`
   - C. `Grid`
   - D. `TabView`

3. How do you handle platform-specific styling?
   - A. Using `if` statements
   - B. Using the `style` modifier
   - C. Using separate files
   - D. Using CSS

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [UI-DSL Reference](../../reference/UI_DSL_MOBILE.md)
- [UI-DSL Roadmap](../../reference/UI_DSL_ROADMAP.md)
- [Energy Profiler](../../reference/ENERGY_PROFILER.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [Desktop Integration](./desktop_integration.md)
- [Mobile Adaptation](./mobile_adaptation.md)
- [Real-time Updates](./realtime_updates.md)
- [Testing & Deployment](./testing_deployment.md)

#[ai::tag(ui_component)]
actor UIManagerActor {
    data ManagerState {
        components: Map<String, Component>,
        layouts: Map<String, Layout>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> ManagerState {
        return ManagerState {
            components: Map::new(),
            layouts: Map::new(),
            energy_budget: 2000.0.joules,
            permissions: Set::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext::new()
        }
    }

    async fn handle_component(self_state: ManagerState, request: ComponentRequest) -> (ManagerState, Result<ComponentResult, ComponentError>) {
        let start_ops = measure_ops();
        
        // Check permissions
        if !has_required_capabilities(self_state.security_context, [Capability::ComponentManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "component",
                request.id,
                false,
                "Missing required capabilities"
            );
            return (ManagerState {
                components: self_state.components,
                layouts: self_state.layouts,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(ComponentError::PermissionDenied))
        }

        // Check energy budget
        let component_energy_cost = calculate_energy_cost(20.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < component_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "component",
                request.id,
                false,
                "Insufficient energy budget"
            );
            return (ManagerState {
                components: self_state.components,
                layouts: self_state.layouts,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(ComponentError::InsufficientEnergy))
        }

        // Handle component
        let component = match self_state.components.get(request.id) {
            Some(c) => c,
            None => return (self_state, Err(ComponentError::ComponentNotFound))
        }

        let result = await component.handle(request)
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "component",
            request.id,
            true,
            null
        );

        return (ManagerState {
            components: self_state.components.insert(request.id, result.component),
            layouts: self_state.layouts,
            energy_budget: self_state.energy_budget - component_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Ok(result))
    }
}

#[ai::tag(layout_component)]
actor LayoutManagerActor {
    data LayoutState {
        layouts: Map<String, Layout>,
        constraints: Map<String, Constraint>,
        energy_budget: Float,
        permissions: Set<Capability>,
        energy_metrics: EnergyMetrics,
        security_context: SecurityContext
    }

    fn init() -> LayoutState {
        return LayoutState {
            layouts: Map::new(),
            constraints: Map::new(),
            energy_budget: 2500.0.joules,
            permissions: Set::new(),
            energy_metrics: EnergyMetrics::new(),
            security_context: SecurityContext::new()
        }
    }

    async fn handle_layout(self_state: LayoutState, request: LayoutRequest) -> (LayoutState, Result<LayoutResult, LayoutError>) {
        let start_ops = measure_ops();
        
        // Check permissions
        if !has_required_capabilities(self_state.security_context, [Capability::LayoutManagement]) {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "layout",
                request.id,
                false,
                "Missing required capabilities"
            );
            return (LayoutState {
                layouts: self_state.layouts,
                constraints: self_state.constraints,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(LayoutError::PermissionDenied))
        }

        // Check energy budget
        let layout_energy_cost = calculate_energy_cost(25.0.joules, self_state.energy_metrics);
        if self_state.energy_budget < layout_energy_cost {
            let end_ops = measure_ops();
            let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
            let new_context = audit_operation(
                self_state.security_context,
                request.principal,
                "layout",
                request.id,
                false,
                "Insufficient energy budget"
            );
            return (LayoutState {
                layouts: self_state.layouts,
                constraints: self_state.constraints,
                energy_budget: self_state.energy_budget,
                permissions: self_state.permissions,
                energy_metrics: new_metrics,
                security_context: new_context
            }, Err(LayoutError::InsufficientEnergy))
        }

        // Handle layout
        let layout = match self_state.layouts.get(request.id) {
            Some(l) => l,
            None => return (self_state, Err(LayoutError::LayoutNotFound))
        }

        let result = await layout.handle(request)
        let end_ops = measure_ops();
        let new_metrics = update_energy_metrics(self_state.energy_metrics, start_ops, end_ops);
        let new_context = audit_operation(
            self_state.security_context,
            request.principal,
            "layout",
            request.id,
            true,
            null
        );

        return (LayoutState {
            layouts: self_state.layouts.insert(request.id, result.layout),
            constraints: self_state.constraints.insert(request.id, result.constraints),
            energy_budget: self_state.energy_budget - layout_energy_cost,
            permissions: self_state.permissions,
            energy_metrics: new_metrics,
            security_context: new_context
        }, Ok(result))
    }
}

// Message definitions
data ComponentRequest {
    id: String
    principal: String
    capabilities: Set<Capability>
    scope: String
}

data ComponentResult {
    id: String
    component: Component
    duration: Duration
    energy_used: Float
}

data ComponentError {
    ComponentNotFound
    PermissionDenied
    InsufficientEnergy
    ComponentError
    InvalidRequest
}

data LayoutRequest {
    id: String
    principal: String
    capabilities: Set<Capability>
    scope: String
}

data LayoutResult {
    id: String
    layout: Layout
    constraints: Constraint
    duration: Duration
    energy_used: Float
}

data LayoutError {
    LayoutNotFound
    PermissionDenied
    InsufficientEnergy
    LayoutError
    InvalidRequest
}

// Security model
data Capability {
    ComponentManagement
    ComponentRendering
    LayoutManagement
    LayoutRendering
    EnergyManagement
} 