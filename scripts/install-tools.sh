#!/usr/bin/env bash
# Installs the tools scripts/check.sh needs beyond the Rust toolchain
# (rust-toolchain.toml covers rustc, rustfmt, clippy and the wasm target):
#
#   wasm-bindgen-cli  pinned to the wasm-bindgen version in Cargo.lock
#   rustfilt          rustc-demangle's CLI, pinned by RUSTFILT_VERSION: names
#                     the raw wasm's functions as wasm-bindgen does, so
#                     wasm-split.ts moves each module's data with it
#   brotli            the size gate's compressor
#   bun               the TypeScript packages' test runner, pinned by BUN_VERSION
#   ShellCheck        the gate scripts' linter, pinned by SHELLCHECK_VERSION (an
#                     older one, Ubuntu's 0.9, reports what 0.11 does not)
#   sccache           the compiler cache, pinned by SCCACHE_VERSION (first: the
#                     cargo installs below go through it)
#   og-sccache-wrap   the rustc wrapper .cargo/config.toml names
#                     (crates/og-sccache-wrap): starts sccache's server without
#                     a build's jobserver; built here from this checkout
#   zig, cargo-zigbuild  `build.sh --linux-x64`'s cross build against glibc
#                     2.31 (release packing), pinned by ZIG_VERSION and
#                     CARGO_ZIGBUILD_VERSION with their sha256; Linux x86_64 only
#   wasm-split        Binaryen's module splitter, pinned by BINARYEN_VERSION and
#                     each release archive's sha256: cuts the shipping wasm
#                     into its primary and its on-demand modules
#                     (packages/engine/scripts/wasm-split.ts)
#   tsrs              `og check`'s type checker, pinned by TSRS_VERSION and the
#                     SRI of its npm platform package; the package version
#                     7.1.0-beta.tsrs5 is a binary that reports
#                     `Version 7.1.0-beta` (TSRS_REPORTS), the
#                     string the present check compares
#   wasmtime          the wasm32-wasip1 test runner of the shape-wasm32 step,
#                     pinned by WASMTIME_VERSION and each release archive's
#                     sha256
#
# Each tool is skipped when the right version is already on PATH. Linux x86_64
# and aarch64 download release binaries into OG_TOOLS_BIN (default
# ~/.local/bin, appended to $GITHUB_PATH on Actions); brotli comes from
# Homebrew on macOS. wasm-bindgen falls back to `cargo install --locked`.
set -eu -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"

BUN_VERSION="${BUN_VERSION:-1.4.5}"
SCCACHE_VERSION="${SCCACHE_VERSION:-0.19.0}"
SHELLCHECK_VERSION="${SHELLCHECK_VERSION:-0.11.0}"
RUSTFILT_VERSION="${RUSTFILT_VERSION:-0.2.1}"
TSRS_VERSION="${TSRS_VERSION:-7.1.0-beta.tsrs5}"
TSRS_REPORTS="7.1.0-beta"
BINARYEN_VERSION="${BINARYEN_VERSION:-133}"
ZIG_VERSION="${ZIG_VERSION:-0.17.0}"
ZIG_SHA256=1cbe9df9f27e6b78d14ccbca43b6703a404ef79ef1c463de901d7f088d4e2026
CARGO_ZIGBUILD_VERSION="${CARGO_ZIGBUILD_VERSION:-0.23.4}"
CARGO_ZIGBUILD_SHA256=6b69963040818ca1ef573d8135a95aa902e17cf7f873c32c6426642dadfe7f59
WASMTIME_VERSION="${WASMTIME_VERSION:-51.0.0}"
BIN="${OG_TOOLS_BIN:-$HOME/.local/bin}"
mkdir -p "$BIN"
export PATH="$BIN:$PATH"
[ -n "${GITHUB_PATH:-}" ] && echo "$BIN" >> "$GITHUB_PATH"

os="$(uname -s)"; arch="$(uname -m)"
case "$arch" in arm64) arch=aarch64 ;; esac
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

