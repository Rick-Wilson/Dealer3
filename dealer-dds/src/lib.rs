//! Double Dummy Solver for Bridge
//!
//! This crate provides double-dummy analysis for bridge deals, calculating
//! the number of tricks that can be made by each side in each denomination
//! when all four hands are visible.

use dealer_core::{Card, Deal, Position, Suit};
use std::collections::HashMap;

/// Helper function to get the next position in clockwise order
fn next_position(pos: Position) -> Position {
    match pos {
        Position::North => Position::East,
        Position::East => Position::South,
        Position::South => Position::West,
        Position::West => Position::North,
    }
}

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

    /// Convert to Suit (NoTrump returns None)
    pub fn to_suit(&self) -> Option<Suit> {
        match self {
            Denomination::Clubs => Some(Suit::Clubs),
            Denomination::Diamonds => Some(Suit::Diamonds),
            Denomination::Hearts => Some(Suit::Hearts),
            Denomination::Spades => Some(Suit::Spades),
            Denomination::NoTrump => None,
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
        Self {
            tricks: [[0; 4]; 5],
        }
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

/// Game state for a single trick in progress
#[derive(Clone, Debug)]
struct TrickState {
    cards_played: Vec<(Position, Card)>,
    leader: Position,
    trump: Option<Suit>,
}

impl TrickState {
    fn new(leader: Position, trump: Option<Suit>) -> Self {
        Self {
            cards_played: Vec::with_capacity(4),
            leader,
            trump,
        }
    }

    /// Get the suit led (if any cards played)
    fn suit_led(&self) -> Option<Suit> {
        self.cards_played.first().map(|(_, card)| card.suit)
    }

    /// Determine the winner of the current trick
    fn winner(&self) -> Option<Position> {
        if self.cards_played.len() < 4 {
            return None;
        }

        let suit_led = self.suit_led().unwrap();
        let mut winning_card = self.cards_played[0].1;
        let mut winning_pos = self.cards_played[0].0;

        for &(pos, card) in &self.cards_played[1..] {
            if self.beats(card, winning_card, suit_led) {
                winning_card = card;
                winning_pos = pos;
            }
        }

        Some(winning_pos)
    }

    /// Check if card1 beats card2
    fn beats(&self, card1: Card, card2: Card, suit_led: Suit) -> bool {
        // Trump beats non-trump
        if let Some(trump) = self.trump {
            if card1.suit == trump && card2.suit != trump {
                return true;
            }
            if card2.suit == trump && card1.suit != trump {
                return false;
            }
        }

        // Must follow suit
        if card1.suit == suit_led && card2.suit != suit_led {
            return true;
        }
        if card2.suit == suit_led && card1.suit != suit_led {
            return false;
        }

        // Same suit - compare ranks
        if card1.suit == card2.suit {
            return card1.rank > card2.rank;
        }

        false
    }
}

/// Complete game state for double-dummy solving
#[derive(Clone)]
struct GameState {
    /// Cards remaining in each hand (by position)
    hands: [Vec<Card>; 4],
    /// Current trick in progress
    current_trick: TrickState,
    /// Tricks won by declarer's side
    declarer_tricks: u8,
    /// Declarer position
    declarer: Position,
    /// Total tricks played so far
    tricks_played: u8,
}

impl GameState {
    fn new(deal: &Deal, declarer: Position, trump: Option<Suit>) -> Self {
        let mut hands = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for position in Position::ALL {
            hands[position as usize] = deal.hand(position).cards().to_vec();
        }

        Self {
            hands,
            current_trick: TrickState::new(declarer, trump),
            declarer_tricks: 0,
            declarer,
            tricks_played: 0,
        }
    }

    /// Get the next player to act
    fn next_player(&self) -> Position {
        let cards_played = self.current_trick.cards_played.len();
        if cards_played == 0 {
            self.current_trick.leader
        } else {
            let last_player = self.current_trick.cards_played[cards_played - 1].0;
            next_position(last_player)
        }
    }

    /// Check if declarer's side is on lead
    fn declarer_side_on_lead(&self) -> bool {
        let next = self.next_player();
        next == self.declarer || next == self.declarer.partner()
    }

    /// Get legal moves for the current player
    fn legal_moves(&self) -> Vec<Card> {
        let player = self.next_player();
        let hand = &self.hands[player as usize];

        if let Some(suit_led) = self.current_trick.suit_led() {
            // Must follow suit if possible
            let following: Vec<Card> = hand
                .iter()
                .filter(|c| c.suit == suit_led)
                .copied()
                .collect();
            if !following.is_empty() {
                return following;
            }
        }

        // Can play any card
        hand.clone()
    }

    /// Play a card and update state
    fn play_card(&mut self, card: Card) -> bool {
        let player = self.next_player();
        let hand = &mut self.hands[player as usize];

        // Remove card from hand
        if let Some(pos) = hand.iter().position(|&c| c == card) {
            hand.remove(pos);
        } else {
            return false; // Invalid move
        }

        self.current_trick.cards_played.push((player, card));

        // Check if trick is complete
        if self.current_trick.cards_played.len() == 4 {
            let winner = self.current_trick.winner().unwrap();

            // Award trick
            if winner == self.declarer || winner == self.declarer.partner() {
                self.declarer_tricks += 1;
            }

            self.tricks_played += 1;

            // Start new trick with winner leading
            self.current_trick = TrickState::new(winner, self.current_trick.trump);
        }

        true
    }

    /// Check if game is over
    fn is_terminal(&self) -> bool {
        self.tricks_played >= 13
    }

    /// Get the final score (tricks for declarer)
    fn score(&self) -> u8 {
        self.declarer_tricks
    }
}

/// Transposition table for caching results
type TranspositionTable = HashMap<u64, (u8, u8)>; // hash -> (alpha, beta)

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
    pub fn solve(&self, denomination: Denomination, declarer: Position) -> u8 {
        let trump = denomination.to_suit();
        let state = GameState::new(&self.deal, declarer, trump);
        let mut tt = HashMap::new();

        self.alpha_beta(&state, 0, 13, &mut tt)
    }

    /// Alpha-beta minimax search
    fn alpha_beta(
        &self,
        state: &GameState,
        mut alpha: u8,
        mut beta: u8,
        tt: &mut TranspositionTable,
    ) -> u8 {
        // Terminal node
        if state.is_terminal() {
            return state.score();
        }

        // Check transposition table
        let hash = self.hash_state(state);
        if let Some(&(cached_alpha, cached_beta)) = tt.get(&hash) {
            if cached_alpha >= beta {
                return cached_alpha;
            }
            if cached_beta <= alpha {
                return cached_beta;
            }
            alpha = alpha.max(cached_alpha);
            beta = beta.min(cached_beta);
        }

        let maximizing = state.declarer_side_on_lead();
        let moves = state.legal_moves();

        if maximizing {
            let mut value = alpha;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, value, beta, tt);
                value = value.max(score);
                if value >= beta {
                    break; // Beta cutoff
                }
            }
            tt.insert(hash, (value, beta));
            value
        } else {
            let mut value = beta;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, alpha, value, tt);
                value = value.min(score);
                if value <= alpha {
                    break; // Alpha cutoff
                }
            }
            tt.insert(hash, (alpha, value));
            value
        }
    }

    /// Simple hash function for game state
    fn hash_state(&self, state: &GameState) -> u64 {
        let mut hash = 0u64;

        // Hash hands (card presence)
        for (pos_idx, hand) in state.hands.iter().enumerate() {
            for card in hand {
                let card_idx = card.to_index() as u64;
                hash ^= card_idx.wrapping_mul((pos_idx as u64 + 1) * 13);
            }
        }

        // Hash current trick state
        hash ^= (state.current_trick.leader as u64) << 56;
        hash ^= (state.declarer_tricks as u64) << 48;

        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dealer_core::{DealGenerator, Rank};

    #[test]
    fn test_denomination_conversion() {
        assert_eq!(Denomination::from_suit(Suit::Spades), Denomination::Spades);
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

    #[test]
    fn test_solver_basic() {
        // Test with a known deal
        let mut gen = DealGenerator::new(42);
        let deal = gen.generate();
        let solver = DoubleDummySolver::new(deal);

        // Solve for spades with North as declarer
        let tricks = solver.solve(Denomination::Spades, Position::North);

        // Should return a valid number of tricks (0-13)
        assert!(tricks <= 13);
    }

    #[test]
    fn test_trick_winner() {
        let mut trick = TrickState::new(Position::North, Some(Suit::Spades));

        // Play a complete trick (Spades are trump)
        trick
            .cards_played
            .push((Position::North, Card::new(Suit::Hearts, Rank::Ace)));
        trick
            .cards_played
            .push((Position::East, Card::new(Suit::Spades, Rank::Two))); // Trump
        trick
            .cards_played
            .push((Position::South, Card::new(Suit::Hearts, Rank::King)));
        trick
            .cards_played
            .push((Position::West, Card::new(Suit::Hearts, Rank::Queen)));

        // East should win with the trump
        assert_eq!(trick.winner(), Some(Position::East));
    }
}
