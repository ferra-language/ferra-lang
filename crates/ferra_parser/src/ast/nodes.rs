//! AST node type definitions
//!
//! Defines the structure of AST nodes representing parsed Ferra code.
//! Detailed implementation will be done during development phase.

use crate::token::{Span, Token};

/// Top-level compilation unit (represents a complete source file)
#[derive(Debug, Clone)]
pub struct CompilationUnit {
    pub items: Vec<Item>,
    pub span: Span,
}

/// Top-level items (declarations)
#[derive(Debug, Clone)]
pub enum Item {
    FunctionDecl(FunctionDecl),
    VariableDecl(VariableDecl),
    DataClassDecl(DataClassDecl),
    ExternBlock(ExternBlock),
}

/// Function declaration
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub generics: Option<GenericParams>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Option<Block>,
    pub is_async: bool,
    pub is_extern: bool,
    pub abi: Option<String>,
    pub modifiers: Modifiers,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Variable declaration
#[derive(Debug, Clone)]
pub struct VariableDecl {
    pub name: String,
    pub var_type: Option<Type>,
    pub initializer: Option<Expression>,
    pub is_mutable: bool,
    pub modifiers: Modifiers,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Data class declaration
#[derive(Debug, Clone)]
pub struct DataClassDecl {
    pub name: String,
    pub generics: Option<GenericParams>,
    pub fields: Vec<Field>,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Data class field
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// External block for FFI
#[derive(Debug, Clone)]
pub struct ExternBlock {
    pub abi: String,
    pub items: Vec<ExternItem>,
    pub span: Span,
}

/// External item (function or variable)
#[derive(Debug, Clone)]
pub enum ExternItem {
    Function(ExternFunction),
    Variable(ExternVariable),
}

/// External function declaration
#[derive(Debug, Clone)]
pub struct ExternFunction {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub span: Span,
}

/// External variable declaration
#[derive(Debug, Clone)]
pub struct ExternVariable {
    pub name: String,
    pub var_type: Type,
    pub span: Span,
}

/// Statement types
#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    VariableDecl(VariableDecl),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Return(ReturnStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
    Block(Block),
}

/// If statement
#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

/// While loop
#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
    pub span: Span,
}

/// For loop
#[derive(Debug, Clone)]
pub struct ForStatement {
    pub variable: String,
    pub iterable: Expression,
    pub body: Block,
    pub span: Span,
}

/// Return statement
#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
    pub span: Span,
}

/// Break statement
#[derive(Debug, Clone)]
pub struct BreakStatement {
    pub span: Span,
}

/// Continue statement
#[derive(Debug, Clone)]
pub struct ContinueStatement {
    pub span: Span,
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub is_braced: bool, // true for {}, false for indented
    pub span: Span,
    // Phase 2.4 enhancements
    pub scope_depth: usize,
    pub is_unsafe: bool,
    pub is_async: bool,
    pub is_try: bool,
    pub label: Option<String>,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            statements: Vec::new(),
            is_braced: true,
            span: Span::dummy(),
            scope_depth: 0,
            is_unsafe: false,
            is_async: false,
            is_try: false,
            label: None,
        }
    }
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    QualifiedIdentifier(QualifiedIdentifier),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Call(CallExpression),
    MemberAccess(MemberAccessExpression),
    Index(IndexExpression),
    Await(AwaitExpression),
    Array(ArrayLiteral),
    Tuple(TupleLiteral),
    If(IfExpression),
    Match(MatchExpression),
    Grouped(Box<Expression>),
    Block(BlockExpression), // Phase 2.4 addition
    Macro(MacroInvocation), // Phase 2.8.4: Macro invocations
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

/// Qualified identifier (e.g., module.function)
#[derive(Debug, Clone)]
pub struct QualifiedIdentifier {
    pub parts: Vec<String>,
    pub span: Span,
}

/// Binary expression
#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    NullCoalesce,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

/// Unary expression
#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
    pub span: Span,
}

/// Unary operators
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    Minus,
    Plus,
}

/// Function call expression
#[derive(Debug, Clone)]
pub struct CallExpression {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

/// Member access expression
#[derive(Debug, Clone)]
pub struct MemberAccessExpression {
    pub object: Box<Expression>,
    pub member: String,
    pub span: Span,
}

/// If expression
#[derive(Debug, Clone)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub then_expr: Box<Expression>,
    pub else_expr: Option<Box<Expression>>,
    pub span: Span,
}

