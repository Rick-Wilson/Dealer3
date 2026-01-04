//! Debug fast tricks calculation for 11-card deal - call actual function

use dealer_dds::solver2::{Hands, NOTRUMP, EAST};

fn main() {
    // 11-card deal
    let pbn = "N:AKQT3.J6.KJ4.9 652.AK4.AQ87.T J74.QT95.T.AK8 98.873.965.QJ7";
    let hands = Hands::from_pbn(pbn).expect("Should parse");

    println!("11-card deal:");
    println!("N: AKQT3.J6.KJ4.9");
    println!("E: 652.AK4.AQ87.T");
    println!("S: J74.QT95.T.AK8");
    println!("W: 98.873.965.QJ7");
    println!();
    
    // We need to access the private fast_tricks method...
    // Let's just run the solver with debug output
}
