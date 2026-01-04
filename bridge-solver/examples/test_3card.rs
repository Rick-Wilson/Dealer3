//! Test DD solver logic with a 3-card ending
//!
//! This duplicates the core solver logic but works with 3 cards per hand
//! to verify the alpha-beta search is correct before testing full deals.

use dealer_core::{Card, Position, Rank, Suit};

fn next_position(pos: Position) -> Position {
    match pos {
        Position::North => Position::East,
        Position::East => Position::South,
        Position::South => Position::West,
        Position::West => Position::North,
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
    num_tricks: u8,  // Total tricks in this game (e.g., 3 for 3-card ending)
}

impl GameState {
    fn new(
        hands: [Vec<Card>; 4],
        declarer: Position,
        trump: Option<Suit>,
    ) -> Self {
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

    fn play_card(&mut self, card: Card) -> Option<Position> {
        let player = self.next_player();
        let hand = &mut self.hands[player as usize];

        if let Some(pos) = hand.iter().position(|&c| c == card) {
            hand.remove(pos);
        } else {
            return None;
        }

        self.current_trick.cards_played.push((player, card));

        if self.current_trick.cards_played.len() == 4 {
            let winner = self.current_trick.winner().unwrap();

            if winner == self.declarer || winner == self.declarer.partner() {
                self.declarer_tricks += 1;
            }

            self.tricks_played += 1;
            self.current_trick = TrickState::new(winner, self.current_trick.trump);
            return Some(winner);
        }

        None
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
        let result = self.alpha_beta(state, 0, state.num_tricks, 0);
        println!("Nodes visited: {}", self.nodes_visited);
        result
    }

    fn alpha_beta(
        &mut self,
        state: &GameState,
        mut alpha: u8,
        mut beta: u8,
        depth: u8,
    ) -> u8 {
        self.nodes_visited += 1;

        if state.is_terminal() {
            return state.score();
        }

        let maximizing = state.declarer_side_on_lead();
        let moves = state.legal_moves();

        let indent = "  ".repeat(depth as usize);
        let player = state.next_player();
        println!(
            "{}Depth {}: {} to play ({}) - moves: {:?}",
            indent,
            depth,
            match player {
                Position::North => "N",
                Position::East => "E",
                Position::South => "S",
                Position::West => "W",
            },
            if maximizing { "MAX" } else { "MIN" },
            moves.iter().map(|c| card_str(*c)).collect::<Vec<_>>()
        );

        if maximizing {
            let mut value = 0u8;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, alpha, beta, depth + 1);
                println!("{}  {} plays {} -> score {}", indent,
                    match player { Position::North => "N", Position::East => "E",
                                   Position::South => "S", Position::West => "W" },
                    card_str(card), score);
                if score > value {
                    value = score;
                }
                if value > alpha {
                    alpha = value;
                }
                if alpha >= beta {
                    println!("{}  Beta cutoff (alpha={} >= beta={})", indent, alpha, beta);
                    break;
                }
            }
            value
        } else {
            let mut value = state.num_tricks;
            for card in moves {
                let mut new_state = state.clone();
                new_state.play_card(card);
                let score = self.alpha_beta(&new_state, alpha, beta, depth + 1);
                println!("{}  {} plays {} -> score {}", indent,
                    match player { Position::North => "N", Position::East => "E",
                                   Position::South => "S", Position::West => "W" },
                    card_str(card), score);
                if score < value {
                    value = score;
                }
                if value < beta {
                    beta = value;
                }
                if alpha >= beta {
                    println!("{}  Alpha cutoff (alpha={} >= beta={})", indent, alpha, beta);
                    break;
                }
            }
            value
        }
    }
}

