#!/usr/bin/env bash
set -euo pipefail

DOTMAN_BIN="/home/v0idbr/Dev/dotman/target/x86_64-unknown-linux-musl/release/dotman"

if [ ! -f "$DOTMAN_BIN" ]; then
    echo "Building release static musl binary..."
    cargo build --release --target x86_64-unknown-linux-musl
fi

echo "==============================================================="
echo "   Running dotman v1.0 Real-World & Edge-Case Suite in Podman  "
echo "==============================================================="

podman run --rm -i \
    -v "${DOTMAN_BIN}:/usr/local/bin/dotman:ro" \
    ubuntu:24.04 \
    bash -s << 'EOF'
set -euo pipefail

echo "[Container] Setting up non-root user 'tester'..."
useradd -m -s /bin/bash tester

su - tester << 'TESTS'
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

PASSED=0
FAILED=0

assert_eq() {
    local expected="$1"
    local actual="$2"
    local desc="$3"
    if [ "$expected" = "$actual" ]; then
        echo -e "    ${GREEN}PASS${NC}: $desc"
        PASSED=$((PASSED + 1))
    else
        echo -e "    ${RED}FAIL${NC}: $desc (Expected '$expected', got '$actual')"
        FAILED=$((FAILED + 1))
    fi
}

assert_file_exists() {
    local path="$1"
    local desc="$2"
    if [ -e "$path" ] || [ -L "$path" ]; then
        echo -e "    ${GREEN}PASS${NC}: $desc"
        PASSED=$((PASSED + 1))
    else
        echo -e "    ${RED}FAIL${NC}: $desc ('$path' does not exist)"
        FAILED=$((FAILED + 1))
    fi
}

assert_symlink() {
    local path="$1"
    local desc="$2"
    if [ -L "$path" ]; then
        echo -e "    ${GREEN}PASS${NC}: $desc"
        PASSED=$((PASSED + 1))
    else
        echo -e "    ${RED}FAIL${NC}: $desc ('$path' is not a symlink)"
        FAILED=$((FAILED + 1))
    fi
}

echo -e "\n${CYAN}=== Scenario 1: Init & Manifest Discovery from Subdirectories ===${NC}"
mkdir -p /home/tester/dotfiles
cd /home/tester/dotfiles
dotman init
assert_file_exists "/home/tester/dotfiles/dot.toml" "Manifest dot.toml created"
assert_file_exists "/home/tester/dotfiles/.bak" "Backup directory .bak/ created"

# Deep subdir manifest discovery
mkdir -p /home/tester/dotfiles/deep/sub/dir
cd /home/tester/dotfiles/deep/sub/dir
dotman status > /tmp/status_sub.out 2>&1
assert_eq "0" "$?" "dotman status successfully ran from deep subdirectory"
cd /home/tester/dotfiles

echo -e "\n${CYAN}=== Scenario 2: Real-World Ingest (Files & Folders) ===${NC}"
mkdir -p /home/tester/.config/nvim/lua/plugins
echo 'return { "lazy.nvim" }' > /home/tester/.config/nvim/lua/plugins/init.lua
echo 'export ZSH=/usr/share/zsh' > /home/tester/.zshrc

dotman add /home/tester/.zshrc --tag shell
dotman add /home/tester/.config/nvim --tag dev

assert_symlink "/home/tester/.zshrc" "Original .zshrc is replaced by symlink"
assert_symlink "/home/tester/.config/nvim" "Original .config/nvim is replaced by symlink"
assert_file_exists "/home/tester/dotfiles/zshrc" "Repo copy of zshrc exists"
assert_file_exists "/home/tester/dotfiles/nvim/lua/plugins/init.lua" "Nested files inside folder migrated intact"

echo -e "\n${CYAN}=== Scenario 3: Trailing Slashes & Whitespace in Paths ===${NC}"
mkdir -p /home/tester/.config/fish
echo 'set -g fish_greeting ""' > /home/tester/.config/fish/config.fish
dotman add /home/tester/.config/fish/ --name fish
assert_symlink "/home/tester/.config/fish" "Fish dir with trailing slash added and symlinked"

