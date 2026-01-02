use gnurandom::GnuRandom;

fn main() {
    let mut rng = GnuRandom::new();
    rng.srandom(1);

    println!("RNG values for seed 1:");
    for i in 0..60 {
        let val = rng.next_u32();
        if !(5..50).contains(&i) {
            println!("  [{}] = 0x{:08x} ({})", i, val, val);
        } else if i == 5 {
            println!("  ... (values 5-49) ...");
        }
    }
}
