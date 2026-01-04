//! Debug bug with N/S leads giving wrong result

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, get_node_count};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH};
use std::time::Instant;

fn main() {
    // 9-card deal from Test 2
    let pbn = "N:AKQT.J6.KJ.9 65.AK4.AQ8.T J7.QT9.T.AK8 98.87.96.QJ7";
    let hands = Hands::from_pbn(pbn).expect("Should parse");

    println!("9-card deal:");
    println!("N: AKQT.J6.KJ.9");
    println!("E: 65.AK4.AQ8.T");
    println!("S: J7.QT9.T.AK8");
    println!("W: 98.87.96.QJ7");
    println!();

    // Just test South lead
    let start = Instant::now();
    let solver = Solver::new(hands, NOTRUMP, SOUTH);
    let result = solver.solve();
    let nodes = get_node_count();
    let elapsed = start.elapsed();
    println!("S leads: NS={} (expected 6) {:?} {} nodes", result, elapsed, nodes);
}
