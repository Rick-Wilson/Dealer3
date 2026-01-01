use clap::Parser;
use dealer_core::{DealGenerator, Position};
use dealer_eval::{eval, eval_program, EvalContext};
use dealer_parser::{ActionType, Expr, Statement, VulnerabilityType};
use dealer_pbn::{format_oneline, format_printall, format_printew, format_printpbn, format_printcompact, Vulnerability};
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "dealer")]
#[command(about = "Bridge hand generator with constraint evaluation", long_about = None)]
struct Args {
    /// Number of deals to produce (defaults to 40, or value from input file if not specified)
    /// Mutually exclusive with --generate
    #[arg(short = 'p', long = "produce", conflicts_with = "generate")]
    produce: Option<usize>,

    /// Maximum number of hands to generate (defaults to 1000000)
    /// Reports all matching deals found. Mutually exclusive with --produce
    #[arg(short = 'g', long = "generate", conflicts_with = "produce")]
    generate: Option<usize>,

    /// Random seed for generation (defaults to current time)
    #[arg(short = 's', long = "seed")]
    seed: Option<u32>,

    /// Output format (defaults to oneline, or value from input file if not specified)
    #[arg(short = 'f', long = "format")]
    format: Option<OutputFormat>,

    /// Dealer position (N/E/S/W) - used with PBN format (defaults to rotating, or value from input file if not specified)
    #[arg(short = 'd', long = "dealer")]
    dealer: Option<DealerPosition>,

    /// Vulnerability (None/NS/EW/All) - used with PBN format (defaults to rotating, or value from input file if not specified)
    #[arg(short = 'v', long = "vulnerable")]
    vulnerability: Option<VulnerabilityArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    PrintAll,
    PrintEW,
    PrintPBN,
    PrintCompact,
    PrintOneLine,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "printall" | "all" => Ok(OutputFormat::PrintAll),
            "printew" | "ew" => Ok(OutputFormat::PrintEW),
            "printpbn" | "pbn" => Ok(OutputFormat::PrintPBN),
            "printcompact" | "compact" => Ok(OutputFormat::PrintCompact),
            "printoneline" | "oneline" => Ok(OutputFormat::PrintOneLine),
            _ => Err(format!(
                "Invalid format '{}'. Valid options: printall, printew, printpbn, printcompact, printoneline",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DealerPosition {
    North,
    East,
    South,
    West,
}

impl std::str::FromStr for DealerPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "N" | "NORTH" => Ok(DealerPosition::North),
            "E" | "EAST" => Ok(DealerPosition::East),
            "S" | "SOUTH" => Ok(DealerPosition::South),
            "W" | "WEST" => Ok(DealerPosition::West),
            _ => Err(format!(
                "Invalid dealer position '{}'. Valid options: N, E, S, W, North, East, South, West",
                s
            )),
        }
    }
}

impl From<DealerPosition> for Position {
    fn from(dp: DealerPosition) -> Self {
        match dp {
            DealerPosition::North => Position::North,
            DealerPosition::East => Position::East,
            DealerPosition::South => Position::South,
            DealerPosition::West => Position::West,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VulnerabilityArg {
    None,
    NS,
    EW,
    All,
}

impl std::str::FromStr for VulnerabilityArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NONE" | "NEITHER" => Ok(VulnerabilityArg::None),
            "NS" | "N-S" | "NORTH-SOUTH" => Ok(VulnerabilityArg::NS),
            "EW" | "E-W" | "EAST-WEST" => Ok(VulnerabilityArg::EW),
            "ALL" | "BOTH" => Ok(VulnerabilityArg::All),
            _ => Err(format!(
                "Invalid vulnerability '{}'. Valid options: None, NS, EW, All",
                s
            )),
        }
    }
}

impl From<VulnerabilityArg> for Vulnerability {
    fn from(va: VulnerabilityArg) -> Self {
        match va {
            VulnerabilityArg::None => Vulnerability::None,
            VulnerabilityArg::NS => Vulnerability::NS,
            VulnerabilityArg::EW => Vulnerability::EW,
            VulnerabilityArg::All => Vulnerability::All,
        }
    }
}

