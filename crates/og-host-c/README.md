# og-host-c

The engine's third host: every binding symbol of `crates/og-api/api/*.ogt`
as an `extern "C"` entry over tagged values (`include/og_host.h`), for a
native shell that embeds its own JavaScript engine. `src/generated/glue.rs`
is `og api gen`'s; every binding is `og_runtime::host_impl`'s, as on the web
and napi hosts.

`hermes/shell.cpp` is the desktop shell: Hermes over JSI. With the
`window` feature (default) the host opens one window (`src/window.rs`,
winit): the engine's renderer presents to its surface, and its keys,
pointer and wheel reach the engine as the web shell's `InjectedInput`. The
game's bundle (`packages/engine/src/hermes/shell.ts`) opens it; the shell
runs the display loop. The engine's audio plays on the default output
device, and gamepads (gilrs) arrive as the web shell's `gamepad` events;
each is logged when it connects. On Linux the build needs libudev's
headers (`libudev-dev`).

og.platform is the local document, saved as `<data>/<title>/platform.json`
in the player's data directory (Application Support, %APPDATA%, the XDG
data home; `OG_DATA_DIR` overrides it), unless the package bundles a
store's provider: `package.ts --provider <module.ts>` (its default export a
`ShellProviderFactory`, `@origingame/engine/provider`) and `--provider-lib
<lib>`, a library beside the shell exporting `const char
*og_provider_call(const char *method, const char *json)` (`src/platform.rs`).
The store's SDK and credentials live there, never in the engine.

## A desktop package

```sh
cargo build --release -p og-host-c
(cd packages/engine && bun src/hermes/package.ts ../../showcase/night-shift /tmp/pkg \
  --hermes hermes --hermes-build hermes/build)
/tmp/pkg/night-shift/night-shift        # macOS: open /tmp/pkg/night-shift.app
```

The package is the shell, `libhermesvm` and `og_host_c` beside it, the
game as `game.hbc`, `project.ogt`, `cooked/` and `shell.env` (title and
size). `OG_SHELL_FRAMES=<n>` closes it after n frames and prints the frame
times; `OG_SHELL_CAPTURE=<file.png>` writes the last frame.

On Windows, build Hermes inside vcvars64 with
`-DCMAKE_C_FLAGS=/utf-8 -DCMAKE_CXX_FLAGS="/utf-8 /EHsc"` (its sources are
UTF-8, which MSVC misreads on other code pages), and have LLVM's `clang++`
on PATH. A showcase's own pinned assets come from
`bun scripts/assets.ts ensure showcase/<name>`, beside `ensure templates`.

## Running a game under Hermes

```sh
# Hermes (tag hermes-v260318099.0.8) with third_party/hermes's patch, which
# adds IHermesArrayBufferDetach (the shell detaches column buffers before they
# move; Hermes has no public way to): hermesc, libhermesvm, libjsi
git -C hermes apply $PWD/third_party/hermes/detach-array-buffer.patch
cmake -S hermes -B hermes/build -G Ninja -DCMAKE_BUILD_TYPE=Release \
  -DHERMES_ENABLE_TEST_SUITE=OFF
ninja -C hermes/build hermesc hermesvm jsi

cargo build --release -p og-host-c
H=hermes; clang++ -std=c++17 -O2 -I$H/API -I$H/API/jsi -I$H/public \
  -I$H/build/include -Icrates/og-host-c/include crates/og-host-c/hermes/shell.cpp \
  -o og-hermes -L$H/build/lib -lhermesvm $H/build/jsi/libjsi.a \
  -Ltarget/release -log_host_c -Wl,-rpath,$H/build/lib -Wl,-rpath,$PWD/target/release

# a cooked project's bundle, compiled with block scoping (without it Hermes
# gives a loop's let and const one binding for the whole function)
(cd packages/engine && bun bench/game-frame/build.ts ../../showcase/night-shift /tmp/ns.js hermes)
hermesc -O -Xes6-block-scoping -emit-binary -out /tmp/ns.hbc /tmp/ns.js
OG_RENDERER=none ./og-hermes /tmp/ns.hbc OG_BENCH_ROOT=showcase/night-shift \
  OG_BENCH_PROJECT=@showcase/night-shift/project.ogt
```

`OG_HERMES_PROFILE=<file>` writes Hermes's sampling profile (Chrome's trace
format). The same bundle for node or bun is `build.ts ... node`.
