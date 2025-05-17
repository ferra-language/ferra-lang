# Ferra Compiler – Abstract Syntax Tree Specification (v0.1)

> **Status:** Draft (Module 1.1 · Step 1.1.3)
>
> *Consumers*: semantic analyser, borrow‐checker, IR lowering, formatter. *Producers*: front‐end parser as described in DESIGN_PARSER.md.

---

## 1  Design Principles

* **Faithful** – One‐to‐one mapping with grammar but *no* redundant punctuation.
* **Typed IDs + arenas** – All nodes are stored in an `AstArena`; references are `Idx<T>` new‐types (u32) for cache‐friendly passes.
* **Span‐rich** – Every node carries a `Span { file_id: FileId, lo: BytePos, hi: BytePos }` for diagnostics & tooling.
* **Immutable** – AST is read‐only after construction; later passes build separate mutable IRs.
* **Evolvable** – Additive; v0.1 avoids breaking fields so future RFCs can extend using `Option<>` or new variants. While v0.x versions may introduce breaking changes to the AST structure as the language evolves, v1.x and later will adhere to semantic versioning for the AST, ensuring stability for tooling that consumes it.
* **Assignment as Expression (v0.1)**: For v0.1, assignment operations (`=`, `+=`, etc.) are parsed as expressions (see `ExprKind::Assign`) and can syntactically appear where other expressions are allowed. Their evaluation semantics (e.g., what value they yield, typically `Unit`) are defined in `CORE_SEMANTICS.md`.

---

## 2  Shared Data Types

```rust
pub type BytePos = u32;              // UTF‐8 byte offset
pub struct FileId(u32);              // ID into a SourceMap/FileManager for multi-file projects
pub struct Span { pub file_id: FileId, pub lo: BytePos, pub hi: BytePos }

pub struct IdentId(u32);             // points into IdentArena (string interner)
// Typed IDs for AST nodes, referencing items in an arena
pub struct ExprId(u32);
pub struct StmtId(u32);
pub struct TypeId(u32);
pub struct ItemId(u32);              // top‐level items (functions, data classes)
pub struct BlockId(u32);
pub struct PatternId(u32);
pub struct PathId(u32); // For simple paths; complex paths might be full nodes
```

### 2.1  LiteralValue

```rust
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(IdentId),  // interned string contents
    Unit,             // ()
    // Char(char), // ⚠️ TBD (AST-LIT) if dedicated char literals like 'x' are supported
}
// Note: When an `ExprKind::Literal` node containing `LiteralValue::Int`, `::Float`, `::Bool`,
// or `::Unit` is evaluated or bound, it exhibits copy semantics as per CORE_SEMANTICS.md.
```

---

## 3  Expression Nodes

```rust
pub struct Expr { // Stored in ExprArena, referenced by ExprId
    pub kind: ExprKind,
    pub span: Span,
    // pub type_id: Option<ResolvedTypeId>, // Added by type checker
}

pub enum ExprKind {
    Literal { value: LiteralValue },
    Identifier { ident: IdentId }, // Variable use, function name in call position
    Unary { op: UnaryOp, operand: ExprId },
    Binary { op: BinaryOp, lhs: ExprId, rhs: ExprId },
    Assign { op: AssignOp, lhs: ExprId, rhs: ExprId }, // Assignment expressions
    Call { callee: ExprId, args: Vec<ExprId> },
    FieldAccess { base: ExprId, field: IdentId },
    Index { base: ExprId, index: ExprId }, // ⚠️ TBD (AST-LIT): Array/Tuple literal syntax needed first
    If { cond: ExprId, then_branch: BlockId, else_branch: BlockId }, // `else` is mandatory
    Match { scrutinee: ExprId, arms: Vec<MatchArm> },
    Await { expr: ExprId },
    ErrorPropagate { expr: ExprId },   // postfix ?
    Coalesce { lhs: ExprId, rhs: ExprId }, // ??
    Range { lhs: ExprId, rhs: ExprId, inclusive: bool }, // .. and ..=
    Grouped { expr: ExprId }, // For `(expr)`
    // ArrayLiteral { elements: Vec<ExprId> }, // ⚠️ TBD (AST-LIT)
    // TupleLiteral { elements: Vec<ExprId> }, // ⚠️ TBD (AST-LIT)
}
```

### 3.1  Helper enums for Operators

