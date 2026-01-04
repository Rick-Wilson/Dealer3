//! Quick test of 13-card deal (full deal from Test 2)

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, get_node_count};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH};
use std::time::Instant;

fn main() {
    // Full 13-card deal from Test 2
    // N: AKQT3.J6.KJ42.95 (5+2+4+2=13)
    // E: 652.AK42.AQ87.T4 (3+4+4+2=13)
    // S: J74.QT95.T.AK863 (3+4+1+5=13)
    // W: 98.873.9653.QJ72 (2+3+4+4=13)
    let pbn = "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72";
    let hands = Hands::from_pbn(pbn).expect("Should parse");

    println!("13-card deal (Test 2):");
    println!("N: AKQT3.J6.KJ42.95");
    println!("E: 652.AK42.AQ87.T4");
    println!("S: J74.QT95.T.AK863");
    println!("W: 98.873.9653.QJ72");
    println!();

    // Expected values from C++ solver (verified)
    // All leaders result in NS=9 tricks
    for (leader, name, expected) in [(WEST, "W", 9), (NORTH, "N", 9), (EAST, "E", 9), (SOUTH, "S", 9)] {
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
