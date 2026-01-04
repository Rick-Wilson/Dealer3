# X-Ray Testing

Comparison testing between the Rust solver and the C++ reference implementation from [macroxue/bridge-solver](https://github.com/macroxue/bridge-solver).

## Directory Structure

```
xray/
├── README.md           # This file
├── compare.py          # Comparison test script
├── deals/              # Test deals in C++ solver format
│   ├── quick_test_8.txt
│   ├── quick_test_9.txt
│   ├── quick_test_10.txt
│   ├── quick_test_11.txt
│   ├── quick_test_12.txt
│   └── quick_test_13.txt
└── runs/               # Test output (gitignored)
    └── NNNN_<test-name>_<strain>_<leader>/
        ├── input.txt       # Input file used
        ├── rust_output.txt # Rust solver output
        ├── cpp_output.txt  # C++ solver output
        ├── rust_xray.txt   # X-ray trace (if -X used)
        ├── cpp_xray.txt    # X-ray trace (if -X used)
        └── comparison.md   # Comparison report
```

## Deal File Format

The deal files use the C++ solver's input format:

```
<North hand>
<West hand>                    <East hand>
<South hand>
<Trump>     (optional: N/S/H/D/C)
<Leader>    (optional: W/N/E/S)
```

Each hand lists cards by suit: `spades hearts diamonds clubs` separated by spaces.
Use `-` for a void.

Example (8-card deal):
```
AKQ J6 KJ 9
65 AK4 AQ T                    J7 QT9 T AK
98 87 96 QJ
```

If trump is omitted, solves for all 5 strains (N, S, H, D, C).
If leader is omitted, solves for all 4 leaders (W, N, E, S).

## Running the Solvers

### Rust Solver

```bash
cd dealer-dds

# Solve for all strains and leaders
cargo run --example solver_rust --release -- -f examples/xray/deals/quick_test_8.txt

# With X-ray tracing (first N iterations)
cargo run --example solver_rust --release -- -f examples/xray/deals/quick_test_8.txt -X 20
```

### C++ Solver

```bash
cd bridge-solver

# Solve for all strains and leaders
./solver -f ../dealer-dds/examples/xray/deals/quick_test_8.txt

# With X-ray tracing (requires solver_xray)
./solver_xray -X 20 -f ../dealer-dds/examples/xray/deals/quick_test_8.txt
```

## Output Format

Both solvers produce similar output:

```
                          ♠ AKQ ♥ J6 ♦ KJ ♣ 9
       ♠ 65 ♥ AK4 ♦ AQ ♣ T                  ♠ J7 ♥ QT9 ♦ T ♣ AK
                          ♠ 98 ♥ 87 ♦ 96 ♣ QJ
N  1  0.00 s 3696.0 M
```

- First column: Trump (N=NT, S=Spades, H=Hearts, D=Diamonds, C=Clubs)
- Second column: NS tricks (when single leader specified)
- Or columns 2-5: NS tricks for W/N/E/S leads (when no leader specified)
- Time in seconds
- Node count (iterations)

## Test Deals

All deals are derived from "Test 2" in the original test suite, with varying card counts:

| File | Cards | Expected NS Tricks (NT) | Notes |
|------|-------|------------------------|-------|
| quick_test_8.txt | 8 | W:1, N:3, E:1, S:3 | Fast (~0.1ms) |
| quick_test_9.txt | 9 | W:4, N:6, E:6, S:6 | Fast (~3ms) |
| quick_test_10.txt | 10 | W:5, N:7, E:7, S:7 | Fast (~15ms) |
| quick_test_11.txt | 11 | W:7, N:8, E:8, S:8 | ~200ms |
| quick_test_12.txt | 12 | W:8, N:9, E:9, S:9 | Rust times out for N/E/S leads |
| quick_test_13.txt | 13 | W:9, N:9, E:9, S:9 | Too slow (needs pattern cache) |

## Known Issues

1. **12-card timeout**: Rust solver times out (>10s) for N/E/S leads on 12-card deals, while C++ completes in ~6ms. West lead works correctly (~2s). The bug appears to be in the search, not in pruning.

2. **13-card performance**: Both solvers are slow on 13-card deals, but C++ completes in reasonable time due to the shape-based pattern cache (not yet ported to Rust).

3. **Iteration count**: Rust uses ~4.5x more iterations than C++ for the same deal. This is due to the pattern cache in C++ that prunes based on hand shapes.

## Comparison Script

The `compare.py` script automates running both solvers and comparing results:

```bash
cd dealer-dds/examples/xray

# Compare all strains and leaders
python3 compare.py deals/quick_test_8.txt

# Compare specific strain and leader
python3 compare.py deals/quick_test_8.txt -s N -l W

# With custom timeout (default: 10s)
python3 compare.py deals/quick_test_12.txt -s N -l W -t 30

# With X-ray tracing (compare search behavior)
python3 compare.py deals/quick_test_8.txt -s N -l W -X 20

# Build Rust solver first if needed
python3 compare.py deals/quick_test_8.txt --build
```

### Options

| Option | Description |
|--------|-------------|
| `-s`, `--strain` | Strain to test (N/S/H/D/C). Default: all 5 |
| `-l`, `--leader` | Leader to test (W/N/E/S). Default: all 4 |
| `-t`, `--timeout` | Timeout in seconds per solver (default: 10) |
| `-X`, `--xray` | Enable X-ray tracing for N iterations |
| `-P`, `--no-pruning` | Disable fast/slow tricks pruning (for debugging) |
| `-T`, `--no-tt` | Disable transposition table (for debugging) |
| `-R`, `--no-rank-skip` | Disable min_relevant_ranks optimization (for debugging) |
| `--v2` | Use solver_v2 (solve_v2 method) instead of solver_rust |
| `--build` | Build Rust solver before running |

### Debug Flags

The `-P`, `-T`, and `-R` flags disable specific optimizations to isolate bugs:

```bash
# Baseline test: all optimizations disabled
python3 compare.py deals/quick_test_4.txt --v2 -s S -l W -X 1000000 -P -T -R

# Test rank-skip only (no TT, no pruning)
python3 compare.py deals/quick_test_4.txt --v2 -s S -l W -X 1000000 -P -T

# Test TT only (no pruning, no rank-skip)
python3 compare.py deals/quick_test_4.txt --v2 -s S -l W -X 1000000 -P -R

# Test pruning only (no TT, no rank-skip)
python3 compare.py deals/quick_test_4.txt --v2 -s S -l W -X 1000000 -T -R
```

### Output

Results are saved to `runs/NNNN_<test-name>_<strain>_<leader>/`:
- `input.txt` - The input file used (with strain/leader appended)
- `rust_output.txt` - Rust solver output
- `cpp_output.txt` - C++ solver output
- `rust_xray.txt` - X-ray trace lines (if -X used)
- `cpp_xray.txt` - X-ray trace lines (if -X used)
- `comparison.md` - Comparison report with results and performance

Run folders are numbered sequentially (0001, 0002, ...) for easy reference and sorting.

## X-Ray Tracing

X-ray tracing logs the parameters of each `SearchWithCache` call at trick boundaries, allowing comparison of the search behavior between solvers.

### X-ray Output Format

```
XRAY 1: depth=0 seat=West beta=5 ns_tricks_won=0
XRAY 2: depth=0 seat=West beta=2 ns_tricks_won=0
XRAY 3: depth=4 seat=West beta=2 ns_tricks_won=0
...
```

- **depth**: Search depth (0, 4, 8, ... for trick boundaries)
- **seat**: Current player to act (West, North, East, South)
- **beta**: MTD(f) search threshold
- **ns_tricks_won**: NS tricks already won at this point

### Building the X-ray Solver (C++)

The C++ x-ray solver is a modified copy of solver.cc:

```bash
cd bridge-solver
g++ -O3 -march=native -std=c++17 -o solver_xray solver_xray.cc
```

### Comparing Traces

When using `-X`, the comparison report shows where the search paths diverge:

```markdown
## X-Ray Trace Comparison

❌ **X-ray traces DIVERGE** at iteration 8

### First Divergence

**Line 8:**
- Rust: `XRAY 8: depth=20 seat=North beta=2 ns_tricks_won=1`
- C++:  `XRAY 8: depth=20 seat=East beta=2 ns_tricks_won=0`
```

## Recent Progress

### solver_v2 Port (2026-01-03)

A new Rust solver (`solver_v2`) is being ported from the C++ reference with instrumented X-ray tracing for comparison testing.

**Current Status:**
- ✅ Baseline search (with `-P -T -R`) matches C++ traces exactly
- ✅ Rank-skip optimization (`-P -T`) matches C++ traces
- ✅ Pruning optimization (`-T -R`) matches C++ traces for all strains
- ✅ Output format fixed to match C++ (declaring-side perspective)
- ⏳ Pattern cache (`common_bounds_cache`) has a bug causing incorrect cutoffs

**Pattern Cache Bug (2026-01-03):**
The pattern cache stores bounds with empty `pattern_hands` when `rank_winners` is empty. These empty patterns (`[0,0,0,0]`) incorrectly match ANY lookup because `new_pattern.is_subset_of(empty)` is always true. This causes false cache hits that prune valid search branches.

Test case: `quick_test_5.txt` with NT, East lead, pruning+TT enabled (`-R` flag):
- With `-P -T -R` (all disabled): Results MATCH ✅
- With `-R` only (pruning+TT): Results DIFFER ❌ (Rust=1, C++=0)
- Individual optimizations work: `-T -R` ✅, `-P -R` ✅, `-P -T` ✅
- Combination `-R` (pruning+TT together) fails

The bug is in how empty pattern hands are handled during lookup. When `rank_winners` is empty, `compute_pattern_hands` produces empty hands for all seats, which then matches any future lookup pattern.

**Next steps:** Investigate why C++ handles this correctly. Options:
1. Don't store patterns when hands are empty
2. Check if there's a difference in how bounds are validated
3. Add a guard to prevent empty patterns from matching

**Fixed - Output Format (2026-01-03):**
Both solvers now output from the declaring side's perspective:
- W/E leads: Shows NS tricks directly
- N/S leads: Shows EW tricks (total - NS tricks)
The compare.py script was updated to parse both outputs consistently.

**Fixed - Pattern Cache Lookup Beta (2026-01-03):**
Pattern cache bounds are stored relative to `ns_tricks_won` at store time. Lookup must use `rel_beta = beta - ns_tricks_won` for cutoff checks.

**Note:** Rust's simple hash-based TT has been removed to match C++. C++ only uses the pattern-based `common_bounds_cache` for caching, not a simple position hash table.

**IMPORTANT: Always rebuild Rust before retesting!**
```bash
cargo build --example solver_v2 --release
```
This has caused false debugging sessions when stale binaries were tested.

**Fixed - Pruning / SlowTrumpTricks (2026-01-03):**
The C++ `SlowTrumpTricks` function has a bug at lines 1618-1619 where `Have(Cards)` silently converts to `Have(1)` via `operator bool()`. This effectively disables the "KQ against A" finesse pattern detection. The Rust implementation has been updated to match this buggy behavior for iteration lockstep. Unit tests in `test_slow_trump_tricks.rs` verify this behavior.

**Fixed - Rank Winners (2026-01-03):**
The `min_relevant_ranks` optimization requires tracking which cards are "rank winners" (cards that beat other cards in the same suit). The C++ `GetTrickRankWinner()` only adds a trick winner to `rank_winners` if another card in the same suit was played in that trick. The Rust `play_card_and_search` and `collect_last_trick` functions have been updated to match this logic.

**Note on X-ray comparison with multiple leaders:** When running all strains/leaders together (no `-s`/`-l` flags), X-ray traces will diverge after the first leader because:
1. The XRAY counter resets for each strain/leader combination
2. C++ carries over the previous solve result as the initial MTD(f) guess for the next leader (`guess_tricks = ns_tricks + 1`), while Rust computes a fresh guess for each solve

This causes different initial `beta` values for subsequent leaders (e.g., East lead may start with `beta=2` in C++ vs `beta=3` in Rust). The search results are still correct - only the search path differs. **Use individual strain/leader tests (with `-s` and `-l` flags) for accurate X-ray trace comparison.**

### Test Commands

```bash
cd dealer-dds/examples/xray

# IMPORTANT: Always rebuild before testing!
cargo build --example solver_v2 --release

# Baseline (all optimizations disabled) - MATCHES ✅
python3 compare.py deals/quick_test_4.txt --v2 -s S -l W -X 1000000 -P -T -R

# Test rank-skip only - MATCHES ✅
python3 compare.py deals/quick_test_4.txt --v2 -s S -l W -X 1000000 -P -T

# Test pruning only - MATCHES ✅
python3 compare.py deals/quick_test_4.txt --v2 -s H -l W -X 200 -T -R

# Test TT only - MATCHES ✅
python3 compare.py deals/quick_test_4.txt --v2 -s S -l W -X 1000000 -P -R

# Test pruning + TT (pattern cache bug) - FAILS on some deals
python3 compare.py deals/quick_test_5.txt --v2 -s N -l E -X 200 -R

# Test all features on quick_test_4 - MATCHES ✅
python3 compare.py deals/quick_test_4.txt --v2

# Test all features on quick_test_5 - FAILS ❌ (pattern cache bug)
python3 compare.py deals/quick_test_5.txt --v2
```

### Unit Tests

```bash
cd dealer-dds

# Run SlowTrumpTricks unit tests
cargo test --example test_slow_trump_tricks

# Run lead/follow order tests
cargo test --example test_lead_order
cargo test --example test_follow_order
```
