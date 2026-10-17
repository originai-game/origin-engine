#!/usr/bin/env bash
# Installs the tools scripts/check.sh needs beyond the Rust toolchain
# (rust-toolchain.toml covers rustc, rustfmt, clippy and the wasm target):
#
#   wasm-bindgen-cli  pinned to the wasm-bindgen version in Cargo.lock
#   wasm-opt          binaryen, pinned by BINARYEN_VERSION
#   brotli            the size gate's compressor
#   bun               the TypeScript packages' test runner, pinned by BUN_VERSION
#
# Each tool is skipped when the right version is already on PATH. Linux x86_64
# and aarch64 download release binaries into OG_TOOLS_BIN (default
# ~/.local/bin, appended to $GITHUB_PATH on Actions); macOS uses Homebrew for
# binaryen and brotli. wasm-bindgen falls back to `cargo install --locked`.
set -eu -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"

BINARYEN_VERSION="${BINARYEN_VERSION:-123}"
BUN_VERSION="${BUN_VERSION:-1.4.2}"
BIN="${OG_TOOLS_BIN:-$HOME/.local/bin}"
mkdir -p "$BIN"
export PATH="$BIN:$PATH"
[ -n "${GITHUB_PATH:-}" ] && echo "$BIN" >> "$GITHUB_PATH"

os="$(uname -s)"; arch="$(uname -m)"
case "$arch" in arm64) arch=aarch64 ;; esac
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

fetch() { curl -fsSL --retry 3 -o "$2" "$1"; }

# wasm-bindgen-cli
wb="$(og_lock_version wasm-bindgen)"
if [ -z "$wb" ]; then
  og_say "wasm-bindgen: not in Cargo.lock; skipped"
elif command -v wasm-bindgen >/dev/null && [ "$(wasm-bindgen --version | awk '{print $2}')" = "$wb" ]; then
  og_say "wasm-bindgen $wb: present"
else
  og_say "wasm-bindgen $wb: installing"
  case "$os-$arch" in
    Linux-x86_64)  triple=x86_64-unknown-linux-musl ;;
    Linux-aarch64) triple=aarch64-unknown-linux-gnu ;;
    Darwin-x86_64) triple=x86_64-apple-darwin ;;
    Darwin-aarch64) triple=aarch64-apple-darwin ;;
    *) triple="" ;;
  esac
  url="https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$wb/wasm-bindgen-$wb-$triple.tar.gz"
  if [ -n "$triple" ] && fetch "$url" "$tmp/wb.tgz" && tar -xzf "$tmp/wb.tgz" -C "$tmp"; then
    cp "$tmp/wasm-bindgen-$wb-$triple/wasm-bindgen" "$BIN/"
  else
    og_say "wasm-bindgen: no release binary for $os-$arch; cargo install"
    cargo install --locked --version "$wb" wasm-bindgen-cli --root "$tmp/cargo"
    cp "$tmp/cargo/bin/wasm-bindgen" "$BIN/"
  fi
  wasm-bindgen --version
fi

# binaryen (wasm-opt)
if command -v wasm-opt >/dev/null && [ "$(wasm-opt --version | awk '{print $3}')" = "$BINARYEN_VERSION" ]; then
  og_say "wasm-opt $BINARYEN_VERSION: present"
elif [ "$os" = Darwin ]; then
  brew install binaryen
else
  og_say "wasm-opt $BINARYEN_VERSION: installing"
  case "$arch" in x86_64) b=x86_64-linux ;; aarch64) b=aarch64-linux ;; *) og_err "binaryen: unsupported arch $arch"; exit 1 ;; esac
  fetch "https://github.com/WebAssembly/binaryen/releases/download/version_$BINARYEN_VERSION/binaryen-version_$BINARYEN_VERSION-$b.tar.gz" "$tmp/binaryen.tgz"
  tar -xzf "$tmp/binaryen.tgz" -C "$tmp"
  rm -rf "$BIN/../lib/binaryen-$BINARYEN_VERSION"; mkdir -p "$BIN/../lib"
  cp -R "$tmp/binaryen-version_$BINARYEN_VERSION" "$BIN/../lib/binaryen-$BINARYEN_VERSION"
  ln -sf "$BIN/../lib/binaryen-$BINARYEN_VERSION/bin/wasm-opt" "$BIN/wasm-opt"
  wasm-opt --version
fi

# brotli
if command -v brotli >/dev/null; then
  og_say "brotli: present"
elif [ "$os" = Darwin ]; then
  brew install brotli
else
  og_say "brotli: installing"
  if [ "$(id -u)" -eq 0 ]; then apt-get update -qq && apt-get install -y -qq brotli
  else sudo apt-get update -qq && sudo apt-get install -y -qq brotli; fi
fi

# bun
if command -v bun >/dev/null && [ "$(bun --version)" = "$BUN_VERSION" ]; then
  og_say "bun $BUN_VERSION: present"
else
  og_say "bun $BUN_VERSION: installing"
  fetch https://bun.sh/install "$tmp/bun-install.sh"
  BUN_INSTALL="$tmp/bun" bash "$tmp/bun-install.sh" "bun-v$BUN_VERSION" >/dev/null
  cp "$tmp/bun/bin/bun" "$BIN/bun"
  bun --version
fi
