//! Test harness for comparing Rust solver2 against C++ reference solver
//!
//! Generates random 5-7 card endgames and compares results from both solvers.
//! This helps find bugs in the Rust implementation.
//!
//! Usage:
//!   cargo run --example endgame_compare --release -- [options]
//!
//! Options:
//!   --count N       Number of random deals to test (default: 100)
//!   --cards N       Cards per hand (default: 5, range: 1-13)
//!   --seed N        Random seed (default: random)
//!   --verbose       Print all deals, not just failures
//!   --stop          Stop on first failure

use dealer_dds::solver2::{Hands, Solver, NOTRUMP, SPADE, HEART, DIAMOND, CLUB};
use dealer_dds::solver2::{WEST, NORTH, EAST, SOUTH};
use dealer_dds::solver2::cards::rank_of;
use dealer_dds::solver2::types::{rank_name, NUM_SUITS};

use std::io::Write;
use std::process::{Command, Stdio};

/// Generate a random endgame with the specified number of cards per hand
fn generate_random_endgame(cards_per_hand: usize, rng: &mut Rng) -> Hands {
    assert!(cards_per_hand >= 1 && cards_per_hand <= 13);

    // Create deck of all 52 cards
    let mut deck: Vec<usize> = (0..52).collect();

    // Shuffle using Fisher-Yates
    for i in (1..deck.len()).rev() {
        let j = rng.next_usize() % (i + 1);
        deck.swap(i, j);
    }

    // Deal cards_per_hand cards to each of 4 hands
    let mut hands = Hands::new();
    for (i, &card) in deck.iter().take(cards_per_hand * 4).enumerate() {
        let seat = i % 4;
        hands[seat].add(card);
    }

    hands
}

/// Simple random number generator (xorshift64)
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: if seed == 0 { 0x853c49e6748fea9b } else { seed } }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_usize(&mut self) -> usize {
        self.next_u64() as usize
    }
}

/// Convert hands to C++ solver format (4 lines: N, W, E, S with suits as SHDC)
fn to_cpp_format(hands: &Hands) -> String {
    let mut result = String::new();

    // Order: North, West, East, South (matches C++ solver expectation)
    for &seat in &[NORTH, WEST, EAST, SOUTH] {
        let hand = hands[seat];
        for suit in 0..NUM_SUITS {
            let suit_cards = hand.suit(suit);
            if suit > 0 {
                result.push(' ');
            }
            if suit_cards.is_empty() {
                result.push('-');
            } else {
                for card in suit_cards.iter() {
                    result.push(rank_name(rank_of(card)));
                }
            }
        }
        result.push('\n');
    }

    result
}

/// Convert hands to PBN format for display
fn to_pbn(hands: &Hands) -> String {
    let mut result = String::from("N:");

    for seat in 0..4 {
        let actual_seat = (NORTH + seat) % 4;
        if seat > 0 {
            result.push(' ');
        }
        let hand = hands[actual_seat];
        for suit in 0..NUM_SUITS {
            if suit > 0 {
                result.push('.');
            }
            let suit_cards = hand.suit(suit);
            if suit_cards.is_empty() {
                // Empty suit - nothing to write (PBN uses nothing or -)
            } else {
                for card in suit_cards.iter() {
                    result.push(rank_name(rank_of(card)));
                }
            }
        }
    }

    result
}

/// Run C++ solver on the given hands and return NS tricks
fn run_cpp_solver(hands: &Hands, trump: usize, leader: usize) -> Result<u8, String> {
    // Try multiple possible paths for the C++ solver
    let possible_paths = [
        std::path::PathBuf::from("/Users/rick/Documents/GitHub/dealer3/bridge-solver/solver"),
        std::env::current_dir()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("bridge-solver/solver")))
            .unwrap_or_default(),
        std::env::current_dir()
            .ok()
            .map(|p| p.join("../bridge-solver/solver"))
            .unwrap_or_default(),
    ];

    let cpp_solver = possible_paths
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| format!("C++ solver not found in any of: {:?}", possible_paths))?;

    // Create input file content
    let mut input = to_cpp_format(hands);

    // Add trump and leader line
    let trump_char = match trump {
        0 => 'S',
        1 => 'H',
        2 => 'D',
        3 => 'C',
        4 => 'N',
        _ => return Err("Invalid trump".to_string()),
    };
    let leader_char = match leader {
        0 => 'W',
        1 => 'N',
        2 => 'E',
        3 => 'S',
        _ => return Err("Invalid leader".to_string()),
    };
    input.push_str(&format!("{} {}\n", trump_char, leader_char));

    // Write to temp file
    let temp_file = "/tmp/endgame_test.txt";
    std::fs::write(temp_file, &input)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Run solver
    let output = Command::new(&cpp_solver)
        .args(&["-f", temp_file])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run solver: {}", e))?;

    if !output.status.success() {
        return Err(format!("Solver failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // Parse output - look for line like "N  8  0.00 s 3712.0 M"
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // Try to parse as trump + tricks format
            if let Ok(tricks) = parts[1].parse::<u8>() {
                // Verify first part is a trump indicator
                if parts[0].len() == 1 && "SHDCN".contains(parts[0]) {
                    // C++ solver returns tricks for the non-leader's side
                    // Convert to NS tricks:
                    // - If EW leads (W=0 or E=2), result is NS tricks (use directly)
                    // - If NS leads (N=1 or S=3), result is EW tricks (convert)
                    let num_tricks = hands.num_tricks() as u8;
                    let ns_tricks = if leader == NORTH || leader == SOUTH {
                        // NS led, so result is EW tricks
                        num_tricks - tricks
                    } else {
                        // EW led, so result is NS tricks
                        tricks
                    };
                    return Ok(ns_tricks);
                }
            }
        }
    }

    Err(format!("Could not parse solver output: {}", stdout))
}

