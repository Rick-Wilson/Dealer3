use dealer_core::{Card, Deal, Position, Suit};
use dealer_parser::{BinaryOp, Expr, Function, Shape, ShapePattern, UnaryOp};

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

        Expr::FunctionCall { func, args } => eval_function(func, args, ctx),

        Expr::ShapePattern(_pattern) => {
            // Shape patterns shouldn't be evaluated directly, they're arguments to shape()
            Err(EvalError::InvalidArgument(
                "Shape pattern can only be used as argument to shape() function".to_string(),
            ))
        }

        Expr::Card(_card) => {
            // Cards shouldn't be evaluated directly, they're arguments to hascard()
            Err(EvalError::InvalidArgument(
                "Card can only be used as argument to hascard() function".to_string(),
            ))
        }

        Expr::Suit(_suit) => {
            // Suits shouldn't be evaluated directly, they're arguments to suit-specific functions
            Err(EvalError::InvalidArgument(
                "Suit can only be used as argument to functions like losers()".to_string(),
            ))
        }
    }
}

/// Evaluate a function call
fn eval_function(function: &Function, args: &[Expr], ctx: &EvalContext) -> Result<i32, EvalError> {
    match function {
        Function::Hcp => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgumentCount {
                    function: "hcp".to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            let position = eval_position_arg(&args[0], ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.hcp() as i32)
        }

        Function::Hearts => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgumentCount {
                    function: "hearts".to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            let position = eval_position_arg(&args[0], ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.suit_length(dealer_core::Suit::Hearts) as i32)
        }

        Function::Spades => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgumentCount {
                    function: "spades".to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            let position = eval_position_arg(&args[0], ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.suit_length(dealer_core::Suit::Spades) as i32)
        }

        Function::Diamonds => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgumentCount {
                    function: "diamonds".to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            let position = eval_position_arg(&args[0], ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.suit_length(dealer_core::Suit::Diamonds) as i32)
        }

        Function::Clubs => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgumentCount {
                    function: "clubs".to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            let position = eval_position_arg(&args[0], ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.suit_length(dealer_core::Suit::Clubs) as i32)
        }

        Function::Controls => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArgumentCount {
                    function: "controls".to_string(),
                    expected: 1,
                    got: args.len(),
                });
            }
            let position = eval_position_arg(&args[0], ctx)?;
            let hand = ctx.deal.hand(position);
            Ok(hand.controls() as i32)
        }

        Function::Shape => {
            if args.len() != 2 {
                return Err(EvalError::InvalidArgumentCount {
                    function: "shape".to_string(),
                    expected: 2,
                    got: args.len(),
                });
            }
            let position = eval_position_arg(&args[0], ctx)?;
            let pattern = match &args[1] {
                Expr::ShapePattern(p) => p,
                _ => {
                    return Err(EvalError::InvalidArgument(
                        "Second argument to shape() must be a shape pattern".to_string(),
                    ))
                }
            };

            let hand = ctx.deal.hand(position);
            let matches = eval_shape_pattern(hand, pattern)?;
            Ok(if matches { 1 } else { 0 })
        }

        Function::Losers => {
            if args.is_empty() || args.len() > 2 {
                return Err(EvalError::InvalidArgumentCount {
                    function: "losers".to_string(),
                    expected: 1, // or 2 with suit
                    got: args.len(),
                });
            }

            let position = eval_position_arg(&args[0], ctx)?;
            let hand = ctx.deal.hand(position);

            if args.len() == 1 {
                // Total losers in hand
                Ok(hand.losers() as i32)
            } else {
                // Losers in specific suit
                let suit = eval_suit_arg(&args[1])?;
                Ok(hand.losers_in_suit(suit) as i32)
            }
        }

        Function::HasCard => {
            if args.len() != 2 {
                return Err(EvalError::InvalidArgumentCount {
                    function: "hascard".to_string(),
                    expected: 2,
                    got: args.len(),
                });
            }

            let position = eval_position_arg(&args[0], ctx)?;
            let card = eval_card_arg(&args[1])?;
            let hand = ctx.deal.hand(position);

            Ok(if hand.has_card(card) { 1 } else { 0 })
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

/// Evaluate an argument that should be a suit
fn eval_suit_arg(arg: &Expr) -> Result<Suit, EvalError> {
    match arg {
        Expr::Suit(suit) => Ok(*suit),
        _ => Err(EvalError::InvalidArgument(
            "Expected suit (spades, hearts, diamonds, clubs)".to_string(),
        )),
    }
}

/// Evaluate an argument that should be a card
fn eval_card_arg(arg: &Expr) -> Result<Card, EvalError> {
    match arg {
        Expr::Card(card) => Ok(*card),
        _ => Err(EvalError::InvalidArgument(
            "Expected card (e.g., AS, KH, TC)".to_string(),
        )),
    }
}

/// Evaluate a shape pattern against a hand
fn eval_shape_pattern(
    hand: &dealer_core::Hand,
    pattern: &ShapePattern,
) -> Result<bool, EvalError> {
    let mut result = false;

    for spec in &pattern.specs {
        let matches = match &spec.shape {
            Shape::Exact(p) => hand.matches_exact_shape(p),
            Shape::Wildcard(p) => hand.matches_wildcard_shape(p),
            Shape::AnyDistribution(p) => hand.matches_distribution(p),
        };

        if spec.include {
            result = result || matches;
        } else {
            // Exclusion: if it matches, we fail
            if matches {
                return Ok(false);
            }
        }
    }

    Ok(result)
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

    #[test]
    fn test_shape_exact() {
        // Seed 1 produces north with 5-2-4-2 shape: AKQT3.J6.KJ42.95
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // North has 5 spades, 2 hearts, 4 diamonds, 2 clubs
        let ast = parse("shape(north, 5242)").unwrap();
        let result = eval(&ast, &ctx).unwrap();
        assert_eq!(result, 1);

        // Should not match different exact shape
        let ast = parse("shape(north, 5431)").unwrap();
        let result = eval(&ast, &ctx).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_shape_any_distribution() {
        // Find a hand with 4-3-3-3 distribution (balanced)
        let mut gen = DealGenerator::new(1);
        let mut found_4333 = false;

        for _ in 0..1000 {
            let deal = gen.generate();
            let north = deal.hand(Position::North);
            let dist = north.distribution();

            if dist == [4, 3, 3, 3] {
                let ctx = EvalContext::new(&deal);
                let ast = parse("shape(north, any 4333)").unwrap();
                let result = eval(&ast, &ctx).unwrap();
                assert_eq!(result, 1);
                found_4333 = true;
                break;
            }
        }

        assert!(found_4333, "Should find at least one 4-3-3-3 hand in 1000 deals");
    }

    #[test]
    fn test_shape_wildcard() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);
        let north = deal.hand(Position::North);
        let lengths = north.suit_lengths();

        // Build pattern 54xx dynamically based on actual hand
        if lengths[0] == 5 && lengths[1] == 4 {
            let ast = parse("shape(north, 54xx)").unwrap();
            let result = eval(&ast, &ctx).unwrap();
            assert_eq!(result, 1);
        }

        // Pattern that definitely won't match
        let ast = parse("shape(north, xx00)").unwrap(); // No voids in first 2 suits
        let result = eval(&ast, &ctx).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_shape_combination() {
        let mut gen = DealGenerator::new(42);
        let mut found = false;

        // Find a hand that's balanced (4333 or 4432 or 5332)
        for _ in 0..1000 {
            let deal = gen.generate();
            let north = deal.hand(Position::North);
            let dist = north.distribution();

            if matches!(dist, [4, 3, 3, 3] | [4, 4, 3, 2] | [5, 3, 3, 2]) {
                let ctx = EvalContext::new(&deal);
                let ast = parse("shape(north, any 4333 + any 4432 + any 5332)").unwrap();
                let result = eval(&ast, &ctx).unwrap();
                assert_eq!(result, 1);
                found = true;
                break;
            }
        }

        assert!(found, "Should find balanced hand in 1000 deals");
    }

    #[test]
    fn test_shape_exclusion() {
        // Test that exclusion pattern works
        let mut gen = DealGenerator::new(1);

        for _ in 0..100 {
            let deal = gen.generate();
            let ctx = EvalContext::new(&deal);
            let north = deal.hand(Position::North);
            let lengths = north.suit_lengths();

            // any 4333 - 4333 should match 4333 distributions that aren't exactly S=4,H=3,D=3,C=3
            let ast = parse("shape(north, any 4333 - 4333)").unwrap();
            let result = eval(&ast, &ctx).unwrap();

            let dist = north.distribution();
            let is_any_4333 = dist == [4, 3, 3, 3];
            let is_exact_4333 = lengths == [4, 3, 3, 3];

            if is_any_4333 && !is_exact_4333 {
                assert_eq!(result, 1, "Should match 4333 distributions except exact 4333");
            } else if is_exact_4333 {
                assert_eq!(result, 0, "Should not match exact 4333 (excluded)");
            }
        }
    }

    #[test]
    fn test_losers_total() {
        // Seed 1 north: AKQT3.J6.KJ42.95
        // Spades AKQ = 0, Hearts doubleton no honors = 2, Diamonds K = 2, Clubs doubleton no honors = 2
        // Total = 6 losers
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        let ast = parse("losers(north)").unwrap();
        let result = eval(&ast, &ctx).unwrap();
        assert_eq!(result, 6);
    }

    #[test]
    fn test_losers_in_suit() {
        // Test losers in a specific suit
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // North has AKQ in spades → 0 losers
        let ast = parse("losers(north, spades)").unwrap();
        let result = eval(&ast, &ctx).unwrap();
        assert_eq!(result, 0);

        // North has J6 in hearts → 2 losers (doubleton without A or K)
        let ast = parse("losers(north, hearts)").unwrap();
        let result = eval(&ast, &ctx).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn test_hascard() {
        // Seed 1 north: AKQT3.J6.KJ42.95
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let ctx = EvalContext::new(&deal);

        // North has AS
        let ast = parse("hascard(north, AS)").unwrap();
        let result = eval(&ast, &ctx).unwrap();
        assert_eq!(result, 1);

        // North doesn't have 2S
        let ast = parse("hascard(north, 2S)").unwrap();
        let result = eval(&ast, &ctx).unwrap();
        assert_eq!(result, 0);

        // North has KD
        let ast = parse("hascard(north, KD)").unwrap();
        let result = eval(&ast, &ctx).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_losers_various_holdings() {
        use dealer_core::{Card, Hand, Rank, Suit};

        // Test specific loser counts for known holdings
        let mut hand = Hand::new();

        // AKQxx in spades = 0 losers
        hand.add_card(Card::new(Suit::Spades, Rank::Ace));
        hand.add_card(Card::new(Suit::Spades, Rank::King));
        hand.add_card(Card::new(Suit::Spades, Rank::Queen));
        hand.add_card(Card::new(Suit::Spades, Rank::Jack));
        hand.add_card(Card::new(Suit::Spades, Rank::Ten));

        assert_eq!(hand.losers_in_suit(Suit::Spades), 0);

        // Singleton Ace = 0 losers
        let mut hand2 = Hand::new();
        hand2.add_card(Card::new(Suit::Hearts, Rank::Ace));
        assert_eq!(hand2.losers_in_suit(Suit::Hearts), 0);

        // Singleton King = 1 loser
        let mut hand3 = Hand::new();
        hand3.add_card(Card::new(Suit::Hearts, Rank::King));
        assert_eq!(hand3.losers_in_suit(Suit::Hearts), 1);

        // AK doubleton = 0 losers
        let mut hand4 = Hand::new();
        hand4.add_card(Card::new(Suit::Diamonds, Rank::Ace));
        hand4.add_card(Card::new(Suit::Diamonds, Rank::King));
        assert_eq!(hand4.losers_in_suit(Suit::Diamonds), 0);

        // Qx doubleton = 2 losers
        let mut hand5 = Hand::new();
        hand5.add_card(Card::new(Suit::Clubs, Rank::Queen));
        hand5.add_card(Card::new(Suit::Clubs, Rank::Two));
        assert_eq!(hand5.losers_in_suit(Suit::Clubs), 2);
    }
}
