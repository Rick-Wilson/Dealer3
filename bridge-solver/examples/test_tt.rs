//! Test TT implementation correctness
//!
//! Compare results with and without TT to verify TT doesn't break correctness

use dealer_core::{Card, Position, Rank, Suit};
use std::collections::HashMap;
use std::time::Instant;

fn next_position(pos: Position) -> Position {
    match pos {
        Position::North => Position::East,
        Position::East => Position::South,
        Position::South => Position::West,
        Position::West => Position::North,
    }
}

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

    fn suit_led(&self) -> Option<Suit> {
        self.cards_played.first().map(|(_, card)| card.suit)
    }

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

    fn beats(&self, card1: Card, card2: Card, suit_led: Suit) -> bool {
        if let Some(trump) = self.trump {
            if card1.suit == trump && card2.suit != trump {
                return true;
            }
            if card2.suit == trump && card1.suit != trump {
                return false;
            }
        }

        if card1.suit == suit_led && card2.suit != suit_led {
            return true;
        }
        if card2.suit == suit_led && card1.suit != suit_led {
            return false;
        }

        if card1.suit == card2.suit {
            return card1.rank > card2.rank;
        }

        false
    }
}

#[derive(Clone)]
struct GameState {
    hands: [Vec<Card>; 4],
    current_trick: TrickState,
    declarer_tricks: u8,
    declarer: Position,
    tricks_played: u8,
    num_tricks: u8,
}

impl GameState {
    fn new(hands: [Vec<Card>; 4], declarer: Position, trump: Option<Suit>) -> Self {
        let num_tricks = hands[0].len() as u8;
        let opening_leader = next_position(declarer);

        Self {
            hands,
            current_trick: TrickState::new(opening_leader, trump),
            declarer_tricks: 0,
            declarer,
            tricks_played: 0,
            num_tricks,
        }
    }

    fn next_player(&self) -> Position {
        let cards_played = self.current_trick.cards_played.len();
        if cards_played == 0 {
            self.current_trick.leader
        } else {
            let last_player = self.current_trick.cards_played[cards_played - 1].0;
            next_position(last_player)
        }
    }

    fn declarer_side_on_lead(&self) -> bool {
        let next = self.next_player();
        next == self.declarer || next == self.declarer.partner()
    }

    fn legal_moves(&self) -> Vec<Card> {
        let player = self.next_player();
        let hand = &self.hands[player as usize];

        if let Some(suit_led) = self.current_trick.suit_led() {
            let following: Vec<Card> = hand
                .iter()
                .filter(|c| c.suit == suit_led)
                .copied()
                .collect();
            if !following.is_empty() {
                return following;
            }
        }

        hand.clone()
    }

    fn play_card(&mut self, card: Card) {
        let player = self.next_player();
        let hand = &mut self.hands[player as usize];

        if let Some(pos) = hand.iter().position(|&c| c == card) {
            hand.remove(pos);
        }

        self.current_trick.cards_played.push((player, card));

        if self.current_trick.cards_played.len() == 4 {
            let winner = self.current_trick.winner().unwrap();

            if winner == self.declarer || winner == self.declarer.partner() {
                self.declarer_tricks += 1;
            }

            self.tricks_played += 1;
            self.current_trick = TrickState::new(winner, self.current_trick.trump);
        }
    }

    fn is_terminal(&self) -> bool {
        self.tricks_played >= self.num_tricks
    }

    fn at_trick_boundary(&self) -> bool {
        self.current_trick.cards_played.is_empty()
    }

    fn score(&self) -> u8 {
        self.declarer_tricks
    }

    /// Hash the game state for TT lookup
    /// Only valid at trick boundaries (no cards in current trick)
    fn hash(&self) -> u64 {
        let mut hash = 0u64;

        // Hash each hand's cards using a simple scheme
        for (pos_idx, hand) in self.hands.iter().enumerate() {
            for card in hand {
                // Use card index (0-51) and position to create unique contribution
                let card_bit = 1u64 << (card.to_index() as u64 % 52);
                hash ^= card_bit.rotate_left((pos_idx * 13) as u32);
            }
        }

        // Include leader and tricks won
        hash ^= (self.current_trick.leader as u64) << 56;
        hash ^= (self.declarer_tricks as u64) << 52;

        hash
    }
}

/// TT entry with bounds
#[derive(Clone, Copy, Debug)]
enum TTEntry {
    Exact(u8),
    LowerBound(u8),  // Alpha cutoff - actual value >= stored
    UpperBound(u8),  // Beta cutoff - actual value <= stored
}

struct SolverNoTT {
    nodes_visited: u64,
}

impl SolverNoTT {
    fn new() -> Self {
        Self { nodes_visited: 0 }
    }

    fn solve(&mut self, state: &GameState) -> u8 {
        self.nodes_visited = 0;
        self.alpha_beta(state, 0, state.num_tricks)
    }

