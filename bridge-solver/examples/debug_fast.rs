//! Debug fast tricks calculation

use dealer_dds::solver2::{Hands, NORTH, SOUTH, EAST, WEST, NUM_SUITS};

const SUIT_NAMES: [&str; 4] = ["Spade", "Heart", "Diamond", "Club"];

fn fast_tricks_ns(hands: &Hands) -> usize {
    let mut tricks = 0;
    let ns_cards = hands[NORTH].union(hands[SOUTH]);
    let ew_cards = hands[EAST].union(hands[WEST]);

    println!("Fast tricks NS analysis:");
    for suit in 0..NUM_SUITS {
        let ns_suit = ns_cards.suit(suit);
        let ew_suit = ew_cards.suit(suit);
        print!("  {}: NS has {} cards, EW has {} cards", 
            SUIT_NAMES[suit], ns_suit.size(), ew_suit.size());
        
        if ns_suit.is_empty() {
            println!(" -> NS void, 0 winners");
            continue;
        }
        if ew_suit.is_empty() {
            println!(" -> EW void, NS wins {} tricks", ns_suit.size());
            tricks += ns_suit.size();
            continue;
        }
        // Count NS top cards above EW's highest
        let ew_top = ew_suit.top();
        let ns_winners = ns_suit.slice(0, ew_top);
        println!(" -> EW top idx={}, NS has {} winners above it", ew_top, ns_winners.size());
        tricks += ns_winners.size();
    }
    println!("  Total fast tricks: {}", tricks);
    tricks
}

fn main() {
    // 9-card deal from Test 2
    let pbn = "N:AKQT.J6.KJ.9 65.AK4.AQ8.T J7.QT9.T.AK8 98.87.96.QJ7";
    let hands = Hands::from_pbn(pbn).expect("Should parse");

    println!("9-card deal:");
    println!("N: AKQT.J6.KJ.9  (spades, hearts, diamonds, clubs)");
    println!("E: 65.AK4.AQ8.T");
    println!("S: J7.QT9.T.AK8");
    println!("W: 98.87.96.QJ7");
    println!();
    
    let _ft = fast_tricks_ns(&hands);
}
