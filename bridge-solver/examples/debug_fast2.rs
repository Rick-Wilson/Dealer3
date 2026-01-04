//! Debug fast tricks calculation for 11-card deal

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, EAST, WEST, NORTH, SOUTH, NUM_SUITS};
use dealer_dds::solver2::cards::{higher_rank, lower_rank};

const SUIT_NAMES: [&str; 4] = ["Spade", "Heart", "Diamond", "Club"];

fn main() {
    // 11-card deal
    let pbn = "N:AKQT3.J6.KJ4.9 652.AK4.AQ87.T J74.QT95.T.AK8 98.873.965.QJ7";
    let hands = Hands::from_pbn(pbn).expect("Should parse");

    println!("11-card deal:");
    println!("N: AKQT3.J6.KJ4.9");
    println!("E: 652.AK4.AQ87.T");
    println!("S: J74.QT95.T.AK8");
    println!("W: 98.873.965.QJ7");
    println!();
    
    // Create solver with East leading
    let solver = Solver::new(hands, NOTRUMP, EAST);
    
    // Call fast_tricks directly
    let all_cards = hands.all_cards();
    let seat = EAST;
    let my_hand = hands[seat];
    let pd_hand = hands[WEST];
    
    println!("East leads, analyzing EW's fast tricks:");
    println!("East: 652.AK4.AQ87.T");
    println!("West: 98.873.965.QJ7");
    println!();
    
    for suit in 0..NUM_SUITS {
        let my_suit = my_hand.suit(suit);
        let pd_suit = pd_hand.suit(suit);
        let all_suit = all_cards.suit(suit);
        
        if my_suit.is_empty() && pd_suit.is_empty() {
            println!("{}: void", SUIT_NAMES[suit]);
            continue;
        }
        
        // Count winners
        let mut my_winners = 0;
        let mut pd_winners = 0;
        for card in all_suit.iter() {
            if my_suit.have(card) {
                my_winners += 1;
            } else if pd_suit.have(card) {
                pd_winners += 1;
            } else {
                break;
            }
        }
        
        println!("{}: E has {} cards ({} winners), W has {} cards ({} winners)", 
            SUIT_NAMES[suit], my_suit.size(), my_winners, pd_suit.size(), pd_winners);
        
        // Check blocking
        if !my_suit.is_empty() && !pd_suit.is_empty() {
            let my_top = my_suit.top();
            let my_bot = my_suit.bottom();
            let pd_top = pd_suit.top();
            let pd_bot = pd_suit.bottom();
            println!("  E: top={}, bot={}", my_top, my_bot);
            println!("  W: top={}, bot={}", pd_top, pd_bot);
            if lower_rank(my_top, pd_bot) {
                println!("  -> Blocked by partner (my top < pd bottom)");
            }
            if higher_rank(my_bot, pd_top) {
                println!("  -> Blocked by me (my bottom > pd top)");
            }
        }
    }
}