    fn alpha_beta(&mut self, state: &GameState, mut alpha: u8, mut beta: u8) -> u8 {
        self.nodes_visited += 1;

        if state.is_terminal() {
            return state.score();
        }

        let maximizing = state.declarer_side_on_lead();
        let moves = state.legal_moves();

        if maximizing {
            let mut value = 0u8;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, alpha, beta);
                value = value.max(score);
                alpha = alpha.max(value);
                if alpha >= beta {
                    break;
                }
            }
            value
        } else {
            let mut value = state.num_tricks;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, alpha, beta);
                value = value.min(score);
                beta = beta.min(value);
                if alpha >= beta {
                    break;
                }
            }
            value
        }
    }
}

struct SolverWithTT {
    nodes_visited: u64,
    tt_hits: u64,
    tt: HashMap<u64, TTEntry>,
}

impl SolverWithTT {
    fn new() -> Self {
        Self {
            nodes_visited: 0,
            tt_hits: 0,
            tt: HashMap::new(),
        }
    }

    fn solve(&mut self, state: &GameState) -> u8 {
        self.nodes_visited = 0;
        self.tt_hits = 0;
        self.tt.clear();
        self.alpha_beta(state, 0, state.num_tricks)
    }

    fn alpha_beta(&mut self, state: &GameState, mut alpha: u8, mut beta: u8) -> u8 {
        self.nodes_visited += 1;

        if state.is_terminal() {
            return state.score();
        }

        // TT lookup - only at trick boundaries
        let hash = if state.at_trick_boundary() {
            let h = state.hash();
            if let Some(&entry) = self.tt.get(&h) {
                self.tt_hits += 1;
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
                let score = self.alpha_beta(&new_state, alpha, beta);
                value = value.max(score);
                alpha = alpha.max(value);
                if alpha >= beta {
                    break;
                }
            }
            value
        } else {
            let mut value = state.num_tricks;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, alpha, beta);
                value = value.min(score);
                beta = beta.min(value);
                if alpha >= beta {
                    break;
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
            self.tt.insert(h, entry);
        }

        value
    }
}

fn create_test_deal(n: usize) -> [Vec<Card>; 4] {
    let ranks = [
        Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Ten,
        Rank::Nine, Rank::Eight, Rank::Seven, Rank::Six, Rank::Five,
        Rank::Four, Rank::Three, Rank::Two,
    ];
    let suits = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

    let mut all_cards: Vec<Card> = Vec::new();
    for suit in suits {
        for &rank in ranks.iter().take(n) {
            all_cards.push(Card::new(suit, rank));
        }
    }

    [
        all_cards[0..n].to_vec(),
        all_cards[n..2*n].to_vec(),
        all_cards[2*n..3*n].to_vec(),
        all_cards[3*n..4*n].to_vec(),
    ]
}

fn main() {
    println!("TT Correctness Test");
    println!("===================\n");

    println!("Comparing results with and without TT\n");

    let mut all_match = true;

    // First test correctness on small sizes (with and without TT)
    for n in 3..=6 {
        let hands = create_test_deal(n);

        // Solve without TT
        let state1 = GameState::new(hands.clone(), Position::North, None);
        let mut solver1 = SolverNoTT::new();
        let start1 = Instant::now();
        let result1 = solver1.solve(&state1);
        let time1 = start1.elapsed();

        // Solve with TT
        let state2 = GameState::new(hands.clone(), Position::North, None);
        let mut solver2 = SolverWithTT::new();
        let start2 = Instant::now();
        let result2 = solver2.solve(&state2);
        let time2 = start2.elapsed();

        let matches = result1 == result2;
        if !matches {
            all_match = false;
        }

        println!(
            "{} cards: no_TT={} ({} nodes, {:.1}ms) | with_TT={} ({} nodes, {} hits, {:.1}ms) {}",
            n,
            result1, solver1.nodes_visited, time1.as_secs_f64() * 1000.0,
            result2, solver2.nodes_visited, solver2.tt_hits, time2.as_secs_f64() * 1000.0,
            if matches { "OK" } else { "MISMATCH!" }
        );
    }

    println!("\nTT-only tests (7-13 cards):");

    // Then test with TT only for larger sizes
    for n in 7..=13 {
        let hands = create_test_deal(n);
        let state = GameState::new(hands, Position::North, None);
        let mut solver = SolverWithTT::new();

        let start = Instant::now();
        let result = solver.solve(&state);
        let elapsed = start.elapsed();

        println!(
            "{:2} cards: with_TT={} ({} nodes, {} hits, {:.1}ms)",
            n, result, solver.nodes_visited, solver.tt_hits, elapsed.as_secs_f64() * 1000.0
        );

        if elapsed.as_secs() > 10 {
            println!("(stopping - taking too long)");
            break;
        }
    }

    println!();
    if all_match {
        println!("All results match - TT implementation is CORRECT!");
    } else {
        println!("ERRORS: TT produces different results!");
    }
}
