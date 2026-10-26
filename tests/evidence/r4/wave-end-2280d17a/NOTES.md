# Mac wave-end at 2280d17a — FAIL

2026-10-26, macrunner, M4 Pro, macOS 27.2. Clean measure lane fetched
and reset to `2280d17acc5e22c29ae9532b069c284afdf07ebc`. Command after
a 60-second pause (one-minute load 2.57):

```sh
CI=1 scripts/wave-end.sh --stages gate,web
```

Full gate and fresh browser references ran serially, without path-aware
skips. No source, baseline or reference edits or workarounds. Total
1,085 s: **gate FAIL (684 s), web PASS (401 s)**. Raw wave summary and
logs are copied unchanged from `target/evidence/wave-end/`, with each
step's log from `target/evidence/gate/`.

| Step | Result | Seconds |
|---|---|---:|
| fmt | PASS | 2 |
| clippy | PASS | 14 |
| wasm-build | PASS | 277 |
| test | PASS | 133 |
| wasm-opt | PASS | 120 |
| size-gate | PASS | 22 |
| drift | PASS | 36 |
| bun-test | FAIL | 132 |
| determinism | PASS, with regenerated lockfile below | 501 |
| Fresh web-reference | PASS | 401 |

The merged Rust test binaries ran: nextest reports 1,360 passed, 18
skipped. No missing test binary/name error occurred. Optional vulkan-cpu
stages were not selected. No native, template or browser frame failed or
moved outside tolerance; no reference was re-recorded.

## Handoff failure

The engine Bun suite reports 331 pass, 1 fail, 7,129 assertions across
58 files. Exact failure from `bun-test.log`:

```text
packages/engine/tests/reload.test.ts:93:30
expect(await first.exited).toBe(0);
Expected: 0
Received: 140
(fail) a handed-off world restores in a new process with a changed layout [591.73ms]
```

The test pipes the child stderr but does not consume or print it, so the
gate log does not contain that child's stderr. The two older watch/reload
tests pass. No workaround or retry was made during this wave.

## Brotli-q11 sizes

Shipping `wasm` profile. All pass the unchanged 4,194,304-byte budget:

| Download | Brotli bytes |
|---|---:|
| 3D lanes | 3,230,271 |
| 2D lanes | 2,989,486 |
| 3D publish | 3,202,439 |
| 2D publish | 2,959,060 |

## Determinism and generated lockfile

All executed builds agree:

| Digest | Value |
|---|---|
| math_digest | `6117185439544271997` |
| bouncing_boxes_hash | `15551159556835976588` |
| physics3d_hash | `10053948282410406339` |
| physics2d_hash | `3002410719564870113` |
| anim_hash | `12806465523503570878` |

**This pass used an automatically regenerated lockfile**, not the
committed tool lockfile unchanged. `generated-lockfile.diff` records the
addition of `syn 3.0.9` to og-core-derive's dependency list. The file was
clean after reset, modified during the gate, and restored before the
receipt commit.

The gate evaluates `rule_determinism` before calling `skip_unchanged`,
even with `CI=1`. It calls `og_crate_dirs --manifest-path
tools/wasm-determinism/Cargo.toml -e normal`, whose implementation in
`scripts/lib.sh` runs **cargo tree without --locked**, suppressing stderr.
That dependency-discovery call can update the tool's lockfile before
`tools/wasm-determinism/run.sh` invokes its `cargo build --locked` commands.
The passing hashes therefore do not prove a direct clean-checkout
`run.sh` can build with the committed lockfile.
