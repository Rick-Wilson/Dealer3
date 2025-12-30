use crate::{Card, Rank, Suit};

/// Represents a single player's hand of 13 cards
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    /// Create a new empty hand
    pub fn new() -> Self {
        Hand { cards: Vec::new() }
    }

    /// Create a hand from a vector of cards
    pub fn from_cards(cards: Vec<Card>) -> Self {
        Hand { cards }
    }

    /// Add a card to the hand
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Get all cards in the hand
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// Get the number of cards in the hand
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Check if the hand is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Count cards of a specific suit
    pub fn suit_length(&self, suit: Suit) -> usize {
        self.cards.iter().filter(|c| c.suit == suit).count()
    }

    /// Get all cards of a specific suit
    pub fn cards_in_suit(&self, suit: Suit) -> Vec<Card> {
        self.cards
            .iter()
            .filter(|c| c.suit == suit)
            .copied()
            .collect()
    }

    /// Calculate total High Card Points (HCP)
    /// A=4, K=3, Q=2, J=1
    pub fn hcp(&self) -> u8 {
        self.cards.iter().map(|c| c.hcp()).sum()
    }

    /// Get the distribution pattern as a sorted array [longest to shortest]
    /// E.g., [5, 4, 3, 1] for a 5-4-3-1 hand
    pub fn distribution(&self) -> [usize; 4] {
        let mut lengths = [
            self.suit_length(Suit::Spades),
            self.suit_length(Suit::Hearts),
            self.suit_length(Suit::Diamonds),
            self.suit_length(Suit::Clubs),
        ];
        lengths.sort_by(|a, b| b.cmp(a)); // Sort descending
        lengths
    }

    /// Get the shape as a sorted string (e.g., "5-4-3-1")
    pub fn shape(&self) -> String {
        let dist = self.distribution();
        format!("{}-{}-{}-{}", dist[0], dist[1], dist[2], dist[3])
    }

    /// Check if hand is balanced (4-3-3-3, 4-4-3-2, or 5-3-3-2)
    pub fn is_balanced(&self) -> bool {
        let dist = self.distribution();
        matches!(
            dist,
            [4, 3, 3, 3] | [4, 4, 3, 2] | [5, 3, 3, 2]
        )
    }

    /// Count controls (A=2, K=1)
    pub fn controls(&self) -> u8 {
        self.cards
            .iter()
            .map(|c| match c.rank {
                Rank::Ace => 2,
                Rank::King => 1,
                _ => 0,
            })
            .sum()
    }

    /// Count honors (A, K, Q, J, T) in a specific suit
    pub fn honors_in_suit(&self, suit: Suit) -> u8 {
        self.cards
            .iter()
            .filter(|c| c.suit == suit && c.rank >= Rank::Ten)
            .count() as u8
    }

    /// Sort the hand by suit (spades first) and rank (high to low)
    pub fn sort(&mut self) {
        self.cards.sort_by(|a, b| {
            // Sort by suit descending (Spades first)
            match b.suit.cmp(&a.suit) {
                std::cmp::Ordering::Equal => {
                    // Within same suit, sort by rank descending (Ace first)
                    b.rank.cmp(&a.rank)
                }
                other => other,
            }
        });
    }

    /// Get a sorted copy of the hand
    pub fn sorted(&self) -> Hand {
        let mut hand = self.clone();
        hand.sort();
        hand
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hcp_calculation() {
        let mut hand = Hand::new();
        hand.add_card(Card::new(Suit::Spades, Rank::Ace)); // 4
        hand.add_card(Card::new(Suit::Hearts, Rank::King)); // 3
        hand.add_card(Card::new(Suit::Diamonds, Rank::Queen)); // 2
        hand.add_card(Card::new(Suit::Clubs, Rank::Jack)); // 1
        hand.add_card(Card::new(Suit::Spades, Rank::Seven)); // 0

        assert_eq!(hand.hcp(), 10);
    }

    #[test]
    fn test_suit_length() {
        let mut hand = Hand::new();
        hand.add_card(Card::new(Suit::Spades, Rank::Ace));
        hand.add_card(Card::new(Suit::Spades, Rank::King));
        hand.add_card(Card::new(Suit::Spades, Rank::Queen));
        hand.add_card(Card::new(Suit::Hearts, Rank::Ace));
        hand.add_card(Card::new(Suit::Hearts, Rank::King));

        assert_eq!(hand.suit_length(Suit::Spades), 3);
        assert_eq!(hand.suit_length(Suit::Hearts), 2);
        assert_eq!(hand.suit_length(Suit::Diamonds), 0);
        assert_eq!(hand.suit_length(Suit::Clubs), 0);
    }

    #[test]
    fn test_balanced_hand() {
        let mut hand = Hand::new();
        // Create a 4-3-3-3 hand
        for _ in 0..4 {
            hand.add_card(Card::new(Suit::Spades, Rank::Two));
        }
        for _ in 0..3 {
            hand.add_card(Card::new(Suit::Hearts, Rank::Two));
        }
        for _ in 0..3 {
            hand.add_card(Card::new(Suit::Diamonds, Rank::Two));
        }
        for _ in 0..3 {
            hand.add_card(Card::new(Suit::Clubs, Rank::Two));
        }

        assert!(hand.is_balanced());
        assert_eq!(hand.distribution(), [4, 3, 3, 3]);
    }

    #[test]
    fn test_controls() {
        let mut hand = Hand::new();
        hand.add_card(Card::new(Suit::Spades, Rank::Ace)); // 2
        hand.add_card(Card::new(Suit::Hearts, Rank::King)); // 1
        hand.add_card(Card::new(Suit::Diamonds, Rank::Ace)); // 2

        assert_eq!(hand.controls(), 5);
    }
}
