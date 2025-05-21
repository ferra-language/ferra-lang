---
title: "Week-1 Tutorial: Building a REST API"
duration: "6h"
level: "intermediate"
---

# Week-1 Tutorial: Building a REST API

> **Duration**: 6 hours
> **Goal**: Build a complete REST API with Ferra

## Overview

This tutorial will guide you through building a REST API using Ferra's HTTP server capabilities, JSON handling, and error management.

## 1. Prerequisites

*   Completed [Day-0 Lab](../day-0/hello_ferra.md)
*   VSCode with [Ferra Plugin](../../reference/VSCODE_PLUGIN_ALPHA_SPEC.md)
*   Basic understanding of HTTP and REST concepts

## 2. Project Setup (30 minutes)

```bash
lang new todo_api
cd todo_api
```

## 3. Dependencies (30 minutes)

```toml
# Ferra.toml
[dependencies]
http = "1.0.0"
json = "1.0.0"
db = "1.0.0"  # For SQLite integration
```

## 4. Data Model (1 hour)

```ferra
data Todo {
    id: Int
    title: String
    completed: Bool
    created_at: DateTime
    updated_at: DateTime
}

impl Todo {
    fn validate(&self) -> Result<()> {
        if self.title.is_empty() {
            return Err("Title cannot be empty")
        }
        if self.title.len() > 100 {
            return Err("Title too long")
        }
        Ok(())
    }

    fn new(title: String) -> Todo {
        let now = DateTime::now()
        Todo {
            id: 0,  // Will be set by database
            title,
            completed: false,
            created_at: now,
            updated_at: now
        }
    }
}
```

**Cross-References**:
- [Data Types](../../reference/SYNTAX_GRAMMAR_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)

## 5. Database Setup (30 minutes)

```ferra
// src/db.ferra
import db

let DB = db::SQLite::new("todos.db")

fn init_db() -> Result<()> {
    DB.execute("""
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        )
    """)
}
```

## 6. API Endpoints (2 hours)

### 6.1. List Todos (GET /todos)

```ferra
async fn list_todos(req: Request) -> Response {
    let todos = DB.query("SELECT * FROM todos ORDER BY created_at DESC")
        .map(|row| Todo {
            id: row.get("id"),
            title: row.get("title"),
            completed: row.get("completed"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at")
        })
    
    Response::new()
        .status(200)
        .json(json!({ "todos": todos }))
}
```

### 6.2. Create Todo (POST /todos)

```ferra
async fn create_todo(req: Request) -> Response {
    let body = req.json()?
    let todo = Todo::new(body.get("title"))
    
    if let Err(e) = todo.validate() {
        return error_response(400, e)
    }
    
    let id = DB.execute("""
        INSERT INTO todos (title, completed, created_at, updated_at)
        VALUES (?, ?, ?, ?)
    """, [todo.title, todo.completed, todo.created_at, todo.updated_at])
    
    Response::new()
        .status(201)
        .json(json!({ "id": id }))
}
```

### 6.3. Get Todo (GET /todos/:id)

```ferra
async fn get_todo(req: Request) -> Response {
    let id = req.param("id")?
    let todo = DB.query_one("SELECT * FROM todos WHERE id = ?", [id])
    
    match todo {
        Some(t) => Response::new()
            .status(200)
            .json(json!({ "todo": t })),
        None => error_response(404, "Todo not found")
    }
}
```

### 6.4. Update Todo (PUT /todos/:id)

```ferra
async fn update_todo(req: Request) -> Response {
    let id = req.param("id")?
    let body = req.json()?
    
    let todo = match DB.query_one("SELECT * FROM todos WHERE id = ?", [id]) {
        Some(t) => t,
        None => return error_response(404, "Todo not found")
    }
    
    let updated = Todo {
        id: todo.id,
        title: body.get("title") ?? todo.title,
        completed: body.get("completed") ?? todo.completed,
        created_at: todo.created_at,
        updated_at: DateTime::now()
    }
    
    if let Err(e) = updated.validate() {
        return error_response(400, e)
    }
    
    DB.execute("""
        UPDATE todos
        SET title = ?, completed = ?, updated_at = ?
        WHERE id = ?
    """, [updated.title, updated.completed, updated.updated_at, id])
    
    Response::new()
        .status(200)
        .json(json!({ "todo": updated }))
}
```

### 6.5. Delete Todo (DELETE /todos/:id)

