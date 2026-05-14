#!/usr/bin/env bash
set -euo pipefail

HOOKS_DIR="$(git rev-parse --git-dir)/hooks"

install_hook() {
    local name="$1"
    local src="scripts/hooks/$name"
    local dst="$HOOKS_DIR/$name"

    if [ ! -f "$src" ]; then
        echo "warning: $src not found, skipping"
        return
    fi

    cp "$src" "$dst"
    chmod +x "$dst"
    echo "installed $name"
}

install_hook pre-push

echo "hooks installed."
