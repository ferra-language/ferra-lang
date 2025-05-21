# Week-1 Tutorial Video Script: Building a REST API with Ferra

> **Format**: 12 episodes, ~30 minutes each
> **Resolution**: 1920x1080, JetBrains Mono, clear narration

---

## Episode 1: Introduction & Setup

[Screen: Ferra logo, project goal]

"Welcome! In this series, you'll build a production-ready Todo REST API in Ferra. Let's get started by creating our project."

[Terminal:]
```bash
lang new todo_api
cd todo_api
```

"Open the project in your editor."

---

## Episode 2: Project Structure

[Screen: VSCode, file tree]

"Organize your code for maintainability. We'll use folders for models, handlers, server, and tests."

[Terminal:]
```bash
mkdir -p src/{models,handlers,server}
touch src/models/todo.lang src/handlers/todo.lang src/server/http.lang tests/api_test.lang
```

---

## Episode 3: Data Models

[Screen: src/models/todo.lang]

"Define the Todo data model and validation logic."

[Type code]
```lang
data Todo {
    id: Int
    title: String
    completed: Bool
    created_at: DateTime
}

impl Todo {
    fn validate(self) -> Result<()> {
        if self.title.len() < 1 {
            return Err("Title cannot be empty")
        }
        if self.title.len() > 100 {
            return Err("Title too long")
        }
        Ok(())
    }
}
```

---

## Episode 4: HTTP Server Basics

[Screen: src/server/http.lang]

"Set up the HTTP server and configuration."

[Type code]
```lang
data Config {
    port: Int = 8080
    host: String = "localhost"
}

fn create_server(config: Config) -> http.Server {
    let server = http.Server::new()
    server.listen(config.host, config.port)
    server
}
```

---

## Episode 5: GET Endpoints

[Screen: src/handlers/todo.lang]

"Implement GET endpoints for listing and fetching todos."

[Type code]
```lang
fn list_todos(req: http.Request) -> http.Response {
    http.Response::new().json(todos)
}

fn get_todo(req: http.Request) -> http.Response {
    let id = req.params.id.to_int()
    match todos.find(t => t.id == id) {
        Some(todo) => http.Response::new().json(todo)
        None => error_response(404, "Todo not found")
    }
}
```

---

## Episode 6: POST Endpoint

[Screen: src/handlers/todo.lang]

"Add the POST endpoint to create new todos."

[Type code]
```lang
fn create_todo(req: http.Request) -> http.Response {
    let body = req.json()
    let todo = Todo {
        id: next_id,
        title: body.title,
        completed: false,
        created_at: DateTime::now()
    }
    match todo.validate() {
        Ok(_) => {
            todos.push(todo)
            next_id += 1
            http.Response::new().status(201).json(todo)
        }
        Err(e) => error_response(400, e)
    }
}
```

---

## Episode 7: PUT Endpoint

[Screen: src/handlers/todo.lang]

"Support updating todos with the PUT endpoint."

[Type code]
```lang
fn update_todo(req: http.Request) -> http.Response {
    let id = req.params.id.to_int()
    let body = req.json()
    match todos.find_index(t => t.id == id) {
        Some(idx) => {
            let todo = todos[idx]
            todo.title = body.title
            todo.completed = body.completed
            match todo.validate() {
                Ok(_) => http.Response::new().json(todo)
                Err(e) => error_response(400, e)
            }
        }
        None => error_response(404, "Todo not found")
    }
}
```

---

## Episode 8: DELETE Endpoint

[Screen: src/handlers/todo.lang]

"Implement the DELETE endpoint to remove todos."

[Type code]
```lang
fn delete_todo(req: http.Request) -> http.Response {
    let id = req.params.id.to_int()
    match todos.find_index(t => t.id == id) {
        Some(idx) => {
            todos.remove(idx)
            http.Response::new().status(204)
        }
        None => error_response(404, "Todo not found")
    }
}
```

---

## Episode 9: Error Handling

[Screen: src/server/http.lang]

"Centralize error responses for consistency."

[Type code]
```lang
fn error_response(status: Int, message: String) -> http.Response {
    http.Response::new().status(status).json({"error": message})
}
```

---

## Episode 10: Input Validation

[Screen: src/models/todo.lang]

"Add input validation to prevent bad data."

[Type code]
```lang
// Add to Todo implementation
fn validate_input(body: Json) -> Result<()> {
    if !body.has("title") {
        return Err("Title is required")
    }
    if body.title.type != "string" {
        return Err("Title must be a string")
    }
    Ok(())
}
```

---

## Episode 11: Testing Basics

[Screen: tests/api_test.lang]

"Write tests for your API endpoints."

[Type code]
```lang
fn test_endpoint(method: String, path: String, body: String = "") -> http.Response {
    let client = http.Client::new()
    match method {
        "GET" => client.get("http://localhost:8080" + path)
        "POST" => client.post("http://localhost:8080" + path, body)
        "PUT" => client.put("http://localhost:8080" + path, body)
        "DELETE" => client.delete("http://localhost:8080" + path)
    }
}
```

---

## Episode 12: API Testing

[Screen: tests/api_test.lang]

"Write and run comprehensive test cases."

[Type code]
```lang
fn test_create_todo() {
    let response = test_endpoint(
        "POST",
        "/todos",
        json.stringify({"title": "Test Todo"})
    )
    assert response.status == 201
}

fn test_get_todo() {
    let response = test_endpoint("GET", "/todos/1")
    assert response.status == 200
}

fn main() {
    test_create_todo()
    test_get_todo()
    println("All tests passed!")
}
```

---

## Accessibility Notes

- All code blocks are high contrast
- Terminal text is enlarged
- Narration is clear and well-paced
- Subtitles provided in .vtt format
- Screen reader friendly
- Chapter markers for navigation 