//! Debug the solver2 on a specific failing case
//!
//! From C++ solver file /tmp/fail_0_NT.txt:
//! 7 76 5 2       <- North: S7 H76 D5 C2
//! A8 - T8 9      <- West:  SA8 - DT8 C9
//! Q J - 865      <- East:  SQ HJ - C865
//! 42 - J9 A      <- South: S42 - DJ9 CA
//! N S            <- NT with South leading
//! C++ result: NS makes 3 tricks

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, SPADE, HEART, DIAMOND, CLUB};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH};

fn main() {
    // Build hands directly using from_solver_format which takes N, W, E, S
    // North: S7 H76 D5 C2
    // West:  SA8 - DT8 C9
    // East:  SQ HJ - C865
    // South: S42 - DJ9 CA
    let hands = Hands::from_solver_format(
        "7 76 5 2",    // North: SHDC
        "A8 - T8 9",   // West: SHDC
        "Q J - 865",   // East: SHDC
        "42 - J9 A",   // South: SHDC
    ).expect("Should parse");

    println!("Testing failing case:");
    println!("Hands by seat:");
    println!("  WEST ({}):  {}", WEST, hands[WEST]);
    println!("  NORTH ({}): {}", NORTH, hands[NORTH]);
    println!("  EAST ({}):  {}", EAST, hands[EAST]);
    println!("  SOUTH ({}): {}", SOUTH, hands[SOUTH]);
    println!();
    println!("Cards per hand: {}", hands.num_tricks());
    println!();

    // Test all trump/leader combinations
    let trumps = [(NOTRUMP, "NT"), (SPADE, "S"), (HEART, "H"), (DIAMOND, "D"), (CLUB, "C")];
    let leaders = [(WEST, "W"), (NORTH, "N"), (EAST, "E"), (SOUTH, "S")];

    println!("Results (Rust solver2):");
    for (trump, trump_name) in trumps.iter() {
        for (leader, leader_name) in leaders.iter() {
            let solver = Solver::new(hands, *trump, *leader);
            let ns_tricks = solver.solve();
            println!("  {} trump, {} leads: NS makes {} tricks", trump_name, leader_name, ns_tricks);
        }
    }

    println!();
    println!("Expected (from C++ solver with NT, South leads): NS makes 3 tricks");
}
