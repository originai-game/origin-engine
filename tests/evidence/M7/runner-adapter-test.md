# M7 runner adapter: real og run of the runner archetype

Platform `agent-runner` (uncommitted M7 working tree), `runner-tools/tests/og.rs`, against the engine at c20e014 (a `git archive` export with `og` built as `CARGO_TARGET_DIR=target/agent-m7 cargo build -p og-cli` and the native addon from `cargo build --release -p og-host-node`, macOS arm64, Metal).

```sh
cd agent-runner
ORIGIN_RUNNER_OG_TESTS=1 ORIGIN_OG_BIN=<origin-engine>/target/agent-m7/debug/og \
  ORIGIN_ENGINE_REPO=<export of c20e014> OG_ENGINE_PACKAGE=<export>/packages/engine \
  ORIGIN_OG_TEST_OUT=<out> cargo test -p runner-tools --features origin-engine-ts --test og -- --nocapture
```

```text
Compiling runner-tools v0.1.4 (/Volumes/app/origingame/agent-runner/crates/runner-tools)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.59s
     Running tests/og.rs (target/debug/deps/og-156dfaa46b6b0c2d)

running 1 test
test the_runner_archetype_cooks_runs_and_is_observed_through_the_adapter ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.63s
```

What it asserted, through the runner's existing tools on the `engine` service `og::start` registered:

- `og::start`: `og run` cooked `assets/` into `cooked/`, og-probe `hello` = substrate `origin-engine`, protocol 5, wire 1, 60 ticks/s, tier `metal`, build `release`.
- `engine_probe` pause, tick, advance 30: exactly +30 ticks while paused.
- `engine_probe` state on the title: facts {"best": 0, "coins": 0, "distance": 0, "phase": "title", "rounds": 0, "score": 0, "speed": 1.5}, entities include `runner`.
- `stage_capture` desktop: `.origin/frames/tu-stage_capture/desktop-1280x720.png`, 1280x720 PNG at tick 33, more than 16 distinct colours, a typed `frames` card (sha256 `e3884aeaa78b4a6d`; filed as `runner-adapter-frame.png`).
- `engine_probe` input `{kind: key, key: Space, action: press}`, advance 30: facts {"best": 0, "coins": 0, "distance": 2.91, "phase": "running", "rounds": 1, "score": 2, "speed": 6.06} at tick 63.
- `og::dump` world: JSON next to the frames, `kind: world` at the current tick.
- `stage_record` 0.5 s: tick-stepped WebM `.origin/frames/tu-stage_record/recording.webm`, 500 ms, 95413 bytes.
- `playtest_replay` 20 ticks with one Space press: before 132, after 152, frame `.origin/frames/tu-playtest_replay/playtest-end.png`, phase `running`.
- `engine_stop`: the process group stopped.

Gates checked: without `--features origin-engine-ts` the test is a compile error; with `ORIGIN_OG_BIN=/nonexistent/og` it fails ("requires og on PATH or ORIGIN_OG_BIN").

The frame, read: Rooftop Run's title at tick 33, 1280x720 — the yellow
"ROOFTOP RUN" wordmark and "屋顶快跑" over a dark cloud band, the Chinese
start prompt "点击或按空格开始" (the English line blinks and is off on this
tick), the moon top right, the runner (white cap, orange scarf, teal suit)
standing at the left on the teal rooftop edge over the brick wall, and a
night skyline of lit windows across the width. Text is crisp, sprites are
not stretched at the 16:9 target although `og run` attached the render
target at the size `og::start` passed (1280x720).
