# A panic inside a lane reaches the page (docs/web-export.md §Errors)

`packages/engine/tests/web/lane-panic.ts`, headless Chrome for Testing 150
on the Apple M4 Pro, 2026-10-20. The `lane-panic` fixture (2,304 boxes, so
each frame's extraction decodes enough og.Mesh rows to split) is built with
`og build --publish`'s web half (3D variant plus the lanes variant) and
served with its COOP/COEP. After a tap on the gate, the test calls the
engine Worker's `fault` (og_web_lanes_fault) over the DevTools protocol, so
the next extraction item panics.

```
cd packages/engine
sh scripts/build.sh                 # the gate's wasm (dist/web-publish, -2d, lanes)
bun tests/web/lane-panic.ts         # 23 checks; also in scripts/check.sh web-reference
```

| File | What it shows |
|---|---|
| `lane-panic.json` | 23 of 23 checks. Isolated, lanes on with 8 Workers: `__runtime_probe__.error` is `{ kind: "panic", message: "a fault injected at the extract site (og_lanes::inject_fault)", tick: 34, lane: "extract", location: "crates/og-lanes/src/lib.rs:181:9" }` 9 ms after the fault; not isolated (single-threaded), the same panic on the engine thread with `lane: null` after 6 ms; in both, a probe call afterwards is refused at once. |
| `panic-card-lanes.png` | The error card: "The game stopped" / "panic at tick 34 in a lane (extract): a fault injected at the extract site (og_lanes::inject_fault) (crates/og-lanes/src/lib.rs:181:9)", Log, Reload, Copy report. |
| `panic-card-single.png` | The same without the lane: "panic at tick 34: …". |
| `broken-rayon-wait.json`, `broken-rayon-wait-hang-card.png` | Break and restore: the lanes wasm rebuilt with the engine thread waiting on rayon's own latch again (og-lanes' `guarded` replaced by a direct call). The isolated case fails 7 checks: the engine Worker sleeps in `Atomics.wait` for the aborted lane, nothing reaches the page until the shell's frame deadline fires after 15,021 ms, and the error is `{ kind: "hang", message: "the engine Worker sent no frame for 15 s (last at tick 25): it is stuck, so it was stopped" }` (the card shows that line). Before this round the page had no deadline either and hung for good. Restored, 23 of 23 again. |

The Rust side: `cargo test -p og-lanes` (a lane that panics and never ends
wakes the waiting thread within the wait at once, with its message, site
and tick; a finished job returns its result; natively a lane's panic
reaches the caller through rayon and is recorded with its site; an
injected fault panics once, at its site only).
