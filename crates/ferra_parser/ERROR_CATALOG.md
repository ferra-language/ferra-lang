# Ferra Parser Error Catalog

**Version**: 1.0  
**Last Updated**: January 2025  
**Phase 2 Complete**: All core error types documented  

---

## Error Classification System

The Ferra parser implements a comprehensive error classification system designed for positive-first error messaging. All errors follow the principle of clarity, actionability, and non-blaming language.

### Error Categories

| Category | Code Range | Description |
|----------|------------|-------------|
| **Syntax Errors** | P001-P099 | Basic syntax violations |
| **Expression Errors** | P100-P199 | Expression parsing failures |
| **Statement Errors** | P200-P299 | Statement structure errors |
| **Type Errors** | P300-P399 | Type annotation and inference errors |
| **Declaration Errors** | P400-P499 | Function, variable, and class declaration errors |
| **Control Flow Errors** | P500-P599 | Control flow statement errors |
| **Pattern Errors** | P600-P699 | Pattern matching errors |
| **Attribute Errors** | P700-P799 | Attribute and modifier errors |
| **Recovery Errors** | P900-P999 | Error recovery and internal parser errors |

---

## Syntax Errors (P001-P099)

### P001: Unexpected Token
**Context**: General token mismatch  
**Message Format**: `Expected {expected}, found {actual}`  
**Example**: `Expected ';', found 'if'`  
**Recovery**: Panic mode to next statement boundary

```rust
let x = 5 if y > 3 { ... }
//        ^^ P001: Expected ';', found 'if'
```

**Suggestions**:
- Add missing semicolon
- Check for incomplete expression

### P002: Unexpected End of File
**Context**: Parser expected more tokens but reached EOF  
**Message Format**: `Unexpected end of file, expected {expected}`  
**Example**: `Unexpected end of file, expected '}'`  
**Recovery**: Report incomplete construct

```rust
fn incomplete() {
    let x = 5;
// Missing closing brace - P002
```

### P003: Invalid Token
**Context**: Lexer produced invalid or unexpected token  
**Message Format**: `Invalid token: {token}`  
**Example**: `Invalid token: '@@'`  
**Recovery**: Skip invalid token and continue

### P004: Missing Delimiter
**Context**: Missing required delimiter (parentheses, brackets, braces)  
**Message Format**: `Missing {delimiter} to match opening {opener}`  
**Example**: `Missing ')' to match opening '('`  
**Recovery**: Insert missing delimiter and continue

```rust
fn call_function(
    arg1: String,
    arg2: i32
// Missing closing parenthesis - P004
{
    // function body
}
```

### P005: Unmatched Delimiter
**Context**: Extra closing delimiter without matching opener  
**Message Format**: `Unmatched '{delimiter}'`  
**Example**: `Unmatched '}'`  
**Recovery**: Skip delimiter and continue

---

## Expression Errors (P100-P199)

### P100: Malformed Binary Expression
**Context**: Binary operator without proper operands  
**Message Format**: `Binary operator '{op}' missing {left|right} operand`  
**Example**: `Binary operator '+' missing left operand`  
**Recovery**: Insert placeholder operand

```rust
let result = + 5;  // P100: Missing left operand for '+'
```

### P101: Invalid Unary Expression
**Context**: Unary operator with invalid operand  
**Message Format**: `Unary operator '{op}' cannot be applied to {type}`  
**Example**: `Unary operator '-' cannot be applied to string literal`  
**Recovery**: Remove unary operator

```rust
let negative_string = -"hello";  // P101: Invalid unary operation
```

### P102: Malformed Function Call
**Context**: Function call syntax errors  
**Message Format**: `Malformed function call: {reason}`  
**Example**: `Malformed function call: missing closing parenthesis`  
**Recovery**: Complete function call syntax

```rust
let result = calculate(1, 2, 3;  // P102: Missing ')'
```

### P103: Invalid Member Access
**Context**: Member access on invalid expression  
**Message Format**: `Invalid member access on {type}`  
**Example**: `Invalid member access on integer literal`  
**Recovery**: Remove member access

```rust
let value = 42.some_field;  // P103: Invalid member access
```

