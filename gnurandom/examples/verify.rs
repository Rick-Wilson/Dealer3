use gnurandom::GnuRandom;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let seed: u32 = if args.len() > 1 {
        args[1].parse().unwrap_or(1)
    } else {
        1
    };

    let count: usize = if args.len() > 2 {
        args[2].parse().unwrap_or(20)
    } else {
        20
    };

    let mut rng = GnuRandom::new();
    rng.srandom(seed);

    for _ in 0..count {
        println!("{}", rng.next_u32());
    }
}
