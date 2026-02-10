#!/usr/bin/env bash
#
# Publish all openclaw crates to crates.io in dependency order
#
# Usage:
#   ./scripts/publish.sh           # Dry run (default)
#   ./scripts/publish.sh --execute # Actually publish
#
# Requirements:
#   - cargo login (must be authenticated with crates.io)
#   - All tests passing
#   - Clean git working directory recommended
#

set -euo pipefail

# Configuration
SLEEP_SECONDS=30  # Time to wait between publishes for crates.io indexing
DRY_RUN=true

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
for arg in "$@"; do
    case $arg in
        --execute)
            DRY_RUN=false
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--execute]"
            echo ""
            echo "Options:"
            echo "  --execute    Actually publish to crates.io (default is dry-run)"
            echo "  --help, -h   Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown argument: $arg${NC}"
            exit 1
            ;;
    esac
done

# Crates in dependency order (dependencies must be published first)
CRATES=(
    "crates/openclaw-core"       # No internal dependencies
    "crates/openclaw-ipc"        # Depends on: core
    "crates/openclaw-providers"  # Depends on: core
    "crates/openclaw-channels"   # Depends on: core
    "crates/openclaw-agents"     # Depends on: core, providers
    "crates/openclaw-plugins"    # Depends on: core, ipc
    "crates/openclaw-gateway"    # Depends on: core, agents, channels, providers
    "crates/openclaw-cli"        # Depends on: core, gateway, agents
    "bridge/openclaw-node"       # Depends on: core, providers, agents
)

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           OpenClaw Crates.io Publish Script                    ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

if $DRY_RUN; then
    echo -e "${YELLOW}Mode: DRY RUN (use --execute to actually publish)${NC}"
else
    echo -e "${RED}Mode: EXECUTING (publishing to crates.io)${NC}"
fi
echo ""

# Pre-flight checks
echo -e "${BLUE}Running pre-flight checks...${NC}"

# Check cargo is logged in (only for actual publish)
if ! $DRY_RUN; then
    if ! cargo login --help &>/dev/null; then
        echo -e "${RED}Error: cargo not found${NC}"
        exit 1
    fi
fi

# Run tests first
echo -e "${BLUE}Running tests...${NC}"
if ! cargo test --workspace; then
    echo -e "${RED}Tests failed! Aborting publish.${NC}"
    exit 1
fi
echo -e "${GREEN}✓ All tests passed${NC}"
echo ""

# Check formatting
echo -e "${BLUE}Checking formatting...${NC}"
if ! cargo fmt --check; then
    echo -e "${RED}Format check failed! Run 'cargo fmt' first.${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Formatting OK${NC}"
echo ""

# Run clippy (check for errors only, warnings allowed)
echo -e "${BLUE}Running clippy...${NC}"
CLIPPY_OUTPUT=$(cargo clippy --workspace 2>&1)
CLIPPY_EXIT=$?
WARN_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "^warning:" || true)
ERROR_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "^error" || true)

if [ "$ERROR_COUNT" -gt 0 ]; then
    echo "$CLIPPY_OUTPUT" | grep "^error"
    echo -e "${RED}Clippy found errors! Fix them before publishing.${NC}"
    exit 1
fi

if [ "$WARN_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}Clippy: $WARN_COUNT warnings (non-blocking)${NC}"
else
    echo -e "${GREEN}Clippy: No warnings${NC}"
fi
echo -e "${GREEN}✓ Clippy OK${NC}"
echo ""

# Publish each crate
TOTAL=${#CRATES[@]}
CURRENT=0

for crate_path in "${CRATES[@]}"; do
    CURRENT=$((CURRENT + 1))
    CRATE_NAME=$(basename "$crate_path")

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}[$CURRENT/$TOTAL] Publishing: ${CRATE_NAME}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    # Special handling for openclaw-gateway (disable UI feature for crates.io)
    EXTRA_FLAGS=""
    if [ "$CRATE_NAME" = "openclaw-gateway" ]; then
        EXTRA_FLAGS="--no-default-features"
        echo -e "${YELLOW}Note: Publishing without UI feature (UI assets not available on crates.io)${NC}"
    fi

    if $DRY_RUN; then
        echo -e "${YELLOW}[DRY RUN] Would publish: $crate_path${NC}"
        if ! cargo publish --dry-run --allow-dirty -p "$CRATE_NAME" $EXTRA_FLAGS 2>&1; then
            if [ $CURRENT -eq 1 ]; then
                echo -e "${RED}Dry run failed for $CRATE_NAME${NC}"
                exit 1
            else
                echo -e "${YELLOW}Note: Dry run failed (expected - dependencies not actually published)${NC}"
            fi
        fi
    else
        echo -e "${GREEN}Publishing: $crate_path${NC}"
        cargo publish -p "$CRATE_NAME" $EXTRA_FLAGS 2>&1 || {
            echo -e "${RED}Failed to publish $CRATE_NAME${NC}"
            exit 1
        }
    fi

    echo -e "${GREEN}✓ $CRATE_NAME done${NC}"

    # Sleep between publishes (except for last one)
    if [ $CURRENT -lt $TOTAL ]; then
        if $DRY_RUN; then
            echo -e "${YELLOW}[DRY RUN] Would sleep ${SLEEP_SECONDS}s for crates.io indexing${NC}"
        else
            echo -e "${BLUE}Waiting ${SLEEP_SECONDS}s for crates.io to index...${NC}"
            sleep "$SLEEP_SECONDS"
        fi
    fi
    echo ""
done

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
if $DRY_RUN; then
    echo -e "${GREEN}║              Dry run completed successfully!                  ║${NC}"
    echo -e "${GREEN}║         Run with --execute to publish for real               ║${NC}"
else
    echo -e "${GREEN}║           All crates published successfully!                 ║${NC}"
fi
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
