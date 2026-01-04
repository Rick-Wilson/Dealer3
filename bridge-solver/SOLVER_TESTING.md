# Double-Dummy Solver Testing Plan

## Overview

We are porting the C++ solver from https://github.com/macroxue/bridge-solver to Rust.
The Rust implementation is in `dealer-dds/src/solver2/`.

## Current Status (Jan 2026)

### What's Working
- Basic solver with MTD(f) driver and alpha-beta search
- Transposition table (TT) with proper bounds storage
- Fast tricks pruning for early cutoffs
- Equivalent card filtering (Phase 1)
- Cutoff card caching (Phase 2)
- Move ordering heuristics (Phase 3)
- Slow tricks analysis for NT (Phase 4)
- Test harness (`endgame_compare.rs`) compares Rust vs C++ solver
- Debug logging available via `--features debug_mtdf`
- Node counter via `get_node_count()` for performance profiling

### Known Issues
1. **12-card deals return wrong result** - After fixing fast_tricks, 12-card N/E/S leads
   return 8 instead of expected 9. The issue is NOT in fast_tricks or slow_tricks pruning
   (tested by disabling them). The bug is somewhere deeper in the search. Investigation needed.
2. **13-card deals** - Still too slow (times out), needs shape-based pattern cache

### Bugs Fixed
1. **MTD(f) alpha bug** - Was passing `alpha=0` instead of `alpha=beta-1` for null-window search
2. **Bounds pruning bug** - When `ns_tricks_won + remaining < beta`, was returning `ns_tricks_won` instead of `ns_tricks_won + remaining`
3. **C++ output interpretation** - C++ solver returns tricks for the NON-LEADER's side:
   - If EW leads (W or E): result = NS tricks directly
   - If NS leads (N or S): result = EW tricks, so NS = num_tricks - result
4. **Test case expected values** - Updated tests.rs with correct values based on proper C++ interpretation
5. **TT bounds storage bug** - Was storing "exact" values after search loop, but null-window search only gives one-sided bounds. Fix: check if `best < beta` (fail low → upper bound) or `best >= beta` (fail high → lower bound)
6. **TT hash function bug** - Was missing the 4th hand (West) in hash computation, causing collisions
7. **Fast tricks overcounting bug** - The naive `fast_tricks_ns/ew` functions were counting all
   top cards without considering entries and blocking. Fixed by implementing proper
   `suit_fast_tricks` logic based on C++ `SuitFastTricks`:
   - Track winners in each hand separately (not combined NS/EW)
   - Check if there are entries between partnership hands
   - Handle suit blocking (my bottom > partner's top, or my top < partner's bottom)
   - Use two entry flags: `my_entry` (I can overtake partner) and `pd_entry` (partner can overtake me)
   - Only use `max(my_tricks, pd_tricks)` if partner can actually take the lead
   - This fixed 9/10/11 card tests that were returning +1 trick for N/S leads

### Performance

**Quick test series** (`examples/quick_test_*.rs`) - Test 2 deal at various trick counts:

| Tricks | W leads | N leads | E leads | S leads | Status |
|--------|---------|---------|---------|---------|--------|
| 8 | 0.3ms, 9K nodes | 0.06ms, 725 nodes | 0.1ms, 7K nodes | 0.06ms, 713 nodes | All OK |
| 9 | 0.2ms, 1.7K nodes | 3ms, 212K nodes | 0.5ms, 31K nodes | 3ms, 210K nodes | All OK |
| 10 | 0.2ms, 1.4K nodes | 15ms, 1.1M nodes | 5ms, 390K nodes | 15ms, 1.1M nodes | All OK |
| 11 | 15ms, 1.2M nodes | 195ms, 14M nodes | 132ms, 9M nodes | 209ms, 15M nodes | All OK |
| 12 | 1.9s, 132M nodes | 29s, 2B nodes | 4.4s, 321M nodes | 30s, 2.1B nodes | N/E/S FAIL (-1) |
| 13 | timeout | timeout | timeout | timeout | Too slow |

