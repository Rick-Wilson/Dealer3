use dealer_core::Position;

/// Abstract Syntax Tree for dealer constraints
/// This is Clone + Send + Sync so it can be shared across threads
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Binary operation: left op right
    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// Unary operation: op expr
    UnaryOp { op: UnaryOp, expr: Box<Expr> },

    /// Function call: func(args...)
    FunctionCall { func: Function, args: Vec<Expr> },

    /// Integer literal
    Literal(i32),

    /// Position identifier (north, south, east, west)
    Position(Position),

    /// Shape pattern for matching hand distributions
    ShapePattern(ShapePattern),
}

/// Shape pattern for hand distribution matching
#[derive(Debug, Clone, PartialEq)]
pub struct ShapePattern {
    /// List of shape specifications combined with + and -
    pub specs: Vec<ShapeSpec>,
}

/// A single shape specification (possibly with operators)
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeSpec {
    /// Whether this is included (+) or excluded (-)
    pub include: bool,
    /// The actual shape
    pub shape: Shape,
}

/// A shape distribution pattern
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// Exact shape: "5431" means exactly 5-4-3-1 in that suit order (S-H-D-C)
    Exact([u8; 4]),
    /// Wildcard shape: "54xx" means 5 spades, 4 hearts, any distribution in minors
    Wildcard([Option<u8>; 4]),
    /// Any distribution: "any 4333" means any hand with 4-3-3-3 distribution regardless of suit order
    AnyDistribution([u8; 4]),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Logical
    And,
    Or,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
}

/// Built-in functions for hand evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Function {
    /// High Card Points (A=4, K=3, Q=2, J=1)
    Hcp,

    /// Number of spades
    Spades,

    /// Number of hearts
    Hearts,

    /// Number of diamonds
    Diamonds,

    /// Number of clubs
    Clubs,

    /// Control count (A=2, K=1)
    Controls,

    /// Losers (future implementation)
    Losers,

    /// Winners (future implementation)
    Winners,

    /// Shape analysis (future implementation)
    Shape,

    /// Has specific card (future implementation)
    HasCard,
}

impl Function {
    /// Parse function name from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hcp" => Some(Function::Hcp),
            "spades" => Some(Function::Spades),
            "hearts" => Some(Function::Hearts),
            "diamonds" => Some(Function::Diamonds),
            "clubs" => Some(Function::Clubs),
            "controls" => Some(Function::Controls),
            "losers" => Some(Function::Losers),
            "winners" => Some(Function::Winners),
            "shape" => Some(Function::Shape),
            "hascard" => Some(Function::HasCard),
            _ => None,
        }
    }
}

impl Expr {
    /// Helper to create a binary operation
    pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Self {
        Expr::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper to create a unary operation
    pub fn unary(op: UnaryOp, expr: Expr) -> Self {
        Expr::UnaryOp {
            op,
            expr: Box::new(expr),
        }
    }

    /// Helper to create a function call with a single argument
    pub fn call(func: Function, arg: Expr) -> Self {
        Expr::FunctionCall {
            func,
            args: vec![arg],
        }
    }

    /// Helper to create a function call with multiple arguments
    pub fn call_multi(func: Function, args: Vec<Expr>) -> Self {
        Expr::FunctionCall { func, args }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_construction() {
        // Build AST for: hcp(north) >= 15
        let ast = Expr::binary(
            BinaryOp::Ge,
            Expr::call(Function::Hcp, Expr::Position(Position::North)),
            Expr::Literal(15),
        );

        match ast {
            Expr::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Ge),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_function_from_str() {
        assert_eq!(Function::from_str("hcp"), Some(Function::Hcp));
        assert_eq!(Function::from_str("hearts"), Some(Function::Hearts));
        assert_eq!(Function::from_str("HCP"), Some(Function::Hcp));
        assert_eq!(Function::from_str("invalid"), None);
    }
}
