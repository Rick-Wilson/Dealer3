use clap::Parser;
use dealer_core::{DealGenerator, Position};
use dealer_eval::eval_program;
use dealer_parser::{ActionType, Statement, VulnerabilityType};
use dealer_pbn::{format_oneline, format_printall, format_printew, format_printpbn, format_printcompact, Vulnerability};
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "dealer")]
#[command(about = "Bridge hand generator with constraint evaluation", long_about = None)]
struct Args {
    /// Number of deals to produce (defaults to 10, or value from input file if not specified)
    #[arg(short = 'p', long = "produce")]
    produce: Option<usize>,

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

    for statement in &program.statements {
        match statement {
            Statement::Produce(n) => produce_count_from_input = Some(*n),
            Statement::Action(action_type) => {
                format_from_input = Some(match action_type {
                    ActionType::PrintAll => OutputFormat::PrintAll,
                    ActionType::PrintEW => OutputFormat::PrintEW,
                    ActionType::PrintPBN => OutputFormat::PrintPBN,
                    ActionType::PrintCompact => OutputFormat::PrintCompact,
                    ActionType::PrintOneLine => OutputFormat::PrintOneLine,
                });
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

    // Command-line flags override input file values
    let produce_count = args.produce
        .or(produce_count_from_input)
        .unwrap_or(40); // dealer.exe default

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

    let mut produced = 0;
    let mut generated = 0;

    // Generate deals until we produce the requested number
    while produced < produce_count {
        let deal = generator.generate();
        generated += 1;

        // Evaluate program (includes variable assignments and final constraint)
        match eval_program(&program, &deal) {
            Ok(result) if result != 0 => {
                // Constraint satisfied (non-zero = true)
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

    // Print statistics to stderr (like dealer.exe does)
    eprintln!("Generated {} hands", generated);
    eprintln!("Produced {} hands", produced);
    eprintln!("Initial random seed {}", seed);
    eprintln!("Time needed  {:7.3} sec", elapsed_secs);
}
