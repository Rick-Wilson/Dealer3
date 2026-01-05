//! Play state and search logic

use super::cards::*;
use super::hands::Hands;
use super::types::*;

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
