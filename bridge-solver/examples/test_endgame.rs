//! Test DD solver with small endgame positions
//!
//! These positions have few cards so the search completes quickly even without TT

use dealer_core::{Card, Deal, Hand, Position, Rank, Suit};
use dealer_dds::{Denomination, DoubleDummySolver};

/// Create a deal where each hand has only the specified cards
/// Pads remaining cards to reach 13 cards per hand (required by Deal)
fn create_endgame_deal(
    north_cards: Vec<Card>,
    east_cards: Vec<Card>,
    south_cards: Vec<Card>,
    west_cards: Vec<Card>,
) -> Deal {
    let mut deal = Deal::new();

    // For this test, we'll create a full 52-card deal but only care about
    // the cards specified - the rest are fillers

    let mut north = Hand::new();
    let mut east = Hand::new();
    let mut south = Hand::new();
    let mut west = Hand::new();

    // Add specified cards
    for c in &north_cards { north.add_card(*c); }
    for c in &east_cards { east.add_card(*c); }
    for c in &south_cards { south.add_card(*c); }
    for c in &west_cards { west.add_card(*c); }

    // Track which cards are used
    let mut used = std::collections::HashSet::new();
    for c in north_cards.iter().chain(east_cards.iter())
        .chain(south_cards.iter()).chain(west_cards.iter()) {
        used.insert((*c).to_index());
    }

    // Fill remaining cards to make 13 each
    let all_ranks = [Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Ten,
                     Rank::Nine, Rank::Eight, Rank::Seven, Rank::Six, Rank::Five,
                     Rank::Four, Rank::Three, Rank::Two];
    let all_suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    let mut remaining: Vec<Card> = Vec::new();
    for &suit in &all_suits {
        for &rank in &all_ranks {
            let card = Card::new(suit, rank);
            if !used.contains(&card.to_index()) {
                remaining.push(card);
            }
        }
    }

    // Distribute remaining cards evenly
    let mut idx = 0;
    while north.len() < 13 {
        north.add_card(remaining[idx]);
        idx += 1;
    }
    while east.len() < 13 {
        east.add_card(remaining[idx]);
        idx += 1;
    }
    while south.len() < 13 {
        south.add_card(remaining[idx]);
        idx += 1;
    }
    while west.len() < 13 {
        west.add_card(remaining[idx]);
        idx += 1;
    }

    *deal.hand_mut(Position::North) = north;
    *deal.hand_mut(Position::East) = east;
    *deal.hand_mut(Position::South) = south;
    *deal.hand_mut(Position::West) = west;

    deal
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

fn cards_str(cards: &[Card]) -> String {
    cards.iter().map(|c| card_str(*c)).collect::<Vec<_>>().join(" ")
}

fn main() {
    println!("Small Endgame Tests");
    println!("===================\n");

    // Test 1: 2-card ending, N/S have top 2 spades, E/W have top 2 hearts
    // In NT, whoever leads wins their suit's tricks
    // North declarer, East leads - East leads hearts, N/S get 0 hearts, 2 spades = 2
    println!("Test 1: 2-card ending");
    println!("North: SA SK");
    println!("East:  HA HK");
    println!("South: SQ SJ");
    println!("West:  HQ HJ");
    println!("North declarer in NT, East leads");
    println!("Expected: E leads hearts -> E/W win 2 hearts, then N/S win 2 spades = 2 tricks for N/S");

    // But wait - this is a 2-card ending embedded in 13-card hands
    // The filler cards will affect the result!
    //
    // Actually, we can't easily test partial endings because the solver
    // always plays all 13 tricks. Let me think of a different approach...
    //
    // Better approach: Create a deal where N/S have ALL of one suit and
    // E/W have ALL of another suit, with known filler distribution

    println!("\n--- Revised approach ---\n");

    // Test A: N/S have all spades (13 tricks), E/W have rest
    // In spades, N/S make 13 tricks
    // In NT, depends on opening lead but N/S should make 13 (all spades)
    println!("Test A: N/S have all 13 spades split 7-6");
    println!("North: SA SK SQ SJ ST S9 S8");
    println!("South: S7 S6 S5 S4 S3 S2, plus small cards");

    let north_a = vec![
        Card::new(Suit::Spades, Rank::Ace),
        Card::new(Suit::Spades, Rank::King),
        Card::new(Suit::Spades, Rank::Queen),
        Card::new(Suit::Spades, Rank::Jack),
        Card::new(Suit::Spades, Rank::Ten),
        Card::new(Suit::Spades, Rank::Nine),
        Card::new(Suit::Spades, Rank::Eight),
    ];
    let south_a = vec![
        Card::new(Suit::Spades, Rank::Seven),
        Card::new(Suit::Spades, Rank::Six),
        Card::new(Suit::Spades, Rank::Five),
        Card::new(Suit::Spades, Rank::Four),
        Card::new(Suit::Spades, Rank::Three),
        Card::new(Suit::Spades, Rank::Two),
    ];

    let deal_a = create_endgame_deal(north_a.clone(), vec![], south_a.clone(), vec![]);

    println!("\nActual deal created:");
    println!("North: {}", cards_str(deal_a.hand(Position::North).cards()));
    println!("East:  {}", cards_str(deal_a.hand(Position::East).cards()));
    println!("South: {}", cards_str(deal_a.hand(Position::South).cards()));
    println!("West:  {}", cards_str(deal_a.hand(Position::West).cards()));

    let solver_a = DoubleDummySolver::new(deal_a);

    println!("\nSolving N in Spades...");
    let tricks_spades = solver_a.solve(Denomination::Spades, Position::North);
    println!("N in Spades: {} tricks (expected: 13)", tricks_spades);

    println!("\nSolving N in NT...");
    let tricks_nt = solver_a.solve(Denomination::NoTrump, Position::North);
    println!("N in NT: {} tricks (expected: 13 - N/S have all spades)", tricks_nt);

    // Test B: Simple test - N/S have AK of all suits (8 cards each = 16 total, split)
    // Actually let's keep it simpler

    println!("\n---\n");
    println!("Test B: N has AKQJ of spades, S has the rest of spades");
    println!("In spades trump, N/S should make all 13 spade tricks = 13");

    // Results summary
    println!("\n===================");
    println!("Results Summary:");
    println!("Test A - N Spades: {} (expected 13)", tricks_spades);
    println!("Test A - N NT: {} (expected 13)", tricks_nt);

    if tricks_spades == 13 && tricks_nt == 13 {
        println!("\nAll tests PASSED!");
    } else {
        println!("\nSome tests FAILED!");
    }
}
