use dealer_core::{Deal, Position};
use dealer_parser::{BinaryOp, Expr, Function, UnaryOp};

/// Evaluation error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    /// Function requires specific number of arguments
    InvalidArgumentCount { function: String, expected: usize, got: usize },
    /// Invalid argument type or value
    InvalidArgument(String),
    /// Function not yet implemented
    NotImplemented(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EvalError::InvalidArgumentCount { function, expected, got } => {
                write!(f, "Function {} expects {} arguments, got {}", function, expected, got)
            }
            EvalError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            EvalError::NotImplemented(feature) => write!(f, "Not implemented: {}", feature),
        }
    }
}

impl std::error::Error for EvalError {}

/// Evaluation context - holds the deal being evaluated
pub struct EvalContext<'a> {
    pub deal: &'a Deal,
}

impl<'a> EvalContext<'a> {
    pub fn new(deal: &'a Deal) -> Self {
        EvalContext { deal }
    }
}

/// Evaluate an expression against a deal
pub fn eval(expr: &Expr, ctx: &EvalContext) -> Result<i32, EvalError> {
    match expr {
        Expr::Literal(value) => Ok(*value),

        Expr::Position(pos) => {
            // Position as a value - not very useful on its own, but valid
            // We'll return the position index (0-3)
            Ok(*pos as i32)
        }

        Expr::BinaryOp { op, left, right } => {
            let left_val = eval(left, ctx)?;
            let right_val = eval(right, ctx)?;

            match op {
                // Arithmetic
                BinaryOp::Add => Ok(left_val + right_val),
                BinaryOp::Sub => Ok(left_val - right_val),
                BinaryOp::Mul => Ok(left_val * right_val),
                BinaryOp::Div => {
                    if right_val == 0 {
                        Err(EvalError::InvalidArgument("Division by zero".to_string()))
                    } else {
                        Ok(left_val / right_val)
                    }
                }
                BinaryOp::Mod => {
                    if right_val == 0 {
                        Err(EvalError::InvalidArgument("Modulo by zero".to_string()))
                    } else {
                        Ok(left_val % right_val)
                    }
                }

                // Comparison (return 1 for true, 0 for false)
                BinaryOp::Eq => Ok(if left_val == right_val { 1 } else { 0 }),
                BinaryOp::Ne => Ok(if left_val != right_val { 1 } else { 0 }),
                BinaryOp::Lt => Ok(if left_val < right_val { 1 } else { 0 }),
                BinaryOp::Le => Ok(if left_val <= right_val { 1 } else { 0 }),
                BinaryOp::Gt => Ok(if left_val > right_val { 1 } else { 0 }),
                BinaryOp::Ge => Ok(if left_val >= right_val { 1 } else { 0 }),

                // Logical (treat 0 as false, non-zero as true)
                BinaryOp::And => Ok(if left_val != 0 && right_val != 0 { 1 } else { 0 }),
                BinaryOp::Or => Ok(if left_val != 0 || right_val != 0 { 1 } else { 0 }),
            }
        }

        Expr::UnaryOp { op, expr } => {
            let val = eval(expr, ctx)?;
            match op {
                UnaryOp::Negate => Ok(-val),
                UnaryOp::Not => Ok(if val == 0 { 1 } else { 0 }),
            }
        }

        Expr::FunctionCall { func, arg } => eval_function(func, arg, ctx),
    }
}

/// Evaluate a function call
fn eval_function(function: &Function, arg: &Expr, ctx: &EvalContext) -> Result<i32, EvalError> {
    match function {
        Function::Hcp => {
            let position = eval_position_arg(arg, ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.hcp() as i32)
        }

        Function::Hearts => {
            let position = eval_position_arg(arg, ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.suit_length(dealer_core::Suit::Hearts) as i32)
        }

        Function::Spades => {
            let position = eval_position_arg(arg, ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.suit_length(dealer_core::Suit::Spades) as i32)
        }

        Function::Diamonds => {
            let position = eval_position_arg(arg, ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.suit_length(dealer_core::Suit::Diamonds) as i32)
        }

        Function::Clubs => {
            let position = eval_position_arg(arg, ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.suit_length(dealer_core::Suit::Clubs) as i32)
        }

        Function::Controls => {
            let position = eval_position_arg(arg, ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.controls() as i32)
        }

        // Not yet implemented
        Function::Losers | Function::Winners | Function::Shape
        | Function::HasCard => {
            Err(EvalError::NotImplemented(format!("{:?}", function)))
        }
    }
}

