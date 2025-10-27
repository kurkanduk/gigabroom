#!/bin/bash

# Rust Cleaner Uninstallation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  Rust Cleaner Uninstall Script        ║${NC}"
echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo ""

# Find rust-cleaner binary
FOUND=0

if [ -f "/usr/local/bin/rust-cleaner" ]; then
    echo -e "${YELLOW}Found rust-cleaner in /usr/local/bin${NC}"
    rm -f /usr/local/bin/rust-cleaner
    echo -e "${GREEN}✓ Removed /usr/local/bin/rust-cleaner${NC}"
    FOUND=1
fi

if [ -f "$HOME/.local/bin/rust-cleaner" ]; then
    echo -e "${YELLOW}Found rust-cleaner in $HOME/.local/bin${NC}"
    rm -f "$HOME/.local/bin/rust-cleaner"
    echo -e "${GREEN}✓ Removed $HOME/.local/bin/rust-cleaner${NC}"
    FOUND=1
fi

if [ -f "$HOME/.cargo/bin/rust-cleaner" ]; then
    echo -e "${YELLOW}Found rust-cleaner in $HOME/.cargo/bin${NC}"
    rm -f "$HOME/.cargo/bin/rust-cleaner"
    echo -e "${GREEN}✓ Removed $HOME/.cargo/bin/rust-cleaner${NC}"
    FOUND=1
fi

# Remove cache file
if [ -f "$HOME/.rust-cleaner-cache.json" ]; then
    echo -e "${YELLOW}Removing cache file...${NC}"
    rm -f "$HOME/.rust-cleaner-cache.json"
    echo -e "${GREEN}✓ Removed cache file${NC}"
fi

if [ $FOUND -eq 0 ]; then
    echo -e "${YELLOW}⚠ rust-cleaner not found in common locations${NC}"
    echo -e "${CYAN}Checked:${NC}"
    echo -e "  - /usr/local/bin/rust-cleaner"
    echo -e "  - $HOME/.local/bin/rust-cleaner"
    echo -e "  - $HOME/.cargo/bin/rust-cleaner"
else
    echo -e "\n${GREEN}✓ Uninstallation complete!${NC}"
fi

echo ""
