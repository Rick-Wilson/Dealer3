/// Represents the four suits in bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Suit {
    Clubs = 0,
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
}

impl Suit {
    /// All suits in standard order
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    /// Convert from numeric index (0-3)
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Suit::Clubs),
            1 => Some(Suit::Diamonds),
            2 => Some(Suit::Hearts),
            3 => Some(Suit::Spades),
            _ => None,
        }
    }

    /// Get the suit as a character symbol
    pub fn symbol(&self) -> char {
        match self {
            Suit::Clubs => '♣',
            Suit::Diamonds => '♦',
            Suit::Hearts => '♥',
            Suit::Spades => '♠',
        }
    }

    /// Get the suit as a single character (C, D, H, S)
    pub fn to_char(&self) -> char {
        match self {
            Suit::Clubs => 'C',
            Suit::Diamonds => 'D',
            Suit::Hearts => 'H',
            Suit::Spades => 'S',
        }
    }
}

/// Represents card ranks from 2 to Ace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

impl Rank {
    /// All ranks from Two to Ace
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    /// Convert from numeric value (2-14)
    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            2 => Some(Rank::Two),
            3 => Some(Rank::Three),
            4 => Some(Rank::Four),
            5 => Some(Rank::Five),
            6 => Some(Rank::Six),
            7 => Some(Rank::Seven),
            8 => Some(Rank::Eight),
            9 => Some(Rank::Nine),
            10 => Some(Rank::Ten),
            11 => Some(Rank::Jack),
            12 => Some(Rank::Queen),
            13 => Some(Rank::King),
            14 => Some(Rank::Ace),
            _ => None,
        }
    }

    /// Get the rank as a character (2-9, T, J, Q, K, A)
    pub fn to_char(&self) -> char {
        match self {
            Rank::Two => '2',
            Rank::Three => '3',
            Rank::Four => '4',
            Rank::Five => '5',
            Rank::Six => '6',
            Rank::Seven => '7',
            Rank::Eight => '8',
            Rank::Nine => '9',
            Rank::Ten => 'T',
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
            Rank::Ace => 'A',
        }
    }

    /// Get HCP (High Card Points) value for this rank
    /// A=4, K=3, Q=2, J=1, others=0
    pub fn hcp(&self) -> u8 {
        match self {
            Rank::Ace => 4,
            Rank::King => 3,
            Rank::Queen => 2,
            Rank::Jack => 1,
            _ => 0,
        }
    }
}

/// Represents a single playing card
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    /// Create a new card
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Card { suit, rank }
    }

    /// Create a card from an index (0-51)
    /// Index is calculated as: suit * 13 + (rank - 2)
    /// This matches common deck representations
    pub fn from_index(index: u8) -> Option<Self> {
        if index >= 52 {
            return None;
        }
        let suit = Suit::from_index(index / 13)?;
        let rank = Rank::from_value((index % 13) + 2)?;
        Some(Card::new(suit, rank))
    }

    /// Convert card to index (0-51)
    pub fn to_index(&self) -> u8 {
        (self.suit as u8) * 13 + (self.rank as u8 - 2)
    }

    /// Get HCP value of this card
    pub fn hcp(&self) -> u8 {
        self.rank.hcp()
    }

    /// Format as a string (e.g., "S:A", "H:K", "C:2")
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.suit.to_char(), self.rank.to_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_index_conversion() {
        // Test all 52 cards
        for i in 0..52 {
            let card = Card::from_index(i).unwrap();
            assert_eq!(card.to_index(), i);
        }
    }

    #[test]
    fn test_hcp_values() {
        assert_eq!(Card::new(Suit::Spades, Rank::Ace).hcp(), 4);
        assert_eq!(Card::new(Suit::Hearts, Rank::King).hcp(), 3);
        assert_eq!(Card::new(Suit::Diamonds, Rank::Queen).hcp(), 2);
        assert_eq!(Card::new(Suit::Clubs, Rank::Jack).hcp(), 1);
        assert_eq!(Card::new(Suit::Spades, Rank::Seven).hcp(), 0);
    }

    #[test]
    fn test_suit_order() {
        assert!(Suit::Clubs < Suit::Diamonds);
        assert!(Suit::Diamonds < Suit::Hearts);
        assert!(Suit::Hearts < Suit::Spades);
    }

    #[test]
    fn test_rank_order() {
        assert!(Rank::Two < Rank::Three);
        assert!(Rank::King < Rank::Ace);
        assert!(Rank::Jack < Rank::Queen);
    }
}
