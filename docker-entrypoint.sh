#!/bin/bash
set -e

mkdir -p /home/developer/.config/opencode
mkdir -p /home/developer/.local/share/opencode
mkdir -p /home/developer/.cache/opencode
mkdir -p /home/developer/.local/state/opencode

SNAPSHOT_DIR="/home/developer/.local/share/opencode/snapshot-container"

git config --global --add safe.directory "$SNAPSHOT_DIR" || true
git config --global --add safe.directory /workspace || true

if [ ! -d "$SNAPSHOT_DIR/.git" ]; then
    mkdir -p "$SNAPSHOT_DIR"
    git init "$SNAPSHOT_DIR"
    cd "$SNAPSHOT_DIR"
    git config user.name "OpenCode"
    git config user.email "opencode@localhost"
    git commit --allow-empty -m "init snapshot" || true
fi

cd /workspace

exec "$@"
