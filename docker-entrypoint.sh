#!/bin/bash
set -e

USER_ID=${CONTAINER_UID:-1000}
GROUP_ID=${CONTAINER_GID:-1000}

if ! getent group developer >/dev/null 2>&1; then
    groupadd -g "$GROUP_ID" developer
else
    groupmod -g "$GROUP_ID" developer
fi

# /home/developer lives inside the container - never mount host home here
if ! id -u developer >/dev/null 2>&1; then
    useradd -u "$USER_ID" -g "$GROUP_ID" -d /home/developer -m -s /bin/bash developer
else
    usermod -u "$USER_ID" -g "$GROUP_ID" -d /home/developer developer
fi

mkdir -p /home/developer/.config/opencode
mkdir -p /home/developer/.local/share/opencode
mkdir -p /home/developer/.cache/opencode
mkdir -p /home/developer/.local/state/opencode
chown -R "$USER_ID:$GROUP_ID" /home/developer

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

# Only chown the project mount - never touch host home
if [ -d /workspace ]; then
    chown -R "$USER_ID:$GROUP_ID" /workspace
fi

exec gosu developer "$@"
