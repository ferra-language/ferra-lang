---
title: "Database & Storage"
duration: "4h"
level: "advanced"
---

# Database & Storage

> **Duration**: 4 hours
> **Goal**: Master data persistence and storage in Ferra applications

## Overview

This tutorial covers database integration, data modeling, and storage solutions in Ferra applications. You'll learn how to work with SQL and NoSQL databases, implement caching, and handle file storage.

## 1. SQL Database Integration (1 hour)

### 1.1. Database Setup

```ferra
// Database configuration
let db = Database {
    provider: .postgres,
    connection: {
        host: "localhost",
        port: 5432,
        database: "myapp",
        user: "admin",
        password: "secret"
    },
    pool: {
        min_connections: 5,
        max_connections: 20,
        idle_timeout: 30.seconds
    }
}

// Schema definition
schema User {
    id: Int @id @auto_increment
    email: String @unique
    name: String
    created_at: DateTime @default(now())
    posts: List<Post> @relation("user_posts")
}

schema Post {
    id: Int @id @auto_increment
    title: String
    content: String
    author: User @relation("user_posts")
    created_at: DateTime @default(now())
}
```

### 1.2. Query Operations

```ferra
// CRUD operations
#[ai::tag(database_component)]
fn user_operations() {
    let energy_metrics = EnergyMetrics::new()
    let security_context = SecurityContext::new()
    
    // Create
    let start_ops = measure_ops()
    let user = db.users.create {
        email: "user@example.com",
        name: "John Doe"
    }
    let end_ops = measure_ops()
    energy_metrics = update_energy_metrics(energy_metrics, start_ops, end_ops)
    security_context = audit_operation(
        security_context,
        "system",
        "create",
        user.id,
        true,
        null
    )
    
    // Read
    let start_ops = measure_ops()
    let users = db.users
        .where { email.contains("@example.com") }
        .order_by { created_at.desc() }
        .limit(10)
        .find()
    let end_ops = measure_ops()
    energy_metrics = update_energy_metrics(energy_metrics, start_ops, end_ops)
    security_context = audit_operation(
        security_context,
        "system",
        "read",
        "users",
        true,
        null
    )
    
    // Update
    let start_ops = measure_ops()
    db.users
        .where { id == user.id }
        .update { name: "John Smith" }
    let end_ops = measure_ops()
    energy_metrics = update_energy_metrics(energy_metrics, start_ops, end_ops)
    security_context = audit_operation(
        security_context,
        "system",
        "update",
        user.id,
        true,
        null
    )
    
    // Delete
    let start_ops = measure_ops()
    db.users
        .where { id == user.id }
        .delete()
    let end_ops = measure_ops()
    energy_metrics = update_energy_metrics(energy_metrics, start_ops, end_ops)
    security_context = audit_operation(
        security_context,
        "system",
        "delete",
        user.id,
        true,
        null
    )
}
```

## 2. NoSQL Database Integration (1 hour)

### 2.1. Document Store

```ferra
// MongoDB integration
let mongo = Mongo {
    uri: "mongodb://localhost:27017",
    database: "myapp",
    options: {
        max_pool_size: 50,
        min_pool_size: 10
    }
}

// Document operations
fn document_ops() {
    let collection = mongo.collection("users")
    
    // Insert
    collection.insert_one {
        name: "Jane",
        email: "jane@example.com",
        preferences: {
            theme: "dark",
            notifications: true
        }
    }
    
    // Query
    let users = collection
        .find {
            "preferences.theme": "dark"
        }
        .sort { "name": 1 }
        .limit(10)
}
```

### 2.2. Key-Value Store

```ferra
// Redis integration
let redis = Redis {
    host: "localhost",
    port: 6379,
    db: 0,
    options: {
        max_connections: 100,
        timeout: 5.seconds
    }
}

// Cache operations
fn cache_ops() {
    // Set with expiration
    redis.set("user:1", user_data, {
        ttl: 1.hour
    })
    
    // Get with fallback
    let data = redis.get("user:1") ?? fetch_user(1)
    
    // Atomic operations
    redis.incr("page_views")
    redis.hset("user:1:stats", {
        "last_login": now(),
        "login_count": 1
    })
}
```

## 3. File Storage (1 hour)

### 3.1. Local Storage

```ferra
// File system operations
fn file_ops() {
    let storage = Storage {
        base_path: "uploads",
        max_size: 10.mb,
        allowed_types: [".jpg", ".png", ".pdf"]
    }
    
    // Upload file
    let file = storage.upload("profile.jpg", {
        content_type: "image/jpeg",
        metadata: {
            "user_id": 1,
            "purpose": "profile"
        }
    })
    
    // Download file
    let content = storage.download(file.path)
    
    // Delete file
    storage.delete(file.path)
}
```

### 3.2. Cloud Storage

```ferra
// S3 integration
let s3 = S3 {
    bucket: "myapp-storage",
    region: "us-west-2",
    credentials: {
        access_key: "AKIA...",
        secret_key: "secret..."
    }
}

// Cloud storage operations
fn cloud_storage() {
    // Upload with progress
    s3.upload("documents/report.pdf", file_data, {
        on_progress: |progress| {
            update_upload_status(progress)
        },
        metadata: {
            "owner": "user:1",
            "type": "document"
        }
    })
    
    // Generate signed URL
    let url = s3.signed_url("documents/report.pdf", {
        expires_in: 1.hour
    })
}
```

## 4. Data Migration (1 hour)

### 4.1. Schema Migration

```ferra
// Migration definition
migration "add_user_roles" {
    up {
        db.execute("""
            ALTER TABLE users
            ADD COLUMN role VARCHAR(20) DEFAULT 'user'
        """)
    }
    
    down {
        db.execute("""
            ALTER TABLE users
            DROP COLUMN role
        """)
    }
}

// Migration runner
fn run_migrations() {
    let migrator = Migrator {
        migrations_dir: "migrations",
        target_version: "latest"
    }
    
    migrator.run()
}
```

### 4.2. Data Transformation

```ferra
// Data transformation
fn transform_data() {
    let transformer = DataTransformer {
        source: old_db,
        target: new_db,
        batch_size: 1000
    }
    
    transformer.transform {
        from: "users",
        to: "profiles",
        map: |user| {
            return {
                id: user.id,
                email: user.email,
                full_name: user.first_name + " " + user.last_name,
                created_at: user.created_at
            }
        }
    }
}
```

## Quiz

1. What is the main benefit of using connection pooling?
   - A. Better security
   - B. Improved performance
   - C. Simpler code
   - D. More features

2. How do you handle file uploads securely?
   - A. Accept all files
   - B. Validate file types
   - C. Ignore size limits
   - D. Skip validation

3. Which strategy is best for data migration?
   - A. Manual migration
   - B. Automated migration
   - C. No migration
   - D. Copy-paste

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Security Model](../../reference/SECURITY_MODEL.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)
- [Energy Profiler](../../reference/ENERGY_PROFILER.md)
- [Concurrency Model](../../reference/CONCURRENCY_MODEL.md)

## Next Steps

- [Authentication & Security](./auth_security.md)
- [API Design & Integration](./api_design.md)
- [Monitoring & Operations](./monitoring_ops.md)
- [Advanced Topics](./advanced_topics.md) 