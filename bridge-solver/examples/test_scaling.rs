//! Test DD solver scaling from 3-card to full 13-card deals
//!
//! Tests progressively larger endgames to see where performance degrades

use dealer_core::{Card, Position, Rank, Suit};
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

    fn score(&self) -> u8 {
        self.declarer_tricks
    }
}

struct Solver {
    nodes_visited: u64,
}

impl Solver {
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
                if score > value {
                    value = score;
                }
                if value > alpha {
                    alpha = value;
                }
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
                if score < value {
                    value = score;
                }
                if value < beta {
                    beta = value;
                }
                if alpha >= beta {
                    break;
                }
            }
            value
        }
    }
}

/// Create a N-card deal where N/S have top N spades, E/W have top N hearts
fn create_simple_deal(n: usize) -> [Vec<Card>; 4] {
    let ranks = [
        Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Ten,
        Rank::Nine, Rank::Eight, Rank::Seven, Rank::Six, Rank::Five,
        Rank::Four, Rank::Three, Rank::Two,
    ];

    // Split N cards between N/S (spades) and E/W (hearts)
    let half = (n + 1) / 2;
    let other_half = n - half;

    let mut north = Vec::new();
    let mut south = Vec::new();
    let mut east = Vec::new();
    let mut west = Vec::new();

    // N gets top half of spades, S gets rest
    for i in 0..half {
        north.push(Card::new(Suit::Spades, ranks[i]));
    }
    for i in half..n {
        south.push(Card::new(Suit::Spades, ranks[i]));
    }

    // E gets top half of hearts, W gets rest
    for i in 0..half {
        east.push(Card::new(Suit::Hearts, ranks[i]));
    }
    for i in half..n {
        west.push(Card::new(Suit::Hearts, ranks[i]));
    }

    [north, east, south, west]
}

/// Create a more complex N-card deal with mixed suits
fn create_mixed_deal(n: usize) -> [Vec<Card>; 4] {
    let ranks = [
        Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Ten,
        Rank::Nine, Rank::Eight, Rank::Seven, Rank::Six, Rank::Five,
        Rank::Four, Rank::Three, Rank::Two,
    ];
    let suits = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

    let mut north = Vec::new();
    let mut east = Vec::new();
    let mut south = Vec::new();
    let mut west = Vec::new();

    // Distribute cards round-robin by rank, cycling through suits
    for i in 0..n {
        let rank = ranks[i % 13];
        let suit = suits[i % 4];
        let card = Card::new(suit, rank);

        match i % 4 {
            0 => north.push(card),
            1 => east.push(card),
            2 => south.push(card),
            3 => west.push(card),
            _ => unreachable!(),
        }
    }

    // Pad to equal sizes
    while north.len() < n / 4 + 1 || east.len() < n / 4 + 1 ||
          south.len() < n / 4 + 1 || west.len() < n / 4 + 1 {
        // This simple distribution might not be balanced, let's try another approach
    }

    [north, east, south, west]
}

fn main() {
    println!("DD Solver Scaling Test");
    println!("======================\n");

    // Test simple deal (spades vs hearts) - these scale well due to no following required
    println!("Test 1: Simple deals (N/S have spades, E/W have hearts)");
    println!("Expected: NT=0, Spades=N for all\n");

    for n in [3, 4, 5, 6, 7, 8] {
        let hands = create_simple_deal(n);
        let state = GameState::new(hands, Position::North, None);
        let mut solver = Solver::new();

        let start = Instant::now();
        let tricks = solver.solve(&state);
        let elapsed = start.elapsed();

        println!(
            "  {} cards: NT={} tricks, {} nodes, {:.3}ms",
            n, tricks, solver.nodes_visited, elapsed.as_secs_f64() * 1000.0
        );
    }

    println!("\n---\n");

    // Test with spades trump (should be fast - N/S always win)
    println!("Test 2: Same deals with Spades trump");
    println!("Expected: Spades=N for all\n");

    for n in [3, 4, 5, 6, 7, 8] {
        let hands = create_simple_deal(n);
        let state = GameState::new(hands, Position::North, Some(Suit::Spades));
        let mut solver = Solver::new();

        let start = Instant::now();
        let tricks = solver.solve(&state);
        let elapsed = start.elapsed();

        println!(
            "  {} cards: S={} tricks, {} nodes, {:.3}ms",
            n, tricks, solver.nodes_visited, elapsed.as_secs_f64() * 1000.0
        );
    }

    println!("\n---\n");

    // Now test with a deal where suit-following matters
    println!("Test 3: All-spade distribution (forces following)");
    println!("Creating deals where all hands have only spades\n");

    for n in [3, 4, 5, 6, 7, 8, 9, 10] {
        let ranks = [
            Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Ten,
            Rank::Nine, Rank::Eight, Rank::Seven, Rank::Six, Rank::Five,
            Rank::Four, Rank::Three, Rank::Two,
        ];

        // Give each hand n/4 spades (rounded)
        let cards_per_hand = n;
        let mut all_cards: Vec<Card> = (0..n*4)
            .map(|i| Card::new(Suit::Spades, ranks[i % 13]))
            .collect();

        // Actually we need unique cards, so let's use all 4 suits
        all_cards.clear();
        let suits = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];
        for suit in suits {
            for rank in ranks.iter().take(n) {
                all_cards.push(Card::new(suit, *rank));
            }
        }

        // Distribute to hands
        let hands = [
            all_cards[0..n].to_vec(),
            all_cards[n..2*n].to_vec(),
            all_cards[2*n..3*n].to_vec(),
            all_cards[3*n..4*n].to_vec(),
        ];

        let state = GameState::new(hands, Position::North, None);
        let mut solver = Solver::new();

        let start = Instant::now();
        let tricks = solver.solve(&state);
        let elapsed = start.elapsed();

        println!(
            "  {} cards: NT={} tricks, {} nodes, {:.3}ms",
            n, tricks, solver.nodes_visited, elapsed.as_secs_f64() * 1000.0
        );

        // Stop if taking too long
        if elapsed.as_secs() > 5 {
            println!("  (stopping - taking too long)");
            break;
        }
    }

    println!("\nDone!");
}
