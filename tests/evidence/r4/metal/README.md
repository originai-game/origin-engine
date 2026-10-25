# Metal candidate 42206f80, 2026-10-25

Candidate: `42206f80476141d38a1d0ed87455762a324138ae`.
The persistent measure checkout switched to a new `r4/metal-candidate`
branch at `1bb341e5`, then fast-forwarded to `origin/r4/candidate`.
The measurement branch and Mac's local main were preserved.

Built the dev package with `OG_GATE=1 sh packages/engine/scripts/build.sh`
under both gate locks. Ran Rumpus Isle's `frames.og.ts` suite first, then
`scripts/check.sh functional`: **PASS, 5/5 steps, 252 seconds**.
fmt 3 s; clippy 18 s; test 247 s; drift 106 s; bun-functional 115 s.
The gate used its normal concurrent steps, not separate gates.
The functional lane does not rebuild wasm or close web acceptance.

## Separate asset mismatch from the contact change

The game needed its ignored assets generated with
`bun examples/rumpus-isle/tools/make-assets.ts`. That copied today's cartoon
atlas and made all eleven reference checks fail: its button corners are
different from the atlas used to record the references. Neither the atlas
generator nor the cartoon style changed between the base and candidate.

For an apples-to-apples collision comparison, used the existing Mac main
checkout's ignored Rumpus atlas in this lane only. SHA-1:

- Today's template atlas: `4218685acafe94ae31b89c60b5fa7b368436f25b`.
- Reference-compatible Rumpus atlas: `526fa0a2b60166086dd05519f8ac42ccf47bd22f`.

With that atlas, all eleven candidate frames pass their existing tolerance.
A separate temporary copy of the game, restoring only `src/critters.ts`
and `src/components.ts` from `1bb341e5`, run with the candidate engine,
matches all eleven committed references pixel-for-pixel. This control
isolates the remaining four changed frames to the merged game-side contact
velocity change, rather than the atlas or engine build.

| Frame | Changed RGB pixels | Outlier % | RGB RMSE |
|---|---:|---:|---:|
| bump.png | 2,954 | 0.014 | 0.00193 |
| splash.png | 2,106 | 0.015 | 0.00236 |
| scores.png | 128 | 0.000 | 0.00004 |
| berry.png | 7 | 0.000 | 0.00001 |

Only those four references were replaced. The other seven are pixel-identical.
The 8× amplified RGB diffs were inspected: bump/splash changes are on the
two colliding critters; scores/berry have only tiny residual pixel changes.
Terrain and UI do not move in this controlled comparison. The first bump
script reports two bumps. Re-ran the same frame suite after replacement.

Commands for both game comparisons:

```sh
OG_ENGINE_PACKAGE="$PWD/packages/engine" packages/engine/native/og.darwin-arm64 test examples/rumpus-isle --filter frames.og.ts
# Control: same command on target/r4-control-rumpus, with the two old source files.
```

The unrelated generated-atlas/reference mismatch is **not fixed** here.
A clean asset regeneration still requires its own reference reconciliation;
these four recordings deliberately retain the references' existing atlas.

During the gate, the owner authorized stopping EchoPlay PID 81366 and
SimMetalHost PID 95801. Confirmed both command lines with `ps -o pid,args -p`,
then sent SIGTERM to exactly those PIDs. SimMetalHost respawned, so ran the
authorized `xcrun simctl shutdown all`. Neither process remained afterward.
No other owner process was stopped. GPU performance measurements wait for
load below 8 after the gate, and remain a separate report.
