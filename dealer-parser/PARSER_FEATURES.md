# Parser Features Analysis

Based on real-world `.dlr` files from Practice-Bidding-Scenarios repository.

## Currently Supported ✅

- **Functions**: `hcp(position)`, `hearts(position)`, `spades(position)`, `diamonds(position)`, `clubs(position)`, `controls(position)`
- **Operators**:
  - Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
  - Logical: `&&`, `||`
  - Arithmetic: `+`, `-`, `*`, `/`, `%`
- **Positions**: `north`/`n`, `south`/`s`, `east`/`e`, `west`/`w` (case-insensitive)
- **Literals**: Integer numbers
- **Parentheses**: For grouping expressions

## Missing Features ❌

### High Priority (Needed for Basic Files)

1. **Variable Assignment**
   ```
   tp = hcp(south)
   s = spades(east)
   ```

2. **Ternary Operator**
   ```
   lp1 = spades(south)>4 ? spades(south)-4 : 0
   ```

3. **`not` Operator** (as a word, not just `!`)
   ```
   notNT = not (shape(south, any 5332))
   oH = not oS and h>4
   ```

4. **Comments**
   ```
   # Single line comment
   // C++ style comment
   /* Multi-line
      comment */
   ```

5. **`shape()` Function**
   ```
   shape(south, 6xxx-x6xx-x5xx-x4xx)
   shape(south, any 5332 +any 4432 +any 4333)
   shape(south, 2452+2425+2245+2254+4522)
   ```

6. **`hascard()` Function**
   ```
   hascard(west,2C)
   hascard(south,AS)
   ```

7. **`top2()`, `top3()`, `top4()`, `top5()` Functions**
   ```
   top4(south,spades)>0
   top2(south,spades)==1
   ```

8. **`losers()` Function**
   ```
   losers(south)<7
   ```

9. **Two-argument function calls** (currently only support one arg)
   ```
   hcp(south,spades)  // HCP in specific suit
   top4(south,spades)
   hascard(south,AS)
   ```

10. **`dealer` Declaration**
    ```
    dealer south
    dealer east
    ```

### Medium Priority

11. **Card Names** (as identifiers, not strings)
    ```
    AS, AH, AD, AC (Aces)
    2C, 3D, etc. (rank + suit)
    ```

12. **`or` as Binary Operator** (currently only parsing, not creating OR nodes)
    ```
    lp1 or lp2 or lp3 or lp4
    ```

13. **Identifier References** (variables)
    ```
    oS = ...
    oH = not oS and ...   // referencing oS variable
    ```

14. **`any` Keyword in Shape**
    ```
    any 5332
    any 4432
    ```

### Lower Priority

15. **`action` Blocks**
    ```
    action
      printpbn
      average ...
    ```

16. **`condition` Statement**
    ```
    condition NT and levelTheDeal
    ```

17. **`generate` and `produce` Commands**
    ```
    generate 100000
    produce 100
    ```

## Implementation Plan

### Phase 1: Core Language Features
- [ ] Variable assignments (`var = expr`)
- [ ] Ternary operator (`cond ? val1 : val2`)
- [ ] `not` keyword operator
- [ ] Comments (`#`, `//`, `/* */`)
- [ ] Identifier references in expressions

### Phase 2: Essential Functions
- [ ] Two-argument function calls
- [ ] `shape(position, pattern)` function
- [ ] `hascard(position, card)` function
- [ ] `top2/3/4/5(position, suit)` functions
- [ ] `losers(position)` function
- [ ] `hcp(position, suit)` - HCP in specific suit

### Phase 3: Card Representation
- [ ] Card name parsing (AS, KH, 2C, etc.)
- [ ] Suit/rank extraction from card names

### Phase 4: Advanced Features
- [ ] `dealer` declaration
- [ ] `condition` statement
- [ ] `action` blocks
- [ ] `generate`/`produce` commands

## Test Coverage Strategy

1. **Simple expressions** - Basic arithmetic and comparisons
2. **Variable assignments** - Store and reference values
3. **Ternary operators** - Conditional value selection
4. **Shape patterns** - Hand distribution matching
5. **Card queries** - Specific card presence
6. **Complex logic** - Real-world dealer files

## Notes

- Most `.dlr` files use a "define then filter" pattern:
  1. Assign intermediate variables
  2. Define complex conditions using those variables
  3. Final `condition` statement combines everything

- We can parse incrementally - start with core features, add functions as needed
- The AST already supports most operators, we just need grammar updates
