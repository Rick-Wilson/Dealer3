use gnurandom::GnuRandom;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let seed: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(42);
    let start_idx: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(310);
    let count: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(20);

    let mut rng = GnuRandom::new();
    rng.srandom(seed);

    // Skip to start index
    for _ in 0..start_idx {
        rng.next_u32();
    }

    println!("RNG values for seed {} starting at call #{}:", seed, start_idx);
    println!("(These are the values used for shuffling deal #{}):", (start_idx / 52) + 1);
    println!();

    for i in start_idx..(start_idx + count) {
        let val = rng.next_u32();
        let k = val >> 15; // Shift as we do in shuffle
        let j_masked = k & 0xFFFF;
        let card_idx = (j_masked % 52) as u8;

        println!("  Call {:3} = 0x{:08x}  >>15 = 0x{:04x}  &0xFFFF = {:5}  %52 = {:2}",
                 i, val, k, j_masked, card_idx);
    }
}
