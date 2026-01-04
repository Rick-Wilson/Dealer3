// Quick test of lib.rs performance on simple distribution
use dealer_core::{Card, Deal, Position, Rank, Suit};
use dealer_dds::{Denomination, DoubleDummySolver};
use std::time::Instant;

fn main() {
    // Create simple 13-card deal: each hand has one suit
    let mut deal = Deal::new();

    let ranks = [
        Rank::Ace,
        Rank::King,
        Rank::Queen,
        Rank::Jack,
        Rank::Ten,
        Rank::Nine,
        Rank::Eight,
        Rank::Seven,
        Rank::Six,
        Rank::Five,
        Rank::Four,
        Rank::Three,
        Rank::Two,
    ];

    // North gets all spades
    for &rank in &ranks {
        deal.hand_mut(Position::North)
            .add_card(Card::new(Suit::Spades, rank));
    }
    // East gets all hearts
    for &rank in &ranks {
        deal.hand_mut(Position::East)
            .add_card(Card::new(Suit::Hearts, rank));
    }
    // South gets all diamonds
    for &rank in &ranks {
        deal.hand_mut(Position::South)
            .add_card(Card::new(Suit::Diamonds, rank));
    }
    // West gets all clubs
    for &rank in &ranks {
        deal.hand_mut(Position::West)
            .add_card(Card::new(Suit::Clubs, rank));
    }

    let solver = DoubleDummySolver::new(deal);

    let start = Instant::now();
    let tricks = solver.solve(Denomination::NoTrump, Position::North);
    let elapsed = start.elapsed();

    println!("Simple 13-card deal (one suit per hand):");
    println!("  NT with North declarer: {} tricks", tricks);
    println!("  Time: {:.1}ms", elapsed.as_secs_f64() * 1000.0);
}
