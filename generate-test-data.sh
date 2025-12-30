#!/bin/bash
# Generate test data from dealer.exe for validating Rust implementation
# This creates reference output to compare against our Rust version

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DATA_DIR="${SCRIPT_DIR}/test-data"
DEALER_SCRIPT="${SCRIPT_DIR}/run-dealer.sh"

# Create test data directory
mkdir -p "${TEST_DATA_DIR}"

echo "Generating test data from dealer.exe in PBN format..."
echo "Output directory: ${TEST_DATA_DIR}"
echo ""

# Helper function to create dealer input with printpbn action
create_dealer_input() {
    local constraint="$1"
    cat << EOF
${constraint}

action
printpbn
EOF
}

# Test 1: Simple HCP constraint with fixed seed
echo "Test 1: Simple HCP constraint (seed=1)"
create_dealer_input "hcp(north) >= 15" | "${DEALER_SCRIPT}" -s 1 -p 10 > "${TEST_DATA_DIR}/test1-hcp-seed1.pbn"
echo "  ✓ Saved to test1-hcp-seed1.pbn"

# Test 2: Same constraint, different seed
echo "Test 2: Simple HCP constraint (seed=42)"
create_dealer_input "hcp(north) >= 15" | "${DEALER_SCRIPT}" -s 42 -p 10 > "${TEST_DATA_DIR}/test2-hcp-seed42.pbn"
echo "  ✓ Saved to test2-hcp-seed42.pbn"

# Test 3: Suit length constraint
echo "Test 3: Suit length (hearts >= 5)"
create_dealer_input "hearts(north) >= 5" | "${DEALER_SCRIPT}" -s 100 -p 10 > "${TEST_DATA_DIR}/test3-hearts-seed100.pbn"
echo "  ✓ Saved to test3-hearts-seed100.pbn"

# Test 4: Combined constraint
echo "Test 4: Combined constraint (hearts >= 5 && hcp <= 13)"
create_dealer_input "hearts(north) >= 5 && hcp(south) <= 13" | "${DEALER_SCRIPT}" -s 200 -p 10 > "${TEST_DATA_DIR}/test4-combined-seed200.pbn"
echo "  ✓ Saved to test4-combined-seed200.pbn"

# Test 5: Generate mode (not produce)
echo "Test 5: Generate 100 deals, report matches"
create_dealer_input "hcp(north) >= 20" | "${DEALER_SCRIPT}" -s 300 -g 100 > "${TEST_DATA_DIR}/test5-generate-seed300.pbn"
echo "  ✓ Saved to test5-generate-seed300.pbn"

echo ""
echo "Test data generation complete!"
echo "Files created in: ${TEST_DATA_DIR}"
echo ""
echo "These files can be used to verify that our Rust implementation"
echo "produces identical output when given the same seed and constraints."
