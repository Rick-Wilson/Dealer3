#!/bin/bash
#
# dealer-compare.sh - Compare dealer3 (Rust) vs dealer.exe (Windows) output
#
# Usage:
#   dealer-compare.sh [options] [dealer-options] [input-file]
#   echo "condition" | dealer-compare.sh [options] [dealer-options]
#
# Options:
#   -t, --timeout SECS   Job timeout in seconds (default: 10)
#   -r, --rust PATH      Path to Rust dealer binary (default: target/release/dealer)
#   -o, --output         Show raw output from both runs after comparison
#   --no-pretest         Skip the quick pretest (pretest is on by default)
#   --pretest-only       Run only the pretest (-p 2), skip the full comparison
#   -h, --help           Show this help message
#
# Examples:
#   dealer-compare.sh -p 10 -s 1 test.dlr
#   echo "hcp(north) >= 20" | dealer-compare.sh -p 10 -s 1
#   dealer-compare.sh -r ~/.cargo/bin/dealer -p 10 -s 1 test.dlr
#   dealer-compare.sh -o -p 10 -s 1 test.dlr
#   dealer-compare.sh --no-pretest -p 1000 -s 1 test.dlr
#   dealer-compare.sh --pretest-only -s 1 test.dlr
#

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WIN_DEALER="$SCRIPT_DIR/win-dealer.sh"

TIMEOUT=10
DEALER_ARGS=()
INPUT_FILE=""
SHOW_OUTPUT=false
DEALER3=""
RUN_PRETEST=true
PRETEST_ONLY=false

show_help() {
    sed -n '2,24p' "$0" | sed 's/^# \?//'
    exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -r|--rust)
            DEALER3="$2"
            shift 2
            ;;
        -o|--output)
            SHOW_OUTPUT=true
            shift
            ;;
        --no-pretest)
            RUN_PRETEST=false
            shift
            ;;
        --pretest-only)
            PRETEST_ONLY=true
            shift
            ;;
        -h|--help)
            show_help
            ;;
        -*)
            # Dealer option - collect it and its argument if needed
            case "$1" in
                -p|-g|-s|-f|-d|--produce|--generate|--seed|--format|--dealer|--vulnerable)
                    DEALER_ARGS+=("$1" "$2")
                    shift 2
                    ;;
                *)
                    DEALER_ARGS+=("$1")
                    shift
                    ;;
            esac
            ;;
        *)
            # Positional argument - treat as input file
            INPUT_FILE="$1"
            shift
            ;;
    esac
done

# Find dealer3 binary if not specified (prefer development build over installed version)
if [[ -z "$DEALER3" ]]; then
    if [[ -x "$SCRIPT_DIR/../target/release/dealer" ]]; then
        DEALER3="$SCRIPT_DIR/../target/release/dealer"
    elif command -v dealer &>/dev/null; then
        DEALER3="dealer"
    else
        echo "Error: dealer3 binary not found in target/release or PATH" >&2
        exit 1
    fi
fi

# Verify dealer3 exists
if [[ ! -x "$DEALER3" ]]; then
    echo "Error: dealer3 binary not found or not executable: $DEALER3" >&2
    exit 1
fi

# Create temp files for output
RUST_OUT=$(mktemp)
WIN_OUT=$(mktemp)
trap "rm -f $RUST_OUT $WIN_OUT" EXIT

# Extract deal lines (exclude stats lines and PBN metadata that differs between environments)
# Normalizes: CRLF, trailing whitespace, Event field (paths differ), Date field (runtime difference)
extract_deals() {
    tr -d '\r' < "$1" | \
        grep -v -E '^(Generated|Produced|Initial|Time|$)' | \
        grep -v -E '^\[Event ' | \
        grep -v -E '^\[Date ' | \
        sed 's/[[:space:]]*$//'
}

# Extract statistics
extract_stat() {
    local file="$1"
    local pattern="$2"
    grep -i "$pattern" "$file" 2>/dev/null | grep -oE '[0-9]+' | head -1
}

