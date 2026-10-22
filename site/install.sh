#!/usr/bin/env bash
set -e

# flareTUI Automated Installer
# Usage: curl -sSL https://raw.githubusercontent.com/KidusAlemayehu/flareTUI/main/install.sh | bash

REPO="KidusAlemayehu/flareTUI"
BIN_NAME="flareTUI"

# Color formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
ORANGE='\033[38;5;208m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${ORANGE}${BOLD}"
echo "   __ _                 _____ _   _ _____ "
echo "  / _| | __ _ _ __ ___ |_   _| | | |_   _|"
echo " | |_| |/ _\` | '__/ _ \  | | | | | | | |  "
echo " |  _| | (_| | | |  __/  | | | |_| | | |  "
echo " |_| |_|\__,_|_|  \___|  |_|  \___/  |_|  "
echo -e "${NC}"
echo -e "${CYAN}Terminal User Interface for Cloudflare${NC}"
echo "--------------------------------------------------"

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     OS_TARGET="unknown-linux-gnu" ;;
    Darwin*)    OS_TARGET="apple-darwin" ;;
    *)
        echo -e "${RED}Error: Unsupported operating system: ${OS}${NC}"
        echo "Please build from source using 'cargo install flare-tui'"
        exit 1
        ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "${ARCH}" in
    x86_64|amd64)   ARCH_TARGET="x86_64" ;;
    aarch64|arm64)  ARCH_TARGET="aarch64" ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: ${ARCH}${NC}"
        echo "Please build from source using 'cargo install flare-tui'"
        exit 1
        ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"
echo -e "Detected platform: ${BOLD}${TARGET}${NC}"

# Fetch latest release tag from GitHub API
echo "Resolving latest release..."
LATEST_TAG=$(curl -sSL -H "Accept: application/vnd.github.v3+json" "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | head -n 1 | cut -d '"' -f 4 || true)

if [ -z "${LATEST_TAG}" ]; then
    LATEST_TAG="v0.9.0"
fi

echo -e "Target version:    ${BOLD}${LATEST_TAG}${NC}"

# Asset details
ARCHIVE_NAME="flareTUI-${LATEST_TAG}-${TARGET}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARCHIVE_NAME}.tar.gz"

TMP_DIR=$(mktemp -d)
cleanup() {
    rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

echo -e "Downloading:       ${CYAN}${DOWNLOAD_URL}${NC}"
if ! curl -sSL --fail "${DOWNLOAD_URL}" -o "${TMP_DIR}/${ARCHIVE_NAME}.tar.gz"; then
    echo -e "${RED}Failed to download ${DOWNLOAD_URL}${NC}"
    echo "Release binary may not be published for ${TARGET} yet."
    echo "You can build directly with: cargo install flare-tui"
    exit 1
fi

echo "Extracting binary..."
tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}.tar.gz" -C "${TMP_DIR}"

# Determine install location
USE_SUDO=false

if [ -n "${INSTALL_DIR:-}" ]; then
    mkdir -p "${INSTALL_DIR}"
elif [ "$(id -u)" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "${INSTALL_DIR}"
fi

echo "Installing to ${INSTALL_DIR}/${BIN_NAME}..."
if [ "${USE_SUDO}" = true ]; then
    sudo mv "${TMP_DIR}/${ARCHIVE_NAME}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
    sudo chmod +x "${INSTALL_DIR}/${BIN_NAME}"
else
    mv "${TMP_DIR}/${ARCHIVE_NAME}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
    chmod +x "${INSTALL_DIR}/${BIN_NAME}"
fi

echo ""
echo -e "${GREEN}${BOLD}✓ flareTUI successfully installed to ${INSTALL_DIR}/${BIN_NAME}!${NC}"

# Check if INSTALL_DIR is in PATH
case ":${PATH}:" in
    *:"${INSTALL_DIR}":*) ;;
    *)
        echo -e "${ORANGE}Warning: ${INSTALL_DIR} is not in your \$PATH.${NC}"
        echo "Add it to your shell configuration:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        ;;
esac

echo ""
echo -e "Run ${BOLD}flareTUI${NC} to start managing your Cloudflare edge infrastructure."
