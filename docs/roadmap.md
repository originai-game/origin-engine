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

Phase 1 of the TODO opened 2026-11-09, after the CSS and UI-infrastructure
round landed. All streams run on macrunner and report to the coordinator
thread [T-38e88112](https://ampcode.com/threads/T-38e88112-c769-4e7b-ab60-ee1cd4065bc7).

| Stream | Thread | Scope |
|---|---|---|
| Capture tooling | [T-01a1bf6c-5afe](https://ampcode.com/threads/T-01a1bf6c-5afe-7689-8295-f4510385bdc8) | Set-of-Mark captures, key frames by event with a timeline, FLIP heat maps, model-sized captures |
| UI pilot P1 | [T-01a1bf6c-62eb](https://ampcode.com/threads/T-01a1bf6c-62eb-71f3-9964-d1ad718af946) | `clean` pack and `shooter` kit, §6 acceptance at desktop and 390 px |
| UI pilot P2 | [T-01a1bf6c-6967](https://ampcode.com/threads/T-01a1bf6c-6967-749b-b5b9-b1b4b8980f4b) | `fantasy` pack and `rpg` kit, §6 acceptance at desktop and 390 px |
| Art review | [T-01a1bf6c-75ca](https://ampcode.com/threads/T-01a1bf6c-75ca-7787-a80d-a39e1194502d) | The queued new and moved looks, then the pilot heads; lands look heads |
| Test cleanup | [T-01a1bf6c-7caa](https://ampcode.com/threads/T-01a1bf6c-7caa-7323-aa28-0ee1999840b6) | `full` lane green, tests that guard nothing removed |

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

Phase 1 is open (Running): the capture tooling, the UI pilot with its
art review, and test cleanup. Then the rest of the catalogue, the
performance pass, the real devices, 0.8.0 and the showcase page.

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