/// Convert VulnerabilityType to Vulnerability
fn vulnerability_type_to_vulnerability(vt: VulnerabilityType) -> Vulnerability {
    match vt {
        VulnerabilityType::None => Vulnerability::None,
        VulnerabilityType::NS => Vulnerability::NS,
        VulnerabilityType::EW => Vulnerability::EW,
        VulnerabilityType::All => Vulnerability::All,
    }
}

fn main() {
    let args = Args::parse();

    // Use provided seed or default to current time (microsecond resolution)
    let seed = args.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros() as u32
    });

    // Read constraint from stdin
    let mut constraint_str = String::new();
    io::stdin()
        .read_to_string(&mut constraint_str)
        .expect("Failed to read constraint from stdin");

    let constraint_str = constraint_str.trim();

    // Preprocess to mark 4-digit numbers in shape() functions
    let preprocessed = dealer_parser::preprocess(constraint_str);

    // Parse the program (may include variable assignments and action blocks)
    let program = match dealer_parser::parse_program(&preprocessed) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Extract action block directives from the program
    let mut produce_count_from_input: Option<usize> = None;
    let mut format_from_input: Option<OutputFormat> = None;
    let mut dealer_from_input: Option<DealerPosition> = None;
    let mut vuln_from_input: Option<VulnerabilityArg> = None;

    // Track average statements: (label, expression, sum, count)
    let mut averages: Vec<(Option<String>, Expr, f64, usize)> = Vec::new();

    // Track frequency statements: (label, expression, histogram, range)
    use std::collections::HashMap;
    let mut frequencies: Vec<(Option<String>, Expr, HashMap<i32, usize>, Option<(i32, i32)>)> = Vec::new();

    for statement in &program.statements {
        match statement {
            Statement::Produce(n) => produce_count_from_input = Some(*n),
            Statement::Action { averages: avg_specs, frequencies: freq_specs, format: action_format } => {
                // Extract format if present
                if let Some(action_type) = action_format {
                    format_from_input = Some(match action_type {
                        ActionType::PrintAll => OutputFormat::PrintAll,
                        ActionType::PrintEW => OutputFormat::PrintEW,
                        ActionType::PrintPBN => OutputFormat::PrintPBN,
                        ActionType::PrintCompact => OutputFormat::PrintCompact,
                        ActionType::PrintOneLine => OutputFormat::PrintOneLine,
                    });
                }
                // Extract averages if present
                for avg_spec in avg_specs {
                    averages.push((avg_spec.label.clone(), avg_spec.expr.clone(), 0.0, 0));
                }
                // Extract frequencies if present
                for freq_spec in freq_specs {
                    frequencies.push((freq_spec.label.clone(), freq_spec.expr.clone(), HashMap::new(), freq_spec.range));
                }
            }
            Statement::Dealer(pos) => {
                dealer_from_input = Some(match pos {
                    Position::North => DealerPosition::North,
                    Position::East => DealerPosition::East,
                    Position::South => DealerPosition::South,
                    Position::West => DealerPosition::West,
                });
            }
            Statement::Vulnerable(vuln) => {
                vuln_from_input = Some(match *vuln {
                    VulnerabilityType::None => VulnerabilityArg::None,
                    VulnerabilityType::NS => VulnerabilityArg::NS,
                    VulnerabilityType::EW => VulnerabilityArg::EW,
                    VulnerabilityType::All => VulnerabilityArg::All,
                });
            }
            _ => {}
        }
    }

    // Determine mode: generate or produce
    let generate_mode = args.generate.is_some();
    let max_generate = args.generate.unwrap_or(1_000_000); // dealer.exe default for -g

    // Command-line flags override input file values
    // In generate mode, produce_count is only used if specified in input file
    let produce_count = if generate_mode {
        produce_count_from_input.unwrap_or(usize::MAX) // No limit in generate mode
    } else {
        args.produce
            .or(produce_count_from_input)
            .unwrap_or(40) // dealer.exe default for -p
    };

    let output_format = args.format
        .or(format_from_input)
        .unwrap_or(OutputFormat::PrintOneLine); // Default format

    let dealer_position = args.dealer
        .or(dealer_from_input);

    let vulnerability = args.vulnerability
        .or(vuln_from_input);

    // Start timing
    let start_time = SystemTime::now();

    // Initialize deal generator
    let mut generator = DealGenerator::new(seed);

    // Apply predeal statements
    for statement in &program.statements {
        if let Statement::Predeal { position, cards } = statement {
            if let Err(e) = generator.predeal(*position, cards) {
                eprintln!("Predeal error: {}", e);
                std::process::exit(1);
            }
        }
    }

    let mut produced = 0;
    let mut generated = 0;

    // Generate deals until we reach the limit
    // In produce mode: stop when we've produced enough matching deals
    // In generate mode: stop when we've generated enough total deals
    while if generate_mode {
        generated < max_generate
    } else {
        produced < produce_count
    } {
        let deal = generator.generate();
        generated += 1;

        // Evaluate program (includes variable assignments and final constraint)
        match eval_program(&program, &deal) {
            Ok(result) if result != 0 => {
                // Constraint satisfied (non-zero = true)

                // Calculate averages for this matching deal
                if !averages.is_empty() || !frequencies.is_empty() {
                    let ctx = EvalContext::new(&deal);

                    for (_, expr, sum, count) in &mut averages {
                        match eval(expr, &ctx) {
                            Ok(val) => {
                                *sum += val as f64;
                                *count += 1;
                            }
                            Err(e) => {
                                eprintln!("Average evaluation error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }

                    // Calculate frequencies for this matching deal
                    for (_, expr, histogram, _) in &mut frequencies {
                        match eval(expr, &ctx) {
                            Ok(val) => {
                                *histogram.entry(val).or_insert(0) += 1;
                            }
                            Err(e) => {
                                eprintln!("Frequency evaluation error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }

                let output = match output_format {
                    OutputFormat::PrintAll => format_printall(&deal, produced),
                    OutputFormat::PrintEW => format_printew(&deal),
                    OutputFormat::PrintPBN => {
                        let dealer_pos = dealer_position.map(|d| d.into());
                        let vuln = vulnerability.map(|v| v.into());
                        format_printpbn(&deal, produced, dealer_pos, vuln, None, Some(seed))
                    }
                    OutputFormat::PrintCompact => format_printcompact(&deal),
                    OutputFormat::PrintOneLine => format_oneline(&deal),
                };
                print!("{}", output);
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

    // Calculate elapsed time
    let elapsed = start_time.elapsed().unwrap();
    let elapsed_secs = elapsed.as_secs_f64();

    // Print averages if any were requested
    if !averages.is_empty() {
        eprintln!();
        for (label, _, sum, count) in &averages {
            let avg = if *count > 0 { sum / (*count as f64) } else { 0.0 };
            if let Some(label_text) = label {
                eprintln!("{}: {:.2}", label_text, avg);
            } else {
                eprintln!("Average: {:.2}", avg);
            }
        }
    }

    // Print frequency tables if any were requested
    if !frequencies.is_empty() {
        for (label, _, histogram, range) in &frequencies {
            eprintln!();
            if let Some(label_text) = label {
                eprintln!("{}:", label_text);
            } else {
                eprintln!("Frequency:");
            }

            // Determine range to display
            let (min_val, max_val) = if let Some((min, max)) = range {
                (*min, *max)
            } else if !histogram.is_empty() {
                let min = *histogram.keys().min().unwrap();
                let max = *histogram.keys().max().unwrap();
                (min, max)
            } else {
                (0, 0)
            };

            // Print frequency table
            for val in min_val..=max_val {
                let count = histogram.get(&val).unwrap_or(&0);
                let percentage = if produced > 0 {
                    (*count as f64 / produced as f64) * 100.0
                } else {
                    0.0
                };
                eprintln!("{:3} {:6} ({:5.2}%)", val, count, percentage);
            }
        }
    }

    // Print statistics to stderr (like dealer.exe does)
    eprintln!();
    eprintln!("Generated {} hands", generated);
    eprintln!("Produced {} hands", produced);
    eprintln!("Initial random seed {}", seed);
    eprintln!("Time needed  {:7.3} sec", elapsed_secs);
}
