# dealer-dds

Double-dummy solver for bridge using alpha-beta minimax search.

## Overview

This crate provides double-dummy analysis for bridge deals, calculating the maximum number of tricks that can be made by each side in each denomination when all four hands are visible. The solver uses alpha-beta pruning with transposition tables to efficiently search the game tree.

## Features

### Core Algorithm
- **Alpha-beta minimax search** - Optimal play calculation with pruning
- **Transposition table** - Position caching to avoid redundant calculations
- **Complete game simulation** - Proper trick-taking rules including:
  - Follow suit requirements
  - Trump handling
  - Trick winner determination
- **All denominations** - Supports all 5 denominations (Clubs, Diamonds, Hearts, Spades, NoTrump)

### API

```rust
use dealer_core::{DealGenerator, Position};
use dealer_dds::{DoubleDummySolver, Denomination};

// Generate a deal
let mut gen = DealGenerator::new(42);
let deal = gen.generate();

// Create solver
let solver = DoubleDummySolver::new(deal);

// Solve for a specific denomination and declarer
let tricks = solver.solve(Denomination::Spades, Position::North);
println!("North can make {} tricks in spades", tricks);

// Solve for all 20 combinations (5 denominations × 4 positions)
let result = solver.solve_all();
println!("North spades: {}", result.get_tricks(Denomination::Spades, Position::North));
```

### Types

- **`Denomination`** - Enum for the 5 denominations
- **`DoubleDummyResult`** - Complete analysis for all 20 denomination/declarer combinations
- **`TrickResult`** - Single result (denomination, declarer, tricks)
- **`DoubleDummySolver`** - Main solver API

## Performance

Current performance (Release mode, measured with seed 42):

- **Single solve**: ~75ms
- **Full solve_all** (20 combinations): ~654ms
- **Per deal**: ~730ms
- **Throughput**: **1.37 deals/second**

### Benchmark

```bash
cargo run --release --example benchmark -p dealer-dds
```

## Implementation Details

### Algorithm
The solver implements a standard alpha-beta minimax search:

1. **Game tree search** - Explores all legal moves from current position
2. **Minimax evaluation** - Maximizes tricks for declarer's side, minimizes for opponents
3. **Alpha-beta pruning** - Cuts off branches that can't affect the result
4. **Transposition table** - Caches positions to avoid re-computation

### State Representation
- **GameState** - Complete deal state (4 hands, current trick, tricks won)
- **TrickState** - Current trick in progress (cards played, leader, trump)
- State cloning on each move (main performance bottleneck)

### Hash Function
Simple XOR-based hashing of:
- Card positions in each hand
- Current trick leader
- Tricks won by declarer

## Comparison to Professional Implementations

Bo Haglund's DDS (industry standard): ~5 deals/second

This implementation: ~1.37 deals/second (**3.6x slower**)

### Why the Difference?

1. **State cloning** - We clone the entire game state for each move explored (expensive)
2. **No move ordering** - Don't try high-value cards first
3. **Simple hashing** - Basic XOR instead of Zobrist hashing
4. **No endgame optimization** - Could use simpler logic for last few tricks

## TODO: Performance Optimizations

### High Impact (2-3x speedup expected)

- [ ] **Make/unmake move pattern**
  - Replace state cloning with in-place make/unmake
  - Store undo information for efficient state restoration
  - Requires careful implementation to avoid bugs
  - Expected: 2-3x speedup → ~3-4 deals/sec

### Medium Impact (20-50% speedup)

- [ ] **Zobrist hashing**
  - Pre-compute random values for each card/position
  - Incremental hash updates on make/unmake
  - Faster than current XOR approach
  - Expected: 20-30% speedup

- [ ] **Move ordering heuristics**
  - Try high cards first (Aces, Kings, Queens)
  - Try trump cards before side suits
  - Better alpha-beta cutoffs
  - Note: Naive sorting made it slower; needs smarter approach
  - Expected: 10-20% speedup

- [ ] **Improved transposition table**
  - Store exact scores vs. bounds
  - Better replacement strategy
  - Track depth for more accurate lookups
  - Expected: 20-30% speedup

### Lower Impact (5-15% speedup)

- [ ] **Endgame tables**
  - Special handling for 1-2 cards remaining
  - Direct calculation instead of search
  - Expected: 5-10% speedup

- [ ] **Parallel solving**
  - Solve different denominations in parallel
  - Rayon-based parallelization
  - Expected: Near-linear speedup for solve_all (4-5x on 4+ cores)

- [ ] **Hand representation optimization**
  - Use bitboards instead of Vec<Card>
  - Faster legal move generation
  - Expected: 10-15% speedup

## Testing

```bash
# Run all tests
cargo test -p dealer-dds

# Run specific test
cargo test -p dealer-dds --lib test_solver_basic

# Run with output
cargo test -p dealer-dds --lib -- --nocapture
```

### Test Coverage

- Denomination conversions
- Trick winner logic (trump handling)
- Solver correctness (returns 0-13 tricks)
- Result storage and retrieval
- Complete solve_all (20 combinations)

## License

This project is released into the **public domain** under [The Unlicense](../LICENSE).

## Credits

Algorithm: Standard alpha-beta minimax with transposition tables

Inspiration: Bo Haglund's DDS library (industry standard for bridge double-dummy solving)

Implementation: Rick Wilson (Unlicense)