/// Evaluate an argument that should be a position
fn eval_position_arg(arg: &Expr, _ctx: &EvalContext) -> Result<Position, EvalError> {
    match arg {
        Expr::Position(pos) => Ok(*pos),
        _ => Err(EvalError::InvalidArgument(
            "Expected position (north, south, east, west)".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dealer_core::{DealGenerator, Suit};
    use dealer_parser::parse;

    #[test]
    fn test_eval_literal() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        let expr = Expr::Literal(42);
        assert_eq!(eval(&expr, &ctx).unwrap(), 42);
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // 5 + 3
        let expr = Expr::binary(
            BinaryOp::Add,
            Expr::Literal(5),
            Expr::Literal(3),
        );
        assert_eq!(eval(&expr, &ctx).unwrap(), 8);

        // 10 - 4
        let expr = Expr::binary(
            BinaryOp::Sub,
            Expr::Literal(10),
            Expr::Literal(4),
        );
        assert_eq!(eval(&expr, &ctx).unwrap(), 6);

        // 6 * 7
        let expr = Expr::binary(
            BinaryOp::Mul,
            Expr::Literal(6),
            Expr::Literal(7),
        );
        assert_eq!(eval(&expr, &ctx).unwrap(), 42);
    }

    #[test]
    fn test_eval_comparison() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // 5 > 3 (true)
        let expr = Expr::binary(
            BinaryOp::Gt,
            Expr::Literal(5),
            Expr::Literal(3),
        );
        assert_eq!(eval(&expr, &ctx).unwrap(), 1);

        // 5 < 3 (false)
        let expr = Expr::binary(
            BinaryOp::Lt,
            Expr::Literal(5),
            Expr::Literal(3),
        );
        assert_eq!(eval(&expr, &ctx).unwrap(), 0);
    }

    #[test]
    fn test_eval_logical() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // 1 && 1 (true)
        let expr = Expr::binary(
            BinaryOp::And,
            Expr::Literal(1),
            Expr::Literal(1),
        );
        assert_eq!(eval(&expr, &ctx).unwrap(), 1);

        // 1 && 0 (false)
        let expr = Expr::binary(
            BinaryOp::And,
            Expr::Literal(1),
            Expr::Literal(0),
        );
        assert_eq!(eval(&expr, &ctx).unwrap(), 0);

        // 0 || 1 (true)
        let expr = Expr::binary(
            BinaryOp::Or,
            Expr::Literal(0),
            Expr::Literal(1),
        );
        assert_eq!(eval(&expr, &ctx).unwrap(), 1);
    }

    #[test]
    fn test_eval_hcp_function() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // Get north's HCP
        let north_hand = deal.hand(Position::North);
        let expected_hcp = north_hand.hcp() as i32;

        let expr = Expr::call(Function::Hcp, Expr::Position(Position::North));
        assert_eq!(eval(&expr, &ctx).unwrap(), expected_hcp);
    }

    #[test]
    fn test_eval_suit_length_functions() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        let south_hand = deal.hand(Position::South);

        // hearts(south)
        let expr = Expr::call(Function::Hearts, Expr::Position(Position::South));
        assert_eq!(eval(&expr, &ctx).unwrap(), south_hand.suit_length(Suit::Hearts) as i32);

        // spades(south)
        let expr = Expr::call(Function::Spades, Expr::Position(Position::South));
        assert_eq!(eval(&expr, &ctx).unwrap(), south_hand.suit_length(Suit::Spades) as i32);

        // diamonds(south)
        let expr = Expr::call(Function::Diamonds, Expr::Position(Position::South));
        assert_eq!(eval(&expr, &ctx).unwrap(), south_hand.suit_length(Suit::Diamonds) as i32);

        // clubs(south)
        let expr = Expr::call(Function::Clubs, Expr::Position(Position::South));
        assert_eq!(eval(&expr, &ctx).unwrap(), south_hand.suit_length(Suit::Clubs) as i32);
    }

    #[test]
    fn test_eval_parsed_constraint() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // Parse and evaluate: hcp(north) >= 15
        let ast = parse("hcp(north) >= 15").unwrap();
        let result = eval(&ast, &ctx).unwrap();

        let north_hcp = deal.hand(Position::North).hcp();
        let expected = if north_hcp >= 15 { 1 } else { 0 };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_complex_constraint() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // Parse and evaluate: hearts(north) >= 5 && hcp(south) <= 13
        let ast = parse("hearts(north) >= 5 && hcp(south) <= 13").unwrap();
        let result = eval(&ast, &ctx).unwrap();

        let north_hearts = deal.hand(Position::North).suit_length(Suit::Hearts);
        let south_hcp = deal.hand(Position::South).hcp();
        let expected = if north_hearts >= 5 && south_hcp <= 13 { 1 } else { 0 };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_arithmetic_combination() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // Parse and evaluate: hcp(north) + hcp(south) >= 25
        let ast = parse("hcp(north) + hcp(south) >= 25").unwrap();
        let result = eval(&ast, &ctx).unwrap();

        let north_hcp = deal.hand(Position::North).hcp();
        let south_hcp = deal.hand(Position::South).hcp();
        let expected = if north_hcp + south_hcp >= 25 { 1 } else { 0 };
        assert_eq!(result, expected);
    }
}
