use dealer_dds::solver2::{Hands, Solver, CLUB, SOUTH, WEST, NORTH, EAST, get_node_count};

fn main() {
    // PBN from endgame_compare failure:
    // N:852..QT64.K Q9.KJT5.K.T 64.A964.2.9 .Q83..AQ763
    // PBN order is N, E, S, W (clockwise from North)
    // So:
    //   N: 852..QT64.K  = S 852, H -, D QT64, C K
    //   E: Q9.KJT5.K.T  = S Q9, H KJT5, D K, C T  (WRONG - this should be EAST not WEST)
    //   S: 64.A964.2.9  = S 64, H A964, D 2, C 9
    //   W: .Q83..AQ763  = S -, H Q83, D -, C AQ763
    let hands = Hands::from_pbn("N:852..QT64.K Q9.KJT5.K.T 64.A964.2.9 .Q83..AQ763").unwrap();
    println!("Hands:\n{}", hands);
    println!("N: {:?}", hands[NORTH]);
    println!("E: {:?}", hands[EAST]);
    println!("S: {:?}", hands[SOUTH]);
    println!("W: {:?}", hands[WEST]);
    println!("Num tricks: {}", hands.num_tricks());

    println!("\nTesting all leaders with Clubs as trump:");
    for (leader, name) in [(WEST, "W"), (NORTH, "N"), (EAST, "E"), (SOUTH, "S")] {
        let solver = Solver::new(hands, CLUB, leader);
        let result = solver.solve();
        println!("  {} leads: NS = {} tricks", name, result);
    }
}
