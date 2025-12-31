use gnurandom::GnuRandom;

fn main() {
    let mut rng = GnuRandom::new();
    rng.srandom(42);

    println!("First 20 values for seed 42:");
    for _i in 0..20 {
        let val = rng.next_u32();
        println!("{}", val);
    }
}
