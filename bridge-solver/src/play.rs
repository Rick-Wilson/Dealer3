//! Play state and search logic
//!
//! TODO: Full port from C++ solver

use super::cards::*;
use super::hands::Hands;
use super::types::*;
use super::cache::*;

/// State for a single trick
#[derive(Clone, Copy, Debug, Default)]
pub struct Trick {
    pub shape: Shape,
    pub all_cards: Cards,
    pub lead_suit: Suit,
}

/// Play state during search - designed for stack allocation
#[derive(Clone, Copy)]
pub struct Play {
    pub trump: usize,
    pub depth: usize,
    pub ns_tricks_won: u8,
    pub seat_to_play: Seat,
    pub card_played: usize,
    pub winning_play: usize,
}

impl Play {
    pub fn new(trump: usize, depth: usize, seat_to_play: Seat) -> Self {
        Play {
            trump,
            depth,
            ns_tricks_won: 0,
            seat_to_play,
            card_played: TOTAL_CARDS,
            winning_play: 0,
        }
    }

    /// Check if at trick start (depth divisible by 4)
    #[inline]
    pub fn trick_starting(&self) -> bool {
        self.depth & 3 == 0
    }

    /// Check if trick is ending
    #[inline]
    pub fn trick_ending(&self) -> bool {
        self.depth & 3 == 3
    }

    /// Check if NS is to play
    #[inline]
    pub fn ns_to_play(&self) -> bool {
        is_ns(self.seat_to_play)
    }

    /// Get next seat
    #[inline]
    pub fn next_seat(&self) -> Seat {
        next_seat(self.seat_to_play)
    }

    /// Get partner
    #[inline]
    pub fn partner(&self) -> Seat {
        partner(self.seat_to_play)
    }

    /// Get left-hand opponent
    #[inline]
    pub fn left_hand_opp(&self) -> Seat {
        left_hand_opp(self.seat_to_play)
    }

    /// Get right-hand opponent
    #[inline]
    pub fn right_hand_opp(&self) -> Seat {
        right_hand_opp(self.seat_to_play)
    }

    /// Check if card1 beats card2 given trump suit and suit led
    #[inline]
    pub fn wins_over(&self, c1: usize, c2: usize, lead_suit: Suit) -> bool {
        let s1 = suit_of(c1);
        let s2 = suit_of(c2);

        // Same suit - higher rank wins
        if s1 == s2 {
            return higher_rank(c1, c2);
        }

        // Trump beats non-trump
        if self.trump < NOTRUMP {
            if s1 == self.trump {
                return true;
            }
            if s2 == self.trump {
                return false;
            }
        }

        // Different non-trump suits - first card (lead) wins
        s2 != lead_suit
    }
}

impl Default for Play {
    fn default() -> Self {
        Play::new(NOTRUMP, 0, WEST)
    }
}

/// Get playable cards for current player
pub fn get_playable_cards(hands: &Hands, seat: Seat, lead_suit: Option<Suit>) -> Cards {
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

/// Determine trick winner
pub fn trick_winner(
    cards: &[usize; 4],
    seats: &[Seat; 4],
    trump: usize,
) -> Seat {
    let lead_suit = suit_of(cards[0]);
    let mut winner_idx = 0;

    for i in 1..4 {
        let winner_card = cards[winner_idx];
        let card = cards[i];
        let s1 = suit_of(winner_card);
        let s2 = suit_of(card);

        let beats = if s1 == s2 {
            higher_rank(card, winner_card)
        } else if trump < NOTRUMP && s2 == trump {
            true
        } else {
            false
        };

        if beats {
            winner_idx = i;
        }
    }

    seats[winner_idx]
}