fetch() { curl -fsSL --retry 3 -o "$2" "$1"; }

# Install the pinned LLVM nightly before cargo installs.
rustup show active-toolchain
if [ "$os" = Linux ] && ! command -v mold >/dev/null; then
  if [ "$(id -u)" -eq 0 ]; then apt-get update -qq && apt-get install -y -qq mold
  else sudo apt-get update -qq && sudo apt-get install -y -qq mold; fi
fi

# sccache, before anything cargo builds here: .cargo/config.toml makes it
# the rustc wrapper (RUSTC_WRAPPER= turns it off, as CI does).
if command -v sccache >/dev/null && [ "$(sccache --version | awk '{print $2}')" = "$SCCACHE_VERSION" ]; then
  og_say "sccache: present ($(sccache --version))"
else
  # A running server keeps the old executable and protocol after replacement.
  if command -v sccache >/dev/null; then sccache --stop-server >/dev/null 2>&1 || true; fi
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

# og-sccache-wrap, built from this checkout without itself (RUSTC_WRAPPER=)
# every time (a std-only crate: seconds), and moved into place by a rename,
# so a compile running the old one keeps it (docs/ci.md, The compiler cache).
og_say "og-sccache-wrap: installing from crates/og-sccache-wrap"
RUSTC_WRAPPER="" cargo install --locked --quiet --path "$(dirname "$0")/../crates/og-sccache-wrap" --root "$tmp/wrap"
for exe in "$tmp/wrap/bin/og-sccache-wrap" "$tmp/wrap/bin/og-sccache-wrap.exe"; do
  [ -f "$exe" ] || continue
  name="$(basename "$exe")"
  cp "$exe" "$BIN/.$name.new"
  mv -f "$BIN/.$name.new" "$BIN/$name"
done
command -v og-sccache-wrap >/dev/null || { og_err "og-sccache-wrap: $BIN is not on PATH"; exit 1; }
# Windows runs only an .exe as cargo's wrapper: .cargo/config.toml's
# scripts/rustc-wrapper is found there as scripts/rustc-wrapper.exe
# (ignored by Git), this same wrapper, renamed into place like the above.
if [ -f "$tmp/wrap/bin/og-sccache-wrap.exe" ]; then
  wrapper="$(dirname "$0")/rustc-wrapper.exe"
  cp "$tmp/wrap/bin/og-sccache-wrap.exe" "$wrapper.new"
  mv -f "$wrapper.new" "$wrapper"
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

# rustfilt (no release binaries)
if command -v rustfilt >/dev/null && [ "$(rustfilt --version | awk '{print $2}')" = "$RUSTFILT_VERSION" ]; then
  og_say "rustfilt $RUSTFILT_VERSION: present"
else
  og_say "rustfilt $RUSTFILT_VERSION: installing (cargo install --locked)"
  cargo install --locked --version "$RUSTFILT_VERSION" rustfilt --root "$tmp/cargo"
  cp "$tmp/cargo/bin/rustfilt" "$BIN/"
  rustfilt --version
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

# wasm-split: Binaryen's release archive, checked against its sha256 (the
# release's own .sha256 files); wasm-split and wasm-opt (whose call graph
# wasm-split.ts reads) are installed. The macOS binaries need the companion dylib.
if command -v wasm-split >/dev/null && [ "$(wasm-split --version | awk '{print $3}')" = "$BINARYEN_VERSION" ] \
    && command -v wasm-opt >/dev/null && [ "$(wasm-opt --version | awk '{print $3}')" = "$BINARYEN_VERSION" ]; then
  og_say "wasm-split and wasm-opt $BINARYEN_VERSION: present"
