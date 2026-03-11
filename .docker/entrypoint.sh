#!/usr/bin/env bash
# Copy the host-mounted opencode config into the agent's config directory.
# The config is mounted read-only at /run/host-opencode-config/opencode.json.
# We copy it on every startup so changes on the host are picked up.

CONFIG_SRC="/Users/eren/.config/opencode/opencode.json"
CONFIG_DST="${HOME}/.config/opencode/opencode.json"

if [ -f "${CONFIG_SRC}" ]; then
    mkdir -p "$(dirname "${CONFIG_DST}")"
    cp "${CONFIG_SRC}" "${CONFIG_DST}"
    echo "[entrypoint] Copied opencode config from ${CONFIG_SRC}"
else
    echo "[entrypoint] No opencode config found at ${CONFIG_SRC}, skipping"
fi

# Hand off to the original opencode entrypoint / default CMD.
exec "$@"
