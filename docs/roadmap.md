# Roadmap

What is running, what is next and what is open. What is done is in git and
the changelog; rules are in `AGENTS.md`, the UI contract in `docs/ui.md`.

## Order

1. **The UI art direction (now).** One design language across the style
   packs: the built-in components and screens (`@ui/`), dressed by the
   fantasy, scifi and cartoon packs through tokens and parts.
2. **Native desktop packages (now)** (§Direction).
3. **The performance pass** (§Below target), then the real-device run.
4. **The engine's showcase page**, rebuilt from scratch as a page of the
   platform (origingame).
5. **Compatibility:** Safari, phones (Pixel 10, PowerVR, Adreno, Mali) and
   HDR on a real display.
6. **Mini-game platforms** (§Direction).

Releases are 0.9.x patches cut at a milestone (`docs/release.md` §3);
between them the platform pins dev builds (§1a).

## Running

Streams report to the coordinator thread
[T-01a1dd0e](https://ampcode.com/threads/T-01a1dd0e-e1c3-747f-9de3-40733ec93490).
These are on branches, not on main.

- **The UI art direction** (`ui-art-*`): the families' colours derived
  from the look's seeds, one selection and one primary action across the
  packs, and the frame review of the built-in screens under each pack on
  the L4.
- **Native desktop** (`native-*`): a C ABI host generated from the IDL
  (`og-host-c`) running the game as Hermes bytecode in a desktop window,
  with the engine's audio, gamepads, and a Windows package.
- **Safari on a real Mac:** the web suites (wire, export, player, device,
  motion ticks) in Safari on macrunner, an M4 Pro.
- **Gate throughput on the L4** (`gate-l4-throughput`): opt-in
  measurements of wasm product reuse, GPU document shards and Cargo unit
  graphs.
- **The structure work** (§Structure).

## Next

1. The web-tier re-records.
2. The performance pass (below).
3. The real-device run.

## Below target

Targets not met yet. The performance pass re-measures each baseline on
the L4 and restates its target there at the same ratio.

| Item | Target | L4, 2026-11-14 (b089eb332, release, Vulkan) |
|---|---|---|
| FSR 3.1 | at most TAAU + 0.5 ms on thin geometry at every scale, within 0.05 dB of the reference, grass not regressed | miss: the resolve is TAAU + 0.37, + 0.46 and + 0.54 ms at scales 0.5, 0.67 and 0.77 (1080p, `fsr::cost`); against the reference (`fsr::compare`) FSR is 0.84–1.71 dB under TAAU on thin static geometry, 3.6–3.8 dB over it while panning, and 0.18–0.29 dB under it on grass |
| GI probes | screen GI at 0.72–0.80 of the path-traced reference, cheaper than now | met in G, 0.01 over in R and B: screen GI (horizon gather over visibility bitmasks, 2 rays × 16 steps) gathers 0.810 / 0.781 / 0.812 of a path-traced reference's indirect light in gi-room (`gi::reference`; the frame's ceiling is 0.812 / 0.752 / 0.737, exceeded in G and B by the half run assumed past the screen edge); at 1080p under TAA a frame is 0.44 ms without GI, 1.99 ms with the probes, 5.09 ms with screen GI (its pass 4.03, was 4.27; 3.18 at 12 steps) and 5.74 ms with both (`gi::cost`, L4, 4452711b0) |
| UI per-frame cost | follows the change, not the tree | miss: the retained tree follows the change (1,022 elements idle 0.23 ms, one label 0.65 ms) but scrolling 1,000 rows costs 59 ms a tick; mounted screens cost their tree when idle (0.5–8.6 ms, leaderboard 8.6 ms for 44 elements) and a dense menu of 356 elements 70 ms, mostly allocation in style resolution (`perf_every_library_and_kit_screen`, `perf_dense_menu_300_elements_40_animating`) |
| Watercolour look | a watercolour primitive (Bousseau, MNPR) in place of today's oil-like result, with the art review | — |
| Distant impostors | their own shadow, the view from below and sway, with the art review | — |
| Latest features on wasm, determinism and browser tiers | measured before a release | — |