**Note**: After fixing fast_tricks with proper entry/blocking logic, 9-11 card tests all pass.
12-card tests now fail with -1 trick for N/E/S leads (returns 8 instead of 9). This is a
different bug - the fast_tricks is now conservative (returns 7 for N leads at depth 0),
so the issue is deeper in the search.

**Scaling from 11 to 12 cards**: 100-670x slowdown (exponential blowup without pattern cache)

**8-card endgame detail**: ~0.1-0.3ms per solve, 700-9K nodes at ~12-30 ns/node

**C++ comparison (8-card)**: Rust is 4-10x worse in iteration count than C++

### C++ vs Rust Comparison (8-card deal, NT)

**After equivalent card filtering (Phase 1 complete):**

| Leader | C++ iters | Rust iters | Ratio   | Rust ns/iter |
|--------|-----------|------------|---------|--------------|
| W      | 1,651     | 11,940     | **7.2x** | 15 ns       |
| N      | 98        | 961        | **9.8x** | 36 ns       |
| E      | 1,640     | 10,522     | **6.4x** | 9 ns        |
| S      | 93        | 947        | **10.2x**| 31 ns       |

**Progress**: Equivalent card filtering reduced iterations by ~12-18x! We went from
~120x worse than C++ to ~7-10x worse.

**Phase 2 (cutoff card caching)**: Implemented but shows no improvement on 8-card
benchmark - the positions are too small for cutoff reuse across MTD(f) iterations.
Will help more on larger deals.

**Phase 3 (move ordering)**: Implemented simple follow ordering with "play low when
can't beat" heuristic. Lead ordering was attempted but hurt some cases - disabled.
Follow ordering helps E leads significantly (~15% improvement).

| Leader | C++ iters | Rust iters | Ratio   | Change vs Phase 1 |
|--------|-----------|------------|---------|-------------------|
| W      | 1,651     | 12,465     | **7.5x** | +4%              |
| N      | 98        | 949        | **9.7x** | -1%              |
| E      | 1,640     | 9,082      | **5.5x** | -14% ✓           |
| S      | 93        | 937        | **10.1x**| -1%              |

**Note**: The N/S vs E/W asymmetry in ratios is likely due to testing with a single deal.
Over a larger deal set, these ratios should converge. TODO: revisit move ordering tuning
once performance allows testing on larger deal sets.

