# Dealer Implementation Status

**Last Updated:** 2025-12-30

## Overview

This document tracks the implementation status of the dealer constraint language, including both filter functions and action keywords.

---

## Language Features

### ⚠️ **User-Defined Expressions (Variables)**

The dealer language supports defining reusable expressions:

```
nt_opener_north = hcp(north) >= 15 && hcp(north) <= 17 && shape(north, any 4333 + any 4432 + any 5332)
weak_hand_south = hcp(south) <= 8
condition nt_opener_north && weak_hand_south
```

**Status**: ⚠️ **Partially implemented**
- Grammar has `assignment` and `ident` rules defined
- Parser can recognize `variable = expression` syntax
- **NOT evaluated**: No variable storage/lookup in evaluator
- **NOT used**: CLI only parses single expressions, not programs

**What's needed**:
1. Symbol table in evaluator to store variable bindings
2. Variable lookup during expression evaluation
3. Support for multi-statement programs (currently only parses single expressions)
4. Update CLI to parse full programs, not just constraints

---

## Filter Functions (Constraints)

### ✅ **Implemented**

| Function | Description | Status |
|----------|-------------|--------|
| `hcp(position)` | High card points (4-3-2-1) | ✅ Working |
| `hearts(position)` | Number of hearts | ✅ Working |
| `spades(position)` | Number of spades | ✅ Working |
| `diamonds(position)` | Number of diamonds | ✅ Working |
| `clubs(position)` | Number of clubs | ✅ Working |
| `controls(position)` | Control count (A=2, K=1) | ✅ Working |

### ⚠️ **Defined but Not Implemented**

These are in the AST but return `NotImplemented` errors:

| Function | Description | Status |
|----------|-------------|--------|
| `losers(position)` | Loser count | ⚠️ Defined, not evaluated |
| `winners(position)` | Winner count | ⚠️ Defined, not evaluated |
| `shape(position, pattern)` | Shape specification | ⚠️ Defined, not evaluated |
| `hascard(position, card)` | Check for specific card | ⚠️ Defined, not evaluated |

### ❌ **Not Yet Defined**

#### Point Counting Functions
- `pt0(position)` through `pt9(position)` - Alternative point count systems
- `tens(position)` - Number of tens
- `jacks(position)` - Number of jacks
- `queens(position)` - Number of queens
- `kings(position)` - Number of kings
- `aces(position)` - Number of aces
- `c13(position)` - C13 point count

#### Top Cards Functions
- `top2(position, suit)` - Top 2 cards in suit
- `top3(position, suit)` - Top 3 cards in suit
- `top4(position, suit)` - Top 4 cards in suit
- `top5(position, suit)` - Top 5 cards in suit

#### Hand Quality Functions
- `quality(position)` - Hand quality metric
- `cccc(position)` - CCCC evaluation algorithm

#### Advanced Functions
- `tricks(position, contract)` - Double-dummy trick count
- `score(contract, result)` - Contract scoring
- `imps(score_diff)` - Convert score to IMPs

---

## Operators

### ✅ **Implemented**

| Category | Operators | Status |
|----------|-----------|--------|
| **Arithmetic** | `+`, `-`, `*`, `/`, `%` | ✅ Working |
| **Comparison** | `==`, `!=`, `<`, `<=`, `>`, `>=` | ✅ Working |
| **Logical** | `&&`, `||`, `!` | ✅ Working |
| **Unary** | `-` (negation), `!` (not) | ✅ Working |

### ❌ **Not Implemented**

- Ternary operator `? :` (removed by design)

---

## Action Keywords

### ✅ **Partially Implemented**

| Action | Description | Status |
|--------|-------------|--------|
| `produce N` | Generate N matching deals | ✅ Via `-p` flag |

### ❌ **Not Implemented**

#### Print Actions
- `printall` - Print all 4 hands (default in dealer.exe)
- `print(expression)` - Print custom expression
- `printew` - Print E/W hands only
- `printpbn` - PBN format output
- `printcompact` - Compact format
- `printoneline` - One-line format (currently hardcoded)
- `printes` - Print in ES format

