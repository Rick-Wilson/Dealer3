#!/usr/bin/env python3
"""
test-filter.py - Test dealer3 filter accuracy against dealer.exe

Generates deals with Windows dealer.exe (via SSH), then feeds them through
dealer3 with the same .dlr filter. If dealer3's evaluation is correct,
100% of input deals should pass.

Usage:
    test-filter.py [-p N] [-s SEED] [-v] <dlr-file>

Options:
    -p N        Produce N deals from dealer.exe (default: 10)
    -s SEED     Random seed for dealer.exe (default: 1)
    -v          Verbose: show dealer outputs on failure
    -h, --help  Show this help message

Examples:
    test-filter.py /path/to/Smolen.dlr
    test-filter.py -p 20 -s 42 /path/to/Misfit6-5.dlr
    test-filter.py -v -p 5 Smolen.dlr
"""
import argparse
import os
import re
import subprocess
import sys
import tempfile

# Add Practice-Bidding-Scenarios build-scripts-mac to path for ssh_runner
PBS_BUILD_SCRIPTS = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "..", "Practice-Bidding-Scenarios", "build-scripts-mac"
)
PBS_BUILD_SCRIPTS = os.path.normpath(PBS_BUILD_SCRIPTS)

if not os.path.isdir(PBS_BUILD_SCRIPTS):
    print(f"Error: Cannot find PBS build-scripts-mac at {PBS_BUILD_SCRIPTS}")
    print("Expected at: ../Practice-Bidding-Scenarios/build-scripts-mac/")
    sys.exit(1)

sys.path.insert(0, PBS_BUILD_SCRIPTS)

from ssh_runner import run_windows_command, mac_to_windows_path


def count_printall_deals(text: str) -> int:
    """Count deals in printall format output (board number lines like '   1.')."""
    return len(re.findall(r'^\s*\d+\.\s*$', text, re.MULTILINE))


def count_oneline_deals(text: str) -> int:
    """Count deals in oneline format output (lines starting with compass letter)."""
    return len(re.findall(r'^[nesw] ', text, re.MULTILINE))


def prepare_dlr_with_printall(dlr_file: str) -> str:
    """Create a temp copy of the .dlr file with 'action printall' forced.

    Some .dlr files have 'action' with no format or a non-deal action,
    so dealer.exe only outputs statistics. We need printall output to
    capture the actual deals.

    Handles three cases:
    - File already has a print action (printall, etc.) → no change
    - File has 'action' with averages/frequencies but no print → insert printall
    - File has no action line at all → append 'action printall'

    Returns the path to the temp file (caller must delete it).
    """
    with open(dlr_file, "r") as f:
        content = f.read()

    print_actions = r'printall|printew|printpbn|printcompact|printoneline'
    has_print = re.search(
        rf'^action\b.*\b({print_actions})\b', content, re.MULTILINE
    )

    if not has_print:
        # Check if 'action' is alone on a line (multi-line action block
        # with averages/frequencies on following lines)
        bare_action = re.search(r'^action\s*$', content, re.MULTILINE)
        if bare_action:
            # Insert 'printall,' so averages/frequencies still chain
            content = re.sub(
                r'^action\s*$', 'action printall,',
                content, flags=re.MULTILINE, count=1
            )
        elif re.search(r'^action\b', content, re.MULTILINE):
            # Action with components on same line but no print type —
            # prepend printall to the component list
            content = re.sub(
                r'^action\b\s*', 'action printall, ',
                content, flags=re.MULTILINE, count=1
            )
        else:
            # No action line at all
            content = content.rstrip() + '\naction printall\n'

    # Write to a temp file in the same directory (so the Windows path
    # mapping works — the file must be under the GitHub folder)
    dlr_dir = os.path.dirname(dlr_file)
    fd, tmp_path = tempfile.mkstemp(suffix=".dlr", dir=dlr_dir)
    with os.fdopen(fd, "w") as f:
        f.write(content)

    return tmp_path


