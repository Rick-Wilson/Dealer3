//! Quick test of 8-card deal

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, get_node_count};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH};
use std::time::Instant;

fn main() {
    // 8-card deal derived from test case 1
    let hands = Hands::from_solver_format(
        "AKQ J6 KJ 9",   // North
        "65 AK4 AQ T",   // West
        "J7 QT9 T AK",   // East
        "98 87 96 QJ",   // South
    ).expect("Should parse");

    println!("8-card deal:");
    println!("N: AKQ J6 KJ 9");
    println!("W: 65 AK4 AQ T");
    println!("E: J7 QT9 T AK");
    println!("S: 98 87 96 QJ");
    println!();

    for (leader, name, expected) in [(WEST, "W", 1), (NORTH, "N", 3), (EAST, "E", 1), (SOUTH, "S", 3)] {
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
