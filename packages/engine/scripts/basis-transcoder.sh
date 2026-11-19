#!/bin/sh
# Builds vendor/basis/basis_transcoder.js and basis_transcoder.wasm from
# Binomial's Basis Universal at UPSTREAM (v1_50_0_2, whose prebuilt
# webgl/transcoder/build/ is what three@0.180.0 ships), with that tag's
# webgl/transcoder CMake build (C++17 instead of 11, below) plus these link
# flags:
#   - -s DYNAMIC_EXECUTION=0 -s EMBIND_AOT=1: the glue never calls eval() or
#     new Function() (embind's wrappers are generated at build time), so a
#     page whose Content-Security-Policy has no 'unsafe-eval' loads it. These
#     are what upstream's later NO_DYNAMIC_EXECUTION option adds;
#   - -s EXPORTED_RUNTIME_METHODS=['HEAP8']: the wrappers read Module.HEAP8,
#     which this Emscripten no longer exports unasked (upstream adds it too);
#   - -s INCOMING_MODULE_JS_API=['wasmBinary']: src/web/basis.ts hands the
#     module its wasm bytes (a build serves the file under a hashed name),
#     which Emscripten 7 no longer reads unasked; it is the only option the
#     engine passes;
#   - -s EXPORT_ES6=1: the script is an ES module (`export default BASIS`),
#     which src/web/basis.ts imports.
# Needs git, cmake and Emscripten EMSCRIPTEN on PATH (emsdk: `./emsdk
# install <version> && ./emsdk activate <version> && . ./emsdk_env.sh`).
#
#   sh packages/engine/scripts/basis-transcoder.sh
set -eu

UPSTREAM=b76a431c6a39c07fe8bb2edf1cbf44781150ed6b # tag v1_50_0_2
EMSCRIPTEN=7.0.0

here=$(cd "$(dirname "$0")/../vendor/basis" && pwd)
have=$(emcc --version 2>/dev/null | sed -n '1s/.* \([0-9][0-9.]*\) .*/\1/p')
[ "$have" = "$EMSCRIPTEN" ] || { echo "build.sh: needs emcc $EMSCRIPTEN on PATH, found '${have:-none}'" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
git -C "$work" init -q src
git -C "$work/src" fetch -q --depth 1 https://github.com/BinomialLLC/basis_universal.git "$UPSTREAM"
git -C "$work/src" checkout -q FETCH_HEAD
# The tag's CMakeLists asks for CMake 3.0, which CMake 4 no longer accepts,
# and C++11, where this Emscripten's embind needs C++17 (upstream's own
# CMakeLists moved to C++17 since).
cmake_lists="$work/src/webgl/transcoder/CMakeLists.txt"
grep -q 'set(CMAKE_CXX_STANDARD 11)' "$cmake_lists" || { echo "build.sh: $cmake_lists no longer sets CMAKE_CXX_STANDARD 11" >&2; exit 1; }
sed -i 's/set(CMAKE_CXX_STANDARD 11)/set(CMAKE_CXX_STANDARD 17)/' "$cmake_lists"
emcmake cmake -S "$work/src/webgl/transcoder" -B "$work/build" -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
  -DCMAKE_EXE_LINKER_FLAGS="-s DYNAMIC_EXECUTION=0 -s EMBIND_AOT=1 -s EXPORTED_RUNTIME_METHODS=['HEAP8'] -s INCOMING_MODULE_JS_API=['wasmBinary'] -s EXPORT_ES6=1" >/dev/null
cmake --build "$work/build" -j >/dev/null
cp "$work/build/basis_transcoder.js" "$work/build/basis_transcoder.wasm" "$here/"
sha256sum "$here/basis_transcoder.js" "$here/basis_transcoder.wasm"
