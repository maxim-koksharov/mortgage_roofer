#!/bin/bash
set -e

USER_ID=${CONTAINER_UID:-1000}
GROUP_ID=${CONTAINER_GID:-1000}

# Create/update group with the host GID
if ! getent group developer >/dev/null 2>&1; then
    groupadd -g "$GROUP_ID" developer
else
    groupmod -g "$GROUP_ID" developer
fi

# Ensure home directory exists and is owned correctly before creating the user.
# Docker creates mount-point directories as root before the entrypoint runs,
# so /home/developer may already exist and be owned by root.
mkdir -p /home/developer
chown "$USER_ID:$GROUP_ID" /home/developer

# Create/update user with the host UID (do NOT use -m because home already exists)
if ! id -u developer >/dev/null 2>&1; then
    useradd -u "$USER_ID" -g "$GROUP_ID" -d /home/developer -M -s /bin/bash developer
else
    usermod -u "$USER_ID" -g "$GROUP_ID" developer
fi

# Ensure opencode data directories exist
mkdir -p /home/developer/.config/opencode
mkdir -p /home/developer/.local/share/opencode
mkdir -p /home/developer/.cache/opencode
mkdir -p /home/developer/.local/state/opencode

# Use a container-local snapshot to avoid the host's bloated snapshot (4.2 GB
# of nested git repos from the previous config iteration).
SNAPSHOT_DIR="/home/developer/.local/share/opencode/snapshot-container"

# Tell git this directory is safe before any git operation
git config --global --add safe.directory "$SNAPSHOT_DIR" || true

# Initialize snapshot git repo if it is missing.
# A missing/corrupted snapshot is what causes opencode to spawn many git processes.
if [ ! -d "$SNAPSHOT_DIR/.git" ]; then
    mkdir -p "$SNAPSHOT_DIR"
    git init "$SNAPSHOT_DIR"
    cd "$SNAPSHOT_DIR"
    git config user.name "OpenCode"
    git config user.email "opencode@localhost"
    git commit --allow-empty -m "init snapshot" || true
fi

# Fix ownership of the whole home directory so opencode can write its files
chown -R "$USER_ID:$GROUP_ID" /home/developer

# Drop privileges and execute the requested command
exec gosu developer "$@"
