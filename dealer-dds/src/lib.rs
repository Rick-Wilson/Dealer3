//! Double Dummy Solver for Bridge
//!
//! This crate provides double-dummy analysis for bridge deals, calculating
//! the number of tricks that can be made by each side in each denomination
//! when all four hands are visible.

use dealer_core::{Deal, Position, Suit};

/// Denomination for double-dummy analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Denomination {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    NoTrump,
}

impl Denomination {
    /// All five denominations
    pub const ALL: [Denomination; 5] = [
        Denomination::Clubs,
        Denomination::Diamonds,
        Denomination::Hearts,
        Denomination::Spades,
        Denomination::NoTrump,
    ];

    /// Convert from Suit
    pub fn from_suit(suit: Suit) -> Self {
        match suit {
            Suit::Clubs => Denomination::Clubs,
            Suit::Diamonds => Denomination::Diamonds,
            Suit::Hearts => Denomination::Hearts,
            Suit::Spades => Denomination::Spades,
        }
    }

    /// Convert to character representation
    pub fn to_char(&self) -> char {
        match self {
            Denomination::Clubs => 'C',
            Denomination::Diamonds => 'D',
            Denomination::Hearts => 'H',
            Denomination::Spades => 'S',
            Denomination::NoTrump => 'N',
        }
    }
}

/// Result of double-dummy analysis for a single denomination and declarer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrickResult {
    pub denomination: Denomination,
    pub declarer: Position,
    pub tricks: u8,
}

/// Complete double-dummy analysis result for all denominations and declarers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoubleDummyResult {
    /// Tricks by denomination and declarer
    /// Index: [denomination][declarer]
    tricks: [[u8; 4]; 5],
}

impl DoubleDummyResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self { tricks: [[0; 4]; 5] }
    }

    /// Set tricks for a specific denomination and declarer
    pub fn set_tricks(&mut self, denomination: Denomination, declarer: Position, tricks: u8) {
        let denom_idx = denomination as usize;
        let decl_idx = declarer as usize;
        self.tricks[denom_idx][decl_idx] = tricks;
    }

    /// Get tricks for a specific denomination and declarer
    pub fn get_tricks(&self, denomination: Denomination, declarer: Position) -> u8 {
        let denom_idx = denomination as usize;
        let decl_idx = declarer as usize;
        self.tricks[denom_idx][decl_idx]
    }

    /// Get all results as a vector of TrickResult
    pub fn all_results(&self) -> Vec<TrickResult> {
        let mut results = Vec::new();
        for denom in Denomination::ALL {
            for position in Position::ALL {
                results.push(TrickResult {
                    denomination: denom,
                    declarer: position,
                    tricks: self.get_tricks(denom, position),
                });
            }
        }
        results
    }
}

impl Default for DoubleDummyResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Solver for double-dummy analysis
pub struct DoubleDummySolver {
    deal: Deal,
}

impl DoubleDummySolver {
    /// Create a new solver for the given deal
    pub fn new(deal: Deal) -> Self {
        Self { deal }
    }

    /// Solve for all denominations and all declarers
    pub fn solve_all(&self) -> DoubleDummyResult {
        let mut result = DoubleDummyResult::new();

        for denomination in Denomination::ALL {
            for declarer in Position::ALL {
                let tricks = self.solve(denomination, declarer);
                result.set_tricks(denomination, declarer, tricks);
            }
        }

        result
    }

    /// Solve for a specific denomination and declarer
    pub fn solve(&self, _denomination: Denomination, _declarer: Position) -> u8 {
        // TODO: Implement actual double-dummy solver
        // For now, return a placeholder value
        let _ = &self.deal;
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dealer_core::DealGenerator;

    #[test]
    fn test_denomination_conversion() {
        assert_eq!(
            Denomination::from_suit(Suit::Spades),
            Denomination::Spades
        );
        assert_eq!(Denomination::from_suit(Suit::Hearts), Denomination::Hearts);
        assert_eq!(
            Denomination::from_suit(Suit::Diamonds),
            Denomination::Diamonds
        );
        assert_eq!(Denomination::from_suit(Suit::Clubs), Denomination::Clubs);
    }

    #[test]
    fn test_denomination_to_char() {
        assert_eq!(Denomination::Spades.to_char(), 'S');
        assert_eq!(Denomination::Hearts.to_char(), 'H');
        assert_eq!(Denomination::Diamonds.to_char(), 'D');
        assert_eq!(Denomination::Clubs.to_char(), 'C');
        assert_eq!(Denomination::NoTrump.to_char(), 'N');
    }

    #[test]
    fn test_double_dummy_result() {
        let mut result = DoubleDummyResult::new();
        result.set_tricks(Denomination::Spades, Position::North, 10);
        assert_eq!(result.get_tricks(Denomination::Spades, Position::North), 10);
    }

    #[test]
    fn test_solver_creation() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();
        let solver = DoubleDummySolver::new(deal);
        let result = solver.solve_all();
        assert_eq!(result.all_results().len(), 20); // 5 denominations × 4 positions
    }
}
