//! Conversion between dealer-core types and bridge-types types.
//!
//! dealer-core defines its own Hand and Deal types with generator-specific methods,
//! while bridge-types defines Hand and Deal with evaluation/parsing methods.
//! This module provides conversions between them.

use crate::Position;

impl From<&crate::Hand> for bridge_types::Hand {
    fn from(hand: &crate::Hand) -> Self {
        bridge_types::Hand::from_cards(hand.cards().to_vec())
    }
}

impl From<crate::Hand> for bridge_types::Hand {
    fn from(hand: crate::Hand) -> Self {
        bridge_types::Hand::from_cards(hand.cards().to_vec())
    }
}

impl From<&bridge_types::Hand> for crate::Hand {
    fn from(hand: &bridge_types::Hand) -> Self {
        crate::Hand::from_cards(hand.cards().to_vec())
    }
}

impl From<bridge_types::Hand> for crate::Hand {
    fn from(hand: bridge_types::Hand) -> Self {
        crate::Hand::from_cards(hand.cards().to_vec())
    }
}

impl From<&crate::Deal> for bridge_types::Deal {
    fn from(deal: &crate::Deal) -> Self {
        let mut bt_deal = bridge_types::Deal::new();
        bt_deal.set_hand(
            bridge_types::Direction::North,
            deal.hand(Position::North).into(),
        );
        bt_deal.set_hand(
            bridge_types::Direction::East,
            deal.hand(Position::East).into(),
        );
        bt_deal.set_hand(
            bridge_types::Direction::South,
            deal.hand(Position::South).into(),
        );
        bt_deal.set_hand(
            bridge_types::Direction::West,
            deal.hand(Position::West).into(),
        );
        bt_deal
    }
}

impl From<crate::Deal> for bridge_types::Deal {
    fn from(deal: crate::Deal) -> Self {
        (&deal).into()
    }
}

impl From<&bridge_types::Deal> for crate::Deal {
    fn from(deal: &bridge_types::Deal) -> Self {
        let mut dc_deal = crate::Deal::new();
        *dc_deal.hand_mut(Position::North) = deal.hand(bridge_types::Direction::North).into();
        *dc_deal.hand_mut(Position::East) = deal.hand(bridge_types::Direction::East).into();
        *dc_deal.hand_mut(Position::South) = deal.hand(bridge_types::Direction::South).into();
        *dc_deal.hand_mut(Position::West) = deal.hand(bridge_types::Direction::West).into();
        dc_deal
    }
}

impl From<bridge_types::Deal> for crate::Deal {
    fn from(deal: bridge_types::Deal) -> Self {
        (&deal).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Card, Rank, Suit};

    #[test]
    fn test_hand_round_trip() {
        let mut hand = crate::Hand::new();
        hand.add_card(Card::new(Suit::Spades, Rank::Ace));
        hand.add_card(Card::new(Suit::Hearts, Rank::King));

        let bt_hand: bridge_types::Hand = (&hand).into();
        let back: crate::Hand = bt_hand.into();

        assert_eq!(hand.len(), back.len());
        assert_eq!(hand.hcp(), back.hcp());
    }

    #[test]
    fn test_deal_round_trip() {
        let mut deal = crate::Deal::new();
        deal.hand_mut(Position::North)
            .add_card(Card::new(Suit::Spades, Rank::Ace));
        deal.hand_mut(Position::East)
            .add_card(Card::new(Suit::Hearts, Rank::King));

        let bt_deal: bridge_types::Deal = (&deal).into();
        let back: crate::Deal = bt_deal.into();

        assert_eq!(
            deal.hand(Position::North).len(),
            back.hand(Position::North).len()
        );
        assert_eq!(
            deal.hand(Position::East).len(),
            back.hand(Position::East).len()
        );
    }
}
