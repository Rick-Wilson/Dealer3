# solver - Bridge Double-Dummy Solver

A command-line double-dummy solver for bridge that calculates optimal trick counts.

## Overview

`solver` is a Rust implementation of a double-dummy bridge solver. Given a deal and trump/leader, it computes the maximum number of tricks that can be made with perfect play by both sides.

The solver uses alpha-beta minimax search with:
- MTD(f) iterative deepening
- Transposition tables
- Fast/slow tricks pruning
- Equivalent card filtering
- Pattern-based caching

## Installation

The solver is built as part of the bridge-solver crate:

```bash
cargo build --release -p bridge-solver
```

The binary will be at `target/release/solver` (or `solver.exe` on Windows).

## Usage

```
solver -f <file> [-X <iterations>] [-P] [-T] [-R] [-V]
```

### Required Arguments

| Flag | Description |
|------|-------------|
| `-f <file>` | Input file containing the deal |

### Optional Arguments

| Flag | Description |
|------|-------------|
| `-X <n>` | X-ray debug mode: limit to n iterations |
| `-P` | Disable pruning (fast/slow tricks) |
| `-T` | Disable transposition table |
| `-R` | Disable rank skip (equivalent card filtering) |
| `-V` | Show performance statistics |

## Input File Format

The input file has 3-5 lines:

```
<North hand>
<West hand>  <East hand>
<South hand>
<Trump>
<Leader>
```

### Hand Format

Each hand lists cards by suit, separated by spaces:
```
SPADES HEARTS DIAMONDS CLUBS
```

Cards use standard notation: A K Q J T 9 8 7 6 5 4 3 2

Use `-` for a void (empty suit).

### Trump Values

| Value | Meaning |
|-------|---------|
| `N` | No Trump |
| `S` | Spades |
| `H` | Hearts |
| `D` | Diamonds |
| `C` | Clubs |

If omitted, solves for all 5 denominations.

### Leader Values

| Value | Meaning |
|-------|---------|
| `W` | West |
| `N` | North |
| `E` | East |
| `S` | South |

If omitted, solves for all 4 leaders.

## Examples

### Example 1: Full Deal, Specific Trump and Leader

Create a file `deal.txt`:
```
AKQ AK AK A
JT9 QJ QJ Q        876 98 98 9
543 65 65 6
N
W
```

Run:
```bash
solver -f deal.txt
```

Output:
```
                          ♠ AKQ ♥ AK ♦ AK ♣ A
       ♠ JT9 ♥ QJ ♦ QJ ♣ Q                  ♠ 876 ♥ 98 ♦ 98 ♣ 9
                          ♠ 543 ♥ 65 ♦ 65 ♣ 6
N  8  0.00 s N/A M
```

The result shows: Trump=NT, NS tricks=8, Time=0.00s

### Example 2: Solve All Trumps and Leaders

Create a file `deal2.txt` with just the hands (no trump/leader lines):
```
AKQ J6 KJ 9
65 AK4 AQ T        J7 QT9 T AK
98 87 96 QJ
```

Run:
```bash
solver -f deal2.txt
```

Output shows results for each trump with all 4 leaders (W E N S order):
```
N  1  1  3  3  0.12 s N/A M
S  1  1  4  4  0.08 s N/A M
H  3  3  5  5  0.06 s N/A M
D  2  2  4  4  0.07 s N/A M
C  2  2  4  4  0.05 s N/A M
```

### Example 3: Debug Mode

To see detailed performance stats:
```bash
solver -f deal.txt -V
```

To limit iterations (for debugging):
```bash
solver -f deal.txt -X 1000
```

## Output Format

### Single Leader Output
```
<Trump>  <NS tricks>  <time> s N/A M
```

### All Leaders Output (W E N S order)
```
<Trump>  <W result>  <E result>  <N result>  <S result>  <time> s N/A M
```

**Note on result interpretation:**
- When W or E leads: result = NS tricks directly
- When N or S leads: result = EW tricks (total - NS tricks)

This matches the convention of the C++ reference solver.

## Endgame Support

The solver supports partial deals (endgames with fewer than 13 cards per hand). This is useful for:
- Testing specific positions
- Analyzing critical endgame decisions
- Faster computation on smaller problems

Simply provide hands with fewer cards:
```
AK AK - -
QJ QJ - -        T9 T9 - -
87 87 - -
N
W
```

## Performance

Typical performance on an 8-card endgame:
- ~0.1-0.3ms per solve
- ~700-9K nodes searched

Performance scales exponentially with deal size:
| Cards | Typical Time |
|-------|--------------|
| 8 | <1ms |
| 9 | 1-5ms |
| 10 | 5-20ms |
| 11 | 15-200ms |
| 12 | 2-30s |
| 13 | varies (may timeout on complex deals) |

## Troubleshooting

### "Failed to read file"
Check that the file path is correct and the file exists.

### "Failed to parse hands"
Verify the hand format:
- Each suit separated by single space
- West and East separated by 2+ spaces
- Valid card characters: A K Q J T 9 8 7 6 5 4 3 2 -

### Slow performance on 13-card deals
Full deals may take significant time. Try:
- Using `-V` to see progress
- Testing with smaller endgames first
- The solver is optimized for endgames; full deals are computationally intensive

## See Also

- [SOLVER_TESTING.md](SOLVER_TESTING.md) - Detailed testing plan and implementation status
- [dealer-dds README](../dealer-dds/README.md) - Library API documentation

## License

This software is released into the public domain under The Unlicense.