fn main() {
    println!("3-Card Ending Test");
    println!("==================\n");

    // Endgame 1: N/S have all spades, E/W have all hearts
    // North: SA SK SQ
    // East:  HA HK HQ
    // South: SJ ST S9
    // West:  HJ HT H9
    //
    // North declarer in NT, East leads
    // E leads hearts, E/W win all 3 hearts, then... wait, there's no more tricks
    // Actually after E/W win 3 hearts, game is over with 0 for N/S
    //
    // Let's try: N declarer, E leads heart. N must follow? No, N has no hearts.
    // N discards a spade. S must follow? S has no hearts. S discards.
    // W plays a heart. E wins trick 1.
    // E leads heart again. Same thing. E/W win all 3 tricks.
    // N/S make 0 tricks in NT.
    //
    // But in Spades trump:
    // E leads heart. N ruffs with a spade (trumps). N wins the trick.
    // N leads SA. E has no spades, discards. S plays SJ. W discards. N wins.
    // N leads SK. Same. N wins. N/S make 3 tricks.

    println!("Endgame 1: Spade vs Hearts");
    println!("North: SA SK SQ");
    println!("East:  HA HK HQ");
    println!("South: SJ ST S9");
    println!("West:  HJ HT H9");
    println!("North declarer, East leads\n");

    let hands_1 = [
        vec![  // North
            Card::new(Suit::Spades, Rank::Ace),
            Card::new(Suit::Spades, Rank::King),
            Card::new(Suit::Spades, Rank::Queen),
        ],
        vec![  // East
            Card::new(Suit::Hearts, Rank::Ace),
            Card::new(Suit::Hearts, Rank::King),
            Card::new(Suit::Hearts, Rank::Queen),
        ],
        vec![  // South
            Card::new(Suit::Spades, Rank::Jack),
            Card::new(Suit::Spades, Rank::Ten),
            Card::new(Suit::Spades, Rank::Nine),
        ],
        vec![  // West
            Card::new(Suit::Hearts, Rank::Jack),
            Card::new(Suit::Hearts, Rank::Ten),
            Card::new(Suit::Hearts, Rank::Nine),
        ],
    ];

    // Test in NT
    println!("--- In NT ---");
    let state_nt = GameState::new(hands_1.clone(), Position::North, None);
    let mut solver = Solver::new();
    let tricks_nt = solver.solve(&state_nt);
    println!("\nResult: N makes {} tricks in NT (expected: 0)\n", tricks_nt);

    // Test in Spades
    println!("--- In Spades ---");
    let state_s = GameState::new(hands_1.clone(), Position::North, Some(Suit::Spades));
    let mut solver = Solver::new();
    let tricks_s = solver.solve(&state_s);
    println!("\nResult: N makes {} tricks in Spades (expected: 3)\n", tricks_s);

    // Endgame 2: Mixed - tricks go both ways
    // North: SA HK DQ
    // East:  SK HA DK
    // South: SQ HQ DA
    // West:  SJ HJ DJ
    //
    // North declarer in NT, East leads:
    // E has SK, HA, DK - all higher than N's cards except SA beats SK
    // Let's trace:
    // E leads SK. N plays SA (wins). S plays SQ. W plays SJ. N wins trick 1.
    // N leads HK. E plays HA (wins). S plays HQ. W plays HJ. E wins trick 2.
    // E leads DK. N plays DQ. S plays DA (wins). W plays DJ. S wins trick 3.
    // N/S make 2 tricks.

    println!("\n==================\n");
    println!("Endgame 2: Mixed tricks");
    println!("North: SA HK DQ");
    println!("East:  SK HA DK");
    println!("South: SQ HQ DA");
    println!("West:  SJ HJ DJ");
    println!("North declarer, East leads\n");

    let hands_2 = [
        vec![  // North
            Card::new(Suit::Spades, Rank::Ace),
            Card::new(Suit::Hearts, Rank::King),
            Card::new(Suit::Diamonds, Rank::Queen),
        ],
        vec![  // East
            Card::new(Suit::Spades, Rank::King),
            Card::new(Suit::Hearts, Rank::Ace),
            Card::new(Suit::Diamonds, Rank::King),
        ],
        vec![  // South
            Card::new(Suit::Spades, Rank::Queen),
            Card::new(Suit::Hearts, Rank::Queen),
            Card::new(Suit::Diamonds, Rank::Ace),
        ],
        vec![  // West
            Card::new(Suit::Spades, Rank::Jack),
            Card::new(Suit::Hearts, Rank::Jack),
            Card::new(Suit::Diamonds, Rank::Jack),
        ],
    ];

    println!("--- In NT ---");
    let state_nt2 = GameState::new(hands_2.clone(), Position::North, None);
    let mut solver2 = Solver::new();
    let tricks_nt2 = solver2.solve(&state_nt2);
    println!("\nResult: N makes {} tricks in NT (expected: 2)\n", tricks_nt2);

    // Summary
    println!("==================");
    println!("Summary:");
    println!("Endgame 1 NT: {} (expected 0) - {}", tricks_nt, if tricks_nt == 0 { "PASS" } else { "FAIL" });
    println!("Endgame 1 Spades: {} (expected 3) - {}", tricks_s, if tricks_s == 3 { "PASS" } else { "FAIL" });
    println!("Endgame 2 NT: {} (expected 2) - {}", tricks_nt2, if tricks_nt2 == 2 { "PASS" } else { "FAIL" });

    let all_pass = tricks_nt == 0 && tricks_s == 3 && tricks_nt2 == 2;
    if all_pass {
        println!("\nAll tests PASSED!");
    } else {
        println!("\nSome tests FAILED!");
    }
}
