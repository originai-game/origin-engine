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

| Item | Target |
|---|---|
| FSR 3.1 | at most TAAU + 0.5 ms on thin geometry at every scale, within 0.05 dB of the reference, grass not regressed |
| GI probes | screen GI at 0.72–0.80 of the path-traced reference, cheaper than now |
| First frame | terrain-near setup and first frame at most 2.0 s, no pipeline compiled alone above 0.5 s, on a cold driver cache |
| Crowds at 50k | at most 12 ms a frame median (perf_crowd_50k), reference frames unmoved |
| UI per-frame cost | follows the change, not the tree |
| Watercolour look | a watercolour primitive (Bousseau, MNPR) in place of today's oil-like result, with the art review |
| Distant impostors | their own shadow, the view from below and sway, with the art review |
| Latest features on wasm, determinism and browser tiers | measured before the release |

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
  engine-internal front end; the documents stop writing through
  `Docs::host_mut`; og-ui's tests stop linking `og-render`.
- **og-render:** culling, batches, the frame uniform and group 0 become
  stages shared by the passes.
- **Tiers (§2.6):** the mini-game tier end to end (passes that read the
  depth prepass build no graph without MRT yet, and secondary views draw
  nothing), then the GL tier on a device.
- **og-runtime:** the remaining 7 `thread_local!` tables; one function per
  feature (`extract/frame.rs`, `destruction::run`); the latent bugs marked
  in place (`followRotation`, baked reflection probes, destruction's body
  weights, a stamp's quaternion without `w`).
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
