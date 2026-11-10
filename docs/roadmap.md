# Roadmap

What is running, what is next and what is open. History is in git and the
threads linked here; rules are in `AGENTS.md`, the UI contract in
`docs/ui.md`.

## Order (the owner, 2026-11-08)

1. **Engine and UI infrastructure:** done 2026-11-09.
2. **UI catalogue (now).** Pilot first (clean and fantasy packs, rpg and shooter
   kits, a separate art-review stream), then the remaining kits and packs
   (`docs/ui.md`: 9 kits, 6 packs) until the whole acceptance matrix
   passes.
3. **0.8.0** ships when the UI catalogue is accepted and every Below target
   item is met. Release and its platform asks per `docs/release.md`.
4. **The engine's showcase page**, rebuilt from scratch as a page of the
   platform (origingame), after the UI catalogue.
5. **Compatibility and real devices last:** Safari, phones (Pixel 10,
   PowerVR, Adreno, Mali), HDR on a real display, the WebGL 2 fallback
   study. `og mcp` after 0.8.0.

## Running

Phase 1 closed 2026-11-10. Streams run on macrunner and report to the
coordinator thread [T-01a1c3df](https://ampcode.com/threads/T-01a1c3df-23c6-73be-9bb8-0da680008e82).

Landed in phase 1:

- The rpg kit and the fantasy pack (pure CSS), art-approved: 7407166332.
- The core and G1 UI parts: 8ef79ce29a.
- `og-rive` and `og-emitter` as owned nodes: 7e4888f5d1.
- CSS gaps 18 (digits beside CJK in one run) and 20 (`rem`): 748b1ef9ef.

Still open:

- G2/G3 parts as nodes: select and tabs landing now, text fields and G3
  next. The fantasy pack's host-rule `pending` markers drop with them.
- Gap 16: the `font-family` fallback list is not consulted per glyph.
- Finding 19: filtering the overflow check's stderr at tick 0.
- The `docs/ui.md` §8 inventory.

Next, in order:

1. The docs pass.
2. The web-tier re-records.
3. Phase 2: the remaining kits and packs, from generated boards.
4. The performance pass.
5. The real-device run.
6. 0.8.0.

## Landed this round (2026-11-08 to 2026-11-09)

- **CSS completion** (the gap audit's sections A, B and C): cascade
  features, `::selection`, pseudo-element boxes, `@font-face`; units,
  box model, flex, grid, table, positioning and anchor positioning,
  multi-column, floats, `direction` and writing modes with vertical text;
  colour, fonts, corner shapes, blend modes and isolation, 3D, `linear()`,
  every property animatable, scroll-driven animations, shared-element and
  nested view transitions, clip-path, inline SVG `<filter>` and
  backdrop-filter, backgrounds, border-image and masks, interaction,
  scrolling, images, `cursor: url()`, `resize`, `contain`,
  `content-visibility`. Head of the round: ae321c70a.
- **Svelte, HTML, SVG and GSAP:** forms with validity, date and time
  pickers, file input, `<video>`/`<audio>`, table display.
- **Retained UI finished:** every TS builder removed (ea7f1ae40,
  708635b81, d4f4d4d1d, 11a2b2488); popovers on CSS anchors;
  `env(safe-area-inset-*)` honours the covered controls and
  `env(-og-hud-inset-*)` keeps the ultrawide HUD band; the og-ui suite
  passes under `OG_UI_CHECK_BOUNDED` (2421a23eb).
- **Publish names:** platform ask 18 landed; publish builds carry
  content-hashed names only (64b33dede).

## Below target

Each must be met before 0.8.0.

| Item | Now | Target | Stream |
|---|---|---|---|
| FSR 3.1 | accumulation cheaper (seven dispatches, 12 bytes a pixel stored, history in 8 fetches, 2929a46): +0.65–1.04 ms over TAAU at 1080p in the thin scene (was +0.71–1.16), 0.90–1.23 ms under it on grass (M4 Pro); the rest is the six passes before the accumulation | at most TAAU + 0.5 ms on thin geometry at every scale, fsr::compare within 0.05 dB, grass not regressed (fused pre-passes, f16 where `shader-f16` exists) | rendering |
| GI probes | main pass 3.70 ms; `both` 12.70 ms; screen GI 0.65 of the path-traced reference | ceiling 0.72–0.80, cheaper | rendering |
| First frame | DX12 cold driver cache (RTX 5060): terrain-near 1.59 s (was 10.46), gltf-reference 0.95 s, materials 1.09 s; lit pipelines are made as the variant for the frame's features first (pipeline_features.rs), the full one in the background; each pipeline compiled alone (cold DXC and driver, one compile worker; terrain-near, gltf-reference, materials) is at most 0.43 s but one, terrain lit at 0.79 s (its layers' taps looped, 0.86 before), the one exception, its pass not split for 0.8.0; a warm driver cache saves about a third (terrain-near 1.16 s); Pixel 14–17 s, not remeasured | terrain-near setup and first frame at most 2.0 s, no pipeline compiled alone above 0.5 s, cold driver cache | rendering |
| Crowds at 50k | 16.40 ms a frame median (was 20.88 on the same main, 24.35 at the last record; prepass 8.75 → 5.73 ms, main 8.68 → 7.39 ms, M4 Pro): skinning reads only weighted joints | at most 12 ms a frame median (M4 Pro, perf_crowd_50k), reference frames unmoved; skinning once a frame tried | rendering |
| UI per-frame cost | baseline idle 1.229 ms, one label 1.254 ms (median, 2560×1440); indicative now idle 0.026 ms, one label 0.160 ms, scroll-1000 about 0.39 ms against 0.29 immediate | follows the change, not the tree | retained UI |
| Watercolour look | reads as oil | a watercolour primitive (Bousseau, MNPR) | with the art review, after the pilot opens |
| Distant impostors | no own shadow, the view from below, no sway | — | with the art review |
| Wasm, determinism and browser tiers of the latest features | measured before a release | — | release |

Deferred by the owner: UI text scale at 200 %, accessibility beyond
AccessKit, the overlay's GPU cost, SSSR and FSR follow-ups, W7 (CPU FFT
band for buoyancy), wgpu 31 (unpublished). Ray tracing is a non-goal;
Radiance Cascades were measured and not ported (`docs/design.md` §2.6).

