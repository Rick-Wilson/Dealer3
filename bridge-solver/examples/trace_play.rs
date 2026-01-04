//! Trace a full DD play to see what's happening
//!
//! Tests Deal 2, North playing NT - our solver says 12, BC says 7

use dealer_core::{Card, Deal, Hand, Position, Rank, Suit};
use std::collections::HashMap;

fn next_position(pos: Position) -> Position {
    match pos {
        Position::North => Position::East,
        Position::East => Position::South,
        Position::South => Position::West,
        Position::West => Position::North,
    }
}

fn parse_hand(s: &str) -> Hand {
    let suits_str: Vec<&str> = s.split('.').collect();
    let suits = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];
    let mut hand = Hand::new();

    for (suit_idx, &suit_str) in suits_str.iter().enumerate() {
        let suit = suits[suit_idx];
        for c in suit_str.chars() {
            let rank = match c {
                'A' => Rank::Ace,
                'K' => Rank::King,
                'Q' => Rank::Queen,
                'J' => Rank::Jack,
                'T' => Rank::Ten,
                '9' => Rank::Nine,
                '8' => Rank::Eight,
                '7' => Rank::Seven,
                '6' => Rank::Six,
                '5' => Rank::Five,
                '4' => Rank::Four,
                '3' => Rank::Three,
                '2' => Rank::Two,
                _ => panic!("Invalid rank: {}", c),
            };
            hand.add_card(Card::new(suit, rank));
        }
    }
    hand
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
    play_history: Vec<String>,
}

impl GameState {
    fn new(deal: &Deal, declarer: Position, trump: Option<Suit>) -> Self {
        let mut hands = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for position in Position::ALL {
            hands[position as usize] = deal.hand(position).cards().to_vec();
        }

        let opening_leader = next_position(declarer);

        Self {
            hands,
            current_trick: TrickState::new(opening_leader, trump),
            declarer_tricks: 0,
            declarer,
            tricks_played: 0,
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

    fn play_card(&mut self, card: Card) -> bool {
        let player = self.next_player();
        let hand = &mut self.hands[player as usize];

        if let Some(pos) = hand.iter().position(|&c| c == card) {
            hand.remove(pos);
        } else {
            return false;
        }

        self.current_trick.cards_played.push((player, card));

        if self.current_trick.cards_played.len() == 4 {
            let winner = self.current_trick.winner().unwrap();
            let won_by_declarer = winner == self.declarer || winner == self.declarer.partner();

            if won_by_declarer {
                self.declarer_tricks += 1;
            }

            // Record the trick
            let trick_str = format!(
                "Trick {}: {} leads. {} {} {} {} -> {} wins{}",
                self.tricks_played + 1,
                pos_char(self.current_trick.leader),
                card_str(self.current_trick.cards_played[0].1),
                card_str(self.current_trick.cards_played[1].1),
                card_str(self.current_trick.cards_played[2].1),
                card_str(self.current_trick.cards_played[3].1),
                pos_char(winner),
                if won_by_declarer { " (N/S)" } else { " (E/W)" }
            );
            self.play_history.push(trick_str);

            self.tricks_played += 1;
            self.current_trick = TrickState::new(winner, self.current_trick.trump);
        }

        true
    }

    fn is_terminal(&self) -> bool {
        self.tricks_played >= 13
    }

    fn score(&self) -> u8 {
        self.declarer_tricks
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

type TranspositionTable = HashMap<u64, u8>;

struct Solver {
    best_play: Option<GameState>,
}

impl Solver {
    fn new() -> Self {
        Self { best_play: None }
    }

    fn solve(&mut self, state: &GameState) -> u8 {
        let mut tt = HashMap::new();
        self.alpha_beta(state, 0, 13, &mut tt)
    }

    fn alpha_beta(
        &mut self,
        state: &GameState,
        mut alpha: u8,
        mut beta: u8,
        tt: &mut TranspositionTable,
    ) -> u8 {
        if state.is_terminal() {
            // Record best play for declarer
            if self.best_play.is_none() || state.score() > self.best_play.as_ref().unwrap().score()
            {
                self.best_play = Some(state.clone());
            }
            return state.score();
        }

        // Disable TT for correctness testing
        let at_trick_boundary = false;
        let hash = 0u64;

        let maximizing = state.declarer_side_on_lead();
        let moves = state.legal_moves();

        if maximizing {
            let mut value = 0u8;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, alpha, beta, tt);
                value = value.max(score);
                if value > alpha {
                    alpha = value;
                }
                if alpha >= beta {
                    break;
                }
            }
            if at_trick_boundary {
                tt.insert(hash, value);
            }
            value
        } else {
            let mut value = 13u8;
            let mut best_move = None;
            for card in moves.iter() {
                let mut new_state = state.clone();
                new_state.play_card(*card);
                let score = self.alpha_beta(&new_state, alpha, beta, tt);
                if score < value {
                    value = score;
                    best_move = Some(*card);
                }
                if value < beta {
                    beta = value;
                }
                if alpha >= beta {
                    break;
                }
            }
            // Debug: show when minimizer picks a move
            if state.tricks_played == 0 && state.current_trick.cards_played.is_empty() {
                println!(
                    "DEBUG: Minimizer at trick 0, {} leads. Best move: {:?} for value {}",
                    pos_char(state.next_player()),
                    best_move.map(|c| card_str(c)),
                    value
                );
                println!("  Available moves: {:?}", moves.iter().map(|c| card_str(*c)).collect::<Vec<_>>());
            }
            if at_trick_boundary {
                tt.insert(hash, value);
            }
            value
        }
    }

    fn hash_state(&self, state: &GameState) -> u64 {
        let mut hash = 0u64;

        for (pos_idx, hand) in state.hands.iter().enumerate() {
            for card in hand {
                let card_idx = card.to_index() as u64;
                hash ^= card_idx.wrapping_mul((pos_idx as u64 + 1) * 13);
            }
        }

        hash ^= (state.current_trick.leader as u64) << 56;
        hash ^= (state.declarer_tricks as u64) << 48;

        hash
    }
}

fn main() {
    // Deal 2 from north_9tricks_nt-bridge_composer.pbn
    // Our solver claims N makes 12 in NT, BC says 7
    let mut deal = Deal::new();
    *deal.hand_mut(Position::North) = parse_hand("AKT52.97.965.J84");
    *deal.hand_mut(Position::East) = parse_hand("9.A864.AT743.T97");
    *deal.hand_mut(Position::South) = parse_hand("J874.KJ3.Q.AKQ32");
    *deal.hand_mut(Position::West) = parse_hand("Q63.QT52.KJ82.65");

    println!("Deal 2: North playing NT");
    println!("========================");
    println!();
    println!("North: AKT52.97.965.J84");
    println!("East:  9.A864.AT743.T97");
    println!("South: J874.KJ3.Q.AKQ32");
    println!("West:  Q63.QT52.KJ82.65");
    println!();
    println!("Expected: North makes 7 tricks");
    println!();

    let state = GameState::new(&deal, Position::North, None);
    let mut solver = Solver::new();
    let tricks = solver.solve(&state);

    println!("Our solver says: North makes {} tricks", tricks);
    println!();

    if let Some(best) = &solver.best_play {
        println!("Best play found (N/S perspective):");
        println!("==================================");
        for line in &best.play_history {
            println!("{}", line);
        }
        println!();
        println!("Final: N/S won {} tricks", best.score());
    }
}
