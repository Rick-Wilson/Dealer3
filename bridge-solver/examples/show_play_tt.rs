//! Show example card play from endgame - WITH transposition table
//!
//! This is a variant of show_play.rs that uses TT to verify correctness

use dealer_core::{Card, Position, Rank, Suit};
use std::collections::HashMap;

fn next_position(pos: Position) -> Position {
    match pos {
        Position::North => Position::East,
        Position::East => Position::South,
        Position::South => Position::West,
        Position::West => Position::North,
    }
}

fn pos_char(pos: Position) -> char {
    match pos {
        Position::North => 'N',
        Position::East => 'E',
        Position::South => 'S',
        Position::West => 'W',
    }
}

fn card_str(card: Card) -> String {
    let suit = match card.suit {
        Suit::Spades => 'S',
        Suit::Hearts => 'H',
        Suit::Diamonds => 'D',
        Suit::Clubs => 'C',
    };
    let rank = match card.rank {
        Rank::Ace => 'A',
        Rank::King => 'K',
        Rank::Queen => 'Q',
        Rank::Jack => 'J',
        Rank::Ten => 'T',
        Rank::Nine => '9',
        Rank::Eight => '8',
        Rank::Seven => '7',
        Rank::Six => '6',
        Rank::Five => '5',
        Rank::Four => '4',
        Rank::Three => '3',
        Rank::Two => '2',
    };
    format!("{}{}", rank, suit)
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
    // Track the play history
    play_history: Vec<(Position, Card)>,
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
            play_history: Vec::new(),
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

        self.play_history.push((player, card));
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

/// TT entry with bounds
#[derive(Clone, Copy, Debug)]
enum TTEntry {
    Exact(u8),
    LowerBound(u8),  // Alpha cutoff - actual value >= stored
    UpperBound(u8),  // Beta cutoff - actual value <= stored
}

type TranspositionTable = HashMap<u64, TTEntry>;

struct Solver {
    best_line: Option<GameState>,
    target_score: u8,
    tt: TranspositionTable,
    tt_hits: u64,
    nodes_visited: u64,
}

impl Solver {
    fn new() -> Self {
        Self {
            best_line: None,
            target_score: 0,
            tt: HashMap::new(),
            tt_hits: 0,
            nodes_visited: 0,
        }
    }

    fn solve(&mut self, state: &GameState) -> u8 {
        self.tt.clear();
        self.tt_hits = 0;
        self.nodes_visited = 0;

        // First pass: find the optimal score
        let score = self.alpha_beta(state, 0, state.num_tricks);
        self.target_score = score;

        println!(
            "  [TT stats: {} nodes, {} TT hits, {} TT entries]",
            self.nodes_visited,
            self.tt_hits,
            self.tt.len()
        );

        // Second pass: find a line that achieves this score
        self.find_line(state, 0, state.num_tricks);

        score
    }

    fn alpha_beta(&mut self, state: &GameState, mut alpha: u8, mut beta: u8) -> u8 {
        self.nodes_visited += 1;

        if state.is_terminal() {
            return state.score();
        }

        // TT lookup - only at trick boundaries for correctness
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

    // Find a concrete line that achieves target_score
    fn find_line(&mut self, state: &GameState, alpha: u8, beta: u8) -> bool {
        if state.is_terminal() {
            if state.score() == self.target_score {
                self.best_line = Some(state.clone());
                return true;
            }
            return false;
        }

        let maximizing = state.declarer_side_on_lead();
        let moves = state.legal_moves();

        for card in moves {
            let mut new_state = state.clone();
            new_state.play_card(card);

            // Check if this move can lead to target
            let score = self.alpha_beta(&new_state, alpha, beta);

            if maximizing {
                if score >= self.target_score {
                    if self.find_line(&new_state, alpha, beta) {
                        return true;
                    }
                }
            } else {
                if score <= self.target_score {
                    if self.find_line(&new_state, alpha, beta) {
                        return true;
                    }
                }
            }
        }

        false
    }
}

fn print_hands(hands: &[Vec<Card>; 4]) {
    println!("         North");
    print!("         ");
    for c in &hands[0] { print!("{} ", card_str(*c)); }
    println!();
    println!();

    print!("West              East");
    println!();
    print!("{:<18}", hands[3].iter().map(|c| card_str(*c)).collect::<Vec<_>>().join(" "));
    println!("{}", hands[1].iter().map(|c| card_str(*c)).collect::<Vec<_>>().join(" "));
    println!();

    println!("         South");
    print!("         ");
    for c in &hands[2] { print!("{} ", card_str(*c)); }
    println!();
}

fn main() {
    println!("Endgame with Example Play - TT VERSION");
    println!("======================================\n");

    // Create a 6-card position
    // North: SA HA DA CA ST HT (aces + tens)
    // East:  SK HK DK CK S9 H9 (kings + nines)
    // South: SQ HQ DQ CQ D9 C9 (queens + nines)
    // West:  SJ HJ DJ CJ DT CT (jacks + tens)
    //
    // North declarer in NT, East leads
    // N/S should make 6 tricks (all aces + winning tens)

    let hands = [
        vec![  // North
            Card::new(Suit::Spades, Rank::Ace),
            Card::new(Suit::Hearts, Rank::Ace),
            Card::new(Suit::Diamonds, Rank::Ace),
            Card::new(Suit::Clubs, Rank::Ace),
            Card::new(Suit::Spades, Rank::Ten),
            Card::new(Suit::Hearts, Rank::Ten),
        ],
        vec![  // East
            Card::new(Suit::Spades, Rank::King),
            Card::new(Suit::Hearts, Rank::King),
            Card::new(Suit::Diamonds, Rank::King),
            Card::new(Suit::Clubs, Rank::King),
            Card::new(Suit::Spades, Rank::Nine),
            Card::new(Suit::Hearts, Rank::Nine),
        ],
        vec![  // South
            Card::new(Suit::Spades, Rank::Queen),
            Card::new(Suit::Hearts, Rank::Queen),
            Card::new(Suit::Diamonds, Rank::Queen),
            Card::new(Suit::Clubs, Rank::Queen),
            Card::new(Suit::Diamonds, Rank::Nine),
            Card::new(Suit::Clubs, Rank::Nine),
        ],
        vec![  // West
            Card::new(Suit::Spades, Rank::Jack),
            Card::new(Suit::Hearts, Rank::Jack),
            Card::new(Suit::Diamonds, Rank::Jack),
            Card::new(Suit::Clubs, Rank::Jack),
            Card::new(Suit::Diamonds, Rank::Ten),
            Card::new(Suit::Clubs, Rank::Ten),
        ],
    ];

    println!("Position 1: Aces+tens vs kings+nines vs queens+nines vs jacks+tens (6 cards)");
    println!();
    print_hands(&hands);
    println!();
    println!("North declarer in NT, East leads");
    println!();

    let state = GameState::new(hands.clone(), Position::North, None);
    let mut solver = Solver::new();
    let tricks = solver.solve(&state);

    println!("Optimal result: N/S make {} tricks\n", tricks);

    if let Some(line) = &solver.best_line {
        println!("Example play achieving {} tricks:", tricks);
        println!("-----------------------------------");

        let mut trick_cards: Vec<(Position, Card)> = Vec::new();
        let mut trick_num = 1;
        let mut leader = Position::East; // Opening leader

        for (pos, card) in &line.play_history {
            trick_cards.push((*pos, *card));

            if trick_cards.len() == 4 {
                // Determine winner
                let suit_led = trick_cards[0].1.suit;
                let mut winner = trick_cards[0].0;
                let mut winning_card = trick_cards[0].1;

                for &(p, c) in &trick_cards[1..] {
                    if c.suit == suit_led && c.rank > winning_card.rank {
                        winner = p;
                        winning_card = c;
                    }
                }

                let won_by_ns = winner == Position::North || winner == Position::South;

                println!(
                    "Trick {}: {} leads {} - {} plays {} - {} plays {} - {} plays {} => {} wins {}",
                    trick_num,
                    pos_char(leader),
                    card_str(trick_cards[0].1),
                    pos_char(trick_cards[1].0),
                    card_str(trick_cards[1].1),
                    pos_char(trick_cards[2].0),
                    card_str(trick_cards[2].1),
                    pos_char(trick_cards[3].0),
                    card_str(trick_cards[3].1),
                    pos_char(winner),
                    if won_by_ns { "(N/S)" } else { "(E/W)" }
                );

                leader = winner;
                trick_num += 1;
                trick_cards.clear();
            }
        }
    }

    // Now a more interesting position
    println!("\n\n=================================\n");
    println!("Position 2: Mixed position with strategy (6 cards)");
    println!();

    // North: SA HK HT D3 C2 S8
    // East:  KS QS HA D4 C3 H8
    // South: TS HQ H9 D9 C4 S7
    // West:  SJ JH DA DK CA D5
    //
    // 6-card endgame with strategic choices

    let hands2 = [
        vec![  // North
            Card::new(Suit::Spades, Rank::Ace),
            Card::new(Suit::Hearts, Rank::King),
            Card::new(Suit::Hearts, Rank::Ten),
            Card::new(Suit::Diamonds, Rank::Three),
            Card::new(Suit::Clubs, Rank::Two),
            Card::new(Suit::Spades, Rank::Eight),
        ],
        vec![  // East
            Card::new(Suit::Spades, Rank::King),
            Card::new(Suit::Spades, Rank::Queen),
            Card::new(Suit::Hearts, Rank::Ace),
            Card::new(Suit::Diamonds, Rank::Four),
            Card::new(Suit::Clubs, Rank::Three),
            Card::new(Suit::Hearts, Rank::Eight),
        ],
        vec![  // South
            Card::new(Suit::Spades, Rank::Ten),
            Card::new(Suit::Hearts, Rank::Queen),
            Card::new(Suit::Hearts, Rank::Nine),
            Card::new(Suit::Diamonds, Rank::Nine),
            Card::new(Suit::Clubs, Rank::Four),
            Card::new(Suit::Spades, Rank::Seven),
        ],
        vec![  // West
            Card::new(Suit::Spades, Rank::Jack),
            Card::new(Suit::Hearts, Rank::Jack),
            Card::new(Suit::Diamonds, Rank::Ace),
            Card::new(Suit::Diamonds, Rank::King),
            Card::new(Suit::Clubs, Rank::Ace),
            Card::new(Suit::Diamonds, Rank::Five),
        ],
    ];

    print_hands(&hands2);
    println!();
    println!("North declarer in NT, East leads");
    println!();

    let state2 = GameState::new(hands2.clone(), Position::North, None);
    let mut solver2 = Solver::new();
    let tricks2 = solver2.solve(&state2);

    println!("Optimal result: N/S make {} tricks\n", tricks2);

    if let Some(line) = &solver2.best_line {
        println!("Example play achieving {} tricks:", tricks2);
        println!("-----------------------------------");

        let mut trick_cards: Vec<(Position, Card)> = Vec::new();
        let mut trick_num = 1;
        let mut leader = Position::East;

        for (pos, card) in &line.play_history {
            trick_cards.push((*pos, *card));

            if trick_cards.len() == 4 {
                let suit_led = trick_cards[0].1.suit;
                let mut winner = trick_cards[0].0;
                let mut winning_card = trick_cards[0].1;

                for &(p, c) in &trick_cards[1..] {
                    if c.suit == suit_led && c.rank > winning_card.rank {
                        winner = p;
                        winning_card = c;
                    }
                }

                let won_by_ns = winner == Position::North || winner == Position::South;

                println!(
                    "Trick {}: {} leads {} - {} plays {} - {} plays {} - {} plays {} => {} wins {}",
                    trick_num,
                    pos_char(leader),
                    card_str(trick_cards[0].1),
                    pos_char(trick_cards[1].0),
                    card_str(trick_cards[1].1),
                    pos_char(trick_cards[2].0),
                    card_str(trick_cards[2].1),
                    pos_char(trick_cards[3].0),
                    card_str(trick_cards[3].1),
                    pos_char(winner),
                    if won_by_ns { "(N/S)" } else { "(E/W)" }
                );

                leader = winner;
                trick_num += 1;
                trick_cards.clear();
            }
        }
    }

    // Compare with non-TT version
    println!("\n\n=================================");
    println!("Verification: Compare TT vs no-TT results");
    println!("=================================\n");

    // Run without TT for comparison
    let state_check = GameState::new(hands2.clone(), Position::North, None);
    let mut solver_no_tt = SolverNoTT::new();
    let tricks_no_tt = solver_no_tt.solve(&state_check);

    println!("Position 2 results:");
    println!("  With TT:    {} tricks ({} nodes)", tricks2, solver2.nodes_visited);
    println!("  Without TT: {} tricks ({} nodes)", tricks_no_tt, solver_no_tt.nodes_visited);

    if tricks2 == tricks_no_tt {
        println!("\n✓ Results MATCH - TT implementation is correct!");
    } else {
        println!("\n✗ Results MISMATCH - TT may have a bug!");
    }
}

// Simple solver without TT for comparison
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
