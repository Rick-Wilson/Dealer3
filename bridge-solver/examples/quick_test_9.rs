//! Quick test of 9-card deal (Test 2 minus 4 lowest cards from each hand)

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, get_node_count};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH};
use std::time::Instant;

fn main() {
    // 9-card deal from Test 2, removing 4 lowest cards from each hand:
    // N: AKQT3.J6.KJ42.95 -> AKQT.J6.KJ.9 (4+2+2+1=9)
    // E: 652.AK42.AQ87.T4 -> 65.AK4.AQ8.T (2+3+3+1=9)
    // S: J74.QT95.T.AK863 -> J7.QT9.T.AK8 (2+3+1+3=9)
    // W: 98.873.9653.QJ72 -> 98.87.96.QJ7 (2+2+2+3=9)
    let pbn = "N:AKQT.J6.KJ.9 65.AK4.AQ8.T J7.QT9.T.AK8 98.87.96.QJ7";
    let hands = Hands::from_pbn(pbn).expect("Should parse");

    println!("9-card deal (Test 2 - 4 lowest cards each):");
    println!("N: AKQT.J6.KJ.9");
    println!("E: 65.AK4.AQ8.T");
    println!("S: J7.QT9.T.AK8");
    println!("W: 98.87.96.QJ7");
    println!();

    // Expected values from C++ solver (verified)
    for (leader, name, expected) in [(WEST, "W", 4), (NORTH, "N", 6), (EAST, "E", 6), (SOUTH, "S", 6)] {
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