/// Match expression
#[derive(Debug, Clone)]
pub struct MatchExpression {
    pub scrutinee: Box<Expression>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub expression: Expression,
    pub span: Span,
}

/// Pattern types for match expressions
#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Wildcard,
    DataClass(DataClassPattern),
    Range(RangePattern),     // Phase 2.8.3: Range patterns (1..=10)
    Slice(SlicePattern),     // Phase 2.8.3: Slice patterns ([head, tail @ ..])
    Or(OrPattern),           // Phase 2.8.3: Or patterns (Some(x) | None)
    Guard(GuardPattern),     // Phase 2.8.3: Guard patterns (x if x > 0)
    Binding(BindingPattern), // Phase 2.8.3: Binding patterns (name @ pattern)
}

/// Data class pattern
#[derive(Debug, Clone)]
pub struct DataClassPattern {
    pub name: String,
    pub fields: Vec<FieldPattern>,
    pub has_rest: bool,
    pub span: Span,
}

/// Field pattern in data class destructuring
#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Option<Pattern>,
    pub span: Span,
}

/// Range pattern for numeric ranges
#[derive(Debug, Clone)]
pub struct RangePattern {
    pub start: Option<Box<Pattern>>, // None for open ranges like ..=5
    pub end: Option<Box<Pattern>>,   // None for open ranges like 5..
    pub inclusive: bool,             // true for ..=, false for ..
    pub span: Span,
}

/// Slice pattern for array destructuring
#[derive(Debug, Clone)]
pub struct SlicePattern {
    pub prefix: Vec<Pattern>, // Patterns before the rest element
    pub rest: Option<String>, // Variable name for rest element (tail @ ..)
    pub suffix: Vec<Pattern>, // Patterns after the rest element
    pub span: Span,
}

/// Or pattern for matching multiple patterns
#[derive(Debug, Clone)]
pub struct OrPattern {
    pub patterns: Vec<Pattern>, // List of alternative patterns
    pub span: Span,
}

/// Guard pattern with conditional expression
#[derive(Debug, Clone)]
pub struct GuardPattern {
    pub pattern: Box<Pattern>, // Base pattern to match
    pub guard: Expression,     // Guard condition
    pub span: Span,
}

/// Binding pattern for named pattern matching
#[derive(Debug, Clone)]
pub struct BindingPattern {
    pub name: String,          // Variable name to bind to
    pub pattern: Box<Pattern>, // Pattern to match
    pub span: Span,
}

/// Type expressions
#[derive(Debug, Clone)]
pub enum Type {
    Identifier(String),
    Generic(GenericType),
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Function(FunctionType),
    Pointer(PointerType),
}

/// Generic type with type parameters (e.g., Vec<T>, HashMap<K, V>)
#[derive(Debug, Clone)]
pub struct GenericType {
    pub base: String,
    pub args: Vec<Type>,
    pub span: Span,
}

/// Function type
#[derive(Debug, Clone)]
pub struct FunctionType {
    pub parameters: Vec<Type>,
    pub return_type: Box<Type>,
    pub is_extern: bool,
    pub abi: Option<String>,
}

/// Pointer type
#[derive(Debug, Clone)]
pub struct PointerType {
    pub target: Box<Type>,
    pub is_mutable: bool,
}

/// Modifiers for declarations
#[derive(Debug, Clone, Default)]
pub struct Modifiers {
    pub is_public: bool,
    pub is_unsafe: bool,
}

/// Attribute for declarations and expressions
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub arguments: Vec<String>,
    pub span: Span,
}

/// Index expression (arr[index])
#[derive(Debug, Clone)]
pub struct IndexExpression {
    pub object: Box<Expression>,
    pub index: Box<Expression>,
    pub span: Span,
}

/// Await expression (expr.await)
#[derive(Debug, Clone)]
pub struct AwaitExpression {
    pub expression: Box<Expression>,
    pub span: Span,
}

/// Array literal ([1, 2, 3])
#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub elements: Vec<Expression>,
    pub span: Span,
}

/// Tuple literal ((1, 2, 3))
#[derive(Debug, Clone)]
pub struct TupleLiteral {
    pub elements: Vec<Expression>,
    pub span: Span,
}

/// Block expression (Phase 2.4)
#[derive(Debug, Clone)]
pub struct BlockExpression {
    pub block: Block,
    pub value: Option<Box<Expression>>,
    pub span: Span,
}

/// Generic type parameter
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeBound>,
    pub default: Option<Type>,
    pub is_lifetime: bool,
    pub span: Span,
}

/// Type bound for generic constraints (T: Clone + Debug)
#[derive(Debug, Clone)]
pub struct TypeBound {
    pub trait_name: String,
    pub span: Span,
}

/// Where clause for complex generic constraints
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub constraints: Vec<WhereConstraint>,
    pub span: Span,
}

/// Individual constraint in where clause
#[derive(Debug, Clone)]
pub struct WhereConstraint {
    pub type_name: String,
    pub bounds: Vec<TypeBound>,
    pub span: Span,
}

/// Generic parameters collection
#[derive(Debug, Clone)]
pub struct GenericParams {
    pub params: Vec<GenericParam>,
    pub where_clause: Option<WhereClause>,
    pub span: Span,
}

/// Macro invocation expression
#[derive(Debug, Clone)]
pub struct MacroInvocation {
    pub name: String,
    pub arguments: Vec<TokenTree>,
    pub span: Span,
}

/// Token tree for macro arguments
#[derive(Debug, Clone)]
pub enum TokenTree {
    Token(Token),
    Group(TokenGroup),
}

/// Grouped tokens in macro arguments
#[derive(Debug, Clone)]
pub struct TokenGroup {
    pub delimiter: GroupDelimiter,
    pub tokens: Vec<TokenTree>,
    pub span: Span,
}

/// Delimiters for token groups
#[derive(Debug, Clone)]
pub enum GroupDelimiter {
    Parentheses, // ()
    Brackets,    // []
    Braces,      // {}
}

/// Macro definition (basic framework)
#[derive(Debug, Clone)]
pub struct MacroDefinition {
    pub name: String,
    pub rules: Vec<MacroRule>,
    pub span: Span,
}

/// Macro rule for pattern matching
#[derive(Debug, Clone)]
pub struct MacroRule {
    pub pattern: Vec<TokenTree>,
    pub replacement: Vec<TokenTree>,
    pub span: Span,
}

impl Statement {
    /// Get the span of this statement
    pub fn span(&self) -> Span {
        match self {
            Statement::Expression(expr) => expr.span(),
            Statement::VariableDecl(var_decl) => var_decl.span.clone(),
            Statement::If(if_stmt) => if_stmt.span.clone(),
            Statement::While(while_stmt) => while_stmt.span.clone(),
            Statement::For(for_stmt) => for_stmt.span.clone(),
            Statement::Return(return_stmt) => return_stmt.span.clone(),
            Statement::Break(break_stmt) => break_stmt.span.clone(),
            Statement::Continue(continue_stmt) => continue_stmt.span.clone(),
            Statement::Block(block) => block.span.clone(),
        }
    }
}

impl Expression {
    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expression::Literal(_) => Span::dummy(), // Literals would have their own spans
            Expression::Identifier(_) => Span::dummy(),
            Expression::QualifiedIdentifier(qi) => qi.span.clone(),
            Expression::Binary(binary) => binary.span.clone(),
            Expression::Unary(unary) => unary.span.clone(),
            Expression::Call(call) => call.span.clone(),
            Expression::MemberAccess(member) => member.span.clone(),
            Expression::Index(index) => index.span.clone(),
            Expression::Await(await_expr) => await_expr.span.clone(),
            Expression::Array(array) => array.span.clone(),
            Expression::Tuple(tuple) => tuple.span.clone(),
            Expression::If(if_expr) => if_expr.span.clone(),
            Expression::Match(match_expr) => match_expr.span.clone(),
            Expression::Grouped(_) => Span::dummy(),
            Expression::Block(block_expr) => block_expr.span.clone(),
            Expression::Macro(macro_invocation) => macro_invocation.span.clone(),
        }
    }
}

impl Pattern {
    /// Get the span for this pattern
    pub fn span(&self) -> Span {
        match self {
            Pattern::Literal(literal) => match literal {
                Literal::String(_)
                | Literal::Integer(_)
                | Literal::Float(_)
                | Literal::Boolean(_) => {
                    // For literals, we'd need to track spans better - for now return dummy
                    Span::dummy()
                }
            },
            Pattern::Identifier(_) => Span::dummy(), // Would need proper span tracking
            Pattern::Wildcard => Span::dummy(),
            Pattern::DataClass(dc) => dc.span.clone(),
            Pattern::Range(r) => r.span.clone(),
            Pattern::Slice(s) => s.span.clone(),
            Pattern::Or(o) => o.span.clone(),
            Pattern::Guard(g) => g.span.clone(),
            Pattern::Binding(b) => b.span.clone(),
        }
    }
}