else
  case "$os-$arch" in
    Linux-x86_64)   b=x86_64-linux;  bsum=2dc9c7813f5375db93d96ead4b78222fcc3e2677bbb832297af4797782a37489 ;;
    Linux-aarch64)  b=aarch64-linux; bsum=89c07ea56faf38d0fbecf36ca8ec0721756716185f265b568e133d427f299bf8 ;;
    Darwin-aarch64) b=arm64-macos;   bsum=ad66da82ac13f163e424b1643f16c6dfcccc98b5966296b43e52d3cab04f84a8 ;;
    Darwin-x86_64)  b=x86_64-macos;  bsum=13a9b90be775c6389ce3d1f879cb8627bea56708ba8c122983941d53a8199b95 ;;
    *) b="" ;;
  esac
  if [ -z "$b" ]; then
    og_say "wasm-split: no Binaryen release for $os-$arch; the release's wasm split needs it"
  else
    og_say "wasm-split $BINARYEN_VERSION: installing"
    a="binaryen-version_$BINARYEN_VERSION-$b.tar.gz"
    fetch "https://github.com/WebAssembly/binaryen/releases/download/version_$BINARYEN_VERSION/$a" "$tmp/$a"
    [ "$( (sha256sum "$tmp/$a" 2>/dev/null || shasum -a 256 "$tmp/$a") | awk '{print $1}')" = "$bsum" ] || { og_err "$a: sha256 is not the pinned $bsum"; exit 1; }
    tar -xzf "$tmp/$a" -C "$tmp"
    if [ "$os" = Darwin ]; then
      mkdir -p "$BIN/../lib"
      cp "$tmp/binaryen-version_$BINARYEN_VERSION/lib/libbinaryen.dylib" "$BIN/../lib/"
    fi
    cp "$tmp/binaryen-version_$BINARYEN_VERSION/bin/wasm-split" "$tmp/binaryen-version_$BINARYEN_VERSION/bin/wasm-opt" "$BIN/"
    wasm-split --version
  fi
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
    Linux-x86_64-gnu)   pkg=linux-x64-gnu;    sri='sha512-hyg70btCCaMT3RvFYU0KEibFyQjFUUfrqk4zTl7rf/7G3HPO3X6sEs8J/AHYUvJB0cA+dXgU/adkiZf1XJI/XA==' ;;
    Linux-aarch64-gnu)  pkg=linux-arm64-gnu;  sri='sha512-o0ofV8Yw5IbaXR7wjIHaqBv9YddJ6Weot+IpYWfUNqG3YpAe4N5G0dU9vsCLX7eNNYx1mMyIZK0eyMV1r6qg2g==' ;;
    Linux-x86_64-musl)  pkg=linux-x64-musl;   sri='sha512-5RI2Ziy8eSkvUi/oeQbrCSzJC/AP7DCtn9IEOWS74BhuQK7WUr1op5ozQMsn4kA6DfuCAuYSQZygTk2Zv3Ib+g==' ;;
    Linux-aarch64-musl) pkg=linux-arm64-musl; sri='sha512-Nki4kxNnp8qO+/8MtZeO0cAt8ifGHPbC9R0BepVlLuDdvR2cqfzym+l75zXtvnwW1nekmKwbNpHqsuHXoITYGQ==' ;;
    Darwin-aarch64-*)   pkg=darwin-arm64;     sri='sha512-wRLJq3IDZjdLrh+lsipipCfYPpFjNW/0zjeM9azNLjA33OWjOIxT2THiKExf7bjkbr6SGCrrjDR+ZZyXYz4Rjg==' ;;
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

# zig and cargo-zigbuild: release packing's Linux x86_64 addon and og
# (packages/engine/scripts/build.sh --linux-x64). Outside the gate; the
# release binaries, checked against their pinned sha256.
sha256_is() { [ "$(sha256sum "$1" | awk '{print $1}')" = "$2" ] || { og_err "$1: sha256 is not the pinned $2"; exit 1; }; }
if [ "$os-$arch" != Linux-x86_64 ]; then
  og_say "zig, cargo-zigbuild: not installed on $os-$arch (release packing runs on Linux x86_64)"
elif [ "${OG_TOOLS_NO_PACKING:-}" = 1 ]; then
  # Explicit opt-out for machines that do not pack Linux releases.
  og_say "zig, cargo-zigbuild: skipped (OG_TOOLS_NO_PACKING=1)"