mkdir -p "/home/tester/.config/my app"
echo '{"theme": "dark"}' > "/home/tester/.config/my app/settings.json"
dotman add "/home/tester/.config/my app/settings.json" --name "my_app/settings.json"
assert_symlink "/home/tester/.config/my app/settings.json" "File with spaces in path added and symlinked"

echo -e "\n${CYAN}=== Scenario 4: Missing Parent Directories on Deployment ===${NC}"
mkdir -p /home/tester/.config/super/deep/path
echo 'mode = "strict"' > /home/tester/.config/super/deep/path/app.toml
dotman add /home/tester/.config/super/deep/path/app.toml --name deep_app/config.toml

# Delete target intermediate directory
rm -rf /home/tester/.config/super
dotman deploy
assert_symlink "/home/tester/.config/super/deep/path/app.toml" "Intermediate parent directories created on deploy"

echo -e "\n${CYAN}=== Scenario 5: Double-Add / Symlink Loop Prevention ===${NC}"
set +e
dotman add /home/tester/.zshrc > /tmp/double_add.out 2>&1
ADD_EXIT="$?"
set -euo pipefail
assert_eq "1" "$ADD_EXIT" "dotman add rejected already-symlinked target"

echo -e "\n${CYAN}=== Scenario 6: Name Collision Guard in Repo ===${NC}"
mkdir -p /tmp/other
echo 'different' > /tmp/other/.zshrc
set +e
dotman add /tmp/other/.zshrc > /tmp/collision.out 2>&1
COLLISION_EXIT="$?"
set -euo pipefail
assert_eq "1" "$COLLISION_EXIT" "dotman add blocked name collision without --name"

echo -e "\n${CYAN}=== Scenario 7: Fresh Machine Deployment ===${NC}"
rm -rf /home/tester/.zshrc /home/tester/.config/nvim /home/tester/.config/fish "/home/tester/.config/my app" /home/tester/.config/super
dotman deploy
assert_symlink "/home/tester/.zshrc" "Restored .zshrc symlink"
assert_symlink "/home/tester/.config/nvim" "Restored .config/nvim symlink"
assert_symlink "/home/tester/.config/fish" "Restored .config/fish symlink"

echo -e "\n${CYAN}=== Scenario 8: Idempotent Deployment ===${NC}"
dotman deploy > /tmp/deploy1.out
dotman deploy > /tmp/deploy2.out
dotman deploy > /tmp/deploy3.out
assert_eq "0" "$?" "Repeated deploy calls succeed idempotently"

echo -e "\n${CYAN}=== Scenario 9: Tag-Based Selective Deployment ===${NC}"
rm -rf /home/tester/.zshrc /home/tester/.config/nvim
dotman deploy --tag dev
assert_symlink "/home/tester/.config/nvim" "Target with tag 'dev' deployed"
if [ ! -e "/home/tester/.zshrc" ]; then
    echo -e "    ${GREEN}PASS${NC}: Untagged or non-matching tag 'shell' skipped"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Non-matching tag was deployed"
    FAILED=$((FAILED + 1))
fi

echo -e "\n${CYAN}=== Scenario 10: Conflict Resolution & Disaster Quarantine (.bak/) ===${NC}"
echo "LOCAL UNTRACKED EDITS THAT MUST NOT BE LOST" > /home/tester/.zshrc
dotman deploy --force
assert_symlink "/home/tester/.zshrc" "Conflicted file replaced by symlink under --force"

BAK_FILE=$(find /home/tester/dotfiles/.bak -type f -name "*zshrc" | head -n 1)
if [ -n "$BAK_FILE" ] && grep -q "LOCAL UNTRACKED EDITS" "$BAK_FILE"; then
    echo -e "    ${GREEN}PASS${NC}: Quarantined local file safely into .bak/ with matching content"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Quarantined file not found or corrupted in .bak/"
    FAILED=$((FAILED + 1))
fi

echo -e "\n${CYAN}=== Scenario 11: Broken Symlink Self-Healing ===${NC}"
rm -f /home/tester/.config/fish
ln -s /tmp/nonexistent_ghost /home/tester/.config/fish
dotman deploy
FISH_TARGET=$(readlink /home/tester/.config/fish)
assert_eq "/home/tester/dotfiles/fish" "$FISH_TARGET" "Broken symlink self-healed to repo source"

