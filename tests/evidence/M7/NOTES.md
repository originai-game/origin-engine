# M7 evidence

One line per file: what it is, how it was produced, at which revision.

- `knowledge-mount.md` — bun packages/knowledge/scripts/mount.ts <copy|/Volumes/app/origingame> [--check] --commit c20e014; platform bun run --cwd packages/game-knowledge test/typecheck (rev c20e014+dirty, 2026-10-17, sha256 7af9bfc88160a181)
- `runner-adapter-test.md` — ORIGIN_RUNNER_OG_TESTS=1 cargo test -p runner-tools --features origin-engine-ts --test og in origingame/agent-runner against og built from c20e014 (rev c20e014+dirty, 2026-10-17, sha256 1253a17482fd928a)
- `runner-adapter-frame.png` — stage_capture desktop on the og run warm process of templates/archetypes/runner (tests/og.rs, ORIGIN_OG_TEST_OUT) (rev c20e014+dirty, 2026-10-17, sha256 e3884aeaa78b4a6d)
- `runner-adapter-run.json` — summary written by origingame agent-runner tests/og.rs (ORIGIN_OG_TEST_OUT): hello, frame, states, recording, replay (rev c20e014+dirty, 2026-10-17, sha256 9b5e025230d3f3f0)
