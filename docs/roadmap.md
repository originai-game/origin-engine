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
| Standard CSS | [T-01a1b996](https://ampcode.com/threads/T-01a1b996-bd22-7771-a0f9-a7e49b7a2fc9) | Instantiated sibling and structural selectors, property transitions and animations, container queries |
| Retained UI renderer | [T-01a1b999](https://ampcode.com/threads/T-01a1b999-f109-72dc-b8c3-1d9e6cb93c1b) | Dirty-only emission and bounded paint, lazy diagnostics, TS builder removal, both-host proof, perf after-numbers |
| og-ui generic capabilities | [T-01a1b99f-0571](https://ampcode.com/threads/T-01a1b99f-0571-7307-b07f-6038d86a5245) | Font packs, password and input types, colour-blind modes, live regions, selectable text, run-time colour emoji, split screen, UI debugger and `dump_ui:motion`, text on a path, untested components, UI perf budgets |
| og explore and invariants | [T-01a1b99f-0ce1](https://ampcode.com/threads/T-01a1b99f-0ce1-7570-9185-d8d1698ac145) | Per-tick invariants stream, invariants follow-ups, `og explore` |
| Presentation checkpoints and og.motion | [T-01a1b99f-1445](https://ampcode.com/threads/T-01a1b99f-1445-70df-9528-b3bc997e0683) | Byte-identical restore of every presentation state, og.motion poses identical on every host, session forking, the jump pitch and the wave goal, module-scope state check |
| Rendering performance | [T-01a1b99f-2748](https://ampcode.com/threads/T-01a1b99f-2748-7173-98de-af62ac91012f) | FSR 3.1, GI probes, crowds at 50k, first frame |

## Landed this round (2026-11-07 to 2026-11-08)

- **UI reset:** the old kits and packs deleted (951b652e5); `docs/ui.md`
  rewritten (beccbf27a and follow-ups).
- **UI documents:** scriptless Svelte templates (6ee803a44, fb2813069,
  01a9a97b2); standard CSS cascade (443078990) and media queries
  (fd41adf81).
- **Retained core:** named stores, `GameDesc.ui`, presentation checkpoint
  and recorded intents (cc6000c29); mounts, topology, keyed virtual rows,
  settlement, stable slots (1167a7566 to 0b6ba3af3).
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
| FSR 3.1 | +0.23–1.87 ms over TAAU at 1080p | cheaper accumulate | rendering |
| GI probes | main pass 3.70 ms; `both` 12.70 ms; screen GI 0.65 of the path-traced reference | ceiling 0.72–0.80, cheaper | rendering |
| First frame | terrain 7–12 s on DX12; Pixel 14–17 s | the shader compile budget | rendering |
| Crowds at 50k | 24.35 ms a frame | prepass and main pass reduced | rendering |
| Presentation checkpoints | og.motion poses differ from native in the last bits on the web; a web tick waits for a late batch; terrain fades by frame seconds | byte-identical on every host | checkpoints |
| og.motion | the jump's pitch clamped, not fixed at the source; the wave goal idles at 12 s | fixed at the source | checkpoints |
| UI per-frame cost | idle 1.229 ms, one label 1.254 ms (median, 2560×1440) | follows the change, not the tree | retained UI |
| Watercolour look | reads as oil | a watercolour primitive (Bousseau, MNPR) | with the art review, after the pilot opens |
| Distant impostors | no own shadow, the view from below, no sway | — | with the art review |
| Wasm, determinism and browser tiers of the latest features | measured before a release | — | release |

Deferred by the owner: UI text scale at 200 %, accessibility beyond
AccessKit, the overlay's GPU cost, SSSR and FSR follow-ups, W7 (CPU FFT
band for buoyancy), wgpu 31 (unpublished). Ray tracing is a non-goal;
Radiance Cascades were measured and not ported (`docs/design.md` §2.6).

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
