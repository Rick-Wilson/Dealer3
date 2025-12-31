use clap::Parser;
use dealer_core::DealGenerator;
use dealer_eval::{eval, EvalContext};
use dealer_pbn::format_oneline;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "dealer")]
#[command(about = "Bridge hand generator with constraint evaluation", long_about = None)]
struct Args {
    /// Number of deals to produce
    #[arg(short = 'p', long = "produce", default_value = "10")]
    produce: usize,

    /// Random seed for generation
    #[arg(short = 's', long = "seed", default_value = "0")]
    seed: u32,
}

fn main() {
    let args = Args::parse();

    // Read constraint from stdin
    let mut constraint_str = String::new();
    io::stdin()
        .read_to_string(&mut constraint_str)
        .expect("Failed to read constraint from stdin");

    let constraint_str = constraint_str.trim();

    // Parse the constraint
    let ast = match dealer_parser::parse(constraint_str) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize deal generator
    let mut generator = DealGenerator::new(args.seed);

    let mut produced = 0;
    let mut generated = 0;

    // Generate deals until we produce the requested number
    while produced < args.produce {
        let deal = generator.generate();
        generated += 1;

        // Evaluate constraint
        let ctx = EvalContext::new(&deal);
        match eval(&ast, &ctx) {
            Ok(result) if result != 0 => {
                // Constraint satisfied (non-zero = true)
                println!("{}", format_oneline(&deal));
                produced += 1;
            }
            Ok(_) => {
                // Constraint not satisfied (zero = false)
                continue;
            }
            Err(e) => {
                eprintln!("Evaluation error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Print statistics to stderr (like dealer.exe does)
    eprintln!("Generated {} deals", generated);
    eprintln!("Produced {} deals", produced);
}
