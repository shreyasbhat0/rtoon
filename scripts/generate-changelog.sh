#!/bin/bash
# Script to generate/update CHANGELOG.md using git-cliff

set -e

if ! command -v git-cliff &> /dev/null; then
    echo "Error: git-cliff is not installed."
    echo "Install it with: cargo install git-cliff"
    exit 1
fi

echo "Fetching tags..."
git fetch --tags --unshallow 2>/dev/null || git fetch --tags || true

echo "Generating CHANGELOG.md..."
if [ -n "$1" ]; then
    VERSION="$1"
    git-cliff --config cliff.toml --output CHANGELOG.md --tag "v${VERSION}"
    echo "Generated CHANGELOG.md for version v${VERSION}"
else
    git-cliff --config cliff.toml --output CHANGELOG.md
    echo "Generated CHANGELOG.md for all versions"
fi

echo "Done! Review CHANGELOG.md and commit if needed."

