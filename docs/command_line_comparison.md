# Command-Line Switch Comparison: dealer.exe vs Our Implementation vs DealerV2_4

## Overview

This document compares command-line switches across three implementations:
1. **dealer.exe** (Original Thomas Andrews version)
2. **Our Rust Implementation** (dealer3 v0.2.0)
3. **DealerV2_4** (Thorvald Aagaard's enhanced version)

**Last Updated**: January 2026

---

## Complete Switch Comparison Table

| Switch | dealer.exe | dealer3 (ours) | DealerV2_4 | Status | Notes |
|--------|------------|----------------|------------|--------|-------|
| **Core Generation** |
| `INPUT_FILE` | ✅ Positional arg | ✅ Same | ✅ Same | **IMPLEMENTED** | Input file as first argument |
| `-p N` | Produce N hands (default 40) | ✅ Same | ✅ Same | **IMPLEMENTED** | Core feature |
| `-g N` | Generate N hands (default 10M) | ✅ Same | ✅ Same | **IMPLEMENTED** | Can combine with `-p` |
| `-s N` | Random seed | ✅ Same | ✅ Same | **IMPLEMENTED** | Core feature |
| **Output Control** |
| `-q` | Suppress deal output | ✅ Same | ✅ PBN Quiet mode | **IMPLEMENTED** | Only show stats |
| `-v` | Verbose (show stats) | ✅ Same | ✅ Toggle EOJ stats | **IMPLEMENTED** | Compatible |
| `-V` | Version info | ✅ Same | ✅ Show version and exit | **IMPLEMENTED** | Standard practice |
| `-h` | Help | ✅ Auto (clap) | ✅ Help | **IMPLEMENTED** | Standard |
| **Format/Display** |
| `-f FORMAT` | ❌ Not in original | ✅ Format selection | ❌ Not in V2_4 | **IMPLEMENTED** | Our enhancement |
| `-u` | Upper/lowercase AKQJT | ❌ Not implemented | ❌ Not in V2_4 | Low | Cosmetic |
| `-m` | Progress meter | ✅ Same | ✅ Progress meter | **IMPLEMENTED** | Shows every 10k hands |
| **Predeal** |
| `-N/E/S/W CARDS` | ❌ Not in original | ✅ Same as V2_4 | ✅ Compass predeal | **IMPLEMENTED** | Format: S8743,HA9,D642,CQT64 |
| predeal keyword | ✅ In input file | ✅ In input file | ✅ In input file | **IMPLEMENTED** | We support via input |
| **Position/Vulnerability** |
| `-d POS` | ❌ Not in original | ✅ Dealer position | ❌ Not in V2_4 | **IMPLEMENTED** | Our enhancement |
| `--vulnerable VULN` | ❌ Not in original | ✅ Vulnerability | ❌ Not in V2_4 | **IMPLEMENTED** | Our enhancement (long form) |
| `-P N` | ❌ Not in original | ❌ Not implemented | ✅ Vulnerability for Par (0-3) | Low | V2_4 only |
| **Metadata** |
| `-T "text"` | ❌ Not in original | ✅ Title metadata | ✅ Title in quotes | **IMPLEMENTED** | For PBN output |
| `--license` | ❌ Not in original | ✅ Show license | ❌ Not in V2_4 | **IMPLEMENTED** | Our addition |
| `--credits` | ❌ Not in original | ✅ Show credits | ❌ Not in V2_4 | **IMPLEMENTED** | Our addition |
| **Export/Reporting** |
| `-C FILE` | ❌ Not in original | ✅ CSV output | ✅ CSV Report filename | **IMPLEMENTED** | Supports append/write modes |
| `-X FILE` | ❌ Not in original | ❌ Not implemented | ✅ Export predeal holdings | Low | V2_4 feature |
| `-Z FILE` | ❌ Not in original | ❌ Not implemented | ✅ RP zrd format export | Low | V2_4 format |
| **Swapping Modes** |
| `-0` | No swapping (default) | ✅ Default behavior | ✅ Same | **IMPLEMENTED** | Default |
| `-2` | 2-way swap (E/W) | ❌ Not implemented | ❌ Not in V2_4 | Low | Not compatible with predeal |
| `-3` | 3-way swap (E/W/S) | ❌ Not implemented | ❌ Not in V2_4 | Low | Not compatible with predeal |
| `-x MODE` | ❌ Not in original | ❌ Not implemented | ✅ eXchange mode 2\|3 | Low | V2_4 adds this |
| **Double-Dummy Analysis** (V2_4 only) |
| `-M MODE` | ❌ Not in original | ❌ Not implemented | ✅ DDS mode 1\|2 | Low | Requires DDS library |
| `-R N` | ❌ Not in original | ❌ Not implemented | ✅ Resources/Threads 1-9 | Low | Requires DDS library |
| **Library/Advanced** (V2_4 only) |
| `-l PATH` | Read from library.dat | ❌ Not implemented | ✅ DL52 format export | Low | Different meaning in V2_4 |
| `-L PATH` | ❌ Not in original | ❌ Not implemented | ✅ RP Library source path | Low | V2_4 advanced feature |
| `-U PATH` | ❌ Not in original | ❌ Not implemented | ✅ DealerServer pathname | Low | V2_4 server mode |
| **OPC** (V2_4 only) |
| `-O POS` | ❌ Not in original | ❌ Not implemented | ✅ OPC evaluation Opener | Low | V2_4 specific |
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

### dealer3 v0.2.0 (Our Implementation)
- **Total switches**: 18
- **Implemented from dealer.exe**: 9/13 (69%)
  - ✅ `-p`, `-g`, `-s`, `-h`, `-q`, `-v`, `-V`, `-m`, `-0`
- **Implemented from DealerV2_4**: 3
  - ✅ `-N/E/S/W`, `-C`, `-T`
- **Our unique enhancements**: 4
  - ✅ `-f` (format), `-d` (dealer), `--vulnerable`, `--license`, `--credits`
- **Not implemented (low priority)**: 4 switches (`-2`, `-3`, `-u`, `-e`)

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
| Progress meter | ✅ | ✅ | ✅ |
| Multi-threading | ❌ | ❌ | ✅ (1-9 threads) |
| Library mode | ✅ | ❌ | ✅ |

---

## Implementation Status (v0.2.0)

### ✅ Fully Implemented
All high-priority features from dealer.exe have been implemented:

1. **`-V/--version`**: Print version and exit ✅
2. **`-v/--verbose`**: Toggle statistics output ✅
3. **`-m/--progress`**: Progress meter for long runs ✅
4. **`-N/E/S/W`**: Predeal via command line (V2_4 feature) ✅
5. **`-C/--csv`**: CSV export for analytics ✅
6. **`-q/--quiet`**: Suppress normal output ✅
7. **`-T/--title`**: Add title/metadata to output ✅
8. **`INPUT_FILE`**: Positional argument for input file ✅

### Remaining Low Priority (Not Planned)
1. **DDS integration**: Double-dummy solver (we have separate `solver` binary)
2. **Multi-threading**: Parallel generation (`-R`)
3. **Export formats**: RP zrd, DL52 formats
4. **Script parameters**: `-0` through `-9` for scripting
5. **Swapping modes**: `-2`, `-3` (not compatible with predeal)

---

## Compatibility Notes

### The `-v` Flag
- **dealer.exe**: `-v` for verbose (toggle stats)
- **dealer3**: `-v` for verbose (same behavior) ✅
- **Vulnerability**: Use `--vulnerable` (long form only)

This maintains compatibility with dealer.exe scripts while providing vulnerability support via a distinct flag.

---

## Unique Features by Implementation

### dealer.exe Only
- 2-way and 3-way swapping modes (`-2`, `-3`)
- Exhaust mode (`-e`)
- Upper/lowercase toggle (`-u`)

### dealer3 (Ours) Only
- Explicit format selection (`-f/--format`)
- Dealer position flag (`-d/--dealer`)
- Vulnerability flag (`--vulnerable`)
- License and credits info (`--license`, `--credits`)
- **Separate `solver` binary** for double-dummy analysis (pure Rust, no external dependencies)

### DealerV2_4 Only
- Double-dummy analysis (DDS integration)
- RP zrd/DL52 export formats
- Multi-threading support
- OPC evaluation
- Par calculation
- Script parameters (`$0`-`$9`)
- Server mode integration

---

## Compatibility Matrix

| Feature | Works with dealer.exe files | Works with V2_4 files |
|---------|----------------------------|---------------------|
| Basic constraints | ✅ Yes | ✅ Yes |
| Predeal syntax | ✅ Yes | ✅ Yes |
| Action blocks | ✅ Yes | ✅ Yes |
| Averages/Frequency | ✅ Yes | ✅ Yes |
| CSV reports | ✅ Yes | ✅ Yes |
| Command-line switches | ✅ 69% compatible | ✅ Key switches supported |
| DDS functions | ❌ N/A | ❌ Use separate `solver` binary |
| OPC functions | ❌ N/A | ❌ Would need implementation |
| Script params | ❌ N/A | ❌ Would need implementation |

---

## Quick Reference: dealer3 v0.2.0 CLI

```
dealer [OPTIONS] [INPUT_FILE]

Arguments:
  [INPUT_FILE]  Input file (reads from stdin if not provided)

Core Options:
  -p, --produce <N>      Produce N matching hands (default: 40)
  -g, --generate <N>     Generate up to N total hands (default: 10000000)
  -s, --seed <N>         Random seed (default: time-based)

Output Options:
  -f, --format <FMT>     Output format (oneline, all, ew, pbn, compact)
  -q, --quiet            Suppress deal output, show only stats
  -v, --verbose          Show statistics at end of run
  -m, --progress         Show progress meter during generation

Predeal Options:
  -N, --north <CARDS>    Predeal cards to North (e.g., S8743,HA9,D642,CQT64)
  -E, --east <CARDS>     Predeal cards to East
  -S, --south <CARDS>    Predeal cards to South
  -W, --west <CARDS>     Predeal cards to West

PBN Options:
  -d, --dealer <POS>     Dealer position (N/E/S/W)
  --vulnerable <VULN>    Vulnerability (None/NS/EW/All)
  -T, --title <TEXT>     Title metadata

Export Options:
  -C, --CSV <FILE>       CSV output file (append mode, use w:file for write)

Info Options:
  -V, --version          Print version and exit
  --license              Print license and exit
  --credits              Print credits and exit
  -h, --help             Print help
```
