# Test Failure Report - 2026-01-06

## Summary

**Results: 282/293 passed, 11 failed (96.2% pass rate)**

94 tests had performance warnings (Rust >1s slower than Windows)

## Failing Scenarios by Type

### 1. Syntax Error - Chained Comparison (1 test)

Unsupported `a==b==(x or y)` chained comparison syntax:

| File | Error |
|------|-------|
| **GIB_1C-P-Resp.dlr** | `spades(west)==hearts(west)==(3 or 4)` - chained comparison not supported |

**Fix Required**: Grammar change to support chained comparisons, or document as unsupported syntax.

---

### 2. Timeout - Evaluation Mismatch (9 tests)

Rust times out (60s) while Windows completes. These have different "generated" counts, indicating condition evaluation differs:

| File | Rust Generated | Win Generated | Likely Cause |
|------|----------------|---------------|--------------|
| **Double_by_Advancer.dlr** | 2.8M (timeout) | completed | Condition matches less often in Rust |
| **GIB_Sandwich_NT_BPH.dlr** | 1.07M (timeout) | 6.2M | Condition matches less often in Rust |
| **Gerber_By_Opener.dlr** | timeout | 2.6M | Condition matches less often in Rust |
| **McCabe_after_WJO.dlr** | timeout | 5.2M | Condition matches less often in Rust |
| **Non_Leaping_Michaels_After_2-Bid.dlr** | timeout | 9.9M | Condition matches less often in Rust |
| **Opps_Bal_Unusual_2N.dlr** | 1.02M, 0 produced | 10M, 8 produced | Condition **never** matches in Rust |
| **Snapdragon_Double.dlr** | timeout | 3.5M | Condition matches less often in Rust |
| **Trap_Pass.dlr** | 1.4M (timeout) | completed | Condition matches less often in Rust |
| **Trap_Pass_Maybe.dlr** | timeout | completed | (same as Trap_Pass) |

**Root Cause**: Most likely caused by:
- `losers()` function implementation differences
- `top3()` / `top2()` / `top4()` / `top5()` function differences
- Shape pattern evaluation edge cases

---

### 3. Different Generated Count - Same Deals (1 test)

Both produce 10 deals with matching content, but generated counts differ significantly:

| File | Rust Generated | Win Generated | Issue |
|------|----------------|---------------|-------|
| **Grand_Slam_Force.dlr** | 8,456,040 | 1,234,935 | `top3()` or `losers()` function returns different values |

**Analysis**: This script uses:
- `top3(north,hearts)==1`
- `losers(north,spades)==0`

The 6.8x difference in iterations suggests our `top3()` or `losers()` implementation differs from dealer.exe for some hands.

---

## Recommended Next Steps

1. **High Priority**: Investigate `losers()` and `top3()` function implementations
   - Compare output for specific hands between Rust and Windows
   - Grand_Slam_Force.dlr is a good test case (same deals, different counts)

2. **Medium Priority**: Fix chained comparison syntax (`a==b==c`)
   - Requires grammar modification
   - Only affects 1 test currently

3. **Low Priority**: Performance optimization
   - 94 tests show Rust is >1s slower than Windows
   - Multi-threading implementation would help (design already documented)

## Test Environment

- **Date**: 2026-01-06
- **Rust Version**: dealer3 v0.2.0 (unreleased)
- **Windows dealer.exe**: Original Henk Uijterwaal version
- **Seed used**: 42
- **Timeout**: 60 seconds
