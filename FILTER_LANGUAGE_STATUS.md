# Dealer Implementation Status

**Last Updated:** 2025-12-30

## Overview

This document tracks the implementation status of the dealer constraint language, including both filter functions and action keywords.

### Quick Summary

**✅ Core Features Working:**
- 10 filter functions (hcp, suits, controls, losers, shape, hascard)
- All arithmetic, comparison, and logical operators
- Shape pattern matching (exact, wildcard, any distribution)
- Card and suit literals
- Produce mode (`-p N`) with seeded generation (`-s SEED`)

**⚠️ Partially Implemented:**
- Variables (grammar ready, evaluation not implemented)

**❌ Not Yet Implemented:**
- Advanced functions (pt0-pt9, top2-5, quality, cccc, tricks, score, imps)
- Action blocks (print formats, statistics, control commands)
- Generate mode (`-g N`)
- Predeal, vulnerability, dealer position

**Test Status:** 58 tests passing across all crates

---

## Language Features

### ⚠️ **User-Defined Expressions (Variables)**

The dealer language supports defining reusable expressions:

```
nt_opener_north = hcp(north) >= 15 && hcp(north) <= 17 && shape(north, any 4333 + any 4432 + any 5332)
weak_hand_south = hcp(south) <= 8
condition nt_opener_north && weak_hand_south
```

**Implementation Model**: Variables are **runtime-evaluated**, not macros. Each variable reference is evaluated in the context of the current deal being analyzed, not textually expanded during parsing. This allows variables to dynamically respond to different hands.

**Status**: ⚠️ **Partially implemented**
- Grammar has `assignment` and `ident` rules defined
- Parser can recognize `variable = expression` syntax
- **NOT evaluated**: No variable storage/lookup in evaluator
- **NOT used**: CLI only parses single expressions, not programs

**What's needed**:
1. Add `Expr::Variable(String)` variant to AST
2. Symbol table in evaluator to store variable → expression bindings
3. Variable lookup during expression evaluation (evaluate stored expression each time)
4. Support for multi-statement programs (currently only parses single expressions)
5. Update CLI to parse full programs, not just constraints

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
| `losers(position)` | Total loser count in hand | ✅ Working |
| `losers(position, suit)` | Losers in specific suit | ✅ Working |
| `shape(position, pattern)` | Shape specification | ✅ Working |
| `hascard(position, card)` | Check for specific card | ✅ Working |

**Loser Count Details:**
- Uses standard losing trick count algorithm
- Void: 0 losers
- Singleton: 0 if Ace, 1 otherwise
- Doubleton: 0 for AK, 1 for Ax/Kx, 2 otherwise
- 3+ cards: Start with 3, subtract 1 for each A/K/Q in top 3 positions
- Examples: `losers(north) <= 7`, `losers(south, spades) == 0`

**Shape Pattern Syntax:**
- Exact shapes: `shape(north, 5431)` - exactly 5-4-3-1 in S-H-D-C order
- Wildcard patterns: `shape(south, 54xx)` - 5 spades, 4 hearts, any minors
- Any distribution: `shape(east, any 4333)` - any 4-3-3-3 regardless of suits
- Combinations: `shape(west, any 4333 + any 5332 - 5332)` - balanced except exact 5-3-3-2
- Uses `+` for inclusion, `-` for exclusion

**Card Syntax:**
- Format: rank + suit (e.g., AS, KH, TC, 2D)
- Ranks: A, K, Q, J, T, 9, 8, 7, 6, 5, 4, 3, 2
- Suits: S (spades), H (hearts), D (diamonds), C (clubs)
- Example: `hascard(north, AS)` checks if north has ace of spades

**Suit Keywords:**
- Used as arguments to functions like `losers(position, suit)`
- Keywords: spades, hearts, diamonds, clubs (case-insensitive)
- Example: `losers(north, spades) == 0` checks for solid spade suit

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
| `dealer-parser` | Constraint parsing | ✅ Expanded (8 tests) |
| `dealer-eval` | Expression evaluation | ✅ Expanded (18 tests) |
| `dealer` | CLI application | ✅ Basic (produce mode) |

### Test Coverage

- **Total Tests**: 58 passing
- **New Tests**: 4 tests for losers/hascard, 6 tests for shape
- **Coverage**: Core constraint functions implemented
- **Missing**: Action language, statistical functions, advanced evaluations

---

## Limitations of Current Implementation

### Parser Limitations
1. Only parses constraint expressions, not action blocks
2. No support for variable assignments
3. No support for multi-statement programs
4. Grammar has `program` and `statement` rules but they're unused

### Evaluator Limitations
1. 10 core functions implemented (hcp, 4 suits, controls, losers, shape, hascard)
2. Missing advanced functions (pt0-pt9, top2-top5, quality, cccc)
3. No double-dummy analysis (tricks)
4. No scoring functions (score, imps)
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