else
  if command -v zig >/dev/null && [ "$(zig version)" = "$ZIG_VERSION" ]; then
    og_say "zig $ZIG_VERSION: present"
  else
    og_say "zig $ZIG_VERSION: installing"
    z="zig-x86_64-linux-$ZIG_VERSION"
    fetch "https://ziglang.org/download/$ZIG_VERSION/$z.tar.xz" "$tmp/$z.tar.xz"
    sha256_is "$tmp/$z.tar.xz" "$ZIG_SHA256"
    tar -xJf "$tmp/$z.tar.xz" -C "$tmp"
    rm -rf "$BIN/../lib/$z"; mkdir -p "$BIN/../lib"
    cp -R "$tmp/$z" "$BIN/../lib/$z"
    ln -sf "$BIN/../lib/$z/zig" "$BIN/zig"
    zig version
  fi
  if command -v cargo-zigbuild >/dev/null && [ "$(cargo-zigbuild --version | awk '{print $2}')" = "$CARGO_ZIGBUILD_VERSION" ]; then
    og_say "cargo-zigbuild $CARGO_ZIGBUILD_VERSION: present"
  else
    og_say "cargo-zigbuild $CARGO_ZIGBUILD_VERSION: installing"
    c=cargo-zigbuild-x86_64-unknown-linux-gnu.tar.xz
    fetch "https://github.com/rust-cross/cargo-zigbuild/releases/download/v$CARGO_ZIGBUILD_VERSION/$c" "$tmp/$c"
    sha256_is "$tmp/$c" "$CARGO_ZIGBUILD_SHA256"
    mkdir -p "$tmp/czb"; tar -xJf "$tmp/$c" -C "$tmp/czb"
    cp "$(find "$tmp/czb" -type f -name cargo-zigbuild | head -1)" "$BIN/"
    cargo-zigbuild --version
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

# wasmtime: runs og-shape's determinism tests built for wasm32-wasip1 (the
# shape-wasm32 step); the release archive, checked against its sha256.
if command -v wasmtime >/dev/null && [ "$(wasmtime --version | awk '{print $2}')" = "$WASMTIME_VERSION" ]; then
  og_say "wasmtime $WASMTIME_VERSION: present"
else
  case "$os-$arch" in
    Linux-x86_64)   w=x86_64-linux;  wsum=a18020674fac4107f329bc9ed720ea9c88e4b30a61e8a527787bcce051123d32 ;;
    Linux-aarch64)  w=aarch64-linux; wsum=a263f34aefad1b24ba4d9ca635ddfea415f4acfe3f3f1bb6d4525c34fc6a7397 ;;
    Darwin-x86_64)  w=x86_64-macos;  wsum=51767ef35ea3c2653205c09892f174b75e507cf8ffccfcba00ed557832f0e72d ;;
    Darwin-aarch64) w=aarch64-macos; wsum=61dd882b44bcf854b301518781f407b9f6db4c6bd157a55f8f44b1f1357f93dc ;;
    *) w="" ;;
  esac
  if [ -z "$w" ]; then
    og_say "wasmtime: no release binary for $os-$arch; the shape-wasm32 step needs it"
  else
    og_say "wasmtime $WASMTIME_VERSION: installing"
    a="wasmtime-v$WASMTIME_VERSION-$w"
    fetch "https://github.com/bytecodealliance/wasmtime/releases/download/v$WASMTIME_VERSION/$a.tar.xz" "$tmp/$a.tar.xz"
    [ "$( (sha256sum "$tmp/$a.tar.xz" 2>/dev/null || shasum -a 256 "$tmp/$a.tar.xz") | awk '{print $1}')" = "$wsum" ] || { og_err "$a.tar.xz: sha256 is not the pinned $wsum"; exit 1; }
    tar -xJf "$tmp/$a.tar.xz" -C "$tmp"
    cp "$tmp/$a/wasmtime" "$BIN/"
    wasmtime --version
  fi
fi
