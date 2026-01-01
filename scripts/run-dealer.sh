#!/bin/bash
# Wrapper script to run dealer.exe on Windows VM from Mac
# Usage: ./run-dealer.sh [dealer options]
# Example: ./run-dealer.sh -p 10 < constraints.dlr

WINDOWS_USER="rick"
WINDOWS_IP="10.211.55.5"
DEALER_PATH="C:\\Dealer\\dealer.exe"

# Run dealer.exe remotely via SSH
# stdin is forwarded automatically
ssh "${WINDOWS_USER}@${WINDOWS_IP}" "${DEALER_PATH}" "$@"
