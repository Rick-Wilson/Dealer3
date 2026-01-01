use clap::Parser;
use dealer_core::{DealGenerator, Position};
use dealer_eval::{eval, eval_program, EvalContext};
use dealer_parser::{ActionType, Expr, Statement, VulnerabilityType};
use dealer_pbn::{format_oneline, format_printall, format_printew, format_printpbn, format_printcompact, format_hand_pbn, Vulnerability};
use std::fs::OpenOptions;
use std::io::{self, Read, Write, BufWriter};
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
    #[arg(long = "vulnerable")]
    vulnerability: Option<VulnerabilityArg>,

    /// Verbose output, prints statistics at the end of the run (matches dealer.exe -v)
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Print version information and exit (matches dealer.exe -V)
    #[arg(short = 'V', long = "version")]
    version: bool,

    /// Print license information and exit
    #[arg(long = "license")]
    license: bool,

    /// Print credits and exit
    #[arg(long = "credits")]
    credits: bool,

    /// Quiet mode - suppress deal output, only show statistics (matches dealer.exe -q)
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Show progress meter during generation (matches dealer.exe -m)
    #[arg(short = 'm', long = "progress")]
    progress: bool,

    /// CSV output file (append mode by default, use 'w:filename' for write mode)
    #[arg(short = 'C', long = "CSV")]
    csv_file: Option<String>,

    /// Title metadata for PBN output
    #[arg(short = 'T', long = "title")]
    title: Option<String>,

    /// Predeal cards to North (format: S8743,HA9,D642,CQT64)
    #[arg(short = 'N', long = "north")]
    north_predeal: Option<String>,

    /// Predeal cards to East (format: S8743,HA9,D642,CQT64)
    #[arg(short = 'E', long = "east")]
    east_predeal: Option<String>,

    /// Predeal cards to South (format: S8743,HA9,D642,CQT64)
    #[arg(short = 'S', long = "south")]
    south_predeal: Option<String>,

    /// Predeal cards to West (format: S8743,HA9,D642,CQT64)
    #[arg(short = 'W', long = "west")]
    west_predeal: Option<String>,

    // Deprecated switches - parse them to show helpful error messages
    /// DEPRECATED: 2-way swapping mode (not supported - incompatible with predeal)
    #[arg(short = '2', hide = true)]
    swap_2: bool,

    /// DEPRECATED: 3-way swapping mode (not supported - incompatible with predeal)
    #[arg(short = '3', hide = true)]
    swap_3: bool,

    /// DEPRECATED: Exhaust mode (experimental feature never completed)
    #[arg(short = 'e', hide = true)]
    exhaust: bool,

    /// DEPRECATED: Upper/lowercase toggle (cosmetic feature not implemented)
    #[arg(short = 'u', hide = true)]
    uppercase: bool,

    /// DEPRECATED: Library mode (conflicting meanings in dealer.exe vs DealerV2_4)
    #[arg(short = 'l', hide = true)]
    library: bool,
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

/// Parse predeal card string (format: S8743,HA9,D642,CQT64)
/// Returns a vector of cards
fn parse_predeal_cards(card_str: &str) -> Result<Vec<dealer_core::Card>, String> {
    use dealer_core::{Card, Suit, Rank};

    let mut cards = Vec::new();

    // Split by comma
    for token in card_str.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }

        // First character is suit indicator
        if token.is_empty() {
            return Err("Empty card token".to_string());
        }

        let mut chars = token.chars();
        let suit_char = chars.next().unwrap().to_uppercase().next().unwrap();

        let suit = match suit_char {
            'S' => Suit::Spades,
            'H' => Suit::Hearts,
            'D' => Suit::Diamonds,
            'C' => Suit::Clubs,
            _ => return Err(format!("Invalid suit character: {}", suit_char)),
        };

        // Remaining characters are ranks
        for rank_char in chars {
            let rank_char = rank_char.to_uppercase().next().unwrap();
            let rank = match rank_char {
                'A' => Rank::Ace,
                'K' => Rank::King,
                'Q' => Rank::Queen,
                'J' => Rank::Jack,
                'T' => Rank::Ten,
                '9' => Rank::Nine,
                '8' => Rank::Eight,
                '7' => Rank::Seven,
                '6' => Rank::Six,
                '5' => Rank::Five,
                '4' => Rank::Four,
                '3' => Rank::Three,
                '2' => Rank::Two,
                _ => return Err(format!("Invalid rank character: {}", rank_char)),
            };

            cards.push(Card::new(suit, rank));
        }
    }

    Ok(cards)
}

