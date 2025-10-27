#!/bin/bash

# Rust Cleaner Installation Script
# Builds and installs rust-cleaner to your PATH

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   Rust Cleaner Installation Script    ║${NC}"
echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Error: Cargo not found${NC}"
    echo -e "  Please install Rust: https://rustup.rs/"
    exit 1
fi

echo -e "${GREEN}✓ Cargo found${NC}"

# Build the release binary
echo -e "\n${YELLOW}Building release binary...${NC}"
cargo build --release

if [ $? -ne 0 ]; then
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Build successful${NC}"

# Determine installation directory
if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ -d "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
else
    mkdir -p "$HOME/.local/bin"
    INSTALL_DIR="$HOME/.local/bin"
fi

echo -e "\n${YELLOW}Installing to: ${INSTALL_DIR}${NC}"

# Copy the binary
cp target/release/rust-cleaner "$INSTALL_DIR/rust-cleaner"
chmod +x "$INSTALL_DIR/rust-cleaner"

echo -e "${GREEN}✓ Binary installed to ${INSTALL_DIR}/rust-cleaner${NC}"

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "\n${YELLOW}⚠ Warning: ${INSTALL_DIR} is not in your PATH${NC}"
    echo -e "\n${CYAN}Add it to your PATH by adding this line to your shell config:${NC}"

    # Detect shell and provide appropriate instructions
    if [ -n "$ZSH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
        echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        echo -e "\n${CYAN}Or run this command to add it automatically:${NC}"
        echo -e "${GREEN}echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> $SHELL_CONFIG${NC}"
    elif [ -n "$BASH_VERSION" ]; then
        if [ "$(uname)" == "Darwin" ]; then
            SHELL_CONFIG="$HOME/.bash_profile"
        else
            SHELL_CONFIG="$HOME/.bashrc"
        fi
        echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        echo -e "\n${CYAN}Or run this command to add it automatically:${NC}"
        echo -e "${GREEN}echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> $SHELL_CONFIG${NC}"
    else
        echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
    fi

    echo -e "\n${YELLOW}After adding to PATH, restart your terminal or run:${NC}"
    echo -e "${GREEN}source $SHELL_CONFIG${NC}"
else
    echo -e "${GREEN}✓ ${INSTALL_DIR} is already in your PATH${NC}"
fi

# Test installation
if command -v rust-cleaner &> /dev/null; then
    VERSION=$(rust-cleaner --version)
    echo -e "\n${GREEN}✓ Installation successful!${NC}"
    echo -e "${CYAN}Version: ${VERSION}${NC}"
    echo -e "\n${CYAN}Try it out:${NC}"
    echo -e "  ${GREEN}rust-cleaner${NC}          # Start interactive menu"
    echo -e "  ${GREEN}rust-cleaner scan${NC}     # Quick scan"
    echo -e "  ${GREEN}rust-cleaner --help${NC}   # Show help"
else
    echo -e "\n${YELLOW}⚠ Installation complete, but 'rust-cleaner' is not yet in your PATH${NC}"
    echo -e "${CYAN}Please follow the instructions above to add it to your PATH${NC}"
    echo -e "\n${CYAN}Or run directly:${NC}"
    echo -e "  ${GREEN}$INSTALL_DIR/rust-cleaner${NC}"
fi

echo ""