Deferred by the owner: UI text scale at 200 %, accessibility beyond
AccessKit, the overlay's GPU cost, SSSR and FSR follow-ups, W7 (CPU FFT
band for buoyancy) and wgpu 31 (unpublished). Ray tracing is a non-goal.

## Direction

`docs/design.md` §1 states the goals; this is the open work.

- **Native store packages:** TypeScript as Hermes bytecode under a C ABI
  host in a thin native shell; desktop first (§Running). Open: the store
  platforms after it and their gate machines.
- **Game-code speed without a JIT:** measure a typical game's frame in
  interpreted Hermes against Chrome's JIT, then move hot paths into batched
  engine primitives.
- **Mini-game platforms:** WeChat on Android over WebGPU first (a host
  adapter and a renderer path without MRT, async pipelines or timestamps),
  then the GL tier for WeChat iOS and older devices, then Douyin, Alipay
  and others on the same adapter.
- **Loaded on demand:** physics, story, nav, motion, destruction and
  video load from the CDN on demand in the single-threaded publish module;
  the lanes module and the GPU extras are next (§Structure).
- **Determinism for every game:** report iteration over containers filled
  in arrival order, report the lint at publish and packaging, ask the
  platform for a `connect-src` policy on the play origin, check that both
  hosts disable floating-point contraction, and pin tick and save proofs in
  the fixtures.
- **HLOD follow-ups** (`docs/design.md` §10.6): a node over several
  cells. Not planned: one draw for a tree's proxies. Each proxy is a
  batch of its own in every pass that sees it (404 draws against 244
  looking across the bench town), but on the L4 cutting that view from 699
  to 404 draws left its frame at 24 ms, and merging the proxies would save
  about 110 more. It would need the tree's textures packed into shared
  atlases and the cluster path's mesh table to hold a level per node.
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
  engine-internal front end.
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
- **Optional runtime modules (§2.1):** the single-threaded publish
  modules are split and load on demand (docs/web-export.md §Modules); open:
  the lanes modules (each pool Worker must load a module into its own
  instance) and the GPU extras (a first-frame profile from the L4).
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
- The `docs/css-coverage.md` inventory's named gaps on PR #40.
- PR #23 stays a draft until the performance pass.
- The GPU host's open portability failures: ui-debug's missing DX12
  reference fallback, core documents compared as bytes, a 2-unit CSS tint,
  and the wasm build's Windows command-line limit under sccache.
- Crowds at 50k are bound by the CPU on the L4, not by a pass: the frame
  is 9.2 ms, its GPU passes 5.5 ms, and the render call takes 11.4 ms of
  CPU in profile mode, 5.4 ms of it the scene's update.
- og-audio's busy mix renders one second of audio in 98 ms on the L4's
  Xeon, against perf.sh's 50 ms bound.
- Mobile: run the UI on at least one real iPhone and one real Android
  phone (the real-device run).
- The asset store's cook stat index: sessions still run the incremental
  cook scan.
- Paused archives (tags), rebased only if still wanted: `archive/ui-catalogue`,
  `archive/ui-text-source`, `archive/platform-scaffold-contract`.
- Platform asks, collected by the coordinator: creation schema
  and labels, retired kit and pack recommendations removed, the engine pin
  and mounted knowledge refreshed together, and the Svelte and store APIs
  once final.
- The pack catalogue for origin-agent's Engine page: `manifest.json`
  lists only the downloadable vfx packs (`docs/release.md` §1). Open:
  `ui`, `fonts` and `looks` sections beside `vfx`, every pack bundled or
  downloadable as `{ version, minEngine, title, description, tags, size,
  bundled, url?, sha256?, preview }` from its own manifest, with a preview
  at `<version>/previews/<kind>/<name>.webp`.
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
