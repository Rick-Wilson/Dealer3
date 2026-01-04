//! Trace through solver2 step by step for debugging
//!
//! Shows full search tree for a small endgame

use dealer_dds::solver2::{Hands, Solver, NOTRUMP};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH, NUM_SEATS};
use dealer_dds::solver2::cards::{rank_of, suit_of, name_of, Cards};
use dealer_dds::solver2::types::{next_seat, is_ns, Seat, Suit, TOTAL_CARDS, TOTAL_TRICKS};

/// Get playable cards for current player
fn get_playable_cards(hands: &Hands, seat: Seat, lead_suit: Option<Suit>) -> Cards {
    let hand = hands[seat];

    if let Some(suit) = lead_suit {
        // Must follow suit if possible
        let suit_cards = hand.suit(suit);
        if !suit_cards.is_empty() {
            return suit_cards;
        }
    }

    // Can play any card
    hand
}

/// Check if card1 beats card2 given trump suit and suit led
fn wins_over(trump: usize, c1: usize, c2: usize, lead_suit: Suit) -> bool {
    let s1 = suit_of(c1);
    let s2 = suit_of(c2);

    // Same suit - higher rank wins (lower card index = higher rank)
    if s1 == s2 {
        return c1 < c2;
    }

    // Trump beats non-trump
    if trump < NOTRUMP {
        if s1 == trump {
            return true;
        }
        if s2 == trump {
            return false;
        }
    }

    // Different non-trump suits - c2 (already played) wins
    false
}

struct TracingSolver {
    hands: Hands,
    trump: usize,
    initial_leader: Seat,
    num_tricks: usize,
    indent: usize,
}

impl TracingSolver {
    fn new(hands: Hands, trump: usize, initial_leader: Seat) -> Self {
        let num_tricks = hands.num_tricks();
        TracingSolver {
            hands,
            trump,
            initial_leader,
            num_tricks,
            indent: 0,
        }
    }

    fn solve(&self) -> u8 {
        let num_tricks = self.num_tricks;
        let guess = num_tricks; // Simple guess: NS wins all

        let mut lower = 0i8;
        let mut upper = num_tricks as i8;
        let mut ns_tricks = guess as i8;

        println!("MTD(f): num_tricks={}, guess={}", num_tricks, guess);

        while lower < upper {
            let beta = if ns_tricks == lower {
                ns_tricks + 1
            } else {
                ns_tricks
            };

            println!("\nMTD(f) iteration: lower={}, upper={}, ns_tricks={}, beta={}", lower, upper, ns_tricks, beta);

            let mut hands = self.hands;
            let mut cards_played = [0usize; TOTAL_CARDS];
            let mut seats = [0usize; TOTAL_CARDS];
            let mut lead_suits = [0usize; TOTAL_TRICKS];
            let mut winning_card_idx = [0usize; TOTAL_TRICKS];

            ns_tricks = self.search(
                &mut hands,
                &mut cards_played,
                &mut seats,
                &mut lead_suits,
                &mut winning_card_idx,
                0,
                0,
                self.initial_leader,
                beta - 1, // alpha = beta - 1 for null-window search
                beta,
                0,
            ) as i8;

            println!("Search returned: {}", ns_tricks);

            if ns_tricks < beta {
                upper = ns_tricks;
            } else {
                lower = ns_tricks;
            }
        }
        lower as u8
    }