#### Statistical Actions
- `average expression` - Calculate averages
- `frequency expression` - Frequency distribution tables

#### Control Commands
- `generate N` - Generate exactly N deals (report all matches)
- `vulnerable none|NS|EW|all` - Set vulnerability
- `dealer N|E|S|W` - Set dealer position
- `predeal player cards` - Pre-assign specific cards
- `pointcount name values` - Define custom point count
- `altcount name values` - Alternative counting method
- `condition expression` - Define filter condition
- `action block` - Define action block

---

## Current CLI Implementation

### Command-Line Arguments

| Argument | Description | Status |
|----------|-------------|--------|
| `-p N` / `--produce N` | Produce N matching deals | ✅ Implemented |
| `-s SEED` / `--seed SEED` | Set random seed | ✅ Implemented |
| `-g N` / `--generate N` | Generate exactly N deals | ❌ Not implemented |

### Default Behavior

- **Input**: Reads constraint from stdin
- **Output**: Printoneline format to stdout (hardcoded)
- **Statistics**: Printed to stderr
- **Seed**: Microsecond-resolution timestamp if not specified

---

## Architecture Status

### Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| `gnurandom` | Exact dealer.exe RNG | ✅ Complete (3 tests) |
| `dealer-core` | Deal generation | ✅ Complete (13 tests) |
| `dealer-pbn` | PBN format I/O | ✅ Basic (9 tests) |
| `dealer-parser` | Constraint parsing | ✅ Basic subset (8 tests) |
| `dealer-eval` | Expression evaluation | ✅ Basic subset (9 tests) |
| `dealer` | CLI application | ✅ Basic (produce mode) |

### Test Coverage

- **Total Tests**: 49 passing
- **Coverage**: Basic functionality only
- **Missing**: Action language, advanced functions, statistics

---

## Limitations of Current Implementation

### Parser Limitations
1. Only parses constraint expressions, not action blocks
2. No support for variable assignments
3. No support for multi-statement programs
4. Grammar has `program` and `statement` rules but they're unused

### Evaluator Limitations
1. Only 6 functions implemented (hcp, 4 suits, controls)
2. No shape analysis
3. No card-specific checks
4. No double-dummy analysis
5. No statistical aggregation

### CLI Limitations
1. Only "produce" mode (no "generate" mode)
2. Output format hardcoded to printoneline
3. No action language support
4. No predeal/vulnerability/dealer position
5. No statistical output (average, frequency)

---

## Dealer Language Architecture

The full dealer language has two parts:

1. **Condition Section** - Filter expressions (partially implemented)
2. **Action Section** - Output and statistics (NOT implemented)

Example full dealer input:
```
# Condition
condition hcp(north) >= 15 && hearts(north) >= 5

# Actions
produce 100
action
    printoneline
    average hcp(north)
    frequency shape(north)
```

**Current implementation only handles simple inline constraints!**

---

## Next Steps for Full Implementation

### High Priority
1. Implement `shape()` function (very common constraint)
2. Implement `hascard()` function
3. Add `-g` / `--generate` mode
4. Add losers/winners functions
5. Parse and handle action blocks

### Medium Priority
6. Multiple output format support
7. Statistical actions (average, frequency)
8. Alternative point counts (pt0-pt9)
9. Predeal support
10. Vulnerability/dealer position

### Low Priority
11. Double-dummy analysis (tricks)
12. Scoring functions (score, imps)
13. Top card functions (top2-top5)
14. Quality metrics (quality, cccc)

---

## Testing Strategy

### Current Testing
- Unit tests for each component
- Golden tests for shuffle algorithm
- Integration tests for basic constraints

### Needed Testing
- Comparison tests against dealer.exe output
- Statistical accuracy tests
- Performance benchmarks
- Edge case coverage (void suits, yarborough, etc.)

---

## References

- **Dealer Manual**: https://www.bridgebase.com/tools/dealer/Manual/input.html
- **Original Dealer**: https://github.com/ThorvaldAagaard/Dealer
- **DealerV2_4**: https://github.com/ThorvaldAagaard/DealerV2_4
