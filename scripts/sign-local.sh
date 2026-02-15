#!/bin/bash
#
# sign-local.sh - Sign and notarize a local dealer3 binary
#
# Usage:
#   sign-local.sh [binary-path]
#
# Defaults to target/release/dealer if no path given.
#
# Required environment variables (set in ~/.zshrc):
#   APPLE_ID          - Apple ID email
#   APPLE_ID_PASSWORD - App-specific password for notarization
#   APPLE_TEAM_ID     - Developer Team ID
#

set -euo pipefail

BINARY="${1:-target/release/dealer}"

if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found: $BINARY" >&2
    echo "Build first: cargo build --release --bin dealer" >&2
    exit 1
fi

if [[ -z "${APPLE_ID:-}" ]] || [[ -z "${APPLE_ID_PASSWORD:-}" ]] || [[ -z "${APPLE_TEAM_ID:-}" ]]; then
    echo "Error: Required environment variables not set." >&2
    echo "Add these to your ~/.zshrc:" >&2
    echo '  export APPLE_ID="your-apple-id@email.com"' >&2
    echo '  export APPLE_ID_PASSWORD="app-specific-password"' >&2
    echo '  export APPLE_TEAM_ID="your-team-id"' >&2
    exit 1
fi

echo "=== Signing $BINARY ==="

# Sign with hardened runtime (required for notarization)
codesign --force --options runtime --sign "Developer ID Application" "$BINARY"
codesign --verify --verbose "$BINARY"
echo "Signing: OK"

echo ""
echo "=== Notarizing ==="

# Notarization requires a zip archive
TMPZIP=$(mktemp /tmp/dealer-notarize-XXXXXX.zip)
trap "rm -f $TMPZIP" EXIT

ditto -c -k "$BINARY" "$TMPZIP"
xcrun notarytool submit "$TMPZIP" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_ID_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" \
    --wait

echo ""
echo "=== Done ==="
echo "Signed and notarized: $BINARY"
