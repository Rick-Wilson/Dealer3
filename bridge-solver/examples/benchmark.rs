use dealer_core::DealGenerator;
use dealer_dds::{Denomination, DoubleDummySolver};
use std::time::Instant;

fn main() {
    let mut gen = DealGenerator::new(42);

    // Benchmark single denomination
    println!("Benchmarking single denomination solve...");
    let start = Instant::now();
    let deal = gen.generate();
    let solver = DoubleDummySolver::new(deal);
    let tricks = solver.solve(Denomination::Spades, dealer_core::Position::North);
    let elapsed = start.elapsed();
    println!("Single solve: {:?} ({} tricks)", elapsed, tricks);

    // Benchmark solve_all (5 denominations × 4 positions = 20 solves)
    println!("\nBenchmarking solve_all (20 solves)...");
    let start = Instant::now();
    let deal = gen.generate();
    let solver = DoubleDummySolver::new(deal);
    let _result = solver.solve_all();
    let elapsed = start.elapsed();
    println!("solve_all: {:?}", elapsed);
    println!("Per solve: {:?}", elapsed / 20);

    // Benchmark multiple deals
    println!("\nBenchmarking 10 deals (200 solves total)...");
    let start = Instant::now();
    for _ in 0..10 {
        let deal = gen.generate();
        let solver = DoubleDummySolver::new(deal);
        let _result = solver.solve_all();
    }
    let elapsed = start.elapsed();
    println!("10 deals: {:?}", elapsed);
    println!("Per deal: {:?}", elapsed / 10);
    println!("Deals per second: {:.2}", 10.0 / elapsed.as_secs_f64());
}
