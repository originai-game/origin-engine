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
3. **UI work, done carefully**, and the below-target items (§Below target)
   are fixed for **0.8.0**; 0.8.0 ships only once both are done (the owner).
   `og mcp` waits until after 0.8.0's UI work, not this round (the owner).
4. **Compatibility and real devices last:** Windows (DX12, Chrome, Edge,
   Firefox), Safari, phones (Pixel 10, PowerVR, Adreno, Mali), HDR on a real
   display, the WebGL 2 fallback study. WebGPU only until then.
5. **Templates and showcase content** are rebuilt with care after the feature
   work.

## Now (the owner, 2026-11-07)

1. **Done at 9665c6734: one destructive cleanup head** (delete-templates): every gameplay
   template, its playtests and references, tests that exist only for them,
   tests/evidence, dead scripts and tools, the stale CI workflow, the
   determinism tool, and tests red because their code is redone in 0.8.0.
   Minimal fixtures only for core features that lost their only coverage.
   No other thread lands until it is on main.
2. **The gate rewritten on the clean tree** (gate-reform), not patched.
3. **Feature work resumes**, rebased and landed in larger complete heads.

Stale branches (the owner, 2026-11-07): 68 obsolete remote branches were
deleted; each head is kept as tag `archive/<branch>`. Live branches and the
compatibility-phase WIP (safari, webboot, android) stay.

Skills are cut hard (the owner): they keep only workflow, judgment and
pitfalls, with no API restatement; names never change.

## Running

