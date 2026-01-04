//! Test lead ordering - uses the actual order_leads/order_follows from solver
//! Run: cargo run --example test_lead_order

use dealer_dds::solver2::{
    Cards, Hands, Seat, WEST, NOTRUMP,
    order_leads, order_follows,
};
use dealer_dds::solver2::cards::name_of;

fn main() {
    // Test deal from quick_test_8.txt:
    // North: AKQ J6 KJ 9
    // West:  65 AK4 AQ T
    // East:  J7 QT9 T AK
    // South: 98 87 96 QJ

    let hands = Hands::from_solver_format(
        "AKQ J6 KJ 9",   // North
        "65 AK4 AQ T",   // West
        "J7 QT9 T AK",   // East
        "98 87 96 QJ"    // South
    ).unwrap();

    println!("Hands:\n{}", hands);

    let all_cards = hands.all_cards();
    let west = hands[WEST];

    println!("=== Test: West leads in NT ===");
    print!("West hand: ");
    for card in west.iter() {
        print!("{} ", name_of(card));
    }
    println!("\n");

    let seat = WEST;
    let trump = NOTRUMP;

    // Call the REAL order_leads function
    let ordered = order_leads(west, &hands, seat, trump, all_cards);

    print!("Ordered cards (from real order_leads): ");
    for card in ordered.iter() {
        print!("{} ", name_of(card));
    }
    println!();
}
