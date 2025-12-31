use dealer_core::DealGenerator;
use dealer_pbn::format_oneline;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let seed: u32 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let count: usize = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    let mut generator = DealGenerator::new(seed);

    for _ in 0..count {
        let deal = generator.generate();
        let oneline = format_oneline(&deal);
        println!("{}", oneline);
    }
}
