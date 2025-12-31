# Dealer Implementation Status

**Last Updated:** 2025-12-30

## Overview

This document tracks the implementation status of the dealer constraint language, including both filter functions and action keywords.

### Quick Summary

**✅ Core Features Working:**
- 22 filter functions (hcp, suits, controls, losers, shape, hascard, tens, jacks, queens, kings, aces, top2-5, c13, quality, cccc)
- All arithmetic, comparison, and logical operators
- Shape pattern matching (exact, wildcard, any distribution)
- Card and suit literals
- Alternative point counts (pt0-pt9)
- Hand quality evaluation (quality, cccc)
- Variables (full support for assignments and references)
- Produce mode (`-p N`) with seeded generation (`-s SEED`)

**❌ Not Yet Implemented:**
- Advanced functions (tricks, score, imps)
- Action blocks (print formats, statistics, control commands)
- Generate mode (`-g N`)
- Predeal, vulnerability, dealer position

**Test Status:** 87 tests passing across all crates

---

## Language Features

### ✅ **User-Defined Expressions (Variables)**

The dealer language supports defining reusable expressions:

```
nt_opener = hcp(north) >= 15 && hcp(north) <= 17 && shape(north, any 4333 + any 4432 + any 5332)
weak_hand = hcp(south) <= 8
nt_opener && weak_hand
```

**Implementation Model**: Variables are **runtime-evaluated**, not macros. Each variable reference is evaluated in the context of the current deal being analyzed, not textually expanded during parsing. This allows variables to dynamically respond to different hands.

**Status**: ✅ **Fully implemented**
- `Expr::Variable(String)` variant in AST
- `Program` and `Statement` types for multi-statement support
- Symbol table in evaluator (HashMap<String, Expr>)
- Variable lookup during expression evaluation (evaluates stored expression each time)
- CLI parses full programs with `parse_program()`, not just single expressions
- Supports variables referencing other variables (recursive evaluation)

**Example Usage**:
```bash
# Simple variable
printf "opener = hcp(north) >= 15\nopener" | dealer -p 10

# Multiple variables
printf "strong = hcp(north) >= 15\nlong_hearts = hearts(north) >= 5\nstrong && long_hearts" | dealer -p 10

# Variables can reference other variables
printf "points = hcp(north)\nopener = points >= 15\nopener" | dealer -p 10
```

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
| `tens(position)` | Number of tens (pt0) | ✅ Working |
| `tens(position, suit)` | Tens in specific suit | ✅ Working |
| `jacks(position)` | Number of jacks (pt1) | ✅ Working |
| `jacks(position, suit)` | Jacks in specific suit | ✅ Working |
| `queens(position)` | Number of queens (pt2) | ✅ Working |
| `queens(position, suit)` | Queens in specific suit | ✅ Working |
| `kings(position)` | Number of kings (pt3) | ✅ Working |
| `kings(position, suit)` | Kings in specific suit | ✅ Working |
| `aces(position)` | Number of aces (pt4) | ✅ Working |
| `aces(position, suit)` | Aces in specific suit | ✅ Working |
| `top2(position)` | Top 2 honors AK (pt5) | ✅ Working |
| `top2(position, suit)` | Top 2 in specific suit | ✅ Working |
| `top3(position)` | Top 3 honors AKQ (pt6) | ✅ Working |
| `top3(position, suit)` | Top 3 in specific suit | ✅ Working |
| `top4(position)` | Top 4 honors AKQJ (pt7) | ✅ Working |
| `top4(position, suit)` | Top 4 in specific suit | ✅ Working |
| `top5(position)` | Top 5 honors AKQJT (pt8) | ✅ Working |
| `top5(position, suit)` | Top 5 in specific suit | ✅ Working |
| `c13(position)` | C13 points A=6,K=4,Q=2,J=1 (pt9) | ✅ Working |
| `c13(position, suit)` | C13 points in specific suit | ✅ Working |
| `quality(position, suit)` | Suit quality metric | ✅ Working |
| `cccc(position)` | CCCC hand evaluation | ✅ Working |

**Alternative Point Counts (pt0-pt9):**
The dealer language provides 10 alternative point count functions with readable synonyms:
- `pt0` / `tens` - Count of tens
- `pt1` / `jacks` - Count of jacks
- `pt2` / `queens` - Count of queens
- `pt3` / `kings` - Count of kings
- `pt4` / `aces` - Count of aces
- `pt5` / `top2` - Top 2 honors (A, K)
- `pt6` / `top3` - Top 3 honors (A, K, Q)
- `pt7` / `top4` - Top 4 honors (A, K, Q, J)
- `pt8` / `top5` - Top 5 honors (A, K, Q, J, T)
- `pt9` / `c13` - C13 point count (A=6, K=4, Q=2, J=1)

Examples: `top3(north) >= 5`, `aces(south, spades) == 1`, `c13(north) + c13(south) >= 40`

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

**Hand Quality Metrics (Bridge World Oct 1982):**