### P104: Malformed Array Index
**Context**: Array indexing syntax errors  
**Message Format**: `Malformed array index: {reason}`  
**Example**: `Malformed array index: missing closing bracket`  
**Recovery**: Complete indexing syntax

```rust
let item = array[0;  // P104: Missing ']'
```

### P105: Empty Expression
**Context**: Expected expression but found empty or invalid construct  
**Message Format**: `Expected expression, found {token}`  
**Example**: `Expected expression, found '}'`  
**Recovery**: Insert placeholder expression

### P106: Invalid Literal
**Context**: Malformed literal values  
**Message Format**: `Invalid {type} literal: {value}`  
**Example**: `Invalid float literal: 3.14.159`  
**Recovery**: Use nearest valid literal

```rust
let pi = 3.14.159;  // P106: Invalid float literal
```

---

## Statement Errors (P200-P299)

### P200: Malformed Variable Declaration
**Context**: Variable declaration syntax errors  
**Message Format**: `Malformed variable declaration: {reason}`  
**Example**: `Malformed variable declaration: missing type annotation`  
**Recovery**: Complete declaration syntax

```rust
let x;  // P200: Missing type annotation or initializer
```

### P201: Invalid Assignment Target
**Context**: Assignment to non-lvalue expression  
**Message Format**: `Cannot assign to {expression_type}`  
**Example**: `Cannot assign to function call`  
**Recovery**: Replace with valid lvalue

```rust
func() = 42;  // P201: Cannot assign to function call
```

### P202: Malformed Block Statement
**Context**: Block statement syntax errors  
**Message Format**: `Malformed block: {reason}`  
**Example**: `Malformed block: missing opening brace`  
**Recovery**: Complete block syntax

### P203: Invalid Statement Position
**Context**: Statement in invalid location  
**Message Format**: `{statement_type} statement not allowed here`  
**Example**: `Return statement not allowed outside function`  
**Recovery**: Remove or relocate statement

```rust
return 42;  // P203: At top level, outside function
```

---

## Control Flow Errors (P500-P599)

### P500: Malformed If Statement
**Context**: If statement syntax errors  
**Message Format**: `Malformed if statement: {reason}`  
**Example**: `Malformed if statement: missing condition`  
**Recovery**: Insert placeholder condition

```rust
if {  // P500: Missing condition
    println("hello");
}
```

### P501: Invalid Loop Construction
**Context**: While/for loop syntax errors  
**Message Format**: `Malformed {loop_type} loop: {reason}`  
**Example**: `Malformed for loop: missing 'in' keyword`  
**Recovery**: Complete loop syntax

```rust
for item items {  // P501: Missing 'in' keyword
    println(item);
}
```

### P502: Break/Continue Outside Loop
**Context**: Break or continue statement outside loop context  
**Message Format**: `{statement} statement not inside loop`  
**Example**: `break statement not inside loop`  
**Recovery**: Remove statement

```rust
fn test() {
    break;  // P502: Not inside loop
}
```

### P503: Malformed Return Statement
**Context**: Return statement syntax errors  
**Message Format**: `Malformed return statement: {reason}`  
**Example**: `Malformed return statement: invalid expression`  
**Recovery**: Fix or remove return value

---

## Declaration Errors (P400-P499)

### P400: Malformed Function Declaration
**Context**: Function declaration syntax errors  
**Message Format**: `Malformed function declaration: {reason}`  
**Example**: `Malformed function declaration: missing parameter list`  
**Recovery**: Complete function signature

```rust
fn calculate -> i32 {  // P400: Missing parameter list
    return 42;
}
```

### P401: Invalid Parameter List
**Context**: Function parameter syntax errors  
**Message Format**: `Invalid parameter list: {reason}`  
**Example**: `Invalid parameter list: missing parameter type`  
**Recovery**: Complete parameter syntax

```rust
fn test(name, age: i32) {  // P401: Missing type for 'name'
}
```

### P402: Malformed Data Class
**Context**: Data class declaration syntax errors  
**Message Format**: `Malformed data class: {reason}`  
**Example**: `Malformed data class: missing field type`  
**Recovery**: Complete class syntax

```rust
data Person {
    name,  // P402: Missing field type
    age: i32,
}
```

