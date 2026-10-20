# og-cli

The `og` tool (design §2.1): `og run`, `og dev`, `og test`, `og polish`,
`og build`, `og cook`, `og probe`, `og doctor`, `og api`, `og sfx`,
`og skin`. Each command is a module of `src/`; the TypeScript halves run in
bun from the engine package (`packages/engine/src/node/`), found through
`OG_ENGINE_PACKAGE`, the project's `node_modules/@origingame/engine` or the
checkout's `packages/engine`. Its tests (`tests/cli.rs`) run the built
binary (`env!("CARGO_BIN_EXE_og")`), with a stand-in engine package where
the real one would need a build.

## Signals and the processes og starts

`og` passes SIGINT, SIGTERM and SIGHUP on to the bun process it runs
(`src/relay.rs`), so a supervisor's stop, a closed terminal or Ctrl-C never
ends `og` alone and leaves its child running:

- **`og dev`** starts the game (`bun src/node/run.ts --watch`) in a process
  group of its own and sends each signal to the whole group, so the bun
  process and anything it started (a browser, a server, a helper process)
  stop with it. It waits up to 3 s for them, kills whatever is left of the
  group, and then ends by the signal it got (its default action, so the
  supervisor sees `killed by SIGTERM`, not an exit code). A game that stops
  on its own ends `og dev` with exit 1 and what is left of its group is
  killed; `--exit-after N` stops the group the same way. SIGKILL to `og`
  cannot be passed on: the group keeps running (stop it with
  `kill -- -<pgid>`, the bun process's pid).
- **`og run --device`** passes the signals to its bun process
  (`src/node/device.ts`), which undoes its adb forwards, Chrome profile,
  WebDriver session and server and exits 130 (SIGINT), 143 (SIGTERM) or
  129 (SIGHUP); `og` waits for it and exits with its code.

`tests/cli.rs` `dev_stopped_by_a_signal_stops_its_game_and_what_it_started`
sends each of the three signals to `og dev` running a stand-in game that
started a child of its own, and checks that `og` ends by that signal and
that neither process is left.
