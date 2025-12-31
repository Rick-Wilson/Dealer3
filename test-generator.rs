#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! dealer-core = { path = "dealer-core" }
//! dealer-pbn = { path = "dealer-pbn" }
//! ```

use dealer_core::{DealGenerator, Position};
use dealer_pbn::format_deal_tag;
use std::env;

fn main() {
    // Get seed from command line, default to 1
    let args: Vec<String> = env::args().collect();
    let seed: u32 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    // Get number of deals from command line, default to 10
    let count: usize = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let mut generator = DealGenerator::new(seed);

    for _ in 0..count {
        let deal = generator.generate();
        let pbn = format_deal_tag(&deal, Position::North);
        println!("{}", pbn);
    }
}