def main():
    parser = argparse.ArgumentParser(
        description="Test dealer3 filter accuracy against dealer.exe"
    )
    parser.add_argument("dlr_file", help="Path to .dlr file")
    parser.add_argument("-p", "--produce", type=int, default=10,
                        help="Number of deals to produce (default: 10)")
    parser.add_argument("-s", "--seed", type=int, default=1,
                        help="Random seed for dealer.exe (default: 1)")
    parser.add_argument("-v", "--verbose", action="store_true",
                        help="Show dealer outputs on failure")
    args = parser.parse_args()

    # Resolve .dlr file to absolute path
    dlr_file = os.path.abspath(args.dlr_file)
    if not os.path.isfile(dlr_file):
        print(f"Error: File not found: {dlr_file}", file=sys.stderr)
        sys.exit(1)

    scenario = os.path.splitext(os.path.basename(dlr_file))[0]
    dealer3_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    print(f"=== Test Filter: {scenario} ===")
    print()

    # --- Step 1: Generate deals with Windows dealer.exe ---
    print(f"Step 1: Generating {args.produce} deals with dealer.exe "
          f"(seed={args.seed})...")

    # Create temp .dlr with 'action printall' to ensure deals are output
    tmp_dlr = prepare_dlr_with_printall(dlr_file)

    try:
        win_dlr_path = mac_to_windows_path(tmp_dlr)
        # dealer.exe is in PATH on Windows (C:\Dealer\dealer.exe)
        dealer_cmd = (
            f'dealer -p {args.produce} -s {args.seed} "{win_dlr_path}"'
        )

        try:
            returncode, stdout, stderr = run_windows_command(
                dealer_cmd, timeout=120, verbose=False
            )
        except Exception as e:
            print(f"  ERROR: dealer.exe failed: {e}", file=sys.stderr)
            sys.exit(1)
    finally:
        os.unlink(tmp_dlr)

    if returncode != 0:
        print(f"  ERROR: dealer.exe exited with code {returncode}",
              file=sys.stderr)
        if stderr:
            print(f"  stderr: {stderr.strip()}", file=sys.stderr)
        sys.exit(1)

    dealer_exe_output = stdout

    # Strip stray characters from dealer.exe block comment bug.
    # The bug echoes <, E, O, F, > characters from inside /* */ comments.
    if dealer_exe_output and dealer_exe_output[0] in '<EOF>':
        dealer_exe_output = dealer_exe_output.lstrip('<EOF>')

    # Count deals from dealer.exe output
    deals_from_exe = (count_printall_deals(dealer_exe_output) +
                      count_oneline_deals(dealer_exe_output))

    if deals_from_exe == 0:
        print("  ERROR: dealer.exe produced 0 deals!")
        print("  Raw output (first 10 lines):")
        for line in dealer_exe_output.splitlines()[:10]:
            print(f"    {line}")
        sys.exit(1)

    print(f"  dealer.exe produced {deals_from_exe} deals")

    # --- Step 2: Feed deals through dealer3 with same filter ---
    print()
    print(f"Step 2: Feeding {deals_from_exe} deals through dealer3 filter...")

    # Build dealer3 if needed
    dealer3_bin = os.path.join(dealer3_root, "target", "release", "dealer")
    if not os.path.isfile(dealer3_bin):
        print("  Building dealer3 (release)...")
        subprocess.run(
            ["cargo", "build", "--release", "--bin", "dealer", "-q"],
            cwd=dealer3_root,
            check=True,
            capture_output=True,
        )

    # Write dealer.exe output to temp file for --input-deals
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".txt", delete=False
    ) as tmp:
        tmp.write(dealer_exe_output)
        tmp_path = tmp.name

    try:
        result = subprocess.run(
            [dealer3_bin, "--input-deals", tmp_path, "-v", "-f", "oneline",
             dlr_file],
            capture_output=True,
            text=True,
        )
        dealer3_output = result.stdout + result.stderr
    finally:
        os.unlink(tmp_path)

    # Extract stats from dealer3 output
    generated_match = re.search(r'Generated\s+(\d+)', dealer3_output)
    produced_match = re.search(r'Produced\s+(\d+)', dealer3_output)

    generated = int(generated_match.group(1)) if generated_match else 0
    produced = int(produced_match.group(1)) if produced_match else 0

    print(f"  dealer3: {produced}/{generated} passed filter")

    # --- Step 3: Report results ---
    print()
    if produced == deals_from_exe:
        print(f"PASS: {produced}/{deals_from_exe} deals passed (100%)")
    else:
        rejected = deals_from_exe - produced
        pct = (produced * 100) // deals_from_exe if deals_from_exe > 0 else 0
        print(f"FAIL: {produced}/{deals_from_exe} deals passed "
              f"({pct}%) — {rejected} rejected")

        if args.verbose:
            print()
            print("--- Dealer.exe output (first 30 lines) ---")
            for line in dealer_exe_output.splitlines()[:30]:
                print(f"  {line}")
            print()
            print("--- Dealer3 output ---")
            for line in dealer3_output.splitlines():
                print(f"  {line}")

        sys.exit(1)


if __name__ == "__main__":
    main()
