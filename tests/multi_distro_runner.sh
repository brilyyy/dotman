#!/usr/bin/env bash
set -euo pipefail

STATIC_BIN="/home/v0idbr/Dev/dotman/target/x86_64-unknown-linux-musl/release/dotman"

if [ ! -f "$STATIC_BIN" ]; then
    echo "Building static musl release binary..."
    cargo build --release --target x86_64-unknown-linux-musl
fi

DISTROS=(
    "Arch Linux|docker.io/library/archlinux:latest|pacman"
    "Fedora|docker.io/library/fedora:latest|dnf"
    "Debian/Ubuntu|docker.io/library/ubuntu:24.04|apt-get"
    "Alpine Linux|docker.io/library/alpine:latest|apk"
    "openSUSE|registry.opensuse.org/opensuse/tumbleweed:latest|zypper"
)

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

RESULTS=()

echo "==============================================================="
echo "   Running dotman Multi-Distribution Test Matrix in Podman     "
echo "==============================================================="

for entry in "${DISTROS[@]}"; do
    IFS="|" read -r DISTRO_NAME IMAGE EXPECTED_PM <<< "$entry"
    echo -e "\n${BOLD}${CYAN}Testing on ${DISTRO_NAME} (${IMAGE})...${NC}"

    set +e
    podman run --rm -i \
        -v "${STATIC_BIN}:/usr/local/bin/dotman:ro" \
        "$IMAGE" \
        sh -s << INNER_EOF
set -eu

export HOME=/tmp/tester_home
mkdir -p "\$HOME"

# 1. Test binary execution
dotman --version >/dev/null 2>&1

# 2. Test Init
mkdir -p "\$HOME/dotfiles"
cd "\$HOME/dotfiles"
dotman init >/dev/null

[ -f "\$HOME/dotfiles/dot.toml" ] || exit 10
[ -d "\$HOME/dotfiles/.bak" ] || exit 11

# 3. Test Add
echo 'export SHELL_INIT=1' > "\$HOME/.zshrc"
mkdir -p "\$HOME/.config/nvim"
echo 'init.lua' > "\$HOME/.config/nvim/init.lua"

dotman add "\$HOME/.zshrc" -t shell >/dev/null
dotman add "\$HOME/.config/nvim" -t dev >/dev/null

[ -L "\$HOME/.zshrc" ] || exit 20
[ -L "\$HOME/.config/nvim" ] || exit 21
[ -f "\$HOME/dotfiles/zshrc" ] || exit 22
[ -f "\$HOME/dotfiles/nvim/init.lua" ] || exit 23

# 4. Test Status
STATUS_OUT=\$(dotman status)
echo "\$STATUS_OUT" | grep -q "Summary: 2 ok" || exit 30

# 5. Test Wipe & Re-Deploy
rm "\$HOME/.zshrc"
dotman deploy >/dev/null
[ -L "\$HOME/.zshrc" ] || exit 40

# 6. Test Package Manager Detection
DEPS_OUT=\$(dotman install-deps --dry-run -c core)
echo "\$DEPS_OUT" | grep -q "${EXPECTED_PM}" || exit 50

exit 0
INNER_EOF

    STATUS=$?
    set -e

    if [ "$STATUS" -eq 0 ]; then
        echo -e "  -> ${GREEN}PASS${NC}: All lifecycle & package manager checks passed on ${DISTRO_NAME} (${EXPECTED_PM})"
        RESULTS+=("${DISTRO_NAME}|${EXPECTED_PM}|PASS")
    else
        echo -e "  -> ${RED}FAIL${NC}: Failed on ${DISTRO_NAME} (Exit code: ${STATUS})"
        RESULTS+=("${DISTRO_NAME}|${EXPECTED_PM}|FAIL (code ${STATUS})")
    fi
done

echo -e "\n==============================================================="
echo -e "             Multi-Distribution Results Matrix                 "
echo -e "==============================================================="
printf "%-18s | %-16s | %-10s\n" "Distribution" "Package Manager" "Status"
echo "-------------------+------------------+-----------"

ALL_PASS=true
for res in "${RESULTS[@]}"; do
    IFS="|" read -r D P S <<< "$res"
    if [[ "$S" =~ PASS ]]; then
        printf "%-18s | %-16s | ${GREEN}%-10s${NC}\n" "$D" "$P" "$S"
    else
        printf "%-18s | %-16s | ${RED}%-10s${NC}\n" "$D" "$P" "$S"
        ALL_PASS=false
    fi
done
echo "==============================================================="

if [ "$ALL_PASS" = true ]; then
    echo -e "${GREEN}All distributions passed successfully!${NC}"
    exit 0
else
    echo -e "${RED}Some distribution checks failed.${NC}"
    exit 1
fi
