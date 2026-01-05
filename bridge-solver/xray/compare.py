#!/usr/bin/env python3
"""
X-Ray Comparison Test Script

Compares Rust and C++ solver outputs for a given deal.

Usage:
    ./compare.py <input-file> [-l LEADER] [-s STRAIN] [--flags FLAGS]

Examples:
    ./compare.py deals/quick_test_8.txt
    ./compare.py deals/quick_test_8.txt -l W -s N
    ./compare.py deals/quick_test_8.txt --leader N --strain N
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path


# Paths (relative to this script's directory)
SCRIPT_DIR = Path(__file__).parent.resolve()
BRIDGE_SOLVER_CRATE = SCRIPT_DIR.parent  # bridge-solver/ (Rust crate)
WORKSPACE_DIR = BRIDGE_SOLVER_CRATE.parent  # dealer3/ (workspace root)
GITHUB_DIR = WORKSPACE_DIR.parent  # ~/Developer/GitHub/
RUNS_DIR = SCRIPT_DIR / "runs"  # Results folder (gitignored)

# C++ solver executables (in separate repos)
CPP_SOLVER_XRAY_REPO = GITHUB_DIR / "bridge-solver-xray"
CPP_SOLVER_UPSTREAM_REPO = GITHUB_DIR / "bridge-solver-upstream"
CPP_SOLVER = CPP_SOLVER_UPSTREAM_REPO / "solver"  # Regular C++ solver
CPP_SOLVER_XRAY = CPP_SOLVER_XRAY_REPO / "xray" / "solver-xray"  # Instrumented C++ solver

# Rust solver executable (workspace builds to workspace root target/)
RUST_SOLVER = WORKSPACE_DIR / "target" / "release" / "solver"


def parse_args():
    parser = argparse.ArgumentParser(
        description="Compare Rust and C++ solver outputs"
    )
    parser.add_argument(
        "input_file",
        help="Input deal file (in C++ solver format)"
    )
    parser.add_argument(
        "-l", "--leader",
        choices=["W", "N", "E", "S"],
        help="Leader (W/N/E/S). If omitted, tests all leaders."
    )
    parser.add_argument(
        "-s", "--strain",
        choices=["N", "S", "H", "D", "C"],
        help="Trump strain (N=NT, S/H/D/C). If omitted, tests all strains."
    )
    parser.add_argument(
        "--flags",
        default="",
        help="Additional flags (TBD)"
    )
    parser.add_argument(
        "--build",
        action="store_true",
        help="Build Rust solver before running"
    )
    parser.add_argument(
        "-t", "--timeout",
        type=int,
        default=10,
        help="Timeout in seconds for each solver (default: 10)"
    )
    parser.add_argument(
        "-X", "--xray",
        type=int,
        default=0,
        help="Enable xray tracing for N iterations (compares search traces)"
    )
    parser.add_argument(
        "--xray-full",
        action="store_true",
        help="Capture full xray stderr output (not just XRAY lines)"
    )
    parser.add_argument(
        "-P", "--no-pruning",
        action="store_true",
        help="Disable fast/slow tricks pruning (for debugging search paths)"
    )
    parser.add_argument(
        "-T", "--no-tt",
        action="store_true",
        help="Disable transposition table (for debugging search paths)"
    )
    parser.add_argument(
        "-R", "--no-rank-skip",
        action="store_true",
        help="Disable min_relevant_ranks optimization in C++ (for debugging search paths)"
    )
    parser.add_argument(
        "--no-perf",
        action="store_true",
        help="Disable performance output (match upstream solver output exactly)"
    )
    return parser.parse_args()


def build_rust_solver():
    """Build the Rust solver in release mode."""
    print("Building Rust solver...")
    result = subprocess.run(
        ["cargo", "build", "--bin", "solver", "--release"],
        cwd=WORKSPACE_DIR,
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        print(f"Build failed:\n{result.stderr}")
        sys.exit(1)
    print("Build complete.")


def count_tricks_in_deal(input_file: Path) -> int:
    """Count the number of cards per hand in the deal file."""
    with open(input_file, "r") as f:
        lines = f.read().strip().split("\n")

    if len(lines) < 3:
        return 13  # Default to full deal

    # Parse North hand (first line)
    north_line = lines[0].strip()
    # Count cards (excluding spaces and dashes for voids)
    card_count = 0
    for c in north_line:
        if c.upper() in "AKQJT98765432":
            card_count += 1

    return card_count


def create_temp_input(input_file: Path, leader: str | None, strain: str | None) -> Path:
    """Create a temporary input file with optional leader/strain appended."""
    with open(input_file, "r") as f:
        content = f.read()

    # Remove any existing strain/leader lines (lines 4 and 5 if they exist)
    lines = content.strip().split("\n")
    # Keep only the first 3 lines (N, W E, S)
    base_lines = lines[:3]

    # Add strain if specified
    if strain:
        base_lines.append(strain)

    # Add leader if specified
    if leader:
        if not strain:
            # Need strain line first (even if empty? No, C++ expects strain before leader)
            # Actually looking at C++ behavior, leader requires strain
            print("Warning: Leader specified without strain. Adding 'N' (NT) as default.")
            base_lines.append("N")
        base_lines.append(leader)

    temp_content = "\n".join(base_lines) + "\n"

    # Create temp file
    fd, temp_path = tempfile.mkstemp(suffix=".txt", prefix="xray_")
    with os.fdopen(fd, "w") as f:
        f.write(temp_content)

    return Path(temp_path)


def run_solver(solver_path: Path, input_file: Path, name: str, timeout: int, xray_iterations: int = 0, no_pruning: bool = False, no_tt: bool = False, no_rank_skip: bool = False, show_perf: bool = True) -> tuple[str, float, bool, list[str], list[str]]:
    """Run a solver and return (output, elapsed_time, timed_out, xray_lines, equiv_lines)."""
    import time

    if not solver_path.exists():
        return f"ERROR: Solver not found at {solver_path}", 0.0, False, [], []

    cmd = [str(solver_path), "-f", str(input_file)]
    if xray_iterations > 0:
        cmd.extend(["-X", str(xray_iterations)])
    if no_pruning:
        cmd.append("-P")
    if no_tt:
        cmd.append("-T")
    if no_rank_skip:
        cmd.append("-R")
    if show_perf:
        cmd.append("-V")

    start = time.time()
    timed_out = False
    xray_lines = []
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout
        )
        elapsed = time.time() - start

        # Combine stdout and stderr (C++ solver outputs [PERF] to stderr)
        output = result.stdout
        equiv_lines = []
        if result.stderr:
            # Extract XRAY and EQUIV lines from stderr
            for line in result.stderr.split("\n"):
                if line.startswith("XRAY "):
                    xray_lines.append(line)
                elif line.startswith("EQUIV:"):
                    equiv_lines.append(line)
            output += result.stderr
    except subprocess.TimeoutExpired:
        elapsed = time.time() - start
        output = f"TIMEOUT after {timeout}s"
        timed_out = True
        equiv_lines = []

    return output, elapsed, timed_out, xray_lines, equiv_lines


def parse_rust_results(output: str, num_tricks: int) -> dict:
    """Parse Rust solver output to extract results.

    Rust outputs NS tricks directly in W, E, N, S order (matching C++).
    """
    results = {
        "raw": output,
        "tricks": {},  # strain -> {leader -> tricks}
        "errors": []
    }

    lines = output.strip().split("\n")
    for line in lines:
        # Skip hand diagram lines and [PERF] lines
        if line.startswith("[PERF]") or "♠" in line or "♥" in line or "♦" in line or "♣" in line:
            continue

        # Single leader: "N  1  0.00 s 0.0 M"
        single_match = re.match(r"^([NSHDC])\s+(\d+)\s+[\d.]+\s*s", line)
        if single_match:
            strain = single_match.group(1)
            tricks = int(single_match.group(2))
            results["tricks"].setdefault(strain, {})["single"] = tricks
            continue

        # All leaders: "N  1  1  3  3  0.00 s 0.0 M" (W E N S order, matching C++)
        # Rust now outputs declaring-side tricks (same as C++): W/E = NS tricks, N/S = EW tricks
        # So we apply the same inverse transform as C++ for N/S leads
        multi_match = re.match(r"^([NSHDC])\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+[\d.]+\s*s", line)
        if multi_match:
            strain = multi_match.group(1)
            results["tricks"].setdefault(strain, {})
            w_raw = int(multi_match.group(2))
            e_raw = int(multi_match.group(3))
            n_raw = int(multi_match.group(4))
            s_raw = int(multi_match.group(5))
            # W (EW leads): raw = NS tricks
            results["tricks"][strain]["W"] = w_raw
            # E (EW leads): raw = NS tricks
            results["tricks"][strain]["E"] = e_raw
            # N (NS leads): raw = EW tricks, NS = total - raw
            results["tricks"][strain]["N"] = num_tricks - n_raw
            # S (NS leads): raw = EW tricks, NS = total - raw
            results["tricks"][strain]["S"] = num_tricks - s_raw
            continue

    # Extract iteration counts from [PERF] lines
    results["total_iterations"] = 0
    for line in lines:
        if line.startswith("[PERF"):
            match = re.search(r"iterations=(\d+)", line)
            if match:
                results["total_iterations"] += int(match.group(1))

    return results


def parse_cpp_results(output: str, num_tricks: int) -> dict:
    """Parse C++ solver output to extract results.

    C++ solver returns tricks for the NON-LEADER's side:
    - If EW leads (W or E): result = NS tricks directly
    - If NS leads (N or S): result = EW tricks, so NS = num_tricks - result

    When outputting all leaders, C++ uses W, E, N, S order (EW first, then NS).
    Output can be on a single line or multiple lines with [PERF] interspersed.
    """
    results = {
        "raw": output,
        "tricks": {},  # strain -> {leader -> tricks}
        "errors": []
    }

    lines = output.strip().split("\n")

    for line in lines:
        # Skip hand diagram lines and [PERF] lines
        if line.startswith("[PERF]") or "♠" in line or "♥" in line or "♦" in line or "♣" in line:
            continue

        # Check for single result: "N  1  0.00 s 3696.0 M"
        single_match = re.match(r"^([NSHDC])\s+(\d+)\s+[\d.]+\s*s", line)
        if single_match:
            strain = single_match.group(1)
            tricks = int(single_match.group(2))
            results["tricks"].setdefault(strain, {})["single"] = tricks
            continue

        # Check for all 4 leaders on one line: "N  1  1  5  5  0.00 s 3728.0 M"
        # C++ outputs in W, E, N, S order
        multi_match = re.match(r"^([NSHDC])\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+[\d.]+\s*s", line)
        if multi_match:
            strain = multi_match.group(1)
            results["tricks"].setdefault(strain, {})
            # C++ order is W, E, N, S and values are raw (non-leader's tricks)
            w_raw = int(multi_match.group(2))
            e_raw = int(multi_match.group(3))
            n_raw = int(multi_match.group(4))
            s_raw = int(multi_match.group(5))
            # W (EW leads): raw = NS tricks
            results["tricks"][strain]["W"] = w_raw
            # E (EW leads): raw = NS tricks
            results["tricks"][strain]["E"] = e_raw
            # N (NS leads): raw = EW tricks, NS = total - raw
            results["tricks"][strain]["N"] = num_tricks - n_raw
            # S (NS leads): raw = EW tricks, NS = total - raw
            results["tricks"][strain]["S"] = num_tricks - s_raw
            continue

    # Extract iteration counts from [PERF] lines
    results["total_iterations"] = 0
    for line in lines:
        if line.startswith("[PERF"):
            match = re.search(r"iterations=(\d+)", line)
            if match:
                results["total_iterations"] += int(match.group(1))

    return results


def compare_xray_traces(rust_xray: list[str], cpp_xray: list[str]) -> dict:
    """Compare xray trace lines from both solvers."""
    comparison = {
        "match": True,
        "first_divergence": None,
        "rust_lines": rust_xray,
        "cpp_lines": cpp_xray,
        "differences": []
    }

    max_lines = max(len(rust_xray), len(cpp_xray))

    for i in range(max_lines):
        rust_line = rust_xray[i] if i < len(rust_xray) else None
        cpp_line = cpp_xray[i] if i < len(cpp_xray) else None

        if rust_line != cpp_line:
            comparison["match"] = False
            if comparison["first_divergence"] is None:
                comparison["first_divergence"] = i + 1
            comparison["differences"].append({
                "line_num": i + 1,
                "rust": rust_line,
                "cpp": cpp_line
            })

    return comparison


def compare_results(rust_results: dict, cpp_results: dict) -> dict:
    """Compare results from both solvers."""
    comparison = {
        "match": True,
        "differences": [],
        "rust_tricks": rust_results["tricks"],
        "cpp_tricks": cpp_results["tricks"],
        "rust_iterations": rust_results.get("total_iterations", 0),
        "cpp_iterations": cpp_results.get("total_iterations", 0),
    }

    # Compare tricks for each strain/leader
    all_strains = set(rust_results["tricks"].keys()) | set(cpp_results["tricks"].keys())

    for strain in sorted(all_strains):
        rust_strain = rust_results["tricks"].get(strain, {})
        cpp_strain = cpp_results["tricks"].get(strain, {})

        all_leaders = set(rust_strain.keys()) | set(cpp_strain.keys())

        for leader in sorted(all_leaders):
            rust_val = rust_strain.get(leader)
            cpp_val = cpp_strain.get(leader)

            if rust_val != cpp_val:
                comparison["match"] = False
                comparison["differences"].append({
                    "strain": strain,
                    "leader": leader,
                    "rust": rust_val,
                    "cpp": cpp_val,
                    "delta": (rust_val - cpp_val) if (rust_val is not None and cpp_val is not None) else None
                })

    return comparison


def get_next_run_number() -> int:
    """Get the next run number by scanning existing folders."""
    if not RUNS_DIR.exists():
        return 1

    max_num = 0
    for folder in RUNS_DIR.iterdir():
        if folder.is_dir():
            # Extract run number from folder name (format: NNNN_...)
            name = folder.name
            if name[:4].isdigit():
                num = int(name[:4])
                max_num = max(max_num, num)

    return max_num + 1


def create_run_folder(input_file: Path, leader: str | None, strain: str | None) -> Path:
    """Create a unique run folder for this test."""
    # Get next run number
    run_num = get_next_run_number()

    # Base name from input file
    base_name = input_file.stem

    # Add strain/leader to name if specified
    if strain:
        base_name += f"_{strain}"
    if leader:
        base_name += f"_{leader}"

    # Format: NNNN_inputfile_strain_leader
    folder_name = f"{run_num:04d}_{base_name}"

    run_folder = RUNS_DIR / folder_name
    run_folder.mkdir(parents=True, exist_ok=True)

    return run_folder


def write_comparison_report(
    run_folder: Path,
    comparison: dict,
    rust_output: str,
    cpp_output: str,
    rust_time: float,
    cpp_time: float,
    rust_timeout: bool,
    cpp_timeout: bool,
    input_file: Path,
    leader: str | None,
    strain: str | None,
    timeout: int,
    xray_comparison: dict | None = None,
    no_pruning: bool = False,
    no_tt: bool = False,
    no_rank_skip: bool = False,
    xray_iterations: int = 0,
    show_perf: bool = True,
):
    """Write the comparison report."""
    report_path = run_folder / "comparison.md"

    with open(report_path, "w") as f:
        f.write("# X-Ray Comparison Report\n\n")

        # Test parameters
        f.write("## Test Parameters\n\n")
        f.write(f"- **Input file**: {input_file.name}\n")
        f.write(f"- **Strain**: {strain or 'all'}\n")
        f.write(f"- **Leader**: {leader or 'all'}\n")
        f.write(f"- **Timeout**: {timeout}s\n")
        if xray_iterations > 0:
            f.write(f"- **X-ray iterations**: {xray_iterations}\n")
        if no_pruning:
            f.write(f"- **No-pruning**: enabled\n")
        if no_tt:
            f.write(f"- **No-TT**: enabled\n")
        if no_rank_skip:
            f.write(f"- **No-rank-skip**: enabled\n")
        f.write(f"- **Timestamp**: {datetime.now().strftime('%Y-%m-%d %I:%M:%S %p')}\n\n")

        # Timeout status
        if rust_timeout or cpp_timeout:
            f.write("## Timeouts\n\n")
            if rust_timeout:
                f.write(f"⏱️ **Rust solver TIMED OUT** after {rust_time:.3f}s\n\n")
            if cpp_timeout:
                f.write(f"⏱️ **C++ solver TIMED OUT** after {cpp_time:.3f}s\n\n")

        # Results summary
        f.write("## Results Summary\n\n")
        if rust_timeout or cpp_timeout:
            f.write("⚠️ **Cannot compare** - one or both solvers timed out\n\n")
        elif comparison["match"]:
            f.write("✅ **Results MATCH**\n\n")
        else:
            f.write("❌ **Results DIFFER**\n\n")
            f.write("### Differences\n\n")
            f.write("| Strain | Leader | Rust | C++ | Delta |\n")
            f.write("|--------|--------|------|-----|-------|\n")
            for diff in comparison["differences"]:
                delta = diff["delta"]
                delta_str = f"{delta:+d}" if delta is not None else "N/A"
                f.write(f"| {diff['strain']} | {diff['leader']} | {diff['rust']} | {diff['cpp']} | {delta_str} |\n")
            f.write("\n")

        # Tricks table
        f.write("### All Results\n\n")
        f.write("| Strain | Leader | Rust | C++ |\n")
        f.write("|--------|--------|------|-----|\n")

        all_strains = set(comparison["rust_tricks"].keys()) | set(comparison["cpp_tricks"].keys())
        for strain_key in ["N", "S", "H", "D", "C"]:
            if strain_key not in all_strains:
                continue
            rust_strain = comparison["rust_tricks"].get(strain_key, {})
            cpp_strain = comparison["cpp_tricks"].get(strain_key, {})

            for leader_key in ["W", "N", "E", "S", "single"]:
                if leader_key not in rust_strain and leader_key not in cpp_strain:
                    continue
                rust_val = rust_strain.get(leader_key, "-")
                cpp_val = cpp_strain.get(leader_key, "-")
                leader_display = leader_key if leader_key != "single" else "(single)"
                f.write(f"| {strain_key} | {leader_display} | {rust_val} | {cpp_val} |\n")
        f.write("\n")

        # Performance
        f.write("## Performance\n\n")
        f.write(f"| Metric | Rust | C++ | Ratio |\n")
        f.write(f"|--------|------|-----|-------|\n")
        f.write(f"| Time | {rust_time:.3f}s | {cpp_time:.3f}s | {rust_time/cpp_time:.1f}x |\n" if cpp_time > 0 else f"| Time | {rust_time:.3f}s | {cpp_time:.3f}s | N/A |\n")

        rust_iters = comparison["rust_iterations"]
        cpp_iters = comparison["cpp_iterations"]
        if not show_perf:
            f.write(f"| Iterations | N/A | N/A | N/A |\n")
        elif cpp_iters > 0:
            f.write(f"| Iterations | {rust_iters:,} | {cpp_iters:,} | {rust_iters/cpp_iters:.1f}x |\n")
        else:
            f.write(f"| Iterations | {rust_iters:,} | {cpp_iters:,} | N/A |\n")
        f.write("\n")

        # X-ray trace comparison
        if xray_comparison is not None:
            f.write("## X-Ray Trace Comparison\n\n")
            if xray_comparison["match"]:
                f.write(f"✅ **X-ray traces MATCH** ({len(xray_comparison['rust_lines'])} iterations traced)\n\n")
            else:
                f.write(f"❌ **X-ray traces DIVERGE** at iteration {xray_comparison['first_divergence']}\n\n")
                f.write("### First Divergence\n\n")
                for diff in xray_comparison["differences"][:5]:  # Show first 5 differences
                    f.write(f"**Line {diff['line_num']}:**\n")
                    f.write(f"- Rust: `{diff['rust'] or '(missing)'}`\n")
                    f.write(f"- C++:  `{diff['cpp'] or '(missing)'}`\n\n")
                if len(xray_comparison["differences"]) > 5:
                    f.write(f"... and {len(xray_comparison['differences']) - 5} more differences\n\n")
        else:
            f.write("## X-Ray Trace Comparison\n\n")
            f.write("*No xray tracing enabled. Use -X <N> to trace first N iterations.*\n\n")

    print(f"Report written to: {report_path}")


def main():
    args = parse_args()

    # Resolve input file path
    input_file = Path(args.input_file)
    if not input_file.is_absolute():
        # Try relative to script dir first, then current dir
        if (SCRIPT_DIR / input_file).exists():
            input_file = SCRIPT_DIR / input_file
        elif not input_file.exists():
            print(f"Error: Input file not found: {input_file}")
            sys.exit(1)

    # Build if requested
    if args.build:
        build_rust_solver()

    # Check solvers exist
    if not RUST_SOLVER.exists():
        print(f"Rust solver not found at {RUST_SOLVER}")
        print("Run with --build to build it, or run:")
        print(f"  cd {WORKSPACE_DIR} && cargo build --bin solver --release")
        sys.exit(1)

    # Always use xray solver (supports -V for perf output)
    cpp_solver = CPP_SOLVER_XRAY
    if not cpp_solver.exists():
        print(f"C++ xray solver not found at {cpp_solver}")
        print("Build it with:")
        print(f"  cd {CPP_SOLVER_XRAY_REPO}/xray && make")
        sys.exit(1)

    # Create temp input file
    temp_input = create_temp_input(input_file, args.leader, args.strain)

    try:
        # Create run folder
        run_folder = create_run_folder(input_file, args.leader, args.strain)

        # Copy temp input to run folder
        shutil.copy(temp_input, run_folder / "input.txt")

        print(f"Running comparison for: {input_file.name}")
        print(f"  Strain: {args.strain or 'all'}")
        print(f"  Leader: {args.leader or 'all'}")
        print(f"  Timeout: {args.timeout}s")
        if args.xray > 0:
            print(f"  X-ray: {args.xray} iterations")
        if args.no_pruning:
            print(f"  No-pruning: enabled")
        if args.no_tt:
            print(f"  No-TT: enabled")
        if args.no_rank_skip:
            print(f"  No-rank-skip: enabled")
        print(f"  Output: {run_folder}")
        print()

        # Run Rust solver
        show_perf = not args.no_perf
        print("Running Rust solver...")
        rust_output, rust_time, rust_timeout, rust_xray, rust_equiv = run_solver(
            RUST_SOLVER, temp_input, "rust", args.timeout, args.xray, args.no_pruning, args.no_tt, args.no_rank_skip, show_perf=show_perf
        )
        with open(run_folder / "rust_output.txt", "w") as f:
            f.write(rust_output)
        if rust_timeout:
            print(f"  TIMEOUT after {rust_time:.3f}s")
        else:
            print(f"  Completed in {rust_time:.3f}s")

        # Run C++ solver
        print("Running C++ solver...")
        cpp_output, cpp_time, cpp_timeout, cpp_xray, cpp_equiv = run_solver(
            cpp_solver, temp_input, "cpp", args.timeout, args.xray, args.no_pruning, args.no_tt, args.no_rank_skip, show_perf=show_perf
        )
        with open(run_folder / "cpp_output.txt", "w") as f:
            f.write(cpp_output)
        if cpp_timeout:
            print(f"  TIMEOUT after {cpp_time:.3f}s")
        else:
            print(f"  Completed in {cpp_time:.3f}s")

        # Count cards to determine number of tricks
        num_tricks = count_tricks_in_deal(input_file)

        # Parse and compare results
        rust_results = parse_rust_results(rust_output, num_tricks)
        cpp_results = parse_cpp_results(cpp_output, num_tricks)
        comparison = compare_results(rust_results, cpp_results)

        # Compare xray traces if enabled
        xray_comparison = None
        if args.xray > 0:
            xray_comparison = compare_xray_traces(rust_xray, cpp_xray)
            # Save xray traces
            with open(run_folder / "rust_xray.txt", "w") as f:
                f.write("\n".join(rust_xray) + "\n")
            with open(run_folder / "cpp_xray.txt", "w") as f:
                f.write("\n".join(cpp_xray) + "\n")
            # Save equiv traces
            if rust_equiv:
                with open(run_folder / "rust_equiv.txt", "w") as f:
                    f.write("\n".join(rust_equiv) + "\n")
            if cpp_equiv:
                with open(run_folder / "cpp_equiv.txt", "w") as f:
                    f.write("\n".join(cpp_equiv) + "\n")

        # Write comparison report
        write_comparison_report(
            run_folder, comparison,
            rust_output, cpp_output,
            rust_time, cpp_time,
            rust_timeout, cpp_timeout,
            input_file, args.leader, args.strain,
            args.timeout,
            xray_comparison,
            args.no_pruning,
            args.no_tt,
            args.no_rank_skip,
            args.xray,
            show_perf,
        )

        # Print summary
        print()
        if rust_timeout or cpp_timeout:
            print("⏱️ TIMEOUT")
            if rust_timeout:
                print("   Rust solver timed out")
            if cpp_timeout:
                print("   C++ solver timed out")
        elif comparison["match"]:
            print("✅ Results MATCH")
        else:
            print("❌ Results DIFFER")
            for diff in comparison["differences"]:
                print(f"   {diff['strain']}/{diff['leader']}: Rust={diff['rust']}, C++={diff['cpp']}")

        # Print xray summary
        if xray_comparison is not None:
            if xray_comparison["match"]:
                print(f"✅ X-ray traces MATCH ({len(rust_xray)} iterations)")
            else:
                print(f"❌ X-ray traces DIVERGE at iteration {xray_comparison['first_divergence']}")

        print(f"\nFull results in: {run_folder}")

    finally:
        # Cleanup temp file
        os.unlink(temp_input)


if __name__ == "__main__":
    main()
