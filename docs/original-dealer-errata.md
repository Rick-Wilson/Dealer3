# Original dealer.exe Errata

This document describes bugs and quirks in the original dealer.exe that dealer3 intentionally does **not** replicate.

## 1. Block Comment Character Echo Bug

**Affected versions**: All known versions of dealer.exe

**Symptom**: When using `/* */` block comments, certain characters inside the comment are echoed to stdout before the normal output.

**Affected characters**: `<`, `E`, `O`, `F`, `>`

**Example**:
```
/* convention-card: 21GF-Puppet */
```
This outputs `F` before the first line of normal output because `GF` contains `F`.

```
/* FGOGE */
```
This outputs `FOE` (the `G` is not echoed, but `F`, `O`, `E` are).

**Root cause**: In `scan.l`, the lexer rules for handling block comments use a character class that incorrectly includes `<<EOF>>`:

**Source location**: `scan.l`, lines ~15-16 (in the COMMENT start condition rules)

```lex
<COMMENT>[^*\n<<EOF>>]*
<COMMENT>"*"+[^*/\n<<EOF>]*
```

The `<<EOF>>` was intended to handle end-of-file within comments, but inside a character class `[...]`, it's interpreted as the literal characters `<`, `E`, `O`, `F`, `>`. These characters are excluded from the pattern, so they don't match and fall through to the default lex action, which echoes unmatched input to stdout.

**Fix**: Replace the character classes with proper EOF handling:

```lex
<COMMENT>[^*\n]*
<COMMENT>"*"+[^*/\n]*
<COMMENT><<EOF>>   { /* handle unterminated comment */ }
```

The `<<EOF>>` token should be its own rule, not embedded in a character class.

**Workaround**: Use `#` single-line comments instead of `/* */` block comments, or avoid the characters `<`, `E`, `O`, `F`, `>` inside block comments.

**dealer3 behavior**: dealer3 correctly ignores all content inside block comments.

---

## 2. PBN Verbose Toggle Bug

**Affected versions**: All known versions of dealer.exe

**Symptom**: When using `printpbn` output format, the verbose statistics output (Generated/Produced/Time) toggles on and off with each deal printed.

**Effect**:
- With an odd number of deals (`-p 1`, `-p 3`, etc.): statistics are suppressed
- With an even number of deals (`-p 2`, `-p 4`, etc.): statistics are shown

**Root cause**: In `pbn.c`, the `printpbn()` function toggles a global verbose flag each time it's called.

**Source location**: `pbn.c`, in the `printpbn()` function (near the top of the function body)

```c
void printpbn(...) {
    verbose = !verbose;  // Bug: toggles each call
    ...
}
```

This appears to be a debugging artifact that was never removed.

**Fix**: Remove the `verbose = !verbose;` line. The verbose flag should be set only by command-line argument parsing, not toggled during output.

**Workaround**: Use `-p` with even numbers, or use `-v` flag to force verbose output (though this has its own interactions).

**dealer3 behavior**: dealer3 does not replicate this bug. The `-v` flag consistently controls verbose output regardless of the number of deals produced or the output format.

---

## Summary

These bugs are documented here for reference when comparing dealer3 output against dealer.exe. The comparison script (`compare-dealer.sh`) filters out some of these differences (e.g., stripping leading garbage characters, filtering `[Event` and `[Date` lines) to allow meaningful output comparison despite these bugs.

When exact compatibility with dealer.exe bug-for-bug is required (rare), users should be aware of these differences.