The quality and cccc functions implement hand evaluation algorithms from Bridge World, October 1982. Both return values multiplied by 100 to use integer math (e.g., 1500 = 15.00 points).

**Quality Function - `quality(position, suit)`:**
Evaluates the quality of a specific suit based on length and honor cards.
- Base values: A=4×SuitFactor, K=3×SuitFactor, Q=2×SuitFactor, J=1×SuitFactor (where SuitFactor = Length × 10)
- Ten bonus: Full SuitFactor if 2+ higher honors or has J; half otherwise
- Nine bonus: Half SuitFactor if 2 higher honors, or has T, or has 8
- Long suit bonus (7+ cards): Adds points for missing honors that would be replaced
- **Note**: Quality values are multiplied by 100 to use integer math (e.g., 1500 = 15.00 points).
- Examples:
  - `quality(north, spades) >= 4000` - Strong spade suit (40.00+ quality points)
  - `quality(south, hearts) < 100` - Weak heart suit (< 1.00 quality points)

**CCCC Function - `cccc(position)`:**
Comprehensive hand evaluation combining honor strength, suit quality, and shape.
- Honor points: A=300, K=200, Q=100, with adjustments for shortage
  - Singleton K: -150, Singleton Q: -75, Doubleton Q: -25
  - Unsupported Q (no higher honor): -25
  - J: +50 if 2 higher honors, +25 if 1 higher
  - T: +25 if 2 higher honors, +25 if 1 higher + nine
- Adds suit_quality for each suit
- Shape points: +100 for each short suit (< 3 cards)
- Balanced adjustment: -50 if balanced, else ShapePoints - 100
- **Note**: CCCC values are multiplied by 100 to use integer math (e.g., 1500 = 15.00 points).
- **Automatic preprocessing**: 4-digit numbers in regular expressions work correctly (e.g., `cccc(north) >= 1500`) thanks to automatic preprocessing that distinguishes shape patterns from numeric literals.
- Examples:
  - `cccc(north) >= 1500` - Strong opening hand (15.00+ points)
  - `cccc(south) + cccc(north) >= 2400` - Game-level partnership (24.00+ combined points)

### ❌ **Not Yet Defined**

#### Double-Dummy Analysis Functions (Requires External Library)
These functions require a double-dummy solver (DDS library) and are deferred:
- `tricks(position, contract)` - Double-dummy trick count
- `score(contract, result)` - Contract scoring (may depend on tricks)
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
| `dealer-parser` | Constraint parsing | ✅ Expanded (20 tests) |
| `dealer-eval` | Expression evaluation | ✅ Expanded (45 tests) |
| `dealer` | CLI application | ✅ Basic (produce mode) |

### Test Coverage

- **Total Tests**: 87 passing
- **Variables**: 9 tests for variable assignment, lookup, and recursive evaluation
- **Preprocessing**: 7 tests for 4-digit number disambiguation
- **Quality/CCCC**: 7 tests for hand evaluation functions (2 unit tests, 3 evaluation tests, 2 integration tests)
- **Coverage**: All core constraint functions, alternative point counts, hand quality metrics, variables, and automatic preprocessing implemented
- **Missing**: Action language, statistical functions, double-dummy analysis

### Preprocessing System

The parser includes an automatic preprocessing step that solves the ambiguity between 4-digit shape patterns and 4-digit numeric literals:

**Problem**: In PEG parsers, `shape(north, 5242)` and `cccc(north) >= 1500` both contain 4-digit numbers, but only the first should be parsed as a shape pattern.

**Solution**: Before parsing, all input is preprocessed to mark 4-digit numbers inside `shape()` functions with a `%s` prefix:
- `shape(north, 5242)` → `shape(north, %s5242)` (marked as shape pattern)
- `cccc(north) >= 1500` → unchanged (numeric literal)
- `shape(north, any 4333 - 4333)` → `shape(north, any 4333 - %s4333)` (only mark non-"any" patterns)

The grammar is then designed to require the `%s` marker for pure-digit shape patterns, while wildcards (e.g., `54xx`) and "any"-prefixed patterns don't need it. This allows users to write natural expressions like `cccc(north) >= 1500` without workarounds.

---

## Limitations of Current Implementation

### Parser Limitations
1. Only parses constraint expressions, not action blocks
2. No support for full dealer input format (e.g., `condition`, `produce`, `action` keywords)

### Evaluator Limitations
1. 22 core functions implemented (hcp, 4 suits, controls, losers, shape, hascard, tens-aces, top2-5, c13, quality, cccc)
2. No double-dummy analysis (tricks)
3. No scoring functions (score, imps)
4. No statistical aggregation

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
1. Add `-g` / `--generate` mode
2. Parse and handle action blocks
3. Multiple output format support
4. Statistical actions (average, frequency)
5. Predeal support

### Medium Priority
6. Vulnerability/dealer position
7. Performance optimization for large deal generation

### Low Priority
9. Double-dummy analysis (tricks) - requires DDS library
10. Scoring functions (score, imps)
11. Additional evaluation metrics

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
