//! Quick test of solve_v2 comparing against original solver

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, WEST, NORTH, EAST, SOUTH};
use dealer_dds::solver2::{set_no_pruning, set_no_tt};

fn main() {
    // Disable TT and pruning for cleaner comparison
    set_no_pruning(true);
    set_no_tt(true);

    // Test 1: Simple 4-trick deal
    test_deal("N:A.A.A.A K.K.K.K 2.2.2.2 3.3.3.3", "4-trick aces", NOTRUMP, WEST);

    // Test 2: 8-card deal from quick_test_8.txt
    // AKQ J6 KJ 9  (North)
    // 65 AK4 AQ T  (West)     J7 QT9 T AK  (East)
    // 98 87 96 QJ  (South)
    test_deal("N:AKQ.J6.KJ.9 J7.QT9.T.AK 98.87.96.QJ 65.AK4.AQ.T", "8-card deal", NOTRUMP, WEST);
    test_deal("N:AKQ.J6.KJ.9 J7.QT9.T.AK 98.87.96.QJ 65.AK4.AQ.T", "8-card deal", NOTRUMP, NORTH);
}

fn test_deal(pbn: &str, name: &str, trump: usize, leader: usize) {
    let hands = match Hands::from_pbn(pbn) {
        Some(h) => h,
        None => {
            println!("Failed to parse {}", name);
            return;
        }
    };

    let leader_name = match leader {
        0 => "West",
        1 => "North",
        2 => "East",
        3 => "South",
        _ => "?",
    };

    let solver = Solver::new(hands, trump, leader);
    let v1 = solver.solve();
    let v2 = solver.solve_v2();

    if v1 == v2 {
        println!("✓ {} ({} leads): solve()={} solve_v2()={}", name, leader_name, v1, v2);
    } else {
        println!("✗ {} ({} leads): solve()={} solve_v2()={} MISMATCH!", name, leader_name, v1, v2);
    }
}