## TODO (the owner, 2026-11-08)

Phase 1 is closed; what is still open and the next steps, in order, are
under Running. The showcase page follows 0.8.0.

- **Screenshots and recordings for agents (running):** UI elements and entities
  marked with numbered boxes on a capture, with a JSON map to path and
  source line (Set-of-Mark); key frames picked by event (visual change,
  input, assertion, screen change) with a synced timeline JSON; perceptual
  diff heat maps (NVIDIA FLIP, BSD) with the worst regions cropped and
  enlarged; captures sized for the model (long side 1280, crops apart). A
  platform ask for video summarised by a video model, the engine supplying
  the video, key frames and timeline.
- **The UI pilot (running), then the whole catalogue;** 0.8.0 with the pack catalogue and the collected platform asks;
  the showcase page on the platform. Each is opened when the owner says so.
- **One full performance pass before the 0.8.0 release, never alongside
  development:** no stream measures while others build or land. It covers
  `scripts/perf.sh` with UI counters and the idle breakdown, the rendering
  budgets (first frame, crowds, FSR, GI) and the release size gate. It also
  finishes the paused rendering work on local branches in macrunner's lanes:
  `gi-screen` (a8b1bfa74, lane-30) needs one fix round on gi-room's non-TAA
  artifacts and an art review; `gate-presentation` (ca1742634, lane-28) is
  the quick presentation step. Crowds pass 2 and further FSR work were
  cancelled.
- **Mobile (the owner, 2026-11-08):** the current CSS round delivers
  `svh`/`lvh`/`dvh` and `env(safe-area-inset-*)` on both hosts; the UI
  pilot accepts every kit at 390 px wide and by touch as well as at 1280;
  before 0.8.0 the whole UI catalogue runs on at least one real iPhone and
  one real Android phone.

## Open

- The asset store's cook stat index: sessions still run the incremental
  cook scan.
- Paused archives, rebase only if still wanted: `archive/ui-catalogue`
  (`og ui find`, fallback icons), `archive/ui-text-source` (text source
  beside locl glyphs, never reviewed), `archive/platform-scaffold-contract`
  (creation schema labels; the platform asks cover it at release).
- Platform asks for 0.8.0 are collected by the coordinator: creation schema
  and labels default-only, retired kit and pack recommendations removed,
  the engine pin and mounted knowledge refreshed together, the Svelte and
  store APIs consumed once final.
- The 0.8.0 release publishes the pack catalogue (`docs/release.md`
  §The pack catalogue): `vfx`, `ui`, `fonts` and `looks` in
  `manifest.json` with a preview per pack, for origin-agent's Engine page
  (asked 2026-11-08); `release-dist.ts` builds it.

## Decided

- **No gameplay templates.** `og create` writes one project skeleton; the
  creation choices go into project.ogt's `create` block.
- **Fixtures** under `tests/fixtures/`, one per feature, a few seconds each.
- **No taste verdicts in the engine:** checks report facts and measures.
- **Invariants** in the game API: `og.assert` always, sometimes, reachable,
  unreachable and each, with a coverage report.
- **Presentation state** lives beside the world snapshot, outside
  world_hash; UI state, sounds and motion are presentation; a control's
  effect on the game is a recorded intent.
- **Familiar syntax first:** Svelte templates and stores, standard CSS,
  GSAP, Yarn, bun:test, Rapier and three.js vocabulary.
- **Not doing:** Cranelift, wasm-opt, Wild, remote compiler caches, Bazel,
  CRIU or process fork, VLM gating, Rust hot patching, splitting tests
  across machines.
- **Landing:** authors land their own heads with `scripts/land.sh`; new
  kits and packs need a separate art sign-off.