### P403: Invalid Modifier Combination
**Context**: Invalid combination of modifiers  
**Message Format**: `Invalid modifier combination: {modifiers}`  
**Example**: `Invalid modifier combination: 'pub pub fn'`  
**Recovery**: Remove duplicate modifiers

```rust
pub pub fn test() {}  // P403: Duplicate 'pub' modifier
```

---

## Type Errors (P300-P399)

### P300: Invalid Type Annotation
**Context**: Type annotation syntax errors  
**Message Format**: `Invalid type annotation: {reason}`  
**Example**: `Invalid type annotation: expected type name`  
**Recovery**: Insert placeholder type

```rust
let x: = 42;  // P300: Missing type name
```

### P301: Malformed Generic Parameters
**Context**: Generic type parameter syntax errors  
**Message Format**: `Malformed generic parameters: {reason}`  
**Example**: `Malformed generic parameters: missing closing bracket`  
**Recovery**: Complete generic syntax

```rust
let map: HashMap<String, i32 = HashMap::new();  // P301: Missing '>'
```

### P302: Invalid Array Type
**Context**: Array type syntax errors  
**Message Format**: `Invalid array type: {reason}`  
**Example**: `Invalid array type: missing element type`  
**Recovery**: Complete array type syntax

```rust
let items: [; 10] = [];  // P302: Missing element type
```

---

## Pattern Errors (P600-P699)

### P600: Malformed Match Pattern
**Context**: Pattern matching syntax errors  
**Message Format**: `Malformed pattern: {reason}`  
**Example**: `Malformed pattern: invalid structure binding`  
**Recovery**: Fix pattern syntax

```rust
match value {
    Person { name age } => {},  // P600: Missing comma or colon
}
```

### P601: Invalid Pattern Type
**Context**: Pattern type mismatch or invalid usage  
**Message Format**: `Invalid pattern type: {reason}`  
**Example**: `Invalid pattern type: cannot destructure integer`  
**Recovery**: Replace with valid pattern

---

## Attribute Errors (P700-P799)

### P700: Malformed Attribute
**Context**: Attribute syntax errors  
**Message Format**: `Malformed attribute: {reason}`  
**Example**: `Malformed attribute: missing closing bracket`  
**Recovery**: Complete attribute syntax

```rust
#[derive(Debug  // P700: Missing ']'
fn test() {}
```

### P701: Invalid Attribute Argument
**Context**: Attribute argument syntax errors  
**Message Format**: `Invalid attribute argument: {reason}`  
**Example**: `Invalid attribute argument: expected string literal`  
**Recovery**: Fix argument syntax

---

## Recovery Errors (P900-P999)

### P900: Parser Internal Error
**Context**: Internal parser state corruption  
**Message Format**: `Internal parser error: {details}`  
**Example**: `Internal parser error: inconsistent state`  
**Recovery**: Restart parsing from safe point

### P901: Recovery Failed
**Context**: Error recovery mechanism failed  
**Message Format**: `Failed to recover from error: {original_error}`  
**Example**: `Failed to recover from error: malformed expression`  
**Recovery**: Report original error and abort

### P902: Maximum Errors Exceeded
**Context**: Too many errors encountered  
**Message Format**: `Maximum error count exceeded ({count}), stopping parser`  
**Example**: `Maximum error count exceeded (50), stopping parser`  
**Recovery**: Stop parsing and report summary

---

## Error Recovery Strategies

### Panic Mode Recovery
Used for most syntax errors. Parser consumes tokens until reaching a synchronization point:

**Synchronization Points**:
- Statement boundaries (`;`, `}`)
- Declaration keywords (`fn`, `let`, `var`, `data`)
- Control flow keywords (`if`, `while`, `for`)

### Error Production Rules
For common mistakes, parser has built-in recovery productions:

- Missing semicolons
- Unmatched delimiters
- Incomplete expressions

### Error Reporting Best Practices

1. **Positive Language**: Focus on what should be done, not what went wrong
2. **Specific Context**: Include exact location and surrounding code context
3. **Actionable Suggestions**: Provide concrete steps to fix the error
4. **Progressive Disclosure**: Start with simple explanation, offer details if needed

### Integration with Diagnostic System

All parser errors integrate with the `miette` diagnostic system for rich error reporting with:

- Source code highlighting
- Multi-line error spans
- Suggestion annotations
- Related error grouping 