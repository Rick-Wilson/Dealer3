//! Quick test of 12-card deal (Test 2 minus lowest card from each hand)

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, get_node_count};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH};
use std::time::Instant;

fn main() {
    // 12-card deal from Test 2, removing lowest card from each hand:
    // N: AKQT3.J6.KJ42.95 -> AKQT3.J6.KJ42.9 (5+2+4+1=12)
    // E: 652.AK42.AQ87.T4 -> 652.AK42.AQ87.T (3+4+4+1=12)
    // S: J74.QT95.T.AK863 -> J74.QT95.T.AK86 (3+4+1+4=12)
    // W: 98.873.9653.QJ72 -> 98.873.9653.QJ7 (2+3+4+3=12)
    let pbn = "N:AKQT3.J6.KJ42.9 652.AK42.AQ87.T J74.QT95.T.AK86 98.873.9653.QJ7";
    let hands = Hands::from_pbn(pbn).expect("Should parse");

    println!("12-card deal (Test 2 - lowest cards):");
    println!("N: AKQT3.J6.KJ42.9");
    println!("E: 652.AK42.AQ87.T");
    println!("S: J74.QT95.T.AK86");
    println!("W: 98.873.9653.QJ7");
    println!();

    // Expected values from C++ solver (verified)
    for (leader, name, expected) in [(WEST, "W", 8), (NORTH, "N", 9), (EAST, "E", 9), (SOUTH, "S", 9)] {
        let start = Instant::now();
        let solver = Solver::new(hands, NOTRUMP, leader);
        let result = solver.solve();
        let nodes = get_node_count();
        let elapsed = start.elapsed();
        let status = if result == expected { "OK" } else { "FAIL" };
        let ns_per_node = elapsed.as_nanos() as f64 / nodes as f64;
        println!("{} leads: NS={} (expected {}) {:?} {} nodes, {:.1} ns/node {}",
            name, result, expected, elapsed, nodes, ns_per_node, status);
    }
}
