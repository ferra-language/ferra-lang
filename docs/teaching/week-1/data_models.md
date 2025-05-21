---
title: "Data Models"
duration: "1h"
level: "intermediate"
---

# Data Models

> **Duration**: 1 hour
> **Goal**: Learn how to define and work with data models in Ferra

## Overview

This tutorial covers data model definitions, implementations, and best practices in Ferra.

## 1. Basic Data Types (15 minutes)

```ferra
// Basic data type definitions
data User {
    id: Int
    name: String
    email: String
    age: Int?
    is_active: Bool
}

// Optional fields with default values
data Settings {
    theme: String = "light"
    notifications: Bool = true
    language: String = "en"
}

// Nested data types
data Address {
    street: String
    city: String
    country: String
    postal_code: String
}

data Profile {
    user: User
    address: Address?
    settings: Settings
}
```

## 2. Data Type Implementation (15 minutes)

```ferra
// Implementation for User data type
impl User {
    fn new(name: String, email: String, age: Int?) -> User {
        User {
            id: generate_id(),
            name: name,
            email: email,
            age: age,
            is_active: true
        }
    }

    fn update(self, name: String?, email: String?, age: Int?) -> User {
        User {
            id: self.id,
            name: name.unwrap_or(self.name),
            email: email.unwrap_or(self.email),
            age: age.or(self.age),
            is_active: self.is_active
        }
    }

    fn deactivate(self) -> User {
        User {
            id: self.id,
            name: self.name,
            email: self.email,
            age: self.age,
            is_active: false
        }
    }
}

// Implementation for Profile data type
impl Profile {
    fn new(user: User, address: Address?, settings: Settings?) -> Profile {
        Profile {
            user: user,
            address: address,
            settings: settings.unwrap_or(Settings::default())
        }
    }

    fn update_address(self, address: Address) -> Profile {
        Profile {
            user: self.user,
            address: Some(address),
            settings: self.settings
        }
    }

    fn update_settings(self, settings: Settings) -> Profile {
        Profile {
            user: self.user,
            address: self.address,
            settings: settings
        }
    }
}
```

## 3. Data Validation (15 minutes)

```ferra
// Validation implementation
impl User {
    fn validate(self) -> Result<User, ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::InvalidName)
        }
        
        if !is_valid_email(self.email) {
            return Err(ValidationError::InvalidEmail)
        }
        
        if let Some(age) = self.age {
            if age < 0 || age > 150 {
                return Err(ValidationError::InvalidAge)
            }
        }
        
        Ok(self)
    }
}

// Validation error type
data ValidationError {
    InvalidName
    InvalidEmail
    InvalidAge
    InvalidAddress
}

// Validation helper functions
fn is_valid_email(email: String) -> Bool {
    // Basic email validation
    email.contains("@") && email.contains(".")
}

fn is_valid_postal_code(code: String) -> Bool {
    // Basic postal code validation
    code.len() >= 5 && code.len() <= 10
}
```

## 4. Data Serialization (15 minutes)

```ferra
// JSON serialization implementation
impl User {
    fn to_json(self) -> Json {
        Json::object([
            ("id", Json::number(self.id)),
            ("name", Json::string(self.name)),
            ("email", Json::string(self.email)),
            ("age", self.age.map_or(Json::null(), |age| Json::number(age))),
            ("is_active", Json::boolean(self.is_active))
        ])
    }

    fn from_json(json: Json) -> Result<User, ParseError> {
        let obj = json.as_object()?
        
        Ok(User {
            id: obj.get("id")?.as_number()?,
            name: obj.get("name")?.as_string()?,
            email: obj.get("email")?.as_string()?,
            age: obj.get("age")?.as_number().ok(),
            is_active: obj.get("is_active")?.as_boolean()?
        })
    }
}

// Parse error type
data ParseError {
    InvalidJson
    MissingField(String)
    InvalidType(String)
}
```

## Quiz

1. How do you define an optional field in a data type?
   - A. Using `Option<T>`
   - B. Using `T?`
   - C. Using `Maybe<T>`
   - D. Using `Optional<T>`

2. What's the correct way to implement methods for a data type?
   - A. Using `class` keyword
   - B. Using `impl` block
   - C. Using `type` keyword
   - D. Using `struct` keyword

3. How do you handle validation in data models?
   - A. Using assertions
   - B. Using Result type
   - C. Using exceptions
   - D. Using null checks

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Syntax Grammar](../../reference/SYNTAX_GRAMMAR_V0.1.md)
- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)

## Next Steps

- [Error Handling](./error_handling.md)
- [Testing](./testing.md)
- [API Design](./api_design.md)

## Video Content

- **Duration**: 1 hour
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (5m)
  2. Basic Data Types (15m)
  3. Data Type Implementation (15m)
  4. Data Validation (15m)
  5. Data Serialization (15m)
  6. Conclusion (5m)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration
- **Chapter Markers**: For easy navigation

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
} 