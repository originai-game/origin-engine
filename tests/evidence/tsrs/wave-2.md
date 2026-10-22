# Wave 2 — og dev with the check in the loop (goals/tsrs/plan.md §六)

Tree `2d7c282a` (main) plus this file; 2026-10-22; an Amp orb: Linux
x86_64, Intel Xeon @ 2.60 GHz, `nproc` 8, load under 1, nothing else
running. `target/release/og` and the release `og-host-node` addon (a copy
named `origin-engine.node`, `OG_NATIVE_ADDON`), `OG_ENGINE_PACKAGE` the
checkout's. The renderer is **vulkan-cpu** (lavapipe): no GPU here.

The project is the template scaffolded by `packages/create`
(`target/evidence/tsrs/game`). A driver started `og dev <game> --json
--exit-after 8`, waited for the first `check` event and 3 s, then 8 times
rewrote `src/main.ts`, alternately adding `export const speed: number =
"fast";` (TS2322) and taking it out, waiting for each `reload` event and
1.5 s. The same driver ran once more with `OG_TSRS=/nonexistent`, which og dev
reports as a broken checker and then reloads unchecked (`og dev: no type
check (...); reloads go on unchecked`): the rebuild + swap baseline.

| Reload | with the check: rebuild + swap (ms) | its check (ms, errors) | without: rebuild + swap (ms) |
|---|---|---|---|
| 1 | 30.8 + 1220.5 | 33.7, 1 | 24.4 + 703.3 |
| 2 | 42.8 + 711.5 | 44.9, 0 | 31.9 + 831.3 |
| 3 | 25.5 + 680.5 | 46.3, 1 | 33.7 + 823.2 |
| 4 | 34.7 + 856.4 | 38.7, 0 | 45.4 + 822.2 |
| 5 | 44.2 + 705.4 | 43.4, 1 | 26.8 + 815.2 |
| 6 | 33.6 + 801.4 | 32.7, 0 | 24.4 + 769.2 |
| 7 | 38.3 + 696.4 | 47.7, 1 | 28.2 + 671.1 |
| 8 | 35.0 + 827.4 | 39.9, 0 | 74.5 + 763.1 |

- **Rebuild** (save to bundle written): median 34.9 ms, worst 44.2 ms with
  the check beside it; median 30.0 ms, worst 74.5 ms without. The check,
  a separate process, does not move the rebuild.
- **The check**: median 39.9 ms, worst 47.7 ms (the starting bundle's 29.3
  ms). Each landed before its reload's event: every `reload` carried
  `check: {ok, ms, errors}`, none `null`; the four with the error carried
  `errors: 1` and their `check` events listed `src/main.ts(30,14) TS2322`.
  Every reload was `ok: true`: a type error never refused one.
- **Swap** (bundle written to first new frame) is 670–1220 ms on both
  runs, over the 300 ms budget with and without the check: it is the
  first frame rendered on lavapipe (`og doctor`: a 64x64 frame in 652 ms),
  not the check. The 300 ms budget's evidence stays the M4 run on Metal
  (`tests/evidence/P2/tooling/m4-m5-tooling.md`, worst 112.8 ms); a rerun
  there with this og is owed to the coordinator (a renderer this orb does
  not have).

Conclusion for D3: the check runs beside the rebuild and lands within
about 50 ms, inside the rebuild + swap window; it is reported on its own
(`check.ms`) and stays outside the 300 ms measure (design §7).

## Sample events

```
{"config":".og/tsconfig.check.json","diagnostics":[{"code":2322,"col":14,"file":"src/main.ts","line":30,"message":"Type 'string' is not assignable to type 'number'."}],"event":"check","ms":33.7,"ok":false,"tool":"tsrs","version":"7.1.0-dev.20260930.4"}
{"budgetMs":300.0,"check":{"errors":1,"ms":33.7,"ok":false},"event":"reload","n":1,"ok":true,"rebuildMs":30.8,...}
```

`og run`, `og test` and `og build` on the same project with the error
(debug og): each printed the diagnostic, `1 type error(s); fix them or
pass --no-check`, exited 1 and left no `.og/game.native.js` or `dist/`;
`og test --json` ended with `{"event":"test","ok":false,"suites":3,
"failed":1,"check":{"ok":false,"ms":30.5,"errors":1}}` and ran no
playtest; `--no-check` ran as before (tick 30, hash 4cff21e8f5070e66, the
same as the clean run); `og build --publish --no-check` exited 2.
