//! Double Dummy Solver for Bridge
//!
//! This crate provides double-dummy analysis for bridge deals, calculating
//! the number of tricks that can be made by each side in each denomination
//! when all four hands are visible.

use dealer_core::{Card, Deal, Position, Suit};

/// New solver implementation (port of macroxue/bridge-solver)
/// Re-exported from bridge-solver crate
pub use bridge_solver as solver2;
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
    /// Total number of tricks in this game
    num_tricks: u8,
    /// Play history for debugging - fixed size array (position in high 2 bits, card index in low 6 bits)
    play_history: [u8; 52],
    /// Number of cards played (index into play_history)
    plays_count: u8,
}

impl GameState {
    fn new(deal: &Deal, declarer: Position, trump: Option<Suit>) -> Self {
        let mut hands = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for position in Position::ALL {
            hands[position as usize] = deal.hand(position).cards().to_vec();
        }

        // Opening lead comes from player to the LEFT of declarer
        let opening_leader = next_position(declarer);
        let num_tricks = hands[0].len() as u8;

        Self {
            hands,
            current_trick: TrickState::new(opening_leader, trump),
            declarer_tricks: 0,
            declarer,
            tricks_played: 0,
            num_tricks,
            play_history: [0; 52],
            plays_count: 0,
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

        // Record play: high 2 bits = position, low 6 bits = card index
        let encoded = ((player as u8) << 6) | card.to_index();
        self.play_history[self.plays_count as usize] = encoded;
        self.plays_count += 1;

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
        self.tricks_played >= self.num_tricks
    }

    /// Check if we're at a trick boundary (no cards in current trick)
    fn at_trick_boundary(&self) -> bool {
        self.current_trick.cards_played.is_empty()
    }

    /// Get the final score (tricks for declarer)
    fn score(&self) -> u8 {
        self.declarer_tricks
    }

    /// Get the play history as a Vec of (Position, Card)
    fn get_play_history(&self) -> Vec<(Position, Card)> {
        (0..self.plays_count as usize)
            .map(|i| {
                let encoded = self.play_history[i];
                let pos_idx = (encoded >> 6) as usize;
                let card_idx = encoded & 0x3F;
                let position = Position::ALL[pos_idx];
                let card = Card::from_index(card_idx).expect("valid card index");
                (position, card)
            })
            .collect()
    }

    /// Hash the game state for TT lookup
    /// Only valid at trick boundaries (no cards in current trick)
    fn hash(&self) -> u64 {
        let mut hash = 0u64;

        // Hash each hand's cards using bit rotation
        for (pos_idx, hand) in self.hands.iter().enumerate() {
            for card in hand {
                let card_bit = 1u64 << (card.to_index() as u64 % 52);
                hash ^= card_bit.rotate_left((pos_idx * 13) as u32);
            }
        }

        // Include leader and tricks won for uniqueness
        hash ^= (self.current_trick.leader as u64) << 56;
        hash ^= (self.declarer_tricks as u64) << 52;

        hash
    }
}

/// TT entry with proper bounds handling
#[derive(Clone, Copy, Debug)]
enum TTEntry {
    /// Exact value - can be returned directly
    Exact(u8),
    /// Lower bound (from beta cutoff) - actual value >= stored
    LowerBound(u8),
    /// Upper bound (from alpha cutoff) - actual value <= stored
    UpperBound(u8),
}

/// Transposition table for caching results
type TranspositionTable = HashMap<u64, TTEntry>;

/// Result with play sequence for debugging
#[derive(Debug, Clone)]
pub struct SolveResultWithLine {
    /// Number of tricks declarer can make
    pub tricks: u8,
    /// Play sequence that achieves this result: (position, card)
    pub play_line: Vec<(Position, Card)>,
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
    pub fn solve(&self, denomination: Denomination, declarer: Position) -> u8 {
        let trump = denomination.to_suit();
        let state = GameState::new(&self.deal, declarer, trump);
        let mut tt = HashMap::new();

        self.alpha_beta(&state, 0, state.num_tricks, &mut tt)
    }

    /// Solve and return a play line that achieves the result (for debugging)
    pub fn solve_with_line(
        &self,
        denomination: Denomination,
        declarer: Position,
    ) -> SolveResultWithLine {
        let trump = denomination.to_suit();
        let state = GameState::new(&self.deal, declarer, trump);
        let mut tt = HashMap::new();

        // First pass: find the optimal score
        let tricks = self.alpha_beta(&state, 0, state.num_tricks, &mut tt);

        // Second pass: find a line that achieves this score
        let play_line = self.find_line(&state, tricks, &mut tt);

        SolveResultWithLine { tricks, play_line }
    }

