//! Exhaustive search to verify solver correctness
//!
//! This implements a simple minimax without any optimizations.

use dealer_dds::solver2::{Hands, NOTRUMP};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH};
use dealer_dds::solver2::cards::{suit_of, name_of, Cards};
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

static mut TRACE: bool = false;
static mut TRACE_DEPTH: usize = 12;

fn seat_name(seat: Seat) -> &'static str {
    match seat {
        WEST => "W",
        NORTH => "N",
        EAST => "E",
        SOUTH => "S",
        _ => "?",
    }
}

struct ExhaustiveSolver {
    hands: Hands,
    trump: usize,
    initial_leader: Seat,
    num_tricks: usize,
}

impl ExhaustiveSolver {
    fn new(hands: Hands, trump: usize, initial_leader: Seat) -> Self {
        let num_tricks = hands.num_tricks();
        ExhaustiveSolver {
            hands,
            trump,
            initial_leader,
            num_tricks,
        }
    }

    fn solve(&self) -> u8 {
        let mut hands = self.hands;
        let mut cards_played = [0usize; TOTAL_CARDS];
        let mut seats = [0usize; TOTAL_CARDS];
        let mut lead_suits = [0usize; TOTAL_TRICKS];
        let mut winning_card_idx = [0usize; TOTAL_TRICKS];

        self.minimax(
            &mut hands,
            &mut cards_played,
            &mut seats,
            &mut lead_suits,
            &mut winning_card_idx,
            0,
            0,
            self.initial_leader,
        )
    }

    fn minimax(
        &self,
        hands: &mut Hands,
        cards_played: &mut [usize; TOTAL_CARDS],
        seats: &mut [usize; TOTAL_CARDS],
        lead_suits: &mut [usize; TOTAL_TRICKS],
        winning_card_idx: &mut [usize; TOTAL_TRICKS],
        depth: usize,
        ns_tricks_won: u8,
        seat_to_play: Seat,
    ) -> u8 {
        let trick_idx = depth / 4;
        let card_in_trick = depth & 3;
        let num_tricks = self.num_tricks;

        // Terminal check
        if trick_idx >= num_tricks {
            unsafe {
                if TRACE && ns_tricks_won >= 3 {
                    println!("Found line with {} tricks for NS!", ns_tricks_won);
                    for t in 0..num_tricks {
                        let cards: Vec<_> = cards_played[t*4..t*4+4]
                            .iter()
                            .map(|&c| name_of(c))
                            .collect();
                        println!("  Trick {}: {} {} {} {}", t + 1, cards[0], cards[1], cards[2], cards[3]);
                    }
                }
            }
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
            return ns_tricks_won;
        }

        let maximizing = is_ns(seat_to_play);
        let mut best = if maximizing { 0u8 } else { num_tricks as u8 };

        unsafe {
            if TRACE && depth < TRACE_DEPTH {
                let indent = "  ".repeat(depth);
                let card_names: Vec<_> = playable.iter().map(|c| name_of(c)).collect();
                println!("{}{}[{}] {} to play, cards: {:?}",
                    indent, if maximizing { "MAX" } else { "MIN" },
                    depth, seat_name(seat_to_play),
                    card_names);
            }
        }

        for card in playable.iter() {
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


                    (ns_tricks_won + ns_won, winner_seat)
                } else {
                    (ns_tricks_won, next_seat(seat_to_play))
                }
            };

            let score = self.minimax(
                hands, cards_played, seats, lead_suits, winning_card_idx,
                depth + 1, next_ns_tricks, next_seat,
            );

            // Restore hand
            hands[seat_to_play].add(card);

            unsafe {
                if TRACE && depth < TRACE_DEPTH {
                    let indent = "  ".repeat(depth);
                    println!("{}{} plays {}: score={}, best={}",
                        indent, seat_name(seat_to_play), name_of(card), score, best);
                }
            }

            if maximizing {
                if score > best {
                    best = score;
                }
            } else {
                if score < best {
                    best = score;
                }
            }
        }

        unsafe {
            if TRACE && depth < TRACE_DEPTH {
                let indent = "  ".repeat(depth);
                println!("{}=> returning {}", indent, best);
            }
        }

        best
    }

    fn enable_trace(&self) {
        unsafe { TRACE = true; }
    }

    fn disable_trace(&self) {
        unsafe { TRACE = false; }
    }
}

fn main() {
    let hands = Hands::from_solver_format(
        "7 76 5 2",    // North: SHDC
        "A8 - T8 9",   // West: SHDC
        "Q J - 865",   // East: SHDC
        "42 - J9 A",   // South: SHDC
    ).expect("Should parse");

    println!("Exhaustive search for NT S leads:");
    println!("Deal: N:7.76.5.2 W:A8.-.T8.9 E:Q.J.-.865 S:42.-.J9.A");
    println!();

    // Only trace first 8 cards (first two tricks)
    unsafe { TRACE_DEPTH = 8; }

    let solver = ExhaustiveSolver::new(hands, NOTRUMP, SOUTH);
    solver.enable_trace();
    let ns_tricks = solver.solve();
    solver.disable_trace();

    println!();
    println!("Result: NS makes {} tricks", ns_tricks);
    println!("C++ solver says: 3 tricks");
}
