#!/usr/bin/env bash

set -euo pipefail

workspace_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
frontend_dir="$workspace_root/frontend"
target_port="${PORT:-3000}"

existing_pid=""

if command -v ss >/dev/null 2>&1; then
  existing_pid="$(
    ss -ltnp "( sport = :$target_port )" 2>/dev/null \
      | sed -n 's/.*pid=\([0-9]\+\).*/\1/p' \
      | head -n 1
  )"
fi

if [ -n "$existing_pid" ]; then
  process_command="$(ps -p "$existing_pid" -o args= 2>/dev/null || true)"
  if [[ "$process_command" == *"$frontend_dir"* ]] || [[ "$process_command" == *"next-server"* ]]; then
    echo "Stopping existing frontend dev server on port $target_port (pid $existing_pid)."
    kill "$existing_pid"

    for _ in $(seq 1 30); do
      if ! ps -p "$existing_pid" >/dev/null 2>&1; then
        break
      fi
      sleep 0.2
    done
  else
    echo "Port $target_port is in use by an unrelated process. Refusing to kill it automatically." >&2
    exit 1
  fi
fi

cd "$frontend_dir"
npm run wasm:build
exec next dev --port "$target_port"
