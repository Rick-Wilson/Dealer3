# Command-Line Switch Comparison: dealer.exe vs Our Implementation vs DealerV2_4

## Overview

This document compares command-line switches across three implementations:
1. **dealer.exe** (Original Thomas Andrews version)
2. **Our Rust Implementation** (dealer3)
3. **DealerV2_4** (Thorvald Aagaard's enhanced version)

---

## Complete Switch Comparison Table

| Switch | dealer.exe | dealer3 (ours) | DealerV2_4 | Priority | Notes |
|--------|------------|----------------|------------|----------|-------|
| **Core Generation** |
| `-p N` | Produce N hands (default 40) | ✅ Same | ✅ Same | **IMPLEMENTED** | Core feature |
| `-g N` | Generate N hands (default 1M) | ✅ Same | ✅ Same | **IMPLEMENTED** | Core feature |
| `-s N` | Random seed | ✅ Same | ✅ Same | **IMPLEMENTED** | Core feature |
| **Output Control** |
| `-q` | Suppress PBN output | ❌ Not implemented | ✅ PBN Quiet mode | Medium | Testing feature |
| `-v` | Verbose (show stats) | ⚠️ Used for vulnerability | ✅ Toggle EOJ stats | High | **CONFLICT**: We use for vulnerability |
| `-V` | Version info | ❌ Not implemented | ✅ Show version and exit | High | Standard practice |
| `-h` | Help | ✅ Auto (clap) | ✅ Help | **IMPLEMENTED** | Standard |
| **Format/Display** |
| `-f FORMAT` | ❌ Not in original | ✅ Format selection | ❌ Not in V2_4 | **IMPLEMENTED** | Our enhancement |
| `-u` | Upper/lowercase AKQJT | ❌ Not implemented | ❌ Not in V2_4 | Low | Cosmetic |
| `-m` | Progress meter | ❌ Not implemented | ✅ Progress meter | Medium | Useful for long runs |
| **Predeal** |
| `-N/E/S/W CARDS` | ❌ Not in original | ❌ Not as switch | ✅ Compass predeal | Medium | V2_4 adds as switch |
| predeal keyword | ✅ In input file | ✅ In input file | ✅ In input file | **IMPLEMENTED** | We support via input |
| **Position/Vulnerability** |
| `-d POS` | ❌ Not in original | ✅ Dealer position | ❌ Not in V2_4 | **IMPLEMENTED** | Our enhancement |
| `-v VULN` | ❌ Not in original | ✅ Vulnerability | ❌ Not in V2_4 | **IMPLEMENTED** | Our enhancement |
| `-P N` | ❌ Not in original | ❌ Not implemented | ✅ Vulnerability for Par (0-3) | Medium | V2_4 only |
| **Swapping Modes** |
| `-0` | No swapping (default) | ✅ Default behavior | ✅ Same | **IMPLEMENTED** | Default |
| `-2` | 2-way swap (E/W) | ❌ Not implemented | ❌ Not in V2_4 | Low | Not compatible with predeal |
| `-3` | 3-way swap (E/W/S) | ❌ Not implemented | ❌ Not in V2_4 | Low | Not compatible with predeal |
| `-x MODE` | ❌ Not in original | ❌ Not implemented | ✅ eXchange mode 2|3 | Low | V2_4 adds this |
| **Double-Dummy Analysis** (V2_4 only) |
| `-M MODE` | ❌ Not in original | ❌ Not implemented | ✅ DDS mode 1|2 | Low | Requires DDS library |
| `-R N` | ❌ Not in original | ❌ Not implemented | ✅ Resources/Threads 1-9 | Low | Requires DDS library |
| **Library/Advanced** (V2_4 only) |
| `-l PATH` | Read from library.dat | ❌ Not implemented | ✅ DL52 format export | Low | Different meaning in V2_4 |
| `-L PATH` | ❌ Not in original | ❌ Not implemented | ✅ RP Library source path | Low | V2_4 advanced feature |
| `-U PATH` | ❌ Not in original | ❌ Not implemented | ✅ DealerServer pathname | Low | V2_4 server mode |
| **Export/Reporting** (V2_4 only) |
| `-C FILE` | ❌ Not in original | ❌ Not implemented | ✅ CSV Report filename | Medium | V2_4 analytics |
| `-X FILE` | ❌ Not in original | ❌ Not implemented | ✅ Export predeal holdings | Low | V2_4 feature |
| `-Z FILE` | ❌ Not in original | ❌ Not implemented | ✅ RP zrd format export | Low | V2_4 format |
| **OPC/Title** (V2_4 only) |
| `-O POS` | ❌ Not in original | ❌ Not implemented | ✅ OPC evaluation Opener | Low | V2_4 specific |
| `-T "text"` | ❌ Not in original | ❌ Not implemented | ✅ Title in quotes | Low | V2_4 metadata |
| **Script Parameters** (V2_4 only) |
| `-0` to `-9` | ❌ Not in original | ❌ Not implemented | ✅ Set $0-$9 script params | Low | V2_4 scripting |
| **Debug** |
| `-D LEVEL` | ❌ Not in original | ❌ Not implemented | ✅ Debug verbosity 0-9 | Low | V2_4 debugging |
| `-e` | Exhaust mode (alpha) | ❌ Not implemented | ❌ Not in V2_4 | Low | Experimental |

---

## Summary Statistics

### dealer.exe (Original)
- **Total switches**: 13
- **Core features**: 5 (`-p`, `-g`, `-s`, `-h`, `-0`)
- **Output control**: 4 (`-q`, `-v`, `-V`, `-u`)
- **Swapping**: 3 (`-0`, `-2`, `-3`)
- **Advanced**: 2 (`-e`, `-l`, `-m`)

### dealer3 (Our Implementation)
- **Total switches**: 6
- **Implemented from original**: 3/13 (23%)
- **Our enhancements**: 3 (`-f`, `-d`, `-v` for vulnerability)
- **Missing from original**: 10 switches
- **Conflict**: `-v` means vulnerability instead of verbose

### DealerV2_4 (Thorvald Aagaard)
- **Total switches**: 29+
- **From original**: 7
- **New features**: 22+
- **Focus areas**:
  - Double-dummy analysis (DDS integration)
  - Export formats (CSV, RP zrd, DL52)
  - Advanced evaluation (OPC, Par)
  - Scripting support

---

## Feature Comparison by Category

### 1. Core Generation Features
| Feature | dealer.exe | dealer3 | DealerV2_4 |
|---------|------------|---------|------------|
| Produce mode | ✅ | ✅ | ✅ |
| Generate mode | ✅ | ✅ | ✅ |
| Seeded RNG | ✅ | ✅ | ✅ |
| Predeal (input) | ✅ | ✅ | ✅ |
| Predeal (switch) | ❌ | ❌ | ✅ |

### 2. Output Formats
| Format | dealer.exe | dealer3 | DealerV2_4 |
|--------|------------|---------|------------|
| PrintOneLine | ✅ | ✅ | ✅ |
| PrintAll | ✅ | ✅ | ✅ |
| PrintEW | ✅ | ✅ | ✅ |
| PrintPBN | ✅ | ✅ | ✅ |
| PrintCompact | ✅ | ✅ | ✅ |
| CSV export | ❌ | ❌ | ✅ |
| RP zrd format | ❌ | ❌ | ✅ |
| DL52 format | ❌ | ❌ | ✅ |

### 3. Analysis Features
| Feature | dealer.exe | dealer3 | DealerV2_4 |
|---------|------------|---------|------------|
| Basic eval (HCP, shape, etc) | ✅ | ✅ | ✅ |
| Double-dummy (tricks) | ❌ | ❌ | ✅ (via DDS) |
| Par calculation | ❌ | ❌ | ✅ |
| OPC evaluation | ❌ | ❌ | ✅ |

### 4. Performance Features
| Feature | dealer.exe | dealer3 | DealerV2_4 |
|---------|------------|---------|------------|
| Progress meter | ✅ | ❌ | ✅ |
| Multi-threading | ❌ | ❌ | ✅ (1-9 threads) |
| Library mode | ✅ | ❌ | ✅ |

---

## Priority Implementation Recommendations

### High Priority (Should Implement)
1. **`-V/--version`**: Print version and exit (standard practice)
2. **`--verbose`**: Toggle statistics output (rename `-v` conflict)
3. **`-m/--progress`**: Progress meter for long runs
4. **`-N/E/S/W`**: Predeal via command line (V2_4 feature)

### Medium Priority (Nice to Have)
1. **`-C/--csv`**: CSV export for analytics
2. **`-q/--quiet`**: Suppress normal output
3. **`-T/--title`**: Add title/metadata to output

### Low Priority (Advanced Features)
1. **DDS integration**: Double-dummy solver (requires external library)
2. **Multi-threading**: Parallel generation (`-R`)
3. **Export formats**: RP zrd, DL52 formats
4. **Script parameters**: `-0` through `-9` for scripting

---

## Conflict Resolution

### The `-v` Conflict

**Problem**:
- dealer.exe uses `-v` for verbose (toggle stats)
- We use `-v` for vulnerability setting
- DealerV2_4 uses `-v` for verbose, `-P` for vulnerability

**Our Current Situation**:
- We always print stats (verbose mode is always on)
- We use `-v` for vulnerability (more useful for PBN output)

**Recommended Solution**:
1. Keep `-v/--vulnerable` for vulnerability (our enhancement)
2. Add `--verbose` (long form only) to optionally suppress stats
3. Add `--quiet` to suppress output (matches `-q` behavior)
4. Document the difference from dealer.exe

**Alternative Solution** (More Compatible):
1. Change `-v` to match dealer.exe (verbose/stats toggle)
2. Use `-V` for vulnerability (capitalized)
3. Add `--version` for version info

---

## Unique Features by Implementation

### dealer.exe Only
- 2-way and 3-way swapping modes (`-2`, `-3`)
- Exhaust mode (`-e`)
- Upper/lowercase toggle (`-u`)

### dealer3 (Ours) Only
- Explicit format selection (`-f/--format`)
- Dealer position flag (`-d/--dealer`)
- Vulnerability flag (`-v/--vulnerable`)

### DealerV2_4 Only
- Double-dummy analysis (DDS integration)
- CSV/RP zrd/DL52 export formats
- Multi-threading support
- OPC evaluation
- Par calculation
- Predeal via command-line switches
- Script parameters (`$0`-`$9`)
- Server mode integration

---

## Recommendations for dealer3

### Immediate (High Value, Low Effort)
1. Add `--version` flag
2. Rename `-v` handling to avoid confusion
3. Add progress meter (`-m`)
4. Add help text improvements

### Near-term (Medium Value, Medium Effort)
1. CSV export capability
2. Predeal via switches (`-N/E/S/W`)
3. Quiet mode (`-q`)

### Long-term (High Value, High Effort)
1. DDS integration for double-dummy analysis
2. Multi-threading support
3. Additional export formats
4. Server mode for integration

---

## Compatibility Matrix

| Feature | Works with dealer.exe files | Works with V2_4 files |
|---------|----------------------------|---------------------|
| Basic constraints | ✅ Yes | ✅ Yes |
| Predeal syntax | ✅ Yes | ✅ Yes |
| Action blocks | ✅ Yes | ✅ Yes |
| Averages/Frequency | ✅ Yes | ✅ Yes |
| DDS functions | ❌ N/A | ❌ Would need implementation |
| OPC functions | ❌ N/A | ❌ Would need implementation |
| Script params | ❌ N/A | ❌ Would need implementation |

