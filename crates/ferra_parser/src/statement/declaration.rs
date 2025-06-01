//! Declaration statement parsing (let, var, fn, data, extern)
//!
//! Implementation will be completed during development phase

use crate::{
    ast::{DataClassDecl, ExternBlock, FunctionDecl, VariableDecl},
    error::ParseResult,
    token::TokenStream,
};

/// Parse variable declarations (let/var)
pub fn parse_variable_declaration<T: TokenStream>(_tokens: &mut T) -> ParseResult<VariableDecl> {
    todo!("Implementation will be done during development phase")
}

/// Parse function declarations
pub fn parse_function_declaration<T: TokenStream>(_tokens: &mut T) -> ParseResult<FunctionDecl> {
    todo!("Implementation will be done during development phase")
}

/// Parse data class declarations
pub fn parse_data_class_declaration<T: TokenStream>(_tokens: &mut T) -> ParseResult<DataClassDecl> {
    todo!("Implementation will be done during development phase")
}

/// Parse extern blocks
pub fn parse_extern_block<T: TokenStream>(_tokens: &mut T) -> ParseResult<ExternBlock> {
    todo!("Implementation will be done during development phase")
}