echo -e "\n${CYAN}=== Scenario 12: Package Manager Integration (install-deps --dry-run) ===${NC}"
dotman install-deps --dry-run -c core > /tmp/deps.out
if grep -qE "(apt|apt-get) install -y git curl zsh tmux" /tmp/deps.out; then
    echo -e "    ${GREEN}PASS${NC}: Identified apt/apt-get and generated expected install command"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Unexpected install-deps output"
    cat /tmp/deps.out
    FAILED=$((FAILED + 1))
fi

echo -e "\n${CYAN}=== Scenario 13: Demigration with dotman remove ===${NC}"
dotman remove zshrc
if [ ! -L "/home/tester/.zshrc" ] && [ -f "/home/tester/.zshrc" ]; then
    echo -e "    ${GREEN}PASS${NC}: Target ~/.zshrc restored as a regular file"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Target ~/.zshrc is not a regular file"
    FAILED=$((FAILED + 1))
fi
if [ ! -e "/home/tester/dotfiles/zshrc" ]; then
    echo -e "    ${GREEN}PASS${NC}: Repo copy of zshrc deleted"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Repo copy of zshrc still exists"
    FAILED=$((FAILED + 1))
fi

echo -e "\n${CYAN}=== Scenario 14: Shell Completions Generation ===${NC}"
dotman completions zsh > /tmp/completions.zsh
if grep -q "compdef _dotman dotman" /tmp/completions.zsh; then
    echo -e "    ${GREEN}PASS${NC}: Generated valid zsh completions script"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Invalid completions script"
    FAILED=$((FAILED + 1))
fi

echo -e "\n${CYAN}=== Scenario 15: Post-Deploy Lifecycle Hooks ===${NC}"
# Add a hook to dot.toml
python3 -c '
import toml
cfg = toml.load("/home/tester/dotfiles/dot.toml")
cfg["hooks"] = {"post_deploy": ["echo \"hook_success\" > /home/tester/hook_token.txt"]}
with open("/home/tester/dotfiles/dot.toml", "w") as f:
    toml.dump(cfg, f)
' 2>/dev/null || cat >> /home/tester/dotfiles/dot.toml << 'HOOK_CFG'

[hooks]
post_deploy = ["echo 'hook_success' > /home/tester/hook_token.txt"]
HOOK_CFG

dotman deploy
if [ -f "/home/tester/hook_token.txt" ] && grep -q "hook_success" "/home/tester/hook_token.txt"; then
    echo -e "    ${GREEN}PASS${NC}: Executed post_deploy lifecycle hook successfully"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Hook was not executed"
    FAILED=$((FAILED + 1))
fi

echo -e "\n${CYAN}=== Scenario 16: Custom Command Templates ({packages} token & execution) ===${NC}"
cat >> /home/tester/dotfiles/dot.toml << 'CMD_CFG'

[dependencies.custom_cmd]
cmd = "echo 'INSTALLING: {packages}' > /home/tester/cmd_receipt.txt"
packages = ["pkg_alpha", "pkg_beta"]
CMD_CFG

dotman install-deps -c custom_cmd
if [ -f "/home/tester/cmd_receipt.txt" ] && grep -q "INSTALLING: pkg_alpha pkg_beta" "/home/tester/cmd_receipt.txt"; then
    echo -e "    ${GREEN}PASS${NC}: Custom command template rendered {packages} and executed successfully"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Custom command failed to execute or substitute packages"
    FAILED=$((FAILED + 1))
fi

# Test CLI --cmd override
dotman install-deps -c custom_cmd --cmd "echo 'CLI_OVERRIDE: {packages}' > /home/tester/cli_cmd_receipt.txt"
if [ -f "/home/tester/cli_cmd_receipt.txt" ] && grep -q "CLI_OVERRIDE: pkg_alpha pkg_beta" "/home/tester/cli_cmd_receipt.txt"; then
    echo -e "    ${GREEN}PASS${NC}: CLI --cmd flag correctly took precedence over manifest"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: CLI --cmd override failed"
    FAILED=$((FAILED + 1))
