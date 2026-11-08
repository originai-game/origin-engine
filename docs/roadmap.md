# Roadmap

What is running, what is next and what is open. History is in git and the
threads linked here; rules are in `AGENTS.md`, the UI contract in
`docs/ui.md`.

## Order (the owner, 2026-11-08)

1. **Engine and UI infrastructure (now).** Everything below under Running.
   The UI kits and style packs start only when all UI infrastructure is on
   main.
2. **UI catalogue.** Pilot first (clean and fantasy packs, rpg and shooter
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

All streams run on macrunner and report to the coordinator thread
[T-38e88112](https://ampcode.com/threads/T-38e88112-c769-4e7b-ab60-ee1cd4065bc7).

| Stream | Thread | Scope |
|---|---|---|
| Retained UI renderer | [T-01a1b999](https://ampcode.com/threads/T-01a1b999-f109-72dc-b8c3-1d9e6cb93c1b) | Wave 1 of builder removal, field replacement, both-host proofs, one-label and scroll-1000 cost, final perf.sh numbers; the final builder removal once the document gaps land |
| og-ui document surface | [T-01a1b99f-0571](https://ampcode.com/threads/T-01a1b99f-0571-7307-b07f-6038d86a5245) | UA block margins, the extern-element hook, bound custom properties, props on existing tags (`border`, `box-shadow`, `outline`, `overflow`), content in buttons, ARIA, device glyphs, inline img/icon/kbd, tooltips and popover, the missing core controls, component documents ([gap brief](https://ampcode.com/artifacts/ui-document-surface-gap-brief-builder-removal-8f5316ab-4604-49d8-8477-6d267aa6ef55)) |
| UI document runtime, graphics and world | [T-01a1bad4-0793](https://ampcode.com/threads/T-01a1bad4-0793-740e-aa3f-5dd65b6b3396) | Mounted documents as layers, snippets as content, inline SVG, drag and drop, `<viewport>`, touch controls, world panels, mass, particles in CSS |
| og-ui features | [T-01a1bad4-117c](https://ampcode.com/threads/T-01a1bad4-117c-709c-bedb-133a6e3d515b) | Colour-blind modes, selectable text, text on a path, colour emoji, live regions, UI debugger and `dump_ui:motion`, split screen, untested components and perf budgets |
| Rendering performance | [T-01a1b99f-2748](https://ampcode.com/threads/T-01a1b99f-2748-7173-98de-af62ac91012f) | FSR 3.1, GI probes, crowds at 50k, first frame |

## Landed this round (2026-11-07 to 2026-11-08)

- **UI reset:** the old kits and packs deleted (951b652e5); `docs/ui.md`
  rewritten (beccbf27a and follow-ups).
- **UI documents:** scriptless Svelte templates (6ee803a44, fb2813069,
  01a9a97b2); standard CSS cascade (443078990), media queries
  (fd41adf81), sibling and structural selectors (17d111bc2), property
  transitions and `@keyframes` animations (7cb6be159), container queries
  (0c5f2cc93) and `@keyframes` inside `@media`/`@container` (dba9940b7);
  the standard CSS stream is complete.
- **Retained core:** named stores, `GameDesc.ui`, presentation checkpoint
  and recorded intents (cc6000c29); mounts, topology, keyed virtual rows,
  settlement, stable slots (1167a7566 to 0b6ba3af3); only changed mounted
  elements are written each frame (dc8a7b38b); the last frame inspected
  on demand with retained paint reused (909c1bb79); layout only along the
  paths to what changed (9909569fe); unchanged subtrees' paint reused and
  scrolled rows kept (9b5a0368e); moved subtrees' paint moved, not
  repainted (ec2553603); the native a11y bridge reads only changed nodes,
  with landmark roles and heading levels (60cedfade); finish and paint
  visit only what changed (0392bec45, a189aa88f).
- **og-ui capabilities:** input types, `inputmode`, `autocomplete` and
  password reveal (4d8b785b9); ten Noto font packs loaded per script on
  demand (84d8ec3e2); HTML element names in documents (a050fc383); core
  `<select>`, `<dialog>` with `::backdrop`, `<details>`, `<meter>`
  (f0c02b795, 3a798e95f); `<p>`, `<span>`, headings, `<div>` and the
  landmarks, lists with CSS markers, `<a href>`, `<label for>`
  (61fb46e22).
- **Checkpoints:** module-scope state fails `og check` as OG9101
  (bf1fd9440); a presentation checkpoint restores the display's whole frame
  history (TAA, SMAA, GI, SSR, FSR, terrain streaming), and
  `og run --capture-only` draws a still frame (1fac50487). The og.motion
  pitch and wave rows were stale since aea4a0f15 (no bounds remain; the
  wave plays past 15 s). Restore checked byte-identical in Chrome WebGPU
  (3dee686b6); the web `renderFrame` readback fixed (dc6ca86c1);
  `og run --forks N` (d0d9a3dde). og.motion runs on the GPU by default
  with no held tick; captures wait for due results; no cross-host pose
  claim; the CPU backend is an opt-in (fb115a7ca, bdc60e28a). The
  checkpoints stream is complete.
- **Invariants and `og explore`:** incremental default invariants, `$`
  path values, `require`, the per-tick `probe.assertionEvents` stream on
  node and web, `RestoreDesc.session`, the `intent:` command and
  `og explore` (29d909847 to dffb7421d); one tick number for every
  assertion kind (e79082c96) and a 20 s explore test (856472296).
- **Core gaps:** CSS length tokens, keyed variable-height virtualization,
  CLDR formatting (a62282dbb, 00a1f5016, e8d43c7b8); structured rich text
  (80ca008ab).
- **Familiar vocabulary:** behaviour trees, GSAP tweens and eases, input
  action maps, `playbackRate`, Rapier names, one shared Ease everywhere
  (bf566370b to 218726bda); playtests on bun:test (d9b237bf0); dialogue in
  Yarn Spinner 3.2.2 (4d4b9ce18).
- **Tooling:** wasm-opt removed (1fe3f19cf); 4 GiB lanes memory; sccache
  0.19.0, local disk only; gate temp files on the checkout volume
  (4c3c8f61c).

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

## TODO (the owner, 2026-11-08: opened later, not now)

- **Screenshots and recordings for agents:** UI elements and entities
  marked with numbered boxes on a capture, with a JSON map to path and
  source line (Set-of-Mark); key frames picked by event (visual change,
  input, assertion, screen change) with a synced timeline JSON; perceptual
  diff heat maps (NVIDIA FLIP, BSD) with the worst regions cropped and
  enlarged; captures sized for the model (long side 1280, crops apart). A
  platform ask for video summarised by a video model, the engine supplying
  the video, key frames and timeline.
- **After the UI infrastructure lands:** the UI pilot (clean and fantasy
  packs, rpg and shooter kits, an art review stream), then the whole
  catalogue; 0.8.0 with the pack catalogue and the collected platform asks;
  the showcase page on the platform. Each is opened when the owner says so.

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