# Run comparison with given args, returns 0 on match, 1 on mismatch
# Arguments: label args...
run_comparison() {
    local label="$1"
    shift
    local args=("$@")

    # Run both dealers (pass timeout to both, strip comment bug from Windows output)
    if [[ -n "$INPUT_FILE" ]]; then
        "$DEALER3" -t "$TIMEOUT" "${args[@]}" "$INPUT_FILE" > "$RUST_OUT" 2>&1
        "$WIN_DEALER" -t "$TIMEOUT" --strip-comment-bug "${args[@]}" "$INPUT_FILE" > "$WIN_OUT" 2>&1
    else
        echo "$INPUT" | "$DEALER3" -t "$TIMEOUT" "${args[@]}" > "$RUST_OUT" 2>&1
        echo "$INPUT" | "$WIN_DEALER" -t "$TIMEOUT" --strip-comment-bug "${args[@]}" > "$WIN_OUT" 2>&1
    fi

    local rust_deals=$(extract_deals "$RUST_OUT")
    local win_deals=$(extract_deals "$WIN_OUT")
    local rust_produced=$(extract_stat "$RUST_OUT" "Produced")
    local win_produced=$(extract_stat "$WIN_OUT" "Produced")
    local rust_generated=$(extract_stat "$RUST_OUT" "Generated")
    local win_generated=$(extract_stat "$WIN_OUT" "Generated")

    if [[ "$rust_deals" == "$win_deals" ]] && [[ "$rust_produced" == "$win_produced" ]] && [[ "$rust_generated" == "$win_generated" ]]; then
        return 0
    else
        return 1
    fi
}

# Save stdin if needed (before pretest consumes it)
INPUT=""
if [[ -z "$INPUT_FILE" ]] && [[ ! -t 0 ]]; then
    INPUT=$(cat)
fi

