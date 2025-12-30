use crate::{Card, Hand};
use gnurandom::GnuRandom;

/// Represents the four positions at a bridge table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Position {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Position {
    /// All positions in standard order
    pub const ALL: [Position; 4] = [
        Position::North,
        Position::East,
        Position::South,
        Position::West,
    ];

    /// Convert from index (0-3)
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Position::North),
            1 => Some(Position::East),
            2 => Some(Position::South),
            3 => Some(Position::West),
            _ => None,
        }
    }

    /// Get position as a character (N, E, S, W)
    pub fn to_char(&self) -> char {
        match self {
            Position::North => 'N',
            Position::East => 'E',
            Position::South => 'S',
            Position::West => 'W',
        }
    }

    /// Get partner position
    pub fn partner(&self) -> Position {
        match self {
            Position::North => Position::South,
            Position::South => Position::North,
            Position::East => Position::West,
            Position::West => Position::East,
        }
    }
}

/// Represents a complete bridge deal (4 hands of 13 cards each)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deal {
    pub north: Hand,
    pub east: Hand,
    pub south: Hand,
    pub west: Hand,
}

impl Deal {
    /// Create a new empty deal
    pub fn new() -> Self {
        Deal {
            north: Hand::new(),
            east: Hand::new(),
            south: Hand::new(),
            west: Hand::new(),
        }
    }

    /// Get a reference to a hand by position
    pub fn hand(&self, position: Position) -> &Hand {
        match position {
            Position::North => &self.north,
            Position::East => &self.east,
            Position::South => &self.south,
            Position::West => &self.west,
        }
    }

    /// Get a mutable reference to a hand by position
    pub fn hand_mut(&mut self, position: Position) -> &mut Hand {
        match position {
            Position::North => &mut self.north,
            Position::East => &mut self.east,
            Position::South => &mut self.south,
            Position::West => &mut self.west,
        }
    }

    /// Sort all hands in the deal
    pub fn sort_all_hands(&mut self) {
        self.north.sort();
        self.east.sort();
        self.south.sort();
        self.west.sort();
    }
}

impl Default for Deal {
    fn default() -> Self {
        Self::new()
    }
}

/// Generator for creating random bridge deals
pub struct DealGenerator {
    rng: GnuRandom,
}

impl DealGenerator {
    /// Create a new deal generator with a given seed
    pub fn new(seed: u32) -> Self {
        let mut rng = GnuRandom::new();
        rng.srandom(seed);
        DealGenerator { rng }
    }

    /// Generate a random deal using Fisher-Yates shuffle
    /// This matches the dealer.exe algorithm
    pub fn generate(&mut self) -> Deal {
        // Create array of 52 cards (indices 0-51)
        let mut deck: [u8; 52] = [0; 52];
        for i in 0..52 {
            deck[i] = i as u8;
        }

        // Fisher-Yates shuffle
        for i in (1..52).rev() {
            // Generate random index from 0 to i (inclusive)
            let j = (self.rng.next_u32() as usize) % (i + 1);
            deck.swap(i, j);
        }

        // Distribute cards to hands
        // First 13 to North, next 13 to East, next 13 to South, last 13 to West
        let mut deal = Deal::new();

        for (idx, &card_index) in deck.iter().enumerate() {
            let card = Card::from_index(card_index).unwrap();
            let position = Position::from_index((idx / 13) as u8).unwrap();
            deal.hand_mut(position).add_card(card);
        }

        // Sort all hands for display
        deal.sort_all_hands();

        deal
    }

    /// Generate multiple deals
    pub fn generate_many(&mut self, count: usize) -> Vec<Deal> {
        (0..count).map(|_| self.generate()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deal_generation() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();

        // Each hand should have exactly 13 cards
        assert_eq!(deal.north.len(), 13);
        assert_eq!(deal.east.len(), 13);
        assert_eq!(deal.south.len(), 13);
        assert_eq!(deal.west.len(), 13);

        // Total HCP should be 40
        let total_hcp = deal.north.hcp() + deal.east.hcp() + deal.south.hcp() + deal.west.hcp();
        assert_eq!(total_hcp, 40);
    }

    #[test]
    fn test_deterministic_generation() {
        // Same seed should produce same deal
        let mut gen1 = DealGenerator::new(42);
        let mut gen2 = DealGenerator::new(42);

        let deal1 = gen1.generate();
        let deal2 = gen2.generate();

        assert_eq!(deal1, deal2);
    }

    #[test]
    fn test_different_seeds_different_deals() {
        let mut gen1 = DealGenerator::new(1);
        let mut gen2 = DealGenerator::new(2);

        let deal1 = gen1.generate();
        let deal2 = gen2.generate();

        // Different seeds should (almost certainly) produce different deals
        assert_ne!(deal1, deal2);
    }

    #[test]
    fn test_all_cards_distributed() {
        let mut gen = DealGenerator::new(123);
        let deal = gen.generate();

        // Collect all cards from all hands
        let mut all_cards = Vec::new();
        all_cards.extend_from_slice(deal.north.cards());
        all_cards.extend_from_slice(deal.east.cards());
        all_cards.extend_from_slice(deal.south.cards());
        all_cards.extend_from_slice(deal.west.cards());

        // Should have exactly 52 unique cards
        assert_eq!(all_cards.len(), 52);

        // Convert to indices and sort
        let mut indices: Vec<u8> = all_cards.iter().map(|c| c.to_index()).collect();
        indices.sort();

        // Should be exactly 0..52
        for (i, &idx) in indices.iter().enumerate() {
            assert_eq!(i as u8, idx);
        }
    }

    #[test]
    fn test_partner_positions() {
        assert_eq!(Position::North.partner(), Position::South);
        assert_eq!(Position::South.partner(), Position::North);
        assert_eq!(Position::East.partner(), Position::West);
        assert_eq!(Position::West.partner(), Position::East);
    }
}
