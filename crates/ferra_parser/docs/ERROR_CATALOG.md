# Ferra Parser Error Catalog

**Version**: 2.0  
**Last Updated**: January 2025  
**Coverage**: All parser error scenarios with positive-first messaging (Based on actual implementation)

---

## Overview

This catalog documents all error messages produced by the Ferra parser, organized by actual ParseError enum variants. The parser follows "positive-first" error messaging principles, providing actionable guidance rather than blame-focused messages.

## Error Classification System

The Ferra parser implements error classification based on the actual ParseError enum with severity levels and optional error codes.

### Error Severity Levels

| Severity | Code | Description | Behavior |
|----------|------|-------------|----------|
| **Warning** | W | Non-blocking issues that should be addressed | Continue parsing |
| **Error** | E | Blocking issues that prevent successful parsing | Attempt recovery |
| **Fatal** | F | Critical issues that stop all parsing | Stop immediately |

### Actual ParseError Variants (From Implementation)

| Variant | Description | Default Code | Severity |
|---------|-------------|--------------|----------|
| **UnexpectedToken** | Token doesn't match expected token | None | Error |
| **ExpectedExpression** | Expected expression, found something else | None | Error |
| **ExpectedStatement** | Expected statement, found something else | None | Error |
| **ExpectedType** | Expected type expression, found something else | None | Error |
| **InvalidBlock** | Block structure is malformed | None | Error |
| **MixedBlockStyles** | Both braced and indented styles in same block | None | Error |
| **InconsistentIndentation** | Indentation levels don't match | None | Error |
| **ExpectedBlock** | Expected block (braced or indented) | None | Error |
| **InvalidIndentation** | Invalid indentation level | None | Error |
| **VariableRedefinition** | Variable already defined in scope | None | Error |
| **UnexpectedEof** | Unexpected end of file | None | Error |
| **Internal** | Internal parser error | I001 | Fatal |
| **SyntaxError** | Generic syntax error | E001 | Error |
| **RecoveryError** | Error recovery failed | R001 | Warning |

---

## Core Parser Errors

### UnexpectedToken
**Message**: `"Expected {expected}, but found {found}"`  
**Context**: Token doesn't match parser expectations  
**Suggested Fix**: Provide the expected token or syntax

```ferra
// ❌ Error
let x = ;  // Expected expression, found semicolon

// ✅ Fix
let x = 42;
let y = "hello";
let z = true;
```

**Error Structure**:
```rust
UnexpectedToken {
    expected: String,        // What was expected
    found: String,          // What was actually found
    span: Span,             // Location of error
    suggestion: Option<String>, // Optional fix suggestion
    severity: ErrorSeverity,    // Error severity level
    error_code: Option<&'static str>, // Optional error code
}
```

### ExpectedExpression
**Message**: `"Expected expression"`  
**Context**: Parser expected an expression but found invalid token  
**Default Suggestion**: "Consider adding a literal, identifier, or parenthesized expression"

```ferra
// ❌ Error
if {  // Expected expression for condition
    do_something();
}

// ✅ Fix
if condition {
    do_something();
}
```

### ExpectedStatement  
**Message**: `"Expected statement"`  
**Context**: Parser expected a statement but found invalid token  
**Default Suggestion**: "Statements can be declarations (let, var, fn, data) or expressions"

```ferra
// ❌ Error
{
    42;  // Expression without proper statement context
    ;    // Empty statement
}

// ✅ Fix
{
    let x = 42;  // Proper variable declaration
    println!(x); // Expression statement
}
```

### ExpectedType
**Message**: `"Expected type expression"`  
**Context**: Parser expected type annotation but found invalid token  
**Default Suggestion**: "Type expressions include identifiers, tuples, arrays, and function types"

```ferra
// ❌ Error
let x: = 42;      // Missing type after colon
fn test(): {      // Missing return type
}

// ✅ Fix
let x: i32 = 42;
fn test() -> i32 {
    42
}
```

