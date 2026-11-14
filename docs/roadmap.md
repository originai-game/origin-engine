# Roadmap

What is running, what is next and what is open. What is done is in git and
the changelog; rules are in `AGENTS.md`, the UI contract in `docs/ui.md`.

## Order

1. **The UI rebuild (now).** Genre kits are dropped: components and screens
   are built into the engine (`@ui/`), and style packs restyle them through
   tokens and parts.
2. **0.8.0** ships when the UI rebuild is accepted and every Below target
   item is met. Release and its platform asks follow `docs/release.md`.
3. **The engine's showcase page**, rebuilt from scratch as a page of the
   platform (origingame), after 0.8.0.
4. **Compatibility and real devices last:** Safari, phones (Pixel 10,
   PowerVR, Adreno, Mali) and HDR on a real display. `og mcp` comes after
   0.8.0.
5. **Native store packages and mini-game platforms** (§Direction), after
   0.8.0 unless the owner moves them.

## Running

Streams report to the coordinator thread
[T-01a1c3df](https://ampcode.com/threads/T-01a1c3df-23c6-73be-9bb8-0da680008e82).

- **The UI rebuild, stage 7:** the component groups (strategy, racing,
  card, puzzle, idle, rhythm, arena, general) are being built into `@ui/`.
  The fantasy style pack ships with the engine.
- **The structure work** (§Structure).
- **Screenshots and recordings for agents:** numbered marks on captures
  mapped to source lines, key frames by event with a timeline, and
  perceptual diff heat maps with the worst regions cropped.

## Next

1. The docs pass (`docs/ui.md` rewritten, the generated wiki).
2. The re-record of every reference set on the GPU host (the L4), with art
   review, which moves `RECORDING_TIER` off `metal`.
3. The web-tier re-records.
4. The performance pass (below).
5. The real-device run.
6. 0.8.0.

## Below target

Each must be met before 0.8.0. The performance pass re-measures each
baseline on the L4 and restates its target there at the same ratio.

| Item | Target | L4, 2026-11-14 (b089eb332, release, Vulkan) |
|---|---|---|
| FSR 3.1 | at most TAAU + 0.5 ms on thin geometry at every scale, within 0.05 dB of the reference, grass not regressed | miss: the resolve is TAAU + 0.37, + 0.46 and + 0.54 ms at scales 0.5, 0.67 and 0.77 (1080p, `fsr::cost`); against the reference (`fsr::compare`) FSR is 0.84–1.71 dB under TAAU on thin static geometry, 3.6–3.8 dB over it while panning, and 0.18–0.29 dB under it on grass |
| GI probes | screen GI at 0.72–0.80 of the path-traced reference, cheaper than now | miss: screen GI gathers 0.63–0.66 of a path-traced reference's indirect light in gi-room (0.58–0.60 with an 8 m reach); at 1080p under TAA a frame is 0.45 ms without GI, 2.01 ms with the probes (main pass 1.24, probes 0.62), 5.25 ms with screen GI (its pass 4.27) and 5.36 ms with both |
| First frame | terrain-near setup and first frame at most 2.0 s, no pipeline compiled alone above 0.5 s, on a cold driver cache | miss: terrain-near 2.57 s (median of 5, NVIDIA's shader cache emptied before each), gltf-reference 1.77 s, materials 1.79 s; compiled alone (one compile worker), a lit variant takes up to 1.57 s, an OIT variant 1.52 s, the terrain's 0.50 s |
| Crowds at 50k | at most 12 ms a frame median (perf_crowd_50k), reference frames unmoved | met: 9.22 ms a frame; the GPU's passes are 5.48 ms (main 2.4, prepass 1.9, cascades 0.8, cull 0.3), so the frame waits on the CPU |
| UI per-frame cost | follows the change, not the tree | miss: the retained tree follows the change (1,022 elements idle 0.23 ms, one label 0.65 ms) but scrolling 1,000 rows costs 59 ms a tick; mounted screens cost their tree when idle (0.5–8.6 ms, leaderboard 8.6 ms for 44 elements) and a dense menu of 356 elements 70 ms, mostly allocation in style resolution (`perf_every_library_and_kit_screen`, `perf_dense_menu_300_elements_40_animating`) |
| Watercolour look | a watercolour primitive (Bousseau, MNPR) in place of today's oil-like result, with the art review | — |
| Distant impostors | their own shadow, the view from below and sway, with the art review | — |
| Latest features on wasm, determinism and browser tiers | measured before the release | — |

Deferred by the owner: UI text scale at 200 %, accessibility beyond
AccessKit, the overlay's GPU cost, SSSR and FSR follow-ups, W7 (CPU FFT
band for buoyancy) and wgpu 31 (unpublished). Ray tracing is a non-goal.

## Direction

`docs/design.md` §1 states the goals; this is the open work.

- **Native store packages:** TypeScript as Hermes bytecode under a C ABI
  host in a thin native shell. Open: which platforms come first, which
  sets the JS engine and the gate machines.
- **Game-code speed without a JIT:** measure a typical game's frame in
  interpreted Hermes against Chrome's JIT, then move hot paths into batched
  engine primitives.
- **Mini-game platforms:** WeChat on Android over WebGPU first (a host
  adapter and a renderer path without MRT, async pipelines or timestamps),
  then the GL tier for WeChat iOS and older devices, then Douyin, Alipay
  and others on the same adapter.
- **Loaded on demand:** optional capabilities (GPU extras, physics,
  navigation, video) as artifacts loaded after the first frame.
- **Determinism for every game:** report iteration over containers filled
  in arrival order, report the lint at publish and packaging, ask the
  platform for a `connect-src` policy on the play origin, check that both
  hosts disable floating-point contraction, and pin tick and save proofs in
  the fixtures.
- **Agent observability:** a `visible` reason code in the probe and
  `dump_entity`, a record of what was drawn after culling, borrow-conflict
  errors naming component, entity and source line, and captures from
  several viewpoints.
- **Features to compare APIs against** (from the review of expo/exact):
  model LOD, dithered fades, a view-model layer, water, glow, multi-light
  shadows, `render_scale` with a native HUD and camera turning between
  ticks. Exact's licence is checked before anything is ported.

## Structure

The rules are `AGENTS.md` (Structure before size) and `docs/design.md`
§2.1, §2.6 and §2.8. A refactor that moves no frame lands on the CPU gate
and is verified once on the GPU host. Open work is collected on PR #40.

- **og-ui:** paint's reuse reads change bits instead of comparing each node
  with the one it replaced; a control becomes an element kind; the tree
  takes explicit insert, update and remove; the immediate API becomes an
  engine-internal front end; og-ui's tests stop linking `og-render`.
- **og-render:** batches, the frame uniform and group 0 become stages
  shared by the passes, as culling is. The sources are grouped by owner
  (its README, Layout); `tests/structure.rs` refuses a mechanism that
  reaches into a world system or the renderer, and one system that
  reaches into another.
- **Tiers (§2.6):** the mini-game tier end to end (passes that read the
  depth prepass build no graph without MRT yet, and secondary views draw
  nothing), then the GL tier on a device.
- **Crate layers (2.1):** `crates/og-core/tests/layers.rs` lists the four
  edges that still break the rule, all in the UI crate and the probe
  (`og-probe` and `og-ui` onto `og-input`, `og-probe` and `og-tween`); that list is worked down to nothing.
- **og-runtime:** one function per feature (`extract/frame.rs`,
  `destruction::run`); the latent bugs marked in place (baked reflection
  probes, destruction's body weights, a stamp's quaternion without `w`).
  The thread-locals left are the process's roots, named where they are
  declared (`engine.rs`); nothing an engine owns is kept on the thread.
- **Engine package:** `startWorker` (`web/worker.ts`) and the `Shell`
  class (`web/shell.ts`) each keep their state in one closure or class of
  about 800 lines; a state object comes first, then a split by concern, as
  `runtime/` was split.
- **Optional runtime modules (§2.1):** variants that leave modules out,
  then separate artifacts loaded after the first frame.
- **One host implementation (§2.1):** native counterparts of the web
  host's TypeScript services, with the native shell.
- **og-cook** links `og-ui` for its parsers behind the `cook` feature; no
  data-only crate replaces that link.
- **Provider contract (§2.8):** `og relay` serves `ws://` only, so a public
  relay needs a TLS proxy; the platform renames its extensions.

## Open

- Parts as nodes: the progress, meter and stepper parts on PR #40, and
  panel, tooltip, slots, the HUD and touch controls behind the part-box
  painter's two ordering issues.
- The `-og` migration's revision-3 extras and the final reflection audit.
- The `docs/ui.md` §8 inventory's named gaps on PR #40.
- PR #9 waits for its author; #23 stays a draft until the performance pass.
- The GPU host's open portability failures: ui-debug's missing DX12
  reference fallback, core documents compared as bytes, a 2-unit CSS tint,
  and the wasm build's Windows command-line limit under sccache.
- The performance pass also finishes the paused `gi-screen` and
  `gate-presentation` branches.
- Crowds at 50k are bound by the CPU on the L4, not by a pass: the frame
  is 9.2 ms, its GPU passes 5.5 ms, and the render call takes 11.4 ms of
  CPU in profile mode, 5.4 ms of it the scene's update.
- og-audio's busy mix renders one second of audio in 98 ms on the L4's
  Xeon, against perf.sh's 50 ms bound.
- Mobile: before 0.8.0 the UI runs on at least one real iPhone and one
  real Android phone.
- The asset store's cook stat index: sessions still run the incremental
  cook scan.
- Paused archives, rebased only if still wanted: `archive/ui-catalogue`,
  `archive/ui-text-source`, `archive/platform-scaffold-contract`.
- Platform asks for 0.8.0, collected by the coordinator: creation schema
  and labels, retired kit and pack recommendations removed, the engine pin
  and mounted knowledge refreshed together, and the Svelte and store APIs
  once final.
- The 0.8.0 release publishes the pack catalogue (`docs/release.md` §The
  pack catalogue) for origin-agent's Engine page.
- Water look, after the structure refactor: the light shafts under water
  are faint, and the lake still shows bands from the wave sum
  (water-underwater-shafts, water-planar).
- UI blending into a linear target: the UI blends into the 8-bit sRGB
  output, so two equal paths can differ by 2 on NVIDIA; a linear target
  moves frames on every tier and is decided with the art review.

## Decided

- **Familiar syntax first:** Svelte templates and stores, standard CSS,
  GSAP, Yarn, bun:test, Rapier and the three.js vocabulary.
- **New style packs** need a separate art sign-off.
- **Not doing:** Cranelift, wasm-opt, Wild, remote compiler caches, Bazel,
  CRIU or process fork, VLM gating, Rust hot patching, splitting tests
  across machines, Radiance Cascades, and from Exact its Contract UI
  language, Rust gameplay and the DOM host.
- **`-og` properties that stay:** `-og-scroll-fade`,
  `-og-layout-transition` and `-og-text-path`, since standard CSS has no
  equivalent that keeps their output.
