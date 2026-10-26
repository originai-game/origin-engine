#!/usr/bin/env bash
# Installs the tools scripts/check.sh needs beyond the Rust toolchain
# (rust-toolchain.toml covers rustc, rustfmt, clippy and the wasm target):
#
#   wasm-bindgen-cli  pinned to the wasm-bindgen version in Cargo.lock
#   wasm-opt          binaryen, pinned by BINARYEN_VERSION
#   brotli            the size gate's compressor
#   bun               the TypeScript packages' test runner, pinned by BUN_VERSION
#   ShellCheck        the gate scripts' linter, pinned by SHELLCHECK_VERSION (an
#                     older one, Ubuntu's 0.9, reports what 0.11 does not)
#   sccache           the compiler cache .cargo/config.toml names, pinned by
#                     SCCACHE_VERSION (first: the cargo installs below go through it)
#   tsrs              `og check`'s type checker, pinned by TSRS_VERSION and the
#                     SRI of its npm platform package; the package version
#                     0.18.0-ts7.1.0-dev.20260929 is a binary that reports
#                     `Version 7.1.0-dev.20260930.4` (TSRS_REPORTS), the
#                     string the present check compares
#
# Each tool is skipped when the right version is already on PATH. Linux x86_64
# and aarch64 download release binaries into OG_TOOLS_BIN (default
# ~/.local/bin, appended to $GITHUB_PATH on Actions); binaryen comes from its
# release archive on macOS as well, brotli from Homebrew there. wasm-bindgen
# falls back to `cargo install --locked`.
set -eu -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"

BINARYEN_VERSION="${BINARYEN_VERSION:-133}"
BUN_VERSION="${BUN_VERSION:-1.4.4}"
SCCACHE_VERSION="${SCCACHE_VERSION:-0.14.0}"
SHELLCHECK_VERSION="${SHELLCHECK_VERSION:-0.11.0}"
TSRS_VERSION="${TSRS_VERSION:-0.18.0-ts7.1.0-dev.20260929}"
TSRS_REPORTS="7.1.0-dev.20260930.4"
BIN="${OG_TOOLS_BIN:-$HOME/.local/bin}"
mkdir -p "$BIN"
export PATH="$BIN:$PATH"
[ -n "${GITHUB_PATH:-}" ] && echo "$BIN" >> "$GITHUB_PATH"

os="$(uname -s)"; arch="$(uname -m)"
case "$arch" in arm64) arch=aarch64 ;; esac
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

fetch() { curl -fsSL --retry 3 -o "$2" "$1"; }

# sccache, before anything cargo builds here: .cargo/config.toml makes it
# the rustc wrapper (RUSTC_WRAPPER= turns it off, as CI does).
if command -v sccache >/dev/null; then
  og_say "sccache: present ($(sccache --version))"
else
  og_say "sccache $SCCACHE_VERSION: installing"
  case "$os-$arch" in
    Linux-x86_64)   triple=x86_64-unknown-linux-musl ;;
    Linux-aarch64)  triple=aarch64-unknown-linux-musl ;;
    Darwin-x86_64)  triple=x86_64-apple-darwin ;;
    Darwin-aarch64) triple=aarch64-apple-darwin ;;
    *) triple="" ;;
  esac
  url="https://github.com/mozilla/sccache/releases/download/v$SCCACHE_VERSION/sccache-v$SCCACHE_VERSION-$triple.tar.gz"
  if [ -n "$triple" ] && fetch "$url" "$tmp/sccache.tgz" && tar -xzf "$tmp/sccache.tgz" -C "$tmp"; then
    cp "$tmp/sccache-v$SCCACHE_VERSION-$triple/sccache" "$BIN/"
  else
    og_say "sccache: no release binary for $os-$arch; cargo install"
    RUSTC_WRAPPER="" cargo install --locked --version "$SCCACHE_VERSION" sccache --root "$tmp/cargo"
    cp "$tmp/cargo/bin/sccache" "$BIN/"
  fi
  sccache --version
fi

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

# binaryen (wasm-opt), from its release archive on every platform (Homebrew's
# is whatever version is current, so macOS used to drift from the pin)
if command -v wasm-opt >/dev/null && [ "$(wasm-opt --version | awk '{print $3}')" = "$BINARYEN_VERSION" ]; then
  og_say "wasm-opt $BINARYEN_VERSION: present"
else
  og_say "wasm-opt $BINARYEN_VERSION: installing"
  case "$os-$arch" in
    Linux-x86_64) b=x86_64-linux ;; Linux-aarch64) b=aarch64-linux ;;
    Darwin-x86_64) b=x86_64-macos ;; Darwin-aarch64) b=arm64-macos ;;
    *) og_err "binaryen: no release archive for $os-$arch"; exit 1 ;;
  esac
  tgz="binaryen-version_$BINARYEN_VERSION-$b.tar.gz"
  base="https://github.com/WebAssembly/binaryen/releases/download/version_$BINARYEN_VERSION"
  fetch "$base/$tgz" "$tmp/$tgz"
  fetch "$base/$tgz.sha256" "$tmp/$tgz.sha256"
  if command -v sha256sum >/dev/null; then (cd "$tmp" && sha256sum -c "$tgz.sha256" >/dev/null)
  else (cd "$tmp" && shasum -a 256 -c "$tgz.sha256" >/dev/null); fi
  tar -xzf "$tmp/$tgz" -C "$tmp"
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