    /// Find a concrete play line that achieves the target score
    fn find_line(
        &self,
        state: &GameState,
        target: u8,
        tt: &mut TranspositionTable,
    ) -> Vec<(Position, Card)> {
        if let Some(terminal) = self.find_line_recursive(state, target, 0, state.num_tricks, tt) {
            terminal.get_play_history()
        } else {
            Vec::new()
        }
    }

    fn find_line_recursive(
        &self,
        state: &GameState,
        target: u8,
        alpha: u8,
        beta: u8,
        tt: &mut TranspositionTable,
    ) -> Option<GameState> {
        if state.is_terminal() {
            if state.score() == target {
                return Some(state.clone());
            }
            return None;
        }

        let maximizing = state.declarer_side_on_lead();
        let moves = state.legal_moves();

        for card in moves {
            let mut new_state = state.clone();
            new_state.play_card(card);

            // Check if this move can lead to target
            let score = self.alpha_beta(&new_state, alpha, beta, tt);

            let dominated = if maximizing {
                score >= target
            } else {
                score <= target
            };

            if dominated {
                if let Some(terminal) =
                    self.find_line_recursive(&new_state, target, alpha, beta, tt)
                {
                    return Some(terminal);
                }
            }
        }

        None
    }

    /// Alpha-beta minimax search with transposition table
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

        // TT lookup - only at trick boundaries for correctness
        let hash = if state.at_trick_boundary() {
            let h = state.hash();
            if let Some(&entry) = tt.get(&h) {
                match entry {
                    TTEntry::Exact(v) => return v,
                    TTEntry::LowerBound(v) => {
                        if v >= beta {
                            return v;
                        }
                        alpha = alpha.max(v);
                    }
                    TTEntry::UpperBound(v) => {
                        if v <= alpha {
                            return v;
                        }
                        beta = beta.min(v);
                    }
                }
            }
            Some(h)
        } else {
            None
        };

        let maximizing = state.declarer_side_on_lead();
        let moves = state.legal_moves();
        let orig_alpha = alpha;

        let value = if maximizing {
            let mut value = 0u8;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, alpha, beta, tt);
                value = value.max(score);
                alpha = alpha.max(value);
                if alpha >= beta {
                    break; // Beta cutoff
                }
            }
            value
        } else {
            let mut value = state.num_tricks;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, alpha, beta, tt);
                value = value.min(score);
                beta = beta.min(value);
                if alpha >= beta {
                    break; // Alpha cutoff
                }
            }
            value
        };

        // TT store - only at trick boundaries
        if let Some(h) = hash {
            let entry = if value <= orig_alpha {
                TTEntry::UpperBound(value)
            } else if value >= beta {
                TTEntry::LowerBound(value)
            } else {
                TTEntry::Exact(value)
            };
            tt.insert(h, entry);
        }

        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dealer_core::Rank;

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

    /// Create a simple deal where each hand has one suit (fast to solve)
    fn create_simple_deal() -> Deal {
        let ranks = [
            Rank::Ace,
            Rank::King,
            Rank::Queen,
            Rank::Jack,
            Rank::Ten,
            Rank::Nine,
            Rank::Eight,
            Rank::Seven,
            Rank::Six,
            Rank::Five,
            Rank::Four,
            Rank::Three,
            Rank::Two,
        ];
        let mut deal = Deal::new();
        for &rank in &ranks {
            deal.hand_mut(Position::North)
                .add_card(Card::new(Suit::Spades, rank));
        }
        for &rank in &ranks {
            deal.hand_mut(Position::East)
                .add_card(Card::new(Suit::Hearts, rank));
        }
        for &rank in &ranks {
            deal.hand_mut(Position::South)
                .add_card(Card::new(Suit::Diamonds, rank));
        }
        for &rank in &ranks {
            deal.hand_mut(Position::West)
                .add_card(Card::new(Suit::Clubs, rank));
        }
        deal
    }

    #[test]
    #[ignore] // Slow: runs DDS solver 20 times
    fn test_solver_creation() {
        let deal = create_simple_deal();
        let solver = DoubleDummySolver::new(deal);
        let result = solver.solve_all();
        assert_eq!(result.all_results().len(), 20); // 5 denominations × 4 positions
    }

    #[test]
    #[ignore] // Slow: runs DDS solver
    fn test_solver_basic() {
        // Test with a simple deal (one suit per hand)
        let deal = create_simple_deal();
        let solver = DoubleDummySolver::new(deal);

        // In NT with N declarer, E leads hearts, E/W win all tricks
        let tricks = solver.solve(Denomination::NoTrump, Position::North);
        assert_eq!(tricks, 0);

        // With Spades trump, N/S win all tricks (N has all spades)
        let tricks_spades = solver.solve(Denomination::Spades, Position::North);
        assert_eq!(tricks_spades, 13);
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