    fn search(
        &self,
        hands: &mut Hands,
        cards_played: &mut [usize; TOTAL_CARDS],
        seats: &mut [usize; TOTAL_CARDS],
        lead_suits: &mut [usize; TOTAL_TRICKS],
        winning_card_idx: &mut [usize; TOTAL_TRICKS],
        depth: usize,
        ns_tricks_won: u8,
        seat_to_play: Seat,
        alpha: i8,
        beta: i8,
        indent: usize,
    ) -> u8 {
        let trick_idx = depth / 4;
        let card_in_trick = depth & 3;
        let num_tricks = self.num_tricks;

        // Terminal check
        if trick_idx >= num_tricks {
            println!("{:indent$}TERMINAL: NS wins {}", "", ns_tricks_won, indent = indent);
            return ns_tricks_won;
        }

        // Quick bounds check
        let remaining = num_tricks - trick_idx;
        if ns_tricks_won as i8 >= beta {
            println!("{:indent$}CUTOFF (ns_tricks >= beta): {} >= {}", "", ns_tricks_won, beta, indent = indent);
            return ns_tricks_won;
        }
        if (ns_tricks_won as usize + remaining) < beta as usize {
            println!("{:indent$}CUTOFF (can't reach beta): {} + {} < {}", "", ns_tricks_won, remaining, beta, indent = indent);
            return ns_tricks_won;
        }

        // Get playable cards
        let lead_suit = if card_in_trick == 0 {
            None
        } else {
            Some(lead_suits[trick_idx])
        };
        let playable = get_playable_cards(hands, seat_to_play, lead_suit);

        if playable.is_empty() {
            println!("{:indent$}NO PLAYABLE CARDS (unexpected)", "", indent = indent);
            return ns_tricks_won;
        }

        let seat_name = ["W", "N", "E", "S"][seat_to_play];
        let maximizing = is_ns(seat_to_play);

        println!("{:indent$}Depth {}, Trick {}.{}, {} to play ({}), alpha={}, beta={}, ns_tricks={}",
            "", depth, trick_idx + 1, card_in_trick, seat_name,
            if maximizing { "MAX" } else { "MIN" }, alpha, beta, ns_tricks_won,
            indent = indent);

        let mut best = if maximizing { 0u8 } else { num_tricks as u8 };
        let mut current_alpha = alpha;
        let mut current_beta = beta;

        for card in playable.iter() {
            println!("{:indent$}  Try {}", "", name_of(card), indent = indent);

            // Play the card
            cards_played[depth] = card;
            seats[depth] = seat_to_play;
            hands[seat_to_play].remove(card);

            let (next_ns_tricks, next_seat) = if card_in_trick == 0 {
                // Leading a new trick
                lead_suits[trick_idx] = suit_of(card);
                winning_card_idx[trick_idx] = depth;
                (ns_tricks_won, next_seat(seat_to_play))
            } else {
                // Following in a trick
                let current_winner_idx = winning_card_idx[trick_idx];
                let current_winner_card = cards_played[current_winner_idx];

                if wins_over(self.trump, card, current_winner_card, lead_suits[trick_idx]) {
                    winning_card_idx[trick_idx] = depth;
                }

                if card_in_trick == 3 {
                    // Trick complete
                    let winner_idx = winning_card_idx[trick_idx];
                    let winner_seat = seats[winner_idx];
                    let ns_won = if is_ns(winner_seat) { 1 } else { 0 };
                    let winning_card = cards_played[winner_idx];
                    println!("{:indent$}  Trick complete: {} wins with {}",
                        "", ["W", "N", "E", "S"][winner_seat], name_of(winning_card),
                        indent = indent);
                    (ns_tricks_won + ns_won, winner_seat)
                } else {
                    (ns_tricks_won, next_seat(seat_to_play))
                }
            };

            let score = self.search(
                hands, cards_played, seats, lead_suits, winning_card_idx,
                depth + 1, next_ns_tricks, next_seat, current_alpha, current_beta, indent + 4,
            );

            // Restore hand
            hands[seat_to_play].add(card);

            println!("{:indent$}  {} score: {}", "", name_of(card), score, indent = indent);

            if maximizing {
                if score > best {
                    best = score;
                    println!("{:indent$}  New best for MAX: {}", "", best, indent = indent);
                }
                if best as i8 >= current_beta {
                    println!("{:indent$}  BETA CUTOFF: {} >= {}", "", best, current_beta, indent = indent);
                    return best;
                }
                if best as i8 > current_alpha {
                    current_alpha = best as i8;
                }
            } else {
                if score < best {
                    best = score;
                    println!("{:indent$}  New best for MIN: {}", "", best, indent = indent);
                }
                if (best as i8) <= current_alpha {
                    println!("{:indent$}  ALPHA CUTOFF: {} <= {}", "", best, current_alpha, indent = indent);
                    return best;
                }
                if (best as i8) < current_beta {
                    current_beta = best as i8;
                }
            }
        }

        println!("{:indent$}Returning best: {}", "", best, indent = indent);
        best
    }
}

fn main() {
    // Failing case:
    // North: S7 H76 D5 C2
    // West: SA8 DT8 C9
    // East: SQ HJ C865
    // South: S42 DJ9 CA
    // C++ says: NT with S leads = 3 tricks
    // Rust says: 2 tricks

    let hands = Hands::from_solver_format(
        "7 76 5 2",    // North: SHDC
        "A8 - T8 9",   // West: SHDC
        "Q J - 865",   // East: SHDC
        "42 - J9 A",   // South: SHDC
    ).expect("Should parse");

    println!("Failing case:");
    println!("North: S7 H76 D5 C2");
    println!("West:  SA8 - DT8 C9");
    println!("East:  SQ HJ - C865");
    println!("South: S42 - DJ9 CA");
    println!();

    // First try the real solver with different leaders
    println!("Real solver2 results:");
    for (leader, name) in [(WEST, "W"), (NORTH, "N"), (EAST, "E"), (SOUTH, "S")] {
        let solver = Solver::new(hands, NOTRUMP, leader);
        let result = solver.solve();
        println!("  NT {} leads: NS makes {} tricks", name, result);
    }
    println!("Expected: all should be 2 (from trusted source)\n");

    println!("Tracing solver with full output...\n");

    let solver = TracingSolver::new(hands, NOTRUMP, SOUTH);
    let result = solver.solve();

    println!("\nTracing result: NS makes {} tricks", result);
    println!("Expected: 3 tricks (from C++ solver)");
}
