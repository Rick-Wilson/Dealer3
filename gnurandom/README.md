# gnurandom

Exact compatibility implementation of the GNU/glibc random() RNG used by dealer.exe.

## Status

✅ **SOLVED!** - Exact dealer.exe compatibility achieved through 64-bit implementation.

## Mystery Solved! 🎉

After extensive investigation including disassembling the original `__random.o` object file, we discovered that **dealer.exe was compiled with 64-bit `long int` types on x86_64 architecture**.

### The Key Discovery

The dealer.exe `__random.o` uses:
- **64-bit state array** (`long int state[31]` where `long` = `int64_t`)
- **64-bit arithmetic** throughout the LCG and warmup phases
- **64-bit arithmetic shift** (`sarq` instruction in x86_64)
- **64-bit LONG_MAX** (0x7fffffffffffffff, not 0x7fffffff)

This causes the LCG to overflow into 64-bit space during initialization, creating negative 64-bit values. When these are arithmetically shifted right by 1 and masked with the 64-bit LONG_MAX, then truncated back to 32-bit for output, it produces the unique bit pattern that differs from a pure 32-bit implementation.

### Assembly Evidence

From disassembly of `__random.o`:
```asm
# LCG initialization with 64-bit arithmetic
52: imulq $0x41c64e09, (%rax,%rcx,8), %rax  # state[i-1] * 1103515145
5a: addq  $0x3039, %rax                     # + 12345
6b: movq  %rax, (%rcx,%rdx,8)               # Store 64-bit result

# Output generation
160: sarq   $0x1, %rax                       # 64-bit arithmetic shift right
164: movabsq $0x7fffffffffffffff, %rcx       # Load 64-bit LONG_MAX
16e: andq   %rcx, %rax                       # AND with 64-bit mask
```

### Why This Matters

A standard 32-bit BSD random() implementation produces different output because:
- 32-bit LCG stays within 32-bit bounds
- 32-bit arithmetic shift behaves differently
- The mask 0x7fffffff clears bit 31 in all cases

The 64-bit implementation:
- LCG overflows into 64-bit negative values (e.g., `state[4] = -7625533116005925835`)
- 64-bit arithmetic shift preserves sign information differently
- The mask 0x7fffffffffffffff only clears bit 63, leaving bit 31 potentially set
- Truncation to u32 preserves the unique bit pattern in the lower 32 bits

### Confirmed Implementation Details

1. **Algorithm**: BSD TYPE_3 (x^31 + x^3 + 1 polynomial)
2. **State Array**: 31 elements of `i64` (64-bit signed integers)
3. **Separation**: 3 (SEP_3)
4. **LCG Constant**: 1103515145 (non-standard, not the typical 1103515245)
   - Source: https://github.com/ThorvaldAagaard/Dealer/blob/main/__random.c
5. **LCG Formula**: `state[i] = state[i-1] * 1103515145 + 12345` (64-bit wrapping arithmetic)
6. **Warmup**: 10 * rand_deg (310 iterations) of `*fptr += *rptr` (64-bit addition)
7. **Output**: `(*fptr >> 1) & 0x7fffffffffffffff` (64-bit operations) truncated to `u32`

### Testing

The implementation has been verified against dealer.exe's `rand.o`/`srand.o` object files using the `rng_probe_x86` test program:

```bash
cargo test
```

**Test Results**: ✅ All tests pass
- Seed 1: First 20 values match exactly
- Seed 2: First 10 values match exactly
- All values verified against dealer.exe binary output

## Usage

```rust
use gnurandom::GnuRandom;

let mut rng = GnuRandom::new();
rng.srandom(1);

let value = rng.next_u32();  // 269167349
```

## Performance Note

This implementation uses 64-bit arithmetic throughout to maintain exact compatibility with the x86_64 dealer.exe binary. While this is slightly more expensive than a pure 32-bit implementation, it ensures perfect compatibility with legacy dealer.exe output for reproducible bridge hand generation.

## License

Apache-2.0
