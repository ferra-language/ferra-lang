//! Precedence and binding power definitions for Pratt parser
//!
//! This module defines the binding power table that drives the Pratt parser,
//! based on the operator precedence table from SYNTAX_GRAMMAR_V0.1.md Appendix A.

use crate::token::TokenType;

/// Binding power for operators (higher = tighter binding)
pub type BindingPower = u8;

/// Associativity of operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
    None, // Non-associative (comparison chains not allowed)
}

/// Operator information for Pratt parsing
#[derive(Debug, Clone)]
pub struct OperatorInfo {
    pub binding_power: BindingPower,
    pub associativity: Associativity,
}

/// Get prefix binding power for tokens that can start expressions (NUD)
pub fn prefix_binding_power(token_type: &TokenType) -> Option<BindingPower> {
    match token_type {
        // Unary prefix operators (precedence level 15)
        TokenType::Minus | TokenType::Plus | TokenType::Bang => Some(150),

        // Primary expressions (highest precedence)
        TokenType::IntegerLiteral(_)
        | TokenType::FloatLiteral(_)
        | TokenType::StringLiteral(_)
        | TokenType::BooleanLiteral(_)
        | TokenType::Identifier(_)
        | TokenType::LeftParen => Some(160),

        _ => None,
    }
}

/// Get infix binding power for tokens that can continue expressions (LED)
pub fn infix_binding_power(token_type: &TokenType) -> Option<OperatorInfo> {
    match token_type {
        // Level 1: Assignment (right associative)
        TokenType::Equal
        | TokenType::PlusEqual
        | TokenType::MinusEqual
        | TokenType::StarEqual
        | TokenType::SlashEqual => Some(OperatorInfo {
            binding_power: 10,
            associativity: Associativity::Right,
        }),

        // Level 2: Logical OR (left associative)
        TokenType::PipePipe => Some(OperatorInfo {
            binding_power: 20,
            associativity: Associativity::Left,
        }),

        // Level 3: Logical AND (left associative)
        TokenType::AmpAmp => Some(OperatorInfo {
            binding_power: 30,
            associativity: Associativity::Left,
        }),

        // Level 4: Equality (left associative)
        TokenType::EqualEqual | TokenType::BangEqual => Some(OperatorInfo {
            binding_power: 40,
            associativity: Associativity::Left,
        }),

        // Level 5: Comparison (non-associative)
        TokenType::Less
        | TokenType::LessEqual
        | TokenType::Greater
        | TokenType::GreaterEqual => Some(OperatorInfo {
            binding_power: 50,
            associativity: Associativity::None,
        }),

        // Level 6: Additive (left associative)
        TokenType::Plus | TokenType::Minus => Some(OperatorInfo {
            binding_power: 60,
            associativity: Associativity::Left,
        }),

        // Level 7: Multiplicative (left associative)
        TokenType::Star | TokenType::Slash | TokenType::Percent => Some(OperatorInfo {
            binding_power: 70,
            associativity: Associativity::Left,
        }),

        // Level 8: Postfix operators (left associative, highest precedence)
        TokenType::Dot             // Member access: obj.field
        | TokenType::LeftParen     // Function call: func()
        | TokenType::LeftBracket   // Indexing: arr[i]
        | TokenType::Question      // Error propagation: expr?
        => Some(OperatorInfo {
            binding_power: 140,
            associativity: Associativity::Left,
        }),

        // Await operator (special case: .await)
        // This is handled specially in the lexer/parser as Dot + Identifier("await")

        _ => None,
    }
}

/// Check if a token can start an expression (has NUD handler)
pub fn can_start_expression(token_type: &TokenType) -> bool {
    prefix_binding_power(token_type).is_some()
}

/// Check if a token can continue an expression (has LED handler)
pub fn can_continue_expression(token_type: &TokenType) -> bool {
    infix_binding_power(token_type).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn test_precedence_ordering() {
        // Test that precedence levels are correctly ordered
        let assignment_bp = infix_binding_power(&TokenType::Equal)
            .unwrap()
            .binding_power;
        let or_bp = infix_binding_power(&TokenType::PipePipe)
            .unwrap()
            .binding_power;
        let and_bp = infix_binding_power(&TokenType::AmpAmp)
            .unwrap()
            .binding_power;
        let equality_bp = infix_binding_power(&TokenType::EqualEqual)
            .unwrap()
            .binding_power;
        let comparison_bp = infix_binding_power(&TokenType::Less).unwrap().binding_power;
        let additive_bp = infix_binding_power(&TokenType::Plus).unwrap().binding_power;
        let multiplicative_bp = infix_binding_power(&TokenType::Star).unwrap().binding_power;
        let postfix_bp = infix_binding_power(&TokenType::Dot).unwrap().binding_power;
        let prefix_bp = prefix_binding_power(&TokenType::Minus).unwrap();

        // Verify precedence ordering: assignment < or < and < equality < comparison < additive < multiplicative < postfix < prefix
        assert!(assignment_bp < or_bp);
        assert!(or_bp < and_bp);
        assert!(and_bp < equality_bp);
        assert!(equality_bp < comparison_bp);
        assert!(comparison_bp < additive_bp);
        assert!(additive_bp < multiplicative_bp);
        assert!(multiplicative_bp < postfix_bp);
        assert!(postfix_bp < prefix_bp);
    }

    #[test]
    fn test_associativity() {
        // Assignment is right associative
        assert_eq!(
            infix_binding_power(&TokenType::Equal)
                .unwrap()
                .associativity,
            Associativity::Right
        );

        // Arithmetic is left associative
        assert_eq!(
            infix_binding_power(&TokenType::Plus).unwrap().associativity,
            Associativity::Left
        );

        // Comparison is non-associative
        assert_eq!(
            infix_binding_power(&TokenType::Less).unwrap().associativity,
            Associativity::None
        );
    }

    #[test]
    fn test_prefix_precedence() {
        // Unary operators should have high precedence
        let unary_bp = prefix_binding_power(&TokenType::Minus).unwrap();
        let postfix_bp = infix_binding_power(&TokenType::Dot).unwrap().binding_power;

        assert!(unary_bp > postfix_bp);
    }
}