```rust
pub enum UnaryOp { Not, Neg, Plus }

pub enum BinaryOp {
    // Arithmetic
    Mul, Div, Mod,
    Add, Sub,
    // Bitwise
    Shl, Shr,
    BitAnd, BitXor, BitOr,
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    // Logical
    AndAnd, OrOr, // &&, || (and their keyword aliases)
    // Range
    Range,      // ..
    RangeEq,    // ..=
    // Coalesce
    Coalesce,   // ??
}

pub enum AssignOp {
    Assign,      // =
    AddAssign,   // +=
    SubAssign,   // -=
    MulAssign,   // *=
    DivAssign,   // /=
    ModAssign,   // %=
    BitAndAssign,// &=
    BitOrAssign, // |=
    BitXorAssign,// ^=
    ShlAssign,   // <<=
    ShrAssign,   // >>=
}
```

### 3.2  MatchArm

```rust
pub struct MatchArm {
    pub pattern: PatternId,
    pub guard: Option<ExprId>, // ⚠️ TBD (AST-GUARD): Match guards
    pub body: BlockId, // Could also be ExprId if match arms can be single expressions
    pub span: Span,
}
```

---

## 4  Statement Nodes

```rust
pub struct Stmt { // Stored in StmtArena, referenced by StmtId
    pub kind: StmtKind,
    pub span: Span,
}

pub enum StmtKind {
    Let { mutable: bool, ident: IdentId, ty_annotation: Option<TypeId>, value_expr: Option<ExprId> }, // Initializer is optional
    Expr { expr: ExprId, has_semicolon: bool }, // Expression used as a statement (includes assignments)
    Return { value_expr: Option<ExprId> },
    Break,
    Continue,
    While { cond_expr: ExprId, body_block: BlockId },
    For   { loop_var_ident: IdentId, iter_expr: ExprId, body_block: BlockId },
    // Block { block: BlockId }, // A standalone block statement can be represented by BlockId directly if needed
}
```

---

## 5  Block & Item Nodes

```rust
pub struct Block { // Stored in BlockArena, referenced by BlockId
    pub stmts: Vec<StmtId>,
    pub uses_indentation: bool, // true if INDENT/DEDENT, false if { }
    pub span: Span,
}

pub struct Item { // Stored in ItemArena, referenced by ItemId
    pub kind: ItemKind,
    pub span: Span,
    // pub attributes: Vec<AttributeId>, // Future
    // pub visibility: Visibility, // Future
}

pub enum ItemKind {
    Function { sig: FnSig, body: BlockId },
    DataClass { name: IdentId, fields: Vec<DataField> /*, generic_params: Vec<GenericParamId>*/ },
    // Import { path: PathId, alias: Option<IdentId> }, // Future
}

pub struct FnSig {
    pub name: IdentId,
    pub params: Vec<Param>,
    pub ret_ty_annotation: Option<TypeId>,
    pub is_async: bool,
    // pub generic_params: Vec<GenericParamId>, // Future
    pub span: Span,
}

pub struct Param { pub ident: IdentId, pub ty_annotation: TypeId, pub span: Span }

pub struct DataField { pub ident: IdentId, pub ty_annotation: TypeId, pub span: Span }
```

---

## 6  Pattern Nodes *(minimal v0.1 for `match`)*

```rust
pub struct Pattern { // Stored in PatternArena, referenced by PatternId
    pub kind: PatternKind,
    pub span: Span,
}

pub enum PatternKind {
    Wildcard,                 // _
    Literal { value: LiteralValue },
    Identifier { ident: IdentId, is_mutable: bool /* if patterns can introduce mut bindings, e.g. `ref mut name` */ }, // ⚠️ TBD (AST-PAT-MUT)
    DataDestruct { type_path: PathId, fields: Vec<FieldPattern>, has_rest: bool /* `..` */ }, // ⚠️ TBD (AST-PAT-REST) clarification on `has_rest` vs. full `..` pattern.
    // Tuple { elements: Vec<PatternId> }, // ⚠️ TBD (AST-LIT)
    // Array { elements: Vec<PatternId> }, // ⚠️ TBD (AST-LIT)
}

pub struct FieldPattern {
    pub field_name: IdentId,
    pub pattern: Option<PatternId>, // If None, implies shorthand `field_name` (binds to `field_name: field_name`)
    pub span: Span,
}

// PathId defined in Shared Data Types
```

---

## 7  Type Nodes

```rust
pub struct Type { // Stored in TypeArena, referenced by TypeId
    pub kind: TypeKind,
    pub span: Span,
}

pub enum TypeKind {
    Simple { ident_path: PathId }, // Can be single IdentId or qualified path e.g. my_module::MyType
    Generic { base_path: PathId, args: Vec<TypeId> },
    Tuple { elems: Vec<TypeId> },
    Array { elem_ty: TypeId }, // For `[T]`
    Function { param_types: Vec<TypeId>, ret_ty: Box<TypeId> }, // Box for recursion
    Infer, // Represents `_` in type position
}
```