fn main() {
    let args = Args::parse();

    // Handle version flag (matches dealer.exe -V behavior)
    if args.version {
        println!("dealer3 version {}", env!("CARGO_PKG_VERSION"));
        println!("Rust implementation of dealer.exe");
        println!("Compatible with dealer.exe and DealerV2_4");
        std::process::exit(0);
    }

    // Handle license flag
    if args.license {
        println!("License");
        println!("-------");
        println!();
        println!("This software is released into the public domain under The Unlicense.");
        println!();
        println!("You are free to use, modify, distribute, and incorporate this software");
        println!("for any purpose, with or without modification.");
        println!();
        println!("The original dealer program was also released into the public domain.");
        println!("Other independent implementations may be licensed differently.");
        println!();
        println!("See the LICENSE file in the source repository for full details.");
        std::process::exit(0);
    }

    // Handle credits flag
    if args.credits {
        println!("Credits");
        println!("-------");
        println!();
        println!("Original dealer");
        println!("  Hans van Staveren (public domain)");
        println!();
        println!("Key contributors");
        println!("  Henk Uijterwaal");
        println!("  Bruce Moore");
        println!("  Francois Dellacherie");
        println!("  Robin Barker");
        println!("  Danil Suits");
        println!("  Alex Martelli");
        println!("  Paul Hankin");
        println!("  Micke Hovmoller");
        println!("  Paul Baxter");
        println!();
        println!("dealer2");
        println!("  Greg Morse (GPLv3, independent)");
        println!();
        println!("dealer3 (Rust edition)");
        println!("  Rick Wilson");
        println!();
        println!("See documentation for full contribution details.");
        std::process::exit(0);
    }

    // Check for deprecated switches and provide helpful error messages
    if args.swap_2 {
        eprintln!("Error: Switch '-2' (2-way swapping) is not supported in dealer3.");
        eprintln!();
        eprintln!("Reason: Swapping modes are incompatible with predeal functionality,");
        eprintln!("        which is a core feature of dealer3.");
        eprintln!();
        eprintln!("Suggestion: Remove the '-2' switch from your command.");
        eprintln!("            If you need swapping, use the original dealer.exe.");
        std::process::exit(1);
    }

    if args.swap_3 {
        eprintln!("Error: Switch '-3' (3-way swapping) is not supported in dealer3.");
        eprintln!();
        eprintln!("Reason: Swapping modes are incompatible with predeal functionality,");
        eprintln!("        which is a core feature of dealer3.");
        eprintln!();
        eprintln!("Suggestion: Remove the '-3' switch from your command.");
        eprintln!("            If you need swapping, use the original dealer.exe.");
        std::process::exit(1);
    }

    if args.exhaust {
        eprintln!("Error: Switch '-e' (exhaust mode) is not supported in dealer3.");
        eprintln!();
        eprintln!("Reason: Exhaust mode was an experimental alpha feature in dealer.exe");
        eprintln!("        that was never completed or documented.");
        eprintln!();
        eprintln!("Suggestion: Remove the '-e' switch from your command.");
        std::process::exit(1);
    }

    if args.uppercase {
        eprintln!("Error: Switch '-u' (upper/lowercase toggle) is not supported in dealer3.");
        eprintln!();
        eprintln!("Reason: This is a cosmetic feature with low priority.");
        eprintln!();
        eprintln!("Suggestion: Remove the '-u' switch from your command.");
        eprintln!("            dealer3 uses standard uppercase card symbols (AKQJT).");
        std::process::exit(1);
    }

    if args.library {
        eprintln!("Error: Switch '-l' (library mode) is not supported in dealer3.");
        eprintln!();
        eprintln!("Reason: The '-l' switch has conflicting meanings:");
        eprintln!("        - In dealer.exe: Read deals from library.dat");
        eprintln!("        - In DealerV2_4: Export to DL52 format");
        eprintln!();
        eprintln!("Suggestion: Remove the '-l' switch from your command.");
        eprintln!("            Future versions may add library support with a different switch.");
        std::process::exit(1);
    }

    // Use provided seed or default to current time (microsecond resolution)
    let seed = args.seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros() as u32
    });

    // Open CSV file if requested
    let mut csv_writer: Option<BufWriter<std::fs::File>> = None;
    if let Some(csv_arg) = &args.csv_file {
        let (filename, write_mode) = if csv_arg.starts_with("w:") {
            (&csv_arg[2..], true)
        } else {
            (csv_arg.as_str(), false)
        };

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(!write_mode)
            .truncate(write_mode)
            .open(filename)
            .unwrap_or_else(|e| {
                eprintln!("ERROR!! Open CSV Report file FAILED");
                eprintln!("ERROR!! Can't open [{}] for {}", filename, if write_mode { "write" } else { "append" });
                eprintln!("{}", e);
                std::process::exit(1);
            });

        csv_writer = Some(BufWriter::new(file));
    }

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

    // Track CSV report statements
    use dealer_parser::{CsvTerm, Side};
    let mut csv_reports: Vec<Vec<CsvTerm>> = Vec::new();

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
            Statement::CsvReport(terms) => {
                csv_reports.push(terms.clone());
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

    // Apply command-line predeal switches (these take precedence over input file predeals)
    if let Some(ref cards_str) = args.north_predeal {
        match parse_predeal_cards(cards_str) {
            Ok(cards) => {
                if let Err(e) = generator.predeal(Position::North, &cards) {
                    eprintln!("Error predealing to North: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error parsing North predeal cards '{}': {}", cards_str, e);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref cards_str) = args.east_predeal {
        match parse_predeal_cards(cards_str) {
            Ok(cards) => {
                if let Err(e) = generator.predeal(Position::East, &cards) {
                    eprintln!("Error predealing to East: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error parsing East predeal cards '{}': {}", cards_str, e);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref cards_str) = args.south_predeal {
        match parse_predeal_cards(cards_str) {
            Ok(cards) => {
                if let Err(e) = generator.predeal(Position::South, &cards) {
                    eprintln!("Error predealing to South: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error parsing South predeal cards '{}': {}", cards_str, e);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref cards_str) = args.west_predeal {
        match parse_predeal_cards(cards_str) {
            Ok(cards) => {
                if let Err(e) = generator.predeal(Position::West, &cards) {
                    eprintln!("Error predealing to West: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error parsing West predeal cards '{}': {}", cards_str, e);
                std::process::exit(1);
            }
        }
    }

    // Apply predeal statements from input file
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

    // Progress meter variables (matches dealer.exe behavior)
    let progress_interval = 10000; // Show progress every 10,000 deals
    let mut last_progress_report = 0;

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

        // Show progress meter if enabled (matches dealer.exe -m)
        if args.progress && generated - last_progress_report >= progress_interval {
            let elapsed = start_time.elapsed().unwrap().as_secs_f64();
            eprintln!(
                "Generated: {} hands, Produced: {} hands, Time: {:.1}s",
                generated, produced, elapsed
            );
            last_progress_report = generated;
        }

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

                // In quiet mode, don't print deals (only statistics)
                if !args.quiet {
                    let output = match output_format {
                        OutputFormat::PrintAll => format_printall(&deal, produced),
                        OutputFormat::PrintEW => format_printew(&deal),
                        OutputFormat::PrintPBN => {
                            let dealer_pos = dealer_position.map(|d| d.into());
                            let vuln = vulnerability.map(|v| v.into());
                            let event_name = args.title.as_deref();
                            format_printpbn(&deal, produced, dealer_pos, vuln, event_name, Some(seed))
                        }
                        OutputFormat::PrintCompact => format_printcompact(&deal),
                        OutputFormat::PrintOneLine => format_oneline(&deal),
                    };
                    print!("{}", output);
                }

                // Write CSV reports if any
                if !csv_reports.is_empty() && csv_writer.is_some() {
                    let ctx = EvalContext::new(&deal);

                    for csv_terms in &csv_reports {
                        let mut line_parts: Vec<String> = Vec::new();

                        for term in csv_terms {
                            match term {
                                CsvTerm::Expression(expr) => {
                                    match eval(expr, &ctx) {
                                        Ok(val) => line_parts.push(val.to_string()),
                                        Err(e) => {
                                            eprintln!("CSV evaluation error: {}", e);
                                            std::process::exit(1);
                                        }
                                    }
                                }
                                CsvTerm::String(s) => {
                                    line_parts.push(format!("'{}'", s));
                                }
                                CsvTerm::Compass(pos) => {
                                    let hand = deal.hand(*pos);
                                    line_parts.push(format_hand_pbn(hand));
                                }
                                CsvTerm::Side(side) => {
                                    let (pos1, pos2) = match side {
                                        Side::NS => (Position::North, Position::South),
                                        Side::EW => (Position::East, Position::West),
                                    };
                                    let hand1 = deal.hand(pos1);
                                    let hand2 = deal.hand(pos2);
                                    line_parts.push(format!("{} {}", format_hand_pbn(hand1), format_hand_pbn(hand2)));
                                }
                                CsvTerm::Deal => {
                                    let n = deal.hand(Position::North);
                                    let e = deal.hand(Position::East);
                                    let s = deal.hand(Position::South);
                                    let w = deal.hand(Position::West);
                                    line_parts.push(format!("{} {} {} {}",
                                        format_hand_pbn(n),
                                        format_hand_pbn(e),
                                        format_hand_pbn(s),
                                        format_hand_pbn(w)));
                                }
                            }
                        }

                        // Write line with space before first item, commas between items
                        if let Some(writer) = csv_writer.as_mut() {
                            write!(writer, " {}\n", line_parts.join(",")).unwrap_or_else(|e| {
                                eprintln!("CSV write error: {}", e);
                                std::process::exit(1);
                            });
                        }
                    }
                }

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
    // In verbose mode (-v), always show stats
    // Without verbose mode, show stats by default (dealer3 behavior)
    // Note: This matches dealer.exe behavior where -v enables verbose output
    if args.verbose {
        eprintln!();
        eprintln!("Generated {} hands", generated);
        eprintln!("Produced {} hands", produced);
        eprintln!("Initial random seed {}", seed);
        eprintln!("Time needed  {:7.3} sec", elapsed_secs);
    }
}