**Phase 4 (slow tricks analysis)**: Implemented slow tricks counting for NT and trump
contracts. Slow tricks are guaranteed tricks that require giving up the lead first
(e.g., Kx behind RHO's Ace is a guaranteed finesse trick). Significant improvement!

| Leader | C++ iters | Rust iters | Ratio   | Change vs Phase 3 |
|--------|-----------|------------|---------|-------------------|
| W      | 1,651     | 9,289      | **5.6x** | -25% ✓           |
| N      | 98        | 797        | **8.1x** | -16% ✓           |
| E      | 1,640     | 6,874      | **4.2x** | -24% ✓           |
| S      | 93        | 785        | **8.4x** | -16% ✓           |

Remaining gap is due to:
- More sophisticated move ordering heuristics
- Enhanced fast tricks (entry analysis, blocking detection)
- Shape-based pattern cache

<details>
<summary>Before equivalent card filtering (for reference)</summary>

| Leader | C++ iters | Rust iters | Pruning ratio |
|--------|-----------|------------|---------------|
| W      | 1,651     | 213,675    | **129x**      |
| N      | 98        | 11,338     | **116x**      |
| E      | 1,640     | 175,366    | **107x**      |
| S      | 93        | 11,332     | **122x**      |

</details>

## Input Formats

### C++ Solver (`bridge-solver/solver`)

**File format** (4 lines for hands, then trump and leader):
```
AKQ AK AK A        # North: spades hearts diamonds clubs
JT9 QJ QJ Q        # West: spades hearts diamonds clubs
876 98 98 9        # East: spades hearts diamonds clubs
543 65 65 6        # South: spades hearts diamonds clubs
N W                # Trump (N=NT, S/H/D/C) and Leader (W/N/E/S)
```

**Command line:**
```bash
./solver -f <file>     # Solve from file
./solver -r            # Random deal
./solver -t <trump>    # Override trump
```

**Output format:**
```
[hand diagram]
N  8  0.00 s 3712.0 M    # Trump, NS tricks, time, nodes
```

### Rust Solver (`dealer-dds/src/solver2`)

**PBN format:**
```
N:AKQJ.AKQ.AKQ.AKQ T987.JT9.JT9.JT9 6543.876.876.876 2.5432.5432.5432
  ^North           ^East             ^South            ^West
```

Each hand: `spades.hearts.diamonds.clubs` separated by spaces.
The `N:` prefix indicates North is listed first, followed by E, S, W.

**Rust API:**
```rust
let hands = Hands::from_pbn("N:A.A.A.A K.K.K.K 2.2.2.2 3.3.3.3").unwrap();
let solver = Solver::new(hands, NOTRUMP, WEST);  // trump, initial_leader
let ns_tricks = solver.solve();
```

**Trump constants:** `SPADE=0, HEART=1, DIAMOND=2, CLUB=3, NOTRUMP=4`

**Seat constants:** `WEST=0, NORTH=1, EAST=2, SOUTH=3`

## Endgame Support

Both solvers support partial deals (endgames with fewer than 13 cards per hand).
This is critical for testing because:
1. Smaller problems are faster to solve and debug
2. Bugs are easier to trace in 5-card endgames than 13-card deals
3. Once correct on small problems, we can focus on performance

## Test Plan

### Phase 1: Verify Correctness on Small Endgames

1. **Generate random endgames** (5-7 cards per hand)
2. **Run both solvers** on the same deal
3. **Compare results** - they should match exactly
4. **Test all trumps and all leaders** for each deal

Example test script approach:
```bash
# Generate random 5-card endgame
# Convert to both formats
# Run C++ solver, capture result
# Run Rust solver, capture result
# Compare
```

### Phase 2: Debug Any Discrepancies

If results differ:
1. Start with smallest failing case
2. Add debug output to trace search
3. Compare search trees between implementations
4. Fix bugs in Rust implementation

### Phase 3: Performance Optimization

Once correctness is verified on endgames:
1. Port pattern-based caching from C++ (more TT hits)
2. Port equivalent card filtering correctly
3. Port move ordering heuristics
4. Port cutoff card caching
5. Benchmark against C++ on full deals

## Key Files

- `dealer-dds/src/solver2/solver.rs` - Main solver, MTD(f), alpha-beta
- `dealer-dds/src/solver2/cards.rs` - Card bitboard (52-bit u64)
- `dealer-dds/src/solver2/hands.rs` - Four hands, PBN parsing
- `dealer-dds/src/solver2/types.rs` - Constants (seats, suits, ranks)
- `bridge-solver/solver.cc` - C++ reference implementation

## Pruning Implementation Plan

The C++ solver has 6 major pruning techniques that we need to port. Listed in order
of likely impact and implementation difficulty:

### Phase 1: Equivalent Card Filtering (High Impact, Medium Difficulty)

**What it does**: Cards that are adjacent in rank with no intervening cards from
other hands are equivalent (e.g., QJ when K is out). Only try one card from each
equivalence class.

**C++ location**: `solver.cc:908-917` (`IsEquivalent`) and `solver.cc:1119-1124`

**Algorithm**:
```
For each card to try:
  - Check if any already-tried card in same suit is "adjacent"
  - Cards are adjacent if all cards between them belong to the current player
  - Skip if adjacent to a tried card
```

**Implementation**:
1. Fix our existing `filter_equivalent_cards` in solver.rs
2. Use `all_cards` (union of all hands) to check for gaps between cards
3. Test: iteration count should drop significantly

### Phase 2: Cutoff Card Caching (High Impact, Low Difficulty)

**What it does**: Remembers which card caused a beta cutoff for a given position.
When revisiting similar positions, try that card first.

**C++ location**: `solver.cc:862-881` (`CutoffEntry`, `cutoff_cache`)

**Algorithm**:
```
On cutoff: Store the card that caused it, keyed by position hash
On search: Check cache, try cached cutoff card first
```

**Implementation**:
1. Create a simple hash table: `HashMap<u64, [u8; 4]>` (one card per seat)
2. On beta cutoff, store the winning card
3. When ordering moves, check cache and put cached card first
4. Key is hash of (shape, seat_to_play) - similar to common_bounds_cache

### Phase 3: Move Ordering Heuristics (High Impact, Medium Difficulty)

**What it does**: Orders candidate moves to maximize alpha-beta cutoffs. Better
moves first means more pruning.

**C++ location**: `solver.cc:1154-1308` (`Lead<>` and `OrderCards`)

**Lead ordering** (when starting a trick):
1. Ruff leads (if partner is void and has trump)
2. Good leads (finesse positions like Kx behind A in partner's hand)
3. High leads (both sides have A/K/Q)
4. Normal leads (top and bottom of suits)
5. Bad leads (high card in front of RHO's higher card)
6. Trump leads (usually last priority)

**Follow ordering** (when following suit):
1. If can't beat current winner → play low
2. If partner winning and trick ending or LHO can't beat → play low
3. If in second seat and partner can win later → play low
4. Otherwise try to win with minimal card

**Implementation**:
1. Create `order_leads()` function for trick-starting moves
2. Create `order_follows()` function for mid-trick moves
3. Replace current unordered iteration with ordered list

### Phase 4: Slow Tricks Analysis (Medium Impact, Medium Difficulty)

**What it does**: Counts guaranteed tricks for the NON-leading side via finesse
positions. Complements fast_tricks which counts leader's guaranteed tricks.

**C++ location**: `solver.cc:1476-1490` (`SlowNoTrumpTricks`) and `solver.cc:1448-1474` (`SlowTrumpTricks`)

**SlowNoTrumpTricks algorithm**:
```
For each suit where current player has cards:
  If our side has the top card → no slow tricks
  Else: that top card is a "rank winner" for opponents
If all rank winners are in one opponent's hand → they get all of them
Else → they get at least 1 (can be blocked)
```

**SlowTrumpTricks algorithm** (for trump contracts):
```
Check for protected honor positions:
  - Kx behind A (finesse for 1 trick)
  - KQ vs A (force out for 1 trick)
  - Qxx behind AK (finesse for 1 trick)
```

**Implementation**:
1. Add `slow_tricks_ns()` and `slow_tricks_ew()` to solver.rs
2. Call after fast_tricks in `search_recursive`
3. Use for additional alpha/beta cutoffs

### Phase 5: Shape-Based Pattern Cache (High Impact, High Difficulty)

**What it does**: A hierarchical cache that stores bounds based on the "pattern"
of cards (relative ranks) rather than absolute cards. Multiple positions with
the same pattern share cached bounds.

**C++ location**: `solver.cc:600-803` (`Pattern` class) and `solver.cc:805-858` (`ShapeEntry`)

**Key concepts**:
- **Shape**: Distribution of cards (e.g., 4-3-3-3 in each hand)
- **Relative hands**: Cards converted to relative ranks (A=highest, K=second, etc.)
- **Pattern matching**: A cached pattern matches if current relative hands are
  a subset (more specific) than the cached pattern

**Algorithm**:
```
On trick boundary:
  1. Compute shape (suit lengths for all 4 hands)
  2. Convert hands to relative ranks
  3. Look up cache by (shape, seat_to_play)
  4. If cached pattern matches → use cached bounds
  5. After search → update cache with new pattern/bounds
```

**Implementation**:
1. Create `Shape` struct (already have a skeleton in cache.rs)
2. Create `Pattern` struct with hierarchical bounds storage
3. Create `common_bounds_cache` with linear probing
4. Integrate into `search_recursive` at trick boundaries

### Phase 6: Enhanced Fast Tricks (Medium Impact, High Difficulty) - DONE

**What it does**: More sophisticated analysis of guaranteed tricks considering
entries, suit lengths, and blocking positions.

**C++ location**: `solver.cc:1492-1560` (`FastTricks`, `SuitFastTricks`)

**Implementation complete** (Jan 2026):
- `suit_fast_tricks()` - per-suit analysis with entry and blocking detection
- `fast_tricks_from_seat()` - computes tricks from a specific seat's perspective
- `fast_tricks()` - main entry point, properly handles entries between partnership hands

**Key logic**:
- Track winners in each hand separately
- Entry: my top winner can cover partner's bottom card
- Blocked by partner: my top < partner's bottom → return partner's winners
- Blocked by me: my bottom > partner's top → return my winners
- If partner has all winners (no small cards), one winner acts as transport
- Only use partner's tricks if `pd_entry` is true (partner can overtake to take lead)

**Result**: Fixed 9/10/11 card tests that were returning +1 trick for N/S leads

---

## Implementation Order

Based on impact vs difficulty:

1. **Equivalent card filtering** - ✅ Done
2. **Cutoff card caching** - ✅ Done (minimal impact on small deals)
3. **Move ordering** - ✅ Done (follow ordering helps ~15%)
4. **Slow tricks** - ✅ Done (NT and trump contracts)
5. **Enhanced fast tricks** - ✅ Done (entries, blocking)
6. **Shape-based cache** - Pending (critical for 12+ cards)

After each step, run the 8-card benchmark to measure improvement in iteration count.
Goal: get within 10x of C++ iteration count, then 13-card deals should complete.

## Next Priority: Debug 12-Card Bug, Then Shape-Based Pattern Cache

### Immediate: Fix 12-Card Correctness Bug

12-card tests fail with N/E/S leads returning 8 instead of expected 9. Analysis:
- Fast tricks returns 7 for N leads at depth 0 (doesn't trigger cutoff for beta=9)
- Slow tricks returns 0 for initial position (no cutoff)
- Disabling both fast and slow tricks pruning makes tests PASS (but very slow)
- Bug is somewhere in the base search, not in the pruning functions

**Debug approach**:
1. Re-run with all pruning disabled to confirm base search is correct
2. Enable pruning one at a time to isolate which one causes the -1 error
3. The bug may be occurring at deeper nodes, not depth 0

### Then: Shape-Based Pattern Cache

The 100-670x blowup from 11 to 12 cards shows the critical need for the shape-based
pattern cache. This is the main remaining optimization from the C++ solver.

**Why it matters**: The current TT only matches exact card positions. The pattern cache
matches positions with the same "shape" (suit distributions) and relative ranks,
dramatically increasing cache hit rate.

**Key C++ code to study**:
- `solver.cc:574-803` - `Pattern` class with hierarchical bounds storage
- `solver.cc:805-860` - `ShapeEntry` for cache entries keyed by (shape, seat_to_play)
- `solver.cc:890-905` - `ComputeRelativeHands` converts cards to relative ranks
- `solver.cc:1028-1063` - Cache lookup/update at trick boundaries

**Implementation approach**:
1. Add `Shape` struct (4x4 nibbles = 64 bits for suit lengths per seat)
2. Add `relative_hands()` function to convert cards to relative ranks
3. Create `PatternCache` with hierarchical pattern matching
4. Integrate at trick boundaries in `search_at_trick_start()`

## Test Commands

```bash
# Run quick test series (8-13 card endgames from Test 2)
cargo run --example quick_test --release      # 8 cards (original)
cargo run --example quick_test_9 --release    # 9 cards
cargo run --example quick_test_10 --release   # 10 cards
cargo run --example quick_test_11 --release   # 11 cards
cargo run --example quick_test_12 --release   # 12 cards (~8 sec total)
cargo run --example quick_test_13 --release   # 13 cards (too slow - will timeout)

# Run comparison test against C++ (requires bridge-solver/solver)
cargo run --example endgame_compare --release -- --count 10 --cards 5

# Run with debug output
cargo run --example quick_test --release --features debug_mtdf

# Run C++ solver on a file
bridge-solver/solver -f test.txt

# Note: C++ solver must be built first - see bridge-solver/README.md
```

## Example Test Deal (8 cards)

```
N: AKQ J6 KJ 9
W: 65 AK4 AQ T
E: J7 QT9 T AK
S: 98 87 96 QJ

NT results (verified with C++):
W leads: NS=1
N leads: NS=3
E leads: NS=1
S leads: NS=3
```