---

## 8  Arena Layout & Memory Strategy

* A top-level `Ast` struct could own all the individual typed arenas (e.g., `expr_arena: Arena<Expr>`, `stmt_arena: Arena<Stmt>`, etc.), providing a single handle for a complete compilation unit's AST.
  ```rust
  // Conceptual
  // pub struct Ast {
  //     expr_arena: Arena<Expr>,
  //     stmt_arena: Arena<Stmt>,
  //     type_arena: Arena<Type>,
  //     pattern_arena: Arena<Pattern>,
  //     block_arena: Arena<Block>,
  //     item_arena: Arena<Item>,
  //     ident_arena: StringInterner, // Or Arena<StringData> for IdentId payload
  //     path_arena: Arena<Vec<IdentId>>, // For PathId payload if not just Vec<IdentId>
  //     // ... potentially other arenas ...
  // }
  ```
* IDs (`ExprId`, `StmtId`, etc.) are `u32` indexes (or `usize` if using `typed-arena` crate directly) into the respective arena.
* This approach offers cache-friendly contiguous allocation for nodes of the same type and avoids `Box` overhead for most inter-node relations (except where recursion in type definitions requires it, e.g., `Box<TypeId>` in `FunctionType`).
* Source text is typically owned separately (e.g., `String` or `&str` if from mmap) and `Span`s refer into it.
* Arenas should typically be reset or created fresh for each distinct compilation unit (e.g., per file being compiled independently) to prevent ID clashes and manage memory lifetimes effectively.
* After parsing and construction, the AST (and its underlying arenas) is typically treated as immutable. If shared across threads for later compiler passes (e.g., parallel code generation), the arena implementation and ID access would need to ensure appropriate `Send + Sync` safety, potentially by "freezing" the arenas or using thread-safe arena types.

---

## 9  Span & Diagnostics Integration

* `Span { file_id: FileId, lo: BytePos, hi: BytePos }` is captured by the parser when a node is created, referencing byte offsets into a specific source file (identified by `FileId`).
* A separate `SourceMap` or `FileManager` utility (outside the AST itself) is responsible for:
    * Managing `FileId`s and mapping them to actual file paths/contents.
    * Mapping `BytePos` (within a given `FileId`) to human-readable (line, column) information.
    * Retrieving source snippets for diagnostics.
* Compound nodes' spans (e.g., a `Block` or `FunctionDecl`) should generally encompass the spans of all their children, from the start of the first child/token to the end of the last child/token.
* Spans for macro-generated code will need special handling (⚠️ **TBD (AST-MACRO-SPAN)**).

---

## 10 Open Questions / ⚠️ TBD (AST Specific)

| Tag                 | Description                                                                      |
|---------------------|----------------------------------------------------------------------------------|
| **AST-LIT**         | Syntax and AST nodes for **Array, Tuple, and Map literals**. (Also Hex-float, byte-string, and potentially Char literals `'x'`). |
| **AST-IMPORT**      | AST for `ImportDeclaration`.                                                     |
| **AST-MACRO-DEF**   | AST for Macro definitions.                                                       |
| **AST-MACRO-INVOKE**| How to represent macro invocations (e.g., `json!(...)`) before expansion.        |
| **AST-MACRO-SPAN**  | Span handling for macro-generated code.                                          |
| **AST-ATTR**        | Representation of attributes/annotations (e.g., `#[gpu]`).                       |
| **AST-PAT-ADV**     | AST for more advanced patterns (ranges, `|` or-patterns, `@` binding).             |
| **AST-PAT-MUT**     | Support for `ref mut name` or similar in patterns.                             |
| **AST-PAT-REST**    | Clarify `has_rest: bool` vs. a full `RestPatternNode` in `DataDestruct`.         |
| **AST-GENERIC-PARAM**| How to represent generic parameters in declarations (e.g., `fn foo<T>(x: T)`).   |
| **AST-LAMBDA**      | Detailed AST for lambda/closure expressions.                                     |
| **AST-TRAIT**       | AST for trait/interface definitions and implementations.                         |
| **AST-VISITOR**     | Finalizing visitor pattern support (e.g., derive, common trait, or manual impls).|
| **AST-PIPELINE**    | AST node variant for pipeline operator `                                          |> ` if added.                |
| **AST-NULLABLE-TYPE**| AST for `Type?` syntax if specific nullable types (not `Option<T>`) are added.   |

---

> *Next*: Parser implementation will emit these IDs; semantic analysis and subsequent compiler passes will traverse the AST using these IDs to access nodes stored in arenas. Visitors or iterative processing queues will be common traversal mechanisms.