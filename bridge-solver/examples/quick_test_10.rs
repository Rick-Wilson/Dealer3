//! Quick test of 10-card deal (Test 2 minus 3 lowest cards from each hand)

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, get_node_count};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH};
use std::time::Instant;

fn main() {
    // 10-card deal from Test 2, removing 3 lowest cards from each hand:
    // N: AKQT3.J6.KJ42.95 -> AKQT3.J6.KJ.9 (5+2+2+1=10)
    // E: 652.AK42.AQ87.T4 -> 652.AK4.AQ8.T (3+3+3+1=10)
    // S: J74.QT95.T.AK863 -> J74.QT9.T.AK8 (3+3+1+3=10)
    // W: 98.873.9653.QJ72 -> 98.87.965.QJ7 (2+2+3+3=10)
    let pbn = "N:AKQT3.J6.KJ.9 652.AK4.AQ8.T J74.QT9.T.AK8 98.87.965.QJ7";
    let hands = Hands::from_pbn(pbn).expect("Should parse");

    println!("10-card deal (Test 2 - 3 lowest cards each):");
    println!("N: AKQT3.J6.KJ.9");
    println!("E: 652.AK4.AQ8.T");
    println!("S: J74.QT9.T.AK8");
    println!("W: 98.87.965.QJ7");
    println!();

    // Expected values from C++ solver (verified)
    for (leader, name, expected) in [(WEST, "W", 5), (NORTH, "N", 7), (EAST, "E", 7), (SOUTH, "S", 7)] {
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
