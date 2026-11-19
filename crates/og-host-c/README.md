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
runs the display loop. Not yet: audio, gamepads, platform services, and
the Hermes API that detaches a buffer without its test methods.

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

## Running a game under Hermes

```sh
# Hermes (tag hermes-v260318099.0.8): hermesc, libhermesvm, libjsi
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
