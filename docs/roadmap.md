# Roadmap

What is running, what is next and what is open. History is in git and the
threads linked here; rules are in `AGENTS.md`.

## Order

1. **Clean-up (now).** Only clean-up and groundwork streams run; everything
   else is paused with its work pushed.
2. **Groundwork after the clean-up:** presentation checkpoints, then
   `og explore` (Go-Explore/Antithesis-style search from snapshots, on the
   invariants and og session's snapshot tree), then og-ui's generic
   capability gaps.
3. **UI work**, then **0.8.0** (it ships only once the UI work is done).
4. **Compatibility and real devices last:** Windows (DX12, Chrome, Edge,
   Firefox), Safari, phones (Pixel 10, PowerVR, Adreno, Mali), HDR on a real
   display, the WebGL 2 fallback study. WebGPU only until then.
5. **Templates and showcase content** are rebuilt with care after the feature
   work.

## Running

| Stream | Thread | Branch | State |
|---|---|---|---|
| Docs simplification (phase-1.md and the showcase archive removed, normative docs cut to current content) | [T-01a1b0a1](https://ampcode.com/threads/T-01a1b0a1-5b3d-7726-9b19-2f0ddd05d359) | docs-simplify | running |

The clean-up round's other threads are archived; their hand-offs are in
artifact 9de31d6e-98f1-479b-8d6f-8e191b620da4. Next threads, briefed from it
after docs-simplify lands: a new art runner, delete-templates (phase 2, WIP
44716c502), og session (WIP og-session e0245a20b), presentation checkpoints,
build-speed experiments (scripts at a3a14d29f), and the og-cook fracture
re-cook with og.Decal on terrain.

Landed this round: examples/ deleted with the path-aware `fixtures` step and
the destruction-yard and vehicle-terrain fixtures (to 876766dc0); meshlet
determinism by stable GPU compaction (b54e9e5a2); Music.start's reflected
minimum removed (b59654b2a).

## Decided in the clean-up

- **No gameplay templates.** `og create` writes one project skeleton; the
  creation choices (genre, view, art style, UI style, platform, input,
  players, languages, music) go into project.ogt's `create` block as
  constraints and hints. examples/ is deleted (876766dc0); templates/archetypes,
  templates/fx-showcase and the templates/ui-kit gallery are deleted next;
  templates/game and game-2d become fixtures. Kept: the UI packs (kits and
  style packs), templates/looks (artBrief is a platform contract), locales,
  packages/design.
- **Fixtures** under `tests/fixtures/`, one per feature, a few seconds each,
  indexed in its README and run only when their inputs change.
- **No taste verdicts in the engine:** `og polish`, the publish style refusal,
  `style check --critic` and `og run --device` scores are gone; `og ui check`
  reports facts and measures (631b5424a).
- **Invariants** in the game API: `og.assert` always/sometimes/reachable/
  unreachable/each, default invariants from reflection, a coverage report
  (443357b6b).
- **Tests:** pinned nightly with LLVM everywhere, 37 → 23 test binaries,
  Rust tests 2,820 → 1,606, engine PNGs 260 → 54, kit frames 906 → 64; a
  missing GPU counts as skipped (090cf6626).
- **Presentation checkpoints** live beside the world snapshot, outside
  world_hash; capture resets frame history and draws a fixed warm-up.
- **Not doing:** Cranelift, CRIU or process fork, VLM gating, Rust hot
  patching, splitting tests across machines.
- **Landing:** authors land non-visual heads themselves; the art runner lands
  visual ones. No showcase archive; evidence is not committed by default;
  platform asks are collected at release.

## Paused, to resume from new briefs

| Item | Where | Left |
|---|---|---|
| Presentation checkpoints | [T-01a1b075](https://ampcode.com/threads/T-01a1b075-9b61-72b7-baee-649cbe0824da) (stopped) | TAA, frame_index, GI/SSR histories not restored; terrain fade per frame; the motion Presenter on wall clock; InterpState and Presenter dropped on restore; a bundler check for module-scope state |
| Creation choices schema and labels | `platform-scaffold-contract` 6a62b9cd0 | reuse the labels and schema for the skeleton's constraints; its 2D overlay is dropped |
| UI catalogue, `og ui find`, fallback icons | `ui-catalogue` c2681af7d | rebase |
| Text source beside locl glyphs | `ui-text-source` f4ae8a0f7 | never reviewed |
| Invariants follow-ups | main | incremental checks of untracked components (changes the snapshot's og.ecs block), `require` over then-steps and room peers, `invariant:` paths, a web run |
| Origin-agent | [T-c41d3348](https://ampcode.com/threads/T-c41d3348-90e2-4342-b2c8-1319ccde89e8) | ask 27 (polish removal) told; the skeleton-and-constraints ask follows delete-examples |

Dropped: `create-starters` a79e6bed8, `ui-own-look` 363ab979b.

## Open defects

- `advance(N)` in one call schedules a timed command differently from
  per-tick stepping, which moves the world hash.
- og-cook reruns destruction fracture before hashing an up-to-date model
  (destruction-yard 13 s, over budget); og.Decal does not project onto
  terrain.
- Nine hover/selected kit frames show no state; the phone's radial aimed
  frame equals unaimed; Tune's live edits are not saved by `ui:save`.

## og-ui generic gaps (after the clean-up)

Ranked: per-frame cost follows the node count, not the change (the tree is
rebuilt and compared every tick; find_overlaps, the accessibility tree and
the probe list run in every build); no perf tests for `docs/ui.md` §15.1's
budgets; numbers and dates not formatted through og.i18n; no password or
input-type hint; no variable-height virtualization; no colour emoji; no
colour-blind modes; no live regions; no selectable read-only text; no split
screen; no visual UI debugger; documented but missing: text on a path,
`dump_ui:motion`, §6.3's layout boundary, §9's per-binding invalidation;
seven components without a test.

## Below target or unverified

| Item | Now | Target |
|---|---|---|
| FSR 3.1 | +0.23–1.87 ms over TAAU at 1080p | cheaper accumulate |
| GI probes | main pass 3.70 ms; `both` 12.70 ms; screen GI 0.65 of the path-traced reference | ceiling 0.72–0.80 |
| First frame | terrain 7–12 s on DX12; Pixel 14–17 s | the shader compile budget |
| Wasm size | text stack +2.08 MB Brotli (publish 6.87 MB, gate 16 MiB) | within the gate; dictionaries on demand |
| Crowds at 50k | 24.35 ms a frame | prepass and main pass dominate |
| Watercolour look | reads as oil | a watercolour primitive (Bousseau, MNPR) |
| og.motion | the jump's pitch clamped, not fixed at the source; the wave goal idles at 12 s | — |
| Distant impostors | no own shadow, the view from below, no sway | — |
| Wasm, determinism and browser tiers of the latest features | measured at wave-end | — |
| W7 (CPU FFT band for buoyancy), wgpu 31 (unpublished) | deferred | — |

Deferred by the owner: UI text scale at 200 %, accessibility beyond
AccessKit, the overlay's GPU cost, SSSR and FSR follow-ups, the platform
asks' joint review. Ray tracing is a non-goal; Radiance Cascades were
measured and not ported (`docs/design.md` §2.6).
