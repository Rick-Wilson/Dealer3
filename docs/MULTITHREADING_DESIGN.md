# Multithreading Design for dealer3

**Date**: 2026-01-07
**Status**: Design Document

## Overview

This document describes a parallel execution design for dealer3 that provides speedup while maintaining deterministic output order compatible with single-threaded dealer.exe.

## Goals

1. **Deterministic output** - Same seed must produce same output, regardless of thread count
2. **Parallel speedup** - Utilize all available CPU cores
3. **RNG compatibility** - Random number sequence matches single-threaded execution
4. **Minimal overhead** - Batch processing to amortize synchronization costs

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                        Supervisor                           │
│  - Owns the single RNG (gnurandom)                         │
│  - Dispatches work in batches of N                         │
│  - Collects and orders results                             │
│  - Outputs passing deals in serial order                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ dispatches (serial_num, rng_state)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       Worker Pool                           │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │ Worker  │  │ Worker  │  │ Worker  │  │ Worker  │  ...  │
│  │   0     │  │   1     │  │   2     │  │   3     │       │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ returns (serial_num, Pass(deal) | Fail)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Result Collector                        │
│  - Receives results (potentially out of order)             │
│  - Sorts by serial number                                  │
│  - Streams to output in order                              │
└─────────────────────────────────────────────────────────────┘
```

### Supervisor Responsibilities

1. **RNG Management**: Maintains the single `gnurandom` instance
2. **Work Distribution**: For each work unit, captures current RNG state, advances RNG by the amount needed for one shuffle (52 random numbers), assigns serial number
3. **Batch Coordination**: Dispatches N work units, waits for completion
4. **Output Ordering**: Ensures results are output in serial number order
5. **Termination**: Tracks produced/generated counts, signals workers to stop

### Worker Responsibilities

1. **Receive**: `(serial_number, rng_state)` tuple
2. **Shuffle**: Initialize local RNG from state, shuffle deck (parallel work)
3. **Evaluate**: Apply filter condition to generated deal
4. **Return**: `(serial_number, Result)` where Result is `Pass(Deal)` or `Fail`

### Work Unit Structure

```rust
struct WorkUnit {
    serial_number: u64,
    rng_state: GnuRandomState,  // Captured RNG state for this deal
}

enum WorkResult {
    Pass(Deal),
    Fail,
}

struct CompletedWork {
    serial_number: u64,
    result: WorkResult,
}
```

## Batch Processing

### Batch Size (N)

- **Default**: `N = 100 × num_cores` (e.g., 800 on 8-core machine)
- **Configurable**: `--batch-size N` or `--batch-size auto`
- **Rationale**: Large batches amortize synchronization overhead; 100× multiplier ensures workers stay busy even with variable evaluation times

### Batch Execution Flow

```
for each batch:
    1. Supervisor captures N RNG states (advancing RNG each time)
    2. Supervisor dispatches N work units to pool
    3. Workers process in parallel (shuffle + evaluate)
    4. Supervisor collects all N results
    5. Supervisor sorts results by serial_number
    6. Supervisor outputs passing deals in order
    7. Update counters (generated += N, produced += passes)
    8. Check termination condition
```

## Early Termination (`-p` mode)

In produce mode (`-p N`), we need exactly N matching deals. The design handles this efficiently:

### Strategy: Sequential Result Processing

Instead of waiting for all N workers, the supervisor processes results **in serial order**:

```
produced = 0
target = args.produce

for serial_num in 0.. {
    wait for result with this serial_num
    generated += 1

    if result is Pass(deal):
        output(deal)
        produced += 1
        if produced >= target:
            cancel remaining workers
            break
}
```

### Implications

- Results from later serial numbers that complete early are buffered
- Once we have enough passes, remaining in-flight work is cancelled/ignored
- A worker with serial_num=50 might find a pass before serial_num=10, but we wait for 10 first
- This preserves deterministic output: we always output the **first N passing deals** in RNG sequence order

### Optimization: Speculative Cancellation

When `produced` is close to `target`, we can avoid dispatching new batches:

```rust
// Don't start new batch if we likely have enough in-flight
let expected_passes = in_flight_count * historical_pass_rate;
if produced + expected_passes >= target * 1.5 {
    // Wait for current batch instead of dispatching more
}
```

## RNG State Capture

The supervisor must capture enough RNG state for workers to reproduce the exact shuffle:

### Option A: Full State Clone (Recommended)

```rust
// Supervisor side
for i in 0..batch_size {
    let state = rng.clone_state();  // Capture full 31-word state
    work_units.push(WorkUnit { serial_number: next_serial++, rng_state: state });
    rng.advance(52);  // Skip the 52 random numbers this shuffle will use
}