| Stream | Thread | Branch |
|---|---|---|
| Art runner (reviews and lands visual heads) | [T-01a1b0c7](https://ampcode.com/threads/T-01a1b0c7-5f8c-7352-a5dc-d024892ea59e) | art-runner |
| og-cook fracture re-cook, og.Decal on terrain | [T-01a1b0c8-3db5](https://ampcode.com/threads/T-01a1b0c8-3db5-7421-9322-4e4629f2ab12) | cook-decal |
| og.motion: geometry diagnosis, jump arc ownership, cases 04/05, skinned art review | [T-01a1b0ff](https://ampcode.com/threads/T-01a1b0ff-58c7-70fd-817c-80f5d7e6ecbd) | og-motion |
| Gate and toolchain reform: landing queue first, then two lanes (quick, full), dependency-derived selection, Cargo overrides re-measured, dead scripts removed | [T-01a1b106](https://ampcode.com/threads/T-01a1b106-008f-775c-b0a3-c4689c659e78) | gate-reform |
| Engine skills rewritten from scratch; docs and leftovers trimmed | [T-01a1b17a](https://ampcode.com/threads/T-01a1b17a-f546-75d5-b368-516abc2476c3) | skills-rewrite |
| origin-agent: platform design aligned (ask 27, skeleton + constraints, unchecked results, invariants) | [T-01a1b0ca](https://ampcode.com/threads/T-01a1b0ca-8527-7155-9ca3-754ac90ba663) | engine-design-sync (origin-agent) |

Hand-offs from the clean-up round: artifact
9de31d6e-98f1-479b-8d6f-8e191b620da4. Waiting: presentation checkpoints, in Claude Code on Opus, once
delete-templates has landed (the owner).

Landed in the clean-up round: examples/ deleted with the path-aware `fixtures` step and
the destruction-yard and vehicle-terrain fixtures (to 876766dc0); meshlet
determinism by stable GPU compaction (b54e9e5a2); Music.start's reflected
minimum removed (b59654b2a).

## Resident sessions

The [session stream](https://ampcode.com/threads/T-01a1b0c8-1b44-773e-9143-d2b43bf515d7)
provides automatic local hosts, isolated game runs and playtests, UI matrices,
affected-suite selection, failure captures, strict unchecked-result handling,
compiler-owned incremental checks, and validated native bundle reuse.

The checker stays a subprocess. One warmed, CPU-only `vehicle-terrain`
sample with the dev CLI measured launch plus `--version` at 5.7 ms out of
a 131 ms checked run (4.4%). This is a launch-cost estimate, not a repeated
benchmark or a shipping-build result; it does not justify checker residency.

Remaining work belongs to these streams:

- The [asset store](https://ampcode.com/threads/T-01a1b1f2-e53a-7673-b610-8c99dfc640d3)
  adds the cook's stat index. Sessions still run the incremental cook scan.
- The [UI foundation](https://ampcode.com/threads/T-01a1b1e4-5d51-73b5-a12b-ee59fd202b61)
  adds document-driven review matrices through `ui::spawn` and `runUi`.
  Review scratch projects must stay beneath the caller's worktree to share
  its host instead of starting one for each temporary project.
- Forking and capture-only remain off until presentation checkpoints cover
  renderer state, interpolation, and the Presenter. Current failure captures
  are diagnostics, not resumable presentation checkpoints.
- Windows keeps the fresh-process path. Unix-only session transport and
  bundle-cache validation remain part of the compatibility work.

## Decided in the clean-up

- **No gameplay templates.** `og create` writes one project skeleton; the
  creation choices (genre, view, art style, UI style, platform, input,
  players, languages, music) go into project.ogt's `create` block as
  constraints and hints. examples/, all gameplay templates, the effects
  showcase and the UI gallery are deleted. Only inputs needed by core
  feature tests remain as small fixtures. Kept: the UI packs (kits and
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
| Creation choices schema and labels | tag `archive/platform-scaffold-contract` 6a62b9cd0 | reuse the labels and schema for the skeleton's constraints; its 2D overlay is dropped |
| UI catalogue, `og ui find`, fallback icons | tag `archive/ui-catalogue` c2681af7d | rebase |
| Text source beside locl glyphs | tag `archive/ui-text-source` f4ae8a0f7 | never reviewed |
| Invariants follow-ups | main | incremental checks of untracked components (changes the snapshot's og.ecs block), `require` over then-steps and room peers, `invariant:` paths, a web run |
| Origin-agent | [T-c41d3348](https://ampcode.com/threads/T-c41d3348-90e2-4342-b2c8-1319ccde89e8) | ask 27 (polish removal) told; the skeleton-and-constraints ask follows delete-examples |

Dropped: tag `archive/create-starters` a79e6bed8, tag `archive/ui-own-look` 363ab979b.

## Open defects

- Rebuild these removed failing assertions in the 0.8.0 UI work: racing
  speedometer tach/shift-light layout; RPG unit/party-frame level badges
  at small text scales; light-style objective text versus HUD-plate ink;
  ability-bar hotkey text scaling (the Q label is absent).
  Their tests are deleted with the template cleanup, not fixed or waived.
- og-cook reruns destruction fracture before hashing an up-to-date model
  (destruction-yard 13 s, over budget); og.Decal does not project onto
  terrain.
- Nine hover/selected kit frames show no state; the phone's radial aimed
  frame equals unaimed; Tune's live edits are not saved by `ui:save`.
- The style packs' frame suites (`templates/styles/tests/<pack>/style-*.og.ts`)
  do not load: `styles.ts` imports the UI gallery's helpers and drives its
  screens, and the gallery went with the template cleanup (9665c6734). Port
  them to the neutral sheet or a small state fixture in the 0.8.0 UI work,
  when the packs are rebuilt and their references move anyway; the packs'
  other tests (style-packs, kit-documents, style-diagnostics) run.

## og-ui generic gaps (after the clean-up)

Font packs (0.8.0 UI work, the owner): fonts become packs like the UI packs
(one per script, with a manifest); installed into a game, og.i18n wires the
font chain, so the `create` block's languages install the matching pack
with no generated code.


Ranked: per-frame cost follows the node count, not the change (the tree is
rebuilt and compared every tick; find_overlaps, the accessibility tree and
the probe list run in every build); no perf tests for `docs/ui.md` §15.1's
budgets; numbers and dates not formatted through og.i18n; no password or
input-type hint; no variable-height virtualization; colour emoji are
cooked images (`font { color true }`), not run-time glyphs; no
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
| Presentation checkpoints | og.motion's poses are the tick's on every host, but the web's differ from native's in the last bits (the model's GPU floats are the backend compiler's; the schedule and token choices agree); on the web a tick whose step needs a batch not back is held (motion-feet in Chrome: 68 of 802 ticks, 19 ms median, 56 ms at most, all where an agent ran dry); terrain tiles fade in by frame seconds on the display (captures draw them settled) | byte-identical on every host |
| Distant impostors | no own shadow, the view from below, no sway | — |
| Wasm, determinism and browser tiers of the latest features | measured before a release | — |
| W7 (CPU FFT band for buoyancy), wgpu 31 (unpublished) | deferred | — |

Deferred by the owner: UI text scale at 200 %, accessibility beyond
AccessKit, the overlay's GPU cost, SSSR and FSR follow-ups, the platform
asks' joint review. Ray tracing is a non-goal; Radiance Cascades were
measured and not ported (`docs/design.md` §2.6).