# ShellCheck: the release binary on Linux, Homebrew's on macOS
if command -v shellcheck >/dev/null && [ "$(shellcheck --version | awk '/^version:/ {print $2}')" = "$SHELLCHECK_VERSION" ]; then
  og_say "shellcheck $SHELLCHECK_VERSION: present"
elif [ "$os" = Darwin ]; then
  if command -v shellcheck >/dev/null; then og_say "shellcheck: present ($(shellcheck --version | awk '/^version:/ {print $2}'), Homebrew's)"
  else brew install shellcheck; fi
else
  og_say "shellcheck $SHELLCHECK_VERSION: installing"
  sc="shellcheck-v$SHELLCHECK_VERSION.linux.$arch.tar.xz"
  fetch "https://github.com/koalaman/shellcheck/releases/download/v$SHELLCHECK_VERSION/$sc" "$tmp/$sc"
  tar -xJf "$tmp/$sc" -C "$tmp"
  cp "$tmp/shellcheck-v$SHELLCHECK_VERSION/shellcheck" "$BIN/"
  shellcheck --version | awk '/^version:/ {print "shellcheck " $2}'
fi

# tsrs: the npm platform package @tsrs/<platform> (GitHub releases carry no
# assets), checked against its pinned dist.integrity (`npm view
# @tsrs/<platform>@<version> dist.integrity`; all five move with
# TSRS_VERSION, and packages/engine/package.json's optionalDependencies
# copy the version). The binary reads lib.*.d.ts from its own directory, so
# the whole package goes to $BIN/../lib/tsrs-<version>/ and $BIN/tsrs is a
# symlink. No package for Windows and Intel macOS: og check falls back to tsgo.
if command -v tsrs >/dev/null && [ "$(tsrs --version | awk '{print $2}')" = "$TSRS_REPORTS" ]; then
  og_say "tsrs $TSRS_VERSION: present"
else
  libc=gnu
  if [ "$os" = Linux ] && ldd --version 2>&1 | grep -qi musl; then libc=musl; fi
  case "$os-$arch-$libc" in
    Linux-x86_64-gnu)   pkg=linux-x64-gnu;    sri='sha512-EdBwBgNICGu1r1h23ixdcKwsMo6E9ll7ZsY3mNERNYqE1Pwdtcat99d5E7zB1J+oy/Xk6YqiizaB6nolIjpDTw==' ;;
    Linux-aarch64-gnu)  pkg=linux-arm64-gnu;  sri='sha512-zm9/z9yylig68JiZF4Aq0Cm26n09KEU8YoQRlG7tdDWTBNAIViYw8knxnWYsjMWVZ2XiU4mYq1xdrOKuOgABHg==' ;;
    Linux-x86_64-musl)  pkg=linux-x64-musl;   sri='sha512-LHk+AOAtXuTlhJhOJ2+elriEIX17ycXljojeNxrrzckOipTG1LksoVWuCSas2nu00EnWAzrdKpt9oqPylUKZfQ==' ;;
    Linux-aarch64-musl) pkg=linux-arm64-musl; sri='sha512-Lw7VVU/YQ9JcarODICEiwzMLdFqZprqh5RGTgbS5Uc6I95P635/g5xA1pOC8RVs4efBvAR30J54iDG3l5JEYiw==' ;;
    Darwin-aarch64-*)   pkg=darwin-arm64;     sri='sha512-sAirMpaGA9uRQm2W+t4zC7PjnijNuFa4PCSIfLKKo0JyAUCRfnMeev0UGBQ9F3g+9Nz187tvikUs5Irh5+eajA==' ;;
    *) pkg="" ;;
  esac
  if [ -z "$pkg" ]; then
    og_say "tsrs: no package for $os-$arch; og check falls back to tsgo (bun x typescript)"
  else
    og_say "tsrs $TSRS_VERSION: installing (@tsrs/$pkg)"
    fetch "https://registry.npmjs.org/@tsrs/$pkg/-/$pkg-$TSRS_VERSION.tgz" "$tmp/tsrs.tgz"
    got="sha512-$(openssl dgst -sha512 -binary "$tmp/tsrs.tgz" | openssl base64 -A)"
    if [ "$got" != "$sri" ]; then
      og_err "tsrs: @tsrs/$pkg-$TSRS_VERSION.tgz is $got, pinned $sri"; exit 1
    fi
    tar -xzf "$tmp/tsrs.tgz" -C "$tmp"
    rm -rf "$BIN/../lib/tsrs-$TSRS_VERSION"; mkdir -p "$BIN/../lib"
    cp -R "$tmp/package" "$BIN/../lib/tsrs-$TSRS_VERSION"
    ln -sf "$BIN/../lib/tsrs-$TSRS_VERSION/tsrs" "$BIN/tsrs"
    tsrs --version
  fi
fi

# cargo-nextest: the gate's parallel test runner (optional; check.sh falls
# back to cargo test without it)
if command -v cargo-nextest >/dev/null; then
  og_say "cargo-nextest: present ($(cargo nextest --version | head -1))"
else
  og_say "cargo-nextest: installing (cargo install --locked)"
  cargo install --locked cargo-nextest >/dev/null
fi
