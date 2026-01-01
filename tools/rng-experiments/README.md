# RNG Experiments

This directory contains experimental code used during the development of dealer3's random number generator (RNG) implementation.

## Background

dealer3 needed to exactly replicate the RNG behavior of the original dealer.exe to ensure identical deal generation for the same seed values. This was critical for:

1. **Compatibility**: Scripts using specific seeds should produce identical deals
2. **Testing**: Ability to verify output matches dealer.exe exactly
3. **Reproducibility**: Same seed = same deals, always

## The Challenge

The original dealer.exe uses the GNU C library's `random()` function, which has:
- 64-bit internal state (not 32-bit!)
- BSD TYPE_3 polynomial: x^31 + x^3 + 1
- Custom LCG constant: 1103515145
- 310-iteration warmup phase (10 × rand_deg)
- Special 64-bit arithmetic with sign extension

These details were discovered through reverse engineering and testing against dealer.exe.

## Files in This Directory

### `test-generator.rs`

A simple tool to test RNG implementation during development.

**Purpose**:
- Generate deals using dealer3's RNG
- Output in PBN format for comparison
- Test different seeds and counts
- Verify RNG produces expected output

**Usage**:
```bash
# Run with rust-script (if you have it installed)
./test-generator.rs [seed] [count]

# Examples:
./test-generator.rs 1 5      # Generate 5 deals with seed 1
./test-generator.rs 42 10    # Generate 10 deals with seed 42
./test-generator.rs          # Use defaults (seed=1, count=10)
```

**How It Works**:
1. Creates a `DealGenerator` with specified seed
2. Generates requested number of deals
3. Outputs each deal in PBN format
4. Can be compared against dealer.exe output

**Example Output**:
```
[Deal "N:QT52.K8.A9732.J3 K73.J542.KQT.AK9 AJ4.AT93.J654.Q7 986.Q76.8.T86542"]
[Deal "N:QJ6.KT9.J9752.K3 K8542.765.KQ.AJ9 A7.AJ42.AT643.Q7 T93.Q83.8.T86542"]
...
```

## Testing Methodology

During development, this tool was used alongside dealer.exe to verify correctness:

```bash
# Test dealer3
./test-generator.rs 1 100 > dealer3-output.txt

# Test dealer.exe (on Windows VM)
echo "" | dealer -s 1 -p 100 -f pbn > dealer-output.txt

# Compare
diff dealer3-output.txt dealer-output.txt
```

If the outputs match exactly, the RNG implementation is correct!

## Key Discoveries

Through this testing process, we discovered:

1. **64-bit arithmetic**: dealer.exe uses 64-bit state, not 32-bit
   - Critical for matching output exactly
   - Affects state array initialization and updates

2. **Verification value**: Seed 1 produces first random value `269167349`
   - Used as a quick sanity check during development
   - Documented in gnurandom crate

3. **Warmup phase**: 310 iterations required before first use
   - Not documented in standard GNU random() docs
   - Essential for matching dealer.exe behavior

4. **State array**: 31 64-bit integers
   - Standard BSD TYPE_3 polynomial
   - Special initialization sequence

## Implementation

The final RNG implementation lives in the `gnurandom` crate:
- `gnurandom/src/lib.rs` - Complete GNU random() port
- Tested against dealer.exe binary output
- 100% compatible with original dealer.exe

## Related Documentation

- [gnurandom/README.md](../../gnurandom/README.md) - Final RNG implementation
- [docs/DEALER_FUNCTIONALITY.md](../../docs/DEALER_FUNCTIONALITY.md) - Overall functionality
- [.clinerules](../../.clinerules) - Project context including RNG details

## Historical Note

This test tool was instrumental in achieving exact dealer.exe compatibility. It allowed rapid iteration and verification during the reverse-engineering process. Once the RNG was proven correct, these tools were archived here for reference.

## Running Modern Tests

For current testing, use the main dealer3 binary:

```bash
# Generate deals
echo "hcp(north) >= 0" | ./target/release/dealer -s 1 -p 10 -f pbn

# Compare with dealer.exe (if available)
diff <(echo "hcp(north) >= 0" | dealer.exe -s 1 -p 10 -f pbn) \
     <(echo "hcp(north) >= 0" | ./target/release/dealer -s 1 -p 10 -f pbn)
```

The dealer3 implementation now includes comprehensive test suites in each crate.