fi

echo -e "\n${CYAN}=== Scenario 17: Dedicated Installer Scripts (Args & Environment) ===${NC}"
mkdir -p /home/tester/dotfiles/scripts
cat > /home/tester/dotfiles/scripts/test_installer.sh << 'SCRIPT_EOF'
#!/bin/sh
echo "PACKAGES: $DOTMAN_PACKAGES" > /home/tester/script_receipt.txt
echo "CATEGORY: $DOTMAN_CATEGORY" >> /home/tester/script_receipt.txt
echo "DRY_RUN: $DOTMAN_DRY_RUN" >> /home/tester/script_receipt.txt
echo "ARGV: $@" >> /home/tester/script_receipt.txt
SCRIPT_EOF
chmod +x /home/tester/dotfiles/scripts/test_installer.sh

cat >> /home/tester/dotfiles/dot.toml << 'SCRIPT_CFG'

[dependencies.script_deps]
script = "scripts/test_installer.sh"
packages = ["tool_x", "tool_y"]
SCRIPT_CFG

dotman install-deps -c script_deps
if [ -f "/home/tester/script_receipt.txt" ] && \
   grep -q "PACKAGES: tool_x tool_y" "/home/tester/script_receipt.txt" && \
   grep -q "CATEGORY: script_deps" "/home/tester/script_receipt.txt" && \
   grep -q "DRY_RUN: 0" "/home/tester/script_receipt.txt" && \
   grep -q "ARGV: tool_x tool_y" "/home/tester/script_receipt.txt"; then
    echo -e "    ${GREEN}PASS${NC}: Dedicated installer script executed with forwarded packages & env vars"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Script output did not match expected environment or arguments"
    cat /home/tester/script_receipt.txt 2>/dev/null || true
    FAILED=$((FAILED + 1))
fi

echo -e "\n${CYAN}=== Scenario 18: Installer Aliases in [installers] & --manager Overrides ===${NC}"
cat >> /home/tester/dotfiles/dot.toml << 'ALIAS_CFG'

[installers]
mock_aur = "echo 'MOCK_AUR: {packages}' > /home/tester/aur_receipt.txt"

[dependencies.aur_category]
manager = "mock_aur"
packages = ["super_pkg"]
ALIAS_CFG

dotman install-deps -c aur_category
if [ -f "/home/tester/aur_receipt.txt" ] && grep -q "MOCK_AUR: super_pkg" "/home/tester/aur_receipt.txt"; then
    echo -e "    ${GREEN}PASS${NC}: Named installer alias in [installers] resolved and executed"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Installer alias failed to resolve"
    FAILED=$((FAILED + 1))
fi

# Test --manager cargo dry run
dotman install-deps -c aur_category --manager cargo --dry-run > /tmp/cargo_override.out
if grep -q "cargo install super_pkg" /tmp/cargo_override.out; then
    echo -e "    ${GREEN}PASS${NC}: CLI --manager cargo overrode category manager"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: CLI --manager cargo override failed"
    FAILED=$((FAILED + 1))
fi

echo -e "\n${CYAN}=== Scenario 19: Ad-Hoc CLI Script & Command Execution ===${NC}"
cat > /home/tester/adhoc.sh << 'ADHOC_EOF'
#!/bin/sh
echo "ADHOC_SUCCESS" > /home/tester/adhoc_token.txt
ADHOC_EOF
chmod +x /home/tester/adhoc.sh

dotman install-deps --script /home/tester/adhoc.sh
if [ -f "/home/tester/adhoc_token.txt" ] && grep -q "ADHOC_SUCCESS" "/home/tester/adhoc_token.txt"; then
    echo -e "    ${GREEN}PASS${NC}: Ad-hoc --script executed directly without category specification"
    PASSED=$((PASSED + 1))
else
    echo -e "    ${RED}FAIL${NC}: Ad-hoc --script failed"
    FAILED=$((FAILED + 1))
fi

echo -e "\n==============================================================="
echo -e "Test Results: ${GREEN}${PASSED} passed${NC}, ${RED}${FAILED} failed${NC}"
echo -e "==============================================================="

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi
TESTS
EOF