/// Run Rust solver on the given hands
fn run_rust_solver(hands: &Hands, trump: usize, leader: usize) -> u8 {
    let solver = Solver::new(*hands, trump, leader);
    solver.solve()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut count = 100;
    let mut cards_per_hand = 5;
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    let mut verbose = false;
    let mut stop_on_failure = false;

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--count" => {
                i += 1;
                count = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(100);
            }
            "--cards" => {
                i += 1;
                cards_per_hand = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(5);
            }
            "--seed" => {
                i += 1;
                seed = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(seed);
            }
            "--verbose" => verbose = true,
            "--stop" => stop_on_failure = true,
            _ => {}
        }
        i += 1;
    }

    println!("Endgame Comparison Test");
    println!("=======================");
    println!("Cards per hand: {}", cards_per_hand);
    println!("Number of deals: {}", count);
    println!("Seed: {}", seed);
    println!();

    let mut rng = Rng::new(seed);
    let mut passed = 0;
    let mut failed = 0;
    let mut errors = 0;

    let trumps = [NOTRUMP, SPADE, HEART, DIAMOND, CLUB];
    let trump_names = ["NT", "S", "H", "D", "C"];
    let leaders = [WEST, NORTH, EAST, SOUTH];
    let leader_names = ["W", "N", "E", "S"];

    for deal_num in 0..count {
        let hands = generate_random_endgame(cards_per_hand, &mut rng);
        let pbn = to_pbn(&hands);

        if verbose {
            println!("Deal {}: {}", deal_num + 1, pbn);
        }

        // Test all trump suits and all leaders
        for (trump_idx, &trump) in trumps.iter().enumerate() {
            for (leader_idx, &leader) in leaders.iter().enumerate() {
                let rust_result = run_rust_solver(&hands, trump, leader);

                match run_cpp_solver(&hands, trump, leader) {
                    Ok(cpp_result) => {
                        if rust_result == cpp_result {
                            passed += 1;
                            if verbose {
                                println!("  {} lead {}: Rust={} C++={} OK",
                                    trump_names[trump_idx], leader_names[leader_idx],
                                    rust_result, cpp_result);
                            }
                        } else {
                            failed += 1;
                            println!("\nFAILURE at deal {}!", deal_num + 1);
                            println!("PBN: {}", pbn);
                            println!("Trump: {} ({}), Leader: {} ({})",
                                trump_names[trump_idx], trump,
                                leader_names[leader_idx], leader);
                            println!("Rust solver: {} tricks", rust_result);
                            println!("C++ solver:  {} tricks", cpp_result);
                            println!("Hands:\n{}", hands);

                            // Save failing case
                            let fail_file = format!("/tmp/fail_{}_{}.txt",
                                deal_num, trump_names[trump_idx]);
                            let mut content = to_cpp_format(&hands);
                            let trump_char = match trump {
                                0 => 'S', 1 => 'H', 2 => 'D', 3 => 'C', _ => 'N'
                            };
                            let leader_char = match leader {
                                0 => 'W', 1 => 'N', 2 => 'E', _ => 'S'
                            };
                            content.push_str(&format!("{} {}\n", trump_char, leader_char));
                            let _ = std::fs::write(&fail_file, &content);
                            println!("Saved to: {}", fail_file);

                            if stop_on_failure {
                                println!("\nStopping on first failure (--stop)");
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        errors += 1;
                        if verbose {
                            println!("  {} lead {}: ERROR - {}",
                                trump_names[trump_idx], leader_names[leader_idx], e);
                        }
                    }
                }
            }
        }

        // Progress indicator
        if !verbose && (deal_num + 1) % 10 == 0 {
            print!(".");
            std::io::stdout().flush().unwrap();
        }
    }

    println!("\n");
    println!("Results:");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("  Errors: {}", errors);

    if failed > 0 {
        println!("\nSome tests FAILED. Check the failure output above.");
        std::process::exit(1);
    } else if errors > 0 {
        println!("\nSome tests had errors (C++ solver issues).");
    } else {
        println!("\nAll tests passed!");
    }
}
