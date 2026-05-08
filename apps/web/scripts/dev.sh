#!/bin/sh
# Run by Moon `web:dev`: Tailwind watch (rebuilds dist/styles.css) + templ watch + static serve.
set -e

cleanup() {
  kill "$TAILWIND_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

bunx @tailwindcss/cli -i src/styles.css -o dist/styles.css --watch=always &
TAILWIND_PID=$!

go tool templ generate --watch --open-browser=false \
  --proxy="http://localhost:${WEB_PORT:-3000}" \
  --cmd "go run . --serve --port ${WEB_PORT:-3000}"
