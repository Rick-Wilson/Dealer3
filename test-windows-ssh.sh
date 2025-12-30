#!/bin/bash
# Test script for Windows SSH connection

# Configuration - UPDATE THESE
WINDOWS_USER="${WINDOWS_USER:-YourUsername}"
WINDOWS_IP="${WINDOWS_IP:-192.168.1.100}"

echo "Testing SSH connection to Windows..."
echo "User: $WINDOWS_USER"
echo "IP: $WINDOWS_IP"
echo ""

# Test 1: Basic connection
echo "Test 1: Basic SSH connection"
if ssh -o BatchMode=yes -o ConnectTimeout=5 "$WINDOWS_USER@$WINDOWS_IP" "echo 'SSH connection successful!'" 2>/dev/null; then
    echo "✅ SSH key authentication works!"
else
    echo "❌ SSH key authentication failed"
    echo "Trying with verbose output..."
    ssh -vvv "$WINDOWS_USER@$WINDOWS_IP" "echo 'test'" 2>&1 | grep -A5 -B5 "publickey"
    exit 1
fi

echo ""

# Test 2: Run Windows command
echo "Test 2: Running Windows command"
WINDOWS_VERSION=$(ssh "$WINDOWS_USER@$WINDOWS_IP" "ver" 2>/dev/null)
echo "Windows version: $WINDOWS_VERSION"

echo ""

# Test 3: Check if dealer.exe exists (common locations)
echo "Test 3: Looking for dealer.exe"
for path in \
    "C:\\dealer\\dealer.exe" \
    "C:\\Program Files\\dealer\\dealer.exe" \
    "C:\\Users\\$WINDOWS_USER\\dealer.exe" \
    "D:\\dealer\\dealer.exe"; do

    if ssh "$WINDOWS_USER@$WINDOWS_IP" "cmd /c \"if exist $path echo FOUND\"" 2>/dev/null | grep -q "FOUND"; then
        echo "✅ Found dealer.exe at: $path"
        DEALER_PATH="$path"
        break
    fi
done

if [ -z "$DEALER_PATH" ]; then
    echo "❌ dealer.exe not found in common locations"
    echo "Please specify the path manually"
else
    echo ""
    echo "Test 4: Running dealer.exe"
    ssh "$WINDOWS_USER@$WINDOWS_IP" "$DEALER_PATH -V" 2>/dev/null || echo "❌ Failed to run dealer.exe"
fi

echo ""
echo "Setup complete! If all tests passed, you can use the run-dealer.sh script."