```ferra
async fn delete_todo(req: Request) -> Response {
    let id = req.param("id")?
    
    match DB.execute("DELETE FROM todos WHERE id = ?", [id]) {
        0 => error_response(404, "Todo not found"),
        _ => Response::new().status(204)
    }
}
```

### 6.6. Main Server Setup

```ferra
#[ai::tag(core_component)]
async fn main() -> Result<()> {
    init_db()?
    
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
    
    let server = http::Server::new()
        .get("/todos", list_todos)
        .post("/todos", create_todo)
        .get("/todos/:id", get_todo)
        .put("/todos/:id", update_todo)
        .delete("/todos/:id", delete_todo)
        .start(":8080")
        .await?
}
```

## 7. Error Handling (1 hour)

```ferra
fn error_response(status: Int, message: String) -> Response {
    Response::new()
        .status(status)
        .json(json!({
            "error": message
        }))
}

// Custom error types
data ApiError {
    code: Int
    message: String
}

impl ApiError {
    fn not_found() -> ApiError {
        ApiError { code: 404, message: "Resource not found" }
    }
    
    fn bad_request(msg: String) -> ApiError {
        ApiError { code: 400, message: msg }
    }
    
    fn to_response(&self) -> Response {
        error_response(self.code, self.message)
    }
}
```

## 8. Testing (1 hour)

### 8.1. Unit Tests

```ferra
#[test]
fn test_todo_validation() {
    let todo = Todo::new("Test")
    assert!(todo.validate().is_ok())
    
    let empty = Todo::new("")
    assert!(empty.validate().is_err())
    
    let long = Todo::new("x".repeat(101))
    assert!(long.validate().is_err())
}
```

### 8.2. Integration Tests

```ferra
#[test]
async fn test_todo_crud() {
    // Create
    let res = http::Client::new()
        .post("http://localhost:8080/todos")
        .json(json!({ "title": "Test Todo" }))
        .send()
        .await?
    
    assert_eq!(res.status, 201)
    let id = res.json().get("id")
    
    // Read
    let res = http::Client::new()
        .get("http://localhost:8080/todos/" + id)
        .send()
        .await?
    
    assert_eq!(res.status, 200)
    let todo = res.json().get("todo")
    assert_eq!(todo.get("title"), "Test Todo")
    
    // Update
    let res = http::Client::new()
        .put("http://localhost:8080/todos/" + id)
        .json(json!({ "completed": true }))
        .send()
        .await?
    
    assert_eq!(res.status, 200)
    
    // Delete
    let res = http::Client::new()
        .delete("http://localhost:8080/todos/" + id)
        .send()
        .await?
    
    assert_eq!(res.status, 204)
}
```

## 9. API Documentation

### 9.1. Endpoints

| Method | Endpoint | Description | Status Codes |
|--------|----------|-------------|--------------|
| GET | /todos | List all todos | 200 |
| POST | /todos | Create a todo | 201, 400 |
| GET | /todos/:id | Get a todo | 200, 404 |
| PUT | /todos/:id | Update a todo | 200, 400, 404 |
| DELETE | /todos/:id | Delete a todo | 204, 404 |

### 9.2. Request/Response Examples

```json
// POST /todos
{
    "title": "Learn Ferra"
}

// Response (201)
{
    "id": 1
}

// GET /todos/1
// Response (200)
{
    "todo": {
        "id": 1,
        "title": "Learn Ferra",
        "completed": false,
        "created_at": "2024-03-20T10:00:00Z",
        "updated_at": "2024-03-20T10:00:00Z"
    }
}
```

## Quiz

1. What is the purpose of the `validate` method in the Todo model?
   - A. To ensure the title is not empty and not too long
   - B. To check if the todo is completed
   - C. To generate a unique ID
   - D. To format the created_at timestamp

2. Which HTTP status code is returned when a todo is successfully created?
   - A. 200
   - B. 201
   - C. 204
   - D. 400

3. What is the purpose of the `error_response` helper function?
   - A. To log errors to a file
   - B. To centralize error responses for consistency
   - C. To validate input data
   - D. To handle database errors

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [HTTP API Reference](../../reference/STDLIB_CORE_V0.1.md)
- [Error Handling Guide](../../reference/DESIGN_DIAGNOSTICS.md)
- [Testing Best Practices](../../reference/CODING_STANDARDS.md)
- [Database Guide](../../reference/STDLIB_CORE_V0.1.md)
- [API Design Patterns](../../reference/CODING_STANDARDS.md) 