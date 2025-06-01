//! AST visitor patterns for traversal and transformation
//!
//! Implementation will be completed during development phase

use super::*;

/// Trait for visiting AST nodes
pub trait Visitor<T> {
    fn visit_compilation_unit(&mut self, node: &CompilationUnit) -> T;
    fn visit_expression(&mut self, node: &Expression) -> T;
    fn visit_statement(&mut self, node: &Statement) -> T;
    fn visit_type(&mut self, node: &Type) -> T;

    // Default implementations can delegate to more specific methods
}

/// Mutable visitor trait for transforming AST nodes
pub trait MutVisitor<T> {
    fn visit_compilation_unit_mut(&mut self, node: &mut CompilationUnit) -> T;
    fn visit_expression_mut(&mut self, node: &mut Expression) -> T;
    fn visit_statement_mut(&mut self, node: &mut Statement) -> T;
    fn visit_type_mut(&mut self, node: &mut Type) -> T;
}

// Implementation will be completed during development phase
