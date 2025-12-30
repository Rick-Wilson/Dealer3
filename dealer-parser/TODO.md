# Parser TODO - Next Session

## Current Status

✅ **Grammar extended** with new features (not yet tested):
- Multi-line comments (`/* */`) and C++ style (`//`)
- Variable assignments (`var = expr`)
- Ternary operator (`cond ? val1 : val2`)
- `and`/`or` keyword operators (in addition to `&&`/`||`)
- `not` keyword operator (in addition to `!`)
- Multi-argument function calls
- Card names (AS, KH, 2C, etc.)
- Identifier references
- New functions: `top2`, `top3`, `top4`, `top5`

✅ **Test files downloaded** (6 real-world `.dlr` files in `tests/fixtures/`):
- 1N.dlr
- 2N.dlr
- Weak_2_Bids.dlr
- Jacoby_2N.dlr
- Takeout_Double.dlr
- Resp_to_1C.dlr

✅ **Feature analysis complete** - See PARSER_FEATURES.md

## Next Steps

### 1. Update AST Types
The current AST in `ast.rs` needs to support:
- **Ternary expressions**: Add `Ternary { cond, true_val, false_val }`
- **Variable references**: Add `Identifier(String)`
- **Card literals**: Add `Card(String)` or parse into rank/suit
- **Multi-arg functions**: Change `FunctionCall` to support `Vec<Expr>` args
- **Assignment statements**: New top-level `Statement` enum

### 2. Update Parser Code
The parser in `parser.rs` needs new match arms for:
- `Rule::ternary`
- `Rule::or_op` / `Rule::and_op` / `Rule::not_op`
- `Rule::card`
- `Rule::ident`
- `Rule::program` / `Rule::statement` / `Rule::assignment`
- Updated `Rule::function_call` for multiple args

### 3. Fix Negation
The `not` keyword should now work with the updated grammar:
```
logical_not = { not_op ~ logical_not | comparison }
not_op = { "!" | ^"not" }
```

Update the parser code to handle this properly.

### 4. Testing Strategy

**Phase 1: Unit tests for new features**
```rust
parse("x = 5")  // Assignment
parse("x > 0 ? 1 : 0")  // Ternary
parse("x and y")  // and keyword
parse("not x")  // not keyword
parse("hcp(south, spades)")  // 2-arg function
parse("hascard(south, AS)")  // Card literal
```

**Phase 2: Integration tests**
- Parse small snippets from actual .dlr files
- Verify they produce correct AST

**Phase 3: Full file parsing**
- Eventually parse complete .dlr files
- Will need to implement more features (shape patterns, etc.)

## Known Limitations

Still need to implement:
- `shape()` function with pattern syntax (`6xxx`, `any 5332`, etc.)
- `dealer` declaration
- `condition` statement
- `action` blocks
- `generate`/`produce` commands

But the grammar foundation is now much more robust!

## Quick Start for Next Session

1. Run `cargo test -p dealer-parser` - will fail (AST doesn't match grammar)
2. Update AST in `ast.rs` to add new types
3. Update parser in `parser.rs` to handle new grammar rules
4. Add tests for each new feature
5. Get tests passing
6. Try parsing snippets from .dlr files

Good luck! 🚀