---

## Block Structure Errors

### InvalidBlock
**Message**: `"Invalid block structure: {message}"`  
**Context**: Block has structural problems beyond just mixed styles  
**Variable Message**: Specific structural issue description

```ferra
// ❌ Error - Various block structure issues
fn test() {
    let x = {
        incomplete block  // Missing closing brace
```

### MixedBlockStyles
**Message**: `"Mixed block styles are not allowed in the same block"`  
**Context**: Both braced and indented styles used in same block  
**Default Suggestion**: "Use either braces {...} OR indentation consistently within a single block"

```ferra
// ❌ Error
fn test() {
    if condition:     // Indented style
        do_something()
    }                 // Mixed with braced style

// ✅ Fix (consistent braces)
fn test() {
    if condition {
        do_something();
    }
}

// ✅ Fix (consistent indentation)  
fn test():
    if condition:
        do_something()
```

### InconsistentIndentation
**Message**: `"Inconsistent indentation"`  
**Context**: Indentation levels don't match within indented block  
**Default Suggestion**: "All statements in an indented block must be at the same level (expected {expected} spaces)"

```ferra
// ❌ Error
if condition:
    statement1()      // 4 spaces
        statement2()  // 8 spaces - inconsistent

// ✅ Fix
if condition:
    statement1()      // 4 spaces
    statement2()      // 4 spaces - consistent
```

### ExpectedBlock
**Message**: `"Expected block (either braced or indented)"`  
**Context**: Parser expected a block but found something else  
**Default Suggestion**: "Consider adding a block (either braced or indented)"

```ferra
// ❌ Error
fn test()     // Missing function body
let x = if condition;  // Missing block after if

// ✅ Fix
fn test() {
    // function body
}
let x = if condition { value } else { default };
```

### InvalidIndentation
**Message**: `"Invalid indentation level"`  
**Context**: Indentation level is invalid for context  
**Default Suggestion**: "Check the indentation level of the block"

---

## Scope and Redefinition Errors

### VariableRedefinition
**Message**: `"Variable '{variable}' is already defined in this scope"`  
**Context**: Attempting to redefine a variable in the same scope  
**No Default Suggestion**: Context-specific

```ferra
// ❌ Error
fn test() {
    let x = 1;
    let x = 2;  // Redefinition in same scope
}

// ✅ Fix
fn test() {
    let x = 1;
    let y = 2;  // Different variable name
}
// OR
fn test() {
    let x = 1;
    {
        let x = 2;  // Different scope
    }
}
```

---

## File and Parsing Errors

### UnexpectedEof
**Message**: `"Unexpected end of file"`  
**Context**: Parser expected more tokens but reached end of file  
**Expected Field**: What was expected before EOF

```ferra
// ❌ Error
fn test() {
    let x = 42;
// Missing closing brace - UnexpectedEof
```

---

## System Errors

### Internal (Fatal)
**Message**: `"Internal parser error: {message}"`  
**Context**: Internal parser state is inconsistent  
**Default Code**: `I001`  
**Severity**: Fatal (stops parsing)

**Recovery**: These should never occur in production. If they do, it's a parser bug.

### SyntaxError (Generic)
**Message**: `"Syntax error: {message}"`  
**Context**: Generic syntax error with custom message  
**Default Code**: `E001`  
**Severity**: Error

### RecoveryError (Warning)
**Message**: `"Recovery error: {message}"`  
**Context**: Error recovery mechanism failed  
**Default Code**: `R001`  
**Severity**: Warning
**Contains**: Original error that triggered recovery

---

## Error Code Assignment

### Current Implementation
The parser supports optional error codes but doesn't assign them by default except for:
- `SyntaxError`: `E001` 
- `Internal`: `I001`
- `RecoveryError`: `R001`

### Error Code Patterns
- **E###**: General parser errors
- **I###**: Internal parser errors  
- **R###**: Recovery-related errors

