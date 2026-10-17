#!/bin/sh
# Keeps a spike page open in headless Chrome for Testing for N seconds (GPU via ANGLE Metal);
# the page POSTs its result to the server. Console output goes to $LOG.
# usage: run-chrome.sh <url> <seconds>
url="$1"; secs="${2:-10}"
chrome="${CHROME:-$HOME/.agent-browser/browsers/chrome-150.0.7871.115/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing}"
profile=$(mktemp -d)
"$chrome" --headless=new --no-first-run --no-default-browser-check --user-data-dir="$profile" \
  --window-size=1280,720 --hide-scrollbars --use-angle=metal --enable-unsafe-webgpu --enable-features=WebGPU \
  --ignore-gpu-blocklist --autoplay-policy=no-user-gesture-required --disable-frame-rate-limit --disable-gpu-vsync --enable-logging=stderr --v=0 \
  --remote-debugging-port=0 "$url" > "${LOG:-/dev/null}" 2>&1 &
pid=$!
sleep "$secs"
kill $pid 2>/dev/null; wait $pid 2>/dev/null; rm -rf "$profile"
