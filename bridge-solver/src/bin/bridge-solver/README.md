# bridge-solver

A command-line tool for adding double-dummy analysis to PBN files.

## Overview

`bridge-solver` reads a PBN file containing bridge deals, performs double-dummy analysis on each deal, and inserts Bridge Composer compatible tags:

- `[DoubleDummyTricks]` - compact 20-character encoding of all results
- `[OptimumResultTable]` - full 20-row table of tricks by declarer and denomination

Note: `[OptimumScore]` and `[ParContract]` tags are not currently generated as proper par calculation requires complex game-theoretic analysis.

## Installation

```bash
cargo build --release -p bridge-solver
```

The binary will be at `target/release/bridge-solver` (or `bridge-solver.exe` on Windows).

## Usage

```
bridge-solver --input <file.pbn> [--output <file.pbn>] [-v]
```

### Arguments

| Flag | Description |
|------|-------------|
| `-i`, `--input <file>` | Input PBN file (required) |
| `-o`, `--output <file>` | Output PBN file (if omitted, writes to stdout) |
| `-v`, `--verbose` | Show progress messages |
| `-V`, `--version` | Show version information |
| `-h`, `--help` | Show help message |

## Examples

### Basic Usage

Add double-dummy results to a PBN file:

```bash
bridge-solver --input deals.pbn --output deals_solved.pbn
```

### With Progress Output

```bash
bridge-solver -i deals.pbn -o solved.pbn -v
```

Output:
```
Processing deal 1...
Processing deal 2...
...
Processed 10 deal(s)
Output written to solved.pbn
```

### Pipe to Stdout

```bash
bridge-solver -i deals.pbn > solved.pbn
```

## Output Format

The tool inserts two tags for each deal. The format matches Bridge Composer's output:

### DoubleDummyTricks

```
[DoubleDummyTricks "9a878987843545354535"]
```

A compact 20-character encoding where each character represents tricks (0-9 as digits, 10-13 as a-d). The order is N(NT,S,H,D,C) S(NT,S,H,D,C) E(NT,S,H,D,C) W(NT,S,H,D,C).

### OptimumResultTable

```
[OptimumResultTable "Declarer;Denomination\2R;Result\2R"]
N NT  9
N  S 10
N  H  8
N  D  7
N  C  8
S NT  9
S  S 10
S  H  8
S  D  7
S  C  8
E NT  4
E  S  3
E  H  4
E  D  6
E  C  5
W NT  4
W  S  3
W  H  4
W  D  6
W  C  5
```

The table shows, for each declarer (N/S/E/W) and denomination (NT/S/H/D/C), the number of tricks that declarer can make with perfect play by both sides.

## Behavior

- **Preserves file structure**: The tool reads the PBN as text and surgically inserts the OptimumResultTable. All other content (comments, formatting, other tags) is preserved.

- **Replaces existing results**: If an `[OptimumResultTable]` already exists for a deal, it is replaced with fresh results.

- **Finds deals by `[Deal]` tag**: The tool extracts the hand data from standard PBN `[Deal "..."]` tags.

- **PBN 2.1 compliant**: Follows the PBN specification for tag ordering. When inserting, places OptimumResultTable after mandatory tags in alphabetical order among supplemental tags. Properly handles brace comments `{...}` which may contain blank lines.

- **Deal separation**: Deals are separated by blank lines. Blank lines inside brace comments `{...}` do not break deal boundaries.

## Performance

Typical performance:
- ~50-60ms per deal on average
- Complex deals may take longer

For a 50-deal file, expect ~3 seconds total.

## See Also

- [solver-diag](../solver-diag/README.md) - Diagnostic/testing tool for the solver
- [dealer-dds README](../../../dealer-dds/README.md) - Library API documentation

## License

This software is released into the public domain under The Unlicense.