### Setting Custom Error Codes
```rust
// In parser code
let error = ParseError::unexpected_token("identifier", &token)
    .with_error_code("E042");

// All ParseError variants support .with_error_code()
```

---

## Error Recovery Strategies

### Recovery Points
The parser attempts recovery at these synchronization points:
- Statement boundaries: `;`, `{`, `}`
- Declaration keywords: `fn`, `let`, `var`, `data`, `extern`
- Control flow keywords: `if`, `while`, `for`, `match`
- Top-level construct boundaries

### Recovery Behavior
1. **Warning/Error Severity**: Continue parsing after collecting error
2. **Fatal Severity**: Stop parsing immediately  
3. **Recovery Errors**: Indicate when recovery itself fails

---

## Positive-First Messaging Principles

### Language Guidelines

#### Instead of Blame Language:
- ❌ "Syntax error"
- ❌ "Invalid input"  
- ❌ "Parse failed"
- ❌ "Unexpected token"

#### Use Constructive Language:
- ✅ "Expected {valid_option}, found {actual}"
- ✅ "Add {missing_element} to complete {construct}"
- ✅ "Use {correct_syntax} for {intended_purpose}"
- ✅ "Consider {alternative_approach}"

### Error Message Structure
All error messages include:
1. **Clear Description**: What went wrong
2. **Location Information**: Span with line/column  
3. **Suggestion**: How to fix it (when applicable)
4. **Severity**: Impact level
5. **Error Code**: Classification (optional)

---

## Diagnostic Integration

### miette Integration
The parser integrates with `miette` for rich diagnostic output:

```rust
// Example diagnostic formatting
error: Expected expression, found 'if'
  ┌─ input.ferra:3:5
  │
3 │     if {
  │     ^^ Expected expression here
  │
  = help: Consider adding a condition before the block

[E001]
```

### Diagnostic Report Structure
```rust
pub struct DiagnosticReport {
    pub errors: Vec<ParseError>,
    pub source_name: Option<String>,
    pub success: bool,
}
```

---

## Testing Error Messages

All error types are tested for:
- **Clarity**: Message is understandable
- **Actionability**: Provides concrete fix suggestions  
- **Consistency**: Follows positive-first principles
- **Accuracy**: Error location is precise
- **Code Coverage**: All ParseError variants tested

### Test Pattern Example
```rust
#[test]
fn test_expected_expression_error() {
    let source = "if { }";  // Missing condition
    let result = parse_expression(source);
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    
    // Verify error type
    assert!(matches!(error, ParseError::ExpectedExpression { .. }));
    
    // Verify positive messaging
    assert!(error.suggestion().unwrap().contains("Consider"));
    assert!(!error.to_string().contains("invalid")); // No blame language
}
```

---

## Error Frequency Analysis (From Tests)

Based on the 429 parser tests, the most common error patterns are:

1. **UnexpectedToken** (~40% of errors): Token mismatches
2. **ExpectedExpression** (~15% of errors): Missing expressions
3. **ExpectedBlock** (~12% of errors): Missing blocks
4. **MixedBlockStyles** (~10% of errors): Block style conflicts
5. **ExpectedStatement** (~8% of errors): Missing statements
6. **Other variants** (~15% of errors): Specialized cases

---

## Migration from Legacy P-Codes

### Legacy System (Deprecated)
The previous P001-P999 error code system has been replaced with the current ParseError enum-based system for better type safety and maintainability.

### Migration Benefits
- **Type Safety**: Compile-time error variant checking
- **Better Recovery**: Structured error information
- **Positive Messaging**: Built-in suggestion system
- **Flexible Severity**: Warning/Error/Fatal levels
- **Structured Data**: Rich error context

---

**Usage**: Reference this catalog when encountering parser errors. Each error includes context, suggested fixes, and examples to help resolve issues quickly and effectively. All error information is based on the actual ParseError implementation in the codebase. 