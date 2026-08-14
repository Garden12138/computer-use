#!/usr/bin/env bash
set -euo pipefail
# Manual acceptance: dedicated Chrome profile + native Webpage Complete save.
# Usage: examples/save_page.sh [url] [out.html]
URL="${1:-https://example.com}"
OUT="${2:-./out/page.html}"
computer-use doctor --json
computer-use browser-open-profile ComputerUse
computer-use browser-open-url "$URL"
computer-use wait 2
computer-use --pacing normal browser-save-page "$OUT" --scrolls 8
ls -la "$(dirname "$OUT")"