// Worker side
fn process(unit: WorkUnit) -> CompletedWork {
    let mut local_rng = GnuRandom::from_state(unit.rng_state);
    let deal = shuffle_deck(&mut local_rng);
    let result = evaluate(deal);
    CompletedWork { serial_number: unit.serial_number, result }
}
```

### Option B: Seed + Skip Count

```rust
// Track how many random numbers have been consumed
// Worker reconstructs by: init(seed), skip(count), then shuffle
// More complex, not recommended
```

## Implementation Plan

### Phase 1: Core Infrastructure

1. Add `GnuRandom::clone_state()` and `GnuRandom::from_state()` methods
2. Create `WorkUnit`, `WorkResult`, `CompletedWork` types
3. Implement basic supervisor/worker split (single-threaded first)

### Phase 2: Parallel Execution

1. Add rayon dependency for thread pool
2. Implement parallel batch dispatch
3. Add result collection and ordering

### Phase 3: CLI Integration

1. Add `--threads N` flag (0 = auto-detect cores)
2. Add `--batch-size N` flag (default: auto)
3. Backward compatibility: `--threads 1` matches current behavior exactly

### Phase 4: Optimization

1. Profile and tune batch sizes
2. Optimize RNG state cloning (it's 31 × 8 = 248 bytes)
3. Consider lock-free result collection

## Configuration

### Command-Line Flags

```
-R N, --threads N     Number of worker threads (default: 0 = auto)
                      Use -R 1 for single-threaded (dealer.exe compatible output)

--batch-size N        Work units per batch (default: auto = 100 × threads)
```

### Auto-Detection

```rust
let num_threads = if args.threads == 0 {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
} else {
    args.threads
};

let batch_size = if args.batch_size == 0 {
    100 * num_threads
} else {
    args.batch_size
};
```

## Compatibility Notes

### Single-Threaded Mode (`-R 1`)

When running with one thread, output must be **identical** to current implementation:
- Same deals in same order
- Same statistics output
- Byte-for-byte compatible with dealer.exe (for same seed)

### Multi-Threaded Mode (`-R N`, N > 1)

- Same deals as single-threaded (deterministic)
- Same order as single-threaded (sorted by serial number)
- May differ from dealer.exe in timing statistics

## Performance Expectations

### Theoretical Speedup

For CPU-bound filter evaluation:
- Linear speedup up to core count (e.g., 8× on 8 cores)
- Diminishing returns beyond physical cores (hyperthreading adds ~20%)

### Realistic Targets

| Threads | Expected Speedup | Notes |
|---------|------------------|-------|
| 1       | 1.0× (baseline)  | Matches current implementation |
| 4       | 3.5-3.8×         | Good scaling |
| 8       | 6-7×             | Some overhead |
| 16      | 8-10×            | Hyperthreading diminishing returns |

### Overhead Sources

1. RNG state cloning (~248 bytes per work unit)
2. Result collection synchronization
3. Output ordering/buffering
4. Thread pool management

## Open Questions

1. **Memory pressure**: With large batch sizes, how much memory for buffered results?
   - Each Deal is ~208 bytes (52 cards × 4 bytes)
   - Batch of 800 = ~166 KB buffered deals (acceptable)

2. **Progress reporting**: How to show `-m` progress meter with parallel execution?
   - Option: Report after each batch completes
   - Option: Atomic counter updated by workers

3. **Statistics**: How to aggregate average/frequency stats across workers?
   - Each worker accumulates local stats
   - Supervisor merges after batch completion

## References

- [docs/implementation_roadmap.md](implementation_roadmap.md) - Section 4.1 Multi-threading
- [docs/PERFORMANCE_OPTIMIZATION_ANALYSIS.md](PERFORMANCE_OPTIMIZATION_ANALYSIS.md) - Performance analysis
- DealerV2_4 uses `-R N` for threading (1-9 threads)
