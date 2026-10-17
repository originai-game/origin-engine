# M5 evidence

One line per file: what it is, how it was produced, at which revision.

- `probe-conformance-test-output.md` — cargo test -p og-probe and cargo test -p og-probe --features publish --test publish (CARGO_TARGET_DIR=target/agent-probe), plus fmt/clippy/wasm32 checks and 21 one-at-a-time mutations of crates/og-probe/src each caught by a failing suite (rev fa2045d+dirty, 2026-10-17, sha256 e4ef03d462e1c8a5)
- `probe-conformance-transcript.json` — every JSON-RPC call and answer of crates/og-probe/tests/conformance.rs every_member_answers_as_protocol_5_says, written to $CARGO_TARGET_TMPDIR by the test (rev fa2045d+dirty, 2026-10-17, sha256 7d5512c3d11af161)
- `scripted-run-dump-world.json` — crates/og-probe/tests/commands.rs a_scripted_run_seeds_advances_dumps_and_quits: script 0.5:seed:7;1.0:advance:30;1.5:dump_world;2.0:quit on the tests/support world (seed 1); the dump_world output at tick 90 (rev fa2045d+dirty, 2026-10-17, sha256 5fc7554bf39447a4)