# Run pretest if enabled (quick test with -p 2)
# Note: We use -p 2 instead of -p 1 because dealer.exe has a bug where PBN output
# toggles verbose each deal, so odd counts suppress stats while even counts show them.
# Using -p 2 ensures stats are shown for comparison.
if [[ "$RUN_PRETEST" == true ]] || [[ "$PRETEST_ONLY" == true ]]; then
    echo "=== Pretest (quick -p 2 check) ==="

    # Build pretest args: replace any -p value with -p 2, or add -p 2 if not present
    PRETEST_ARGS=()
    FOUND_P=false
    i=0
    while [[ $i -lt ${#DEALER_ARGS[@]} ]]; do
        if [[ "${DEALER_ARGS[$i]}" == "-p" ]] || [[ "${DEALER_ARGS[$i]}" == "--produce" ]]; then
            PRETEST_ARGS+=("-p" "2")
            ((i+=2))
            FOUND_P=true
        else
            PRETEST_ARGS+=("${DEALER_ARGS[$i]}")
            ((i++))
        fi
    done
    if [[ "$FOUND_P" == false ]]; then
        PRETEST_ARGS+=("-p" "2")
    fi

    if run_comparison "Pretest" "${PRETEST_ARGS[@]}"; then
        echo "Pretest:     ✅ PASS"
        echo ""
    else
        echo "Pretest:     ❌ FAIL"
        echo ""
        echo "Pretest failed - skipping full test."
        echo "Run with --no-pretest to force full test, or -o to see output."

        if [[ "$SHOW_OUTPUT" == true ]]; then
            echo ""
            echo "=== Rust Output (pretest) ==="
            cat "$RUST_OUT"
            echo ""
            echo "=== Windows Output (pretest) ==="
            tr -d '\r' < "$WIN_OUT"
        fi
        exit 1
    fi
fi

# Exit early if pretest-only mode
if [[ "$PRETEST_ONLY" == true ]]; then
    exit 0
fi

# Run full test
echo "=== Full Comparison ==="
echo "Using: $DEALER3"
echo ""

# Run both dealers with full args (pass timeout to both, strip comment bug from Windows output)
if [[ -n "$INPUT_FILE" ]]; then
    "$DEALER3" -t "$TIMEOUT" "${DEALER_ARGS[@]}" "$INPUT_FILE" > "$RUST_OUT" 2>&1
    RUST_EXIT=$?
    "$WIN_DEALER" -t "$TIMEOUT" --strip-comment-bug "${DEALER_ARGS[@]}" "$INPUT_FILE" > "$WIN_OUT" 2>&1
    WIN_EXIT=$?
else
    echo "$INPUT" | "$DEALER3" -t "$TIMEOUT" "${DEALER_ARGS[@]}" > "$RUST_OUT" 2>&1
    RUST_EXIT=$?
    echo "$INPUT" | "$WIN_DEALER" -t "$TIMEOUT" --strip-comment-bug "${DEALER_ARGS[@]}" > "$WIN_OUT" 2>&1
    WIN_EXIT=$?
fi

# Compare results
RUST_DEALS=$(extract_deals "$RUST_OUT")
WIN_DEALS=$(extract_deals "$WIN_OUT")

RUST_PRODUCED=$(extract_stat "$RUST_OUT" "Produced")
WIN_PRODUCED=$(extract_stat "$WIN_OUT" "Produced")

RUST_GENERATED=$(extract_stat "$RUST_OUT" "Generated")
WIN_GENERATED=$(extract_stat "$WIN_OUT" "Generated")

# Extract time (handle decimal)
RUST_TIME=$(grep -i "Time" "$RUST_OUT" 2>/dev/null | grep -oE '[0-9]+\.[0-9]+' | head -1)
WIN_TIME=$(grep -i "Time" "$WIN_OUT" 2>/dev/null | grep -oE '[0-9]+\.[0-9]+' | head -1)

# Deals comparison (text diff)
if [[ "$RUST_DEALS" == "$WIN_DEALS" ]]; then
    DEAL_COUNT=$(echo "$RUST_DEALS" | grep -c '^' 2>/dev/null || echo 0)
    echo "Deals:       ✅ MATCH ($DEAL_COUNT lines)"
else
    echo "Deals:       ❌ FAIL"
    echo "  Rust lines: $(echo "$RUST_DEALS" | wc -l | tr -d ' ')"
    echo "  Win lines:  $(echo "$WIN_DEALS" | wc -l | tr -d ' ')"
fi

# Produced comparison
if [[ "$RUST_PRODUCED" == "$WIN_PRODUCED" ]]; then
    echo "Produced:    ✅ MATCH ($RUST_PRODUCED)"
else
    echo "Produced:    ❌ FAIL (Rust: $RUST_PRODUCED, Win: $WIN_PRODUCED)"
fi

# Generated comparison
if [[ "$RUST_GENERATED" == "$WIN_GENERATED" ]]; then
    echo "Generated:   ✅ MATCH ($RUST_GENERATED)"
else
    echo "Generated:   ❌ FAIL (Rust: $RUST_GENERATED, Win: $WIN_GENERATED)"
fi

# Time comparison - show times and warn if Rust is significantly slower (>1s difference)
TIME_WARNING=""
if [[ -n "$RUST_TIME" ]] && [[ -n "$WIN_TIME" ]]; then
    # Calculate difference using awk (Rust - Win)
    TIME_DIFF=$(awk "BEGIN {printf \"%.3f\", $RUST_TIME - $WIN_TIME}")
    if awk "BEGIN {exit !($TIME_DIFF > 1.0)}"; then
        TIME_WARNING=" ⚠️  Rust is ${TIME_DIFF}s slower"
    fi
fi
echo "Time:        Rust: ${RUST_TIME:-N/A}s, Win: ${WIN_TIME:-N/A}s${TIME_WARNING}"

# Exit codes
if [[ $RUST_EXIT -ne 0 ]] || [[ $WIN_EXIT -ne 0 ]]; then
    echo ""
    echo "Exit codes:  Rust: $RUST_EXIT, Win: $WIN_EXIT"
fi

# Overall result
echo ""
if [[ "$RUST_DEALS" == "$WIN_DEALS" ]] && [[ "$RUST_PRODUCED" == "$WIN_PRODUCED" ]] && [[ "$RUST_GENERATED" == "$WIN_GENERATED" ]]; then
    echo "Overall:     ✅ PASS"
    RESULT=0
else
    echo "Overall:     ❌ FAIL"
    RESULT=1
fi

# Show raw output if requested
if [[ "$SHOW_OUTPUT" == true ]]; then
    echo ""
    echo "=== Rust Output ==="
    cat "$RUST_OUT"
    echo ""
    echo "=== Windows Output ==="
    tr -d '\r' < "$WIN_OUT"
fi

exit $RESULT
