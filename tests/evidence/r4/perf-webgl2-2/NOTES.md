# r4/perf-webgl2-2: the WebGL2 frame's wasm work and wgpu-hal's GL calls

Base: `origin/r4/candidate` @ 7ca4e251. Goal: WebGL2 ≥ 1.5× three.js 0.180
instanced on `tools/bench/threejs-compare.ts`'s reference scene (Mac,
70c120c0: engine 1.996 ms sustained vs three 2.540 ms, 1.273×; 1.5× needs
the engine at ≤ 1.693 ms, −0.30 ms or −15 %).

**Everything here was measured in a CPU-only orb** (Chrome 156 headless,
`--use-angle=gl-egl`, Mesa llvmpipe, 16 cores). Call counts are exact on
any GPU. Times are relative only and are not acceptance evidence. This
branch cannot show 1.5× on a GPU; the Mac lane has to.

## Profile first (base 7ca4e251)

`bun tools/bench/webgl2-trace.ts --only reference --times-only --gap 300
--frames 90 --profile`, V8 sampling at 100 µs in the engine Worker, three
rounds. Inclusive ms per frame (the frame call's own work; the gap's idle
excluded):

| phase (base) | ms / frame | share of the frame |
|---|---:|---:|
| `og_engine_frame` (all wasm work of a frame) | 3.38–3.94 | 100 % |
| render (`Renderer::render`, encode + submit) | 1.51–1.86 | ~46 % |
|  of it wgpu-hal GLES `Queue::submit` (GL calls, the JS filter) | 0.74–0.92 | ~23 % |
|  of it `CommandEncoder::finish` (wgpu-core) | 0.32–0.36 | ~9 % |
| tick (`Probe::tick_once`) | 1.13–1.25 | ~33 % |
|  of it the raycast record (`geometry::targets`, probe `before_tick`) | 0.22–0.46 | ~10 % |
|  of it one TS system's dispatch: `format!` of its time payload (Dragon4) | ~0.22 | ~6 % |
| extract (`extract::extract`) | 0.36–0.70 | ~13 % |
|  of it resource decodes to JSON (camera, settings, post) and their drops | ~0.15 | ~4 % |
| `World::system_info` (probe timings, every frame) | ~0.09 | ~2.5 % |

The NOTES of r4/perf-webgl2 put extraction at 2.9 ms inclusive. That is
gone: r4/perf-ui's slot reads already took it to ~0.4 ms. What remained was
in the tick and in rereading unchanged state.

## What changed (six commits, runtime first)

1. `a96bb691` **perf(probe)**: `World::system_ms` (og-ecs) answers the
   systems' names and times; `with_probe` stops copying every declaration
   each frame.
2. `8907c9d6` **perf(runtime)**: the raycast record (the probe's
   `before_tick`) no longer composes a world matrix per drawn mesh at every
   tick start. It keeps the drawn rows and the tick-start hierarchy and
   composes on a system's first raycast of the tick, from the same
   transforms, so answers are unchanged bit for bit. The og.Mesh half is
   kept while its columns (entities, geometry, visible, layers) and the
   geometry count are unchanged. og.Model rows are read in place by slot.
   `Hierarchy` keeps its per-entity flags apart from its matrices (16 B vs
   190 B per entity walked on a read where nothing moved) and makes a
   local matrix only when one is composed.
3. `a86ea722` **perf(runtime)**: `Layout::decode_kept`. A resource read
   every frame is decoded again only when its bytes changed.
   `extract::resource` and `fx::resource` answer `Arc<Value>`.
4. `2da3211c` **perf(runtime)**: a system call's time payload formats its
   f64 elapsed time with zmij (already in the tree for serde_json) instead
   of core's `{}`. The f32 fields keep `{}`, because zmij breaks exact ties
   to even and core rounds them up.
5. `c4e0f4c7` **build**: wgpu-hal 30.0.1 copied unchanged into
   `third_party/wgpu-hal` and put in place by `[patch.crates-io]`, as
   `basis-universal-sys` is (the workspace already excludes `third_party`).
   No code changes. Git normalised one CRLF file
   (`src/vulkan/descriptor.rs`) to LF.
6. `c5f57381` **perf(web)**: the patch, `src/gles/shadow.rs` plus routing
   in `queue.rs`. The GLES queue skips setters whose value it saw set in
   the same submit. It defers attachments, capabilities and attribute
   enables to the next command that observes them, and forgets everything
   after a command it does not follow (copies, buffer clears, barriers).
   Rules: `third_party/wgpu-hal/ORIGIN.md`.

Commits 1–4 build and pass on their own (`cargo check -p og-runtime` at
`8907c9d6` and `2da3211c`), so they can merge without 5–6.

## After

### WebGL calls per frame, reference scene (`counts.json` beside this file)

| | base | branch |
|---|---:|---:|
| without the JS filter (what leaves wasm) | 3,797 | **1,335** |
| with the JS filter (what reaches WebGL) | 761 | **732** |
| `framebufferTexture2D` (with filter) | 49 | 24 |
| `bindFramebuffer` / `disable` / `enable` (without filter) | 68 / 452 / 60 | 46 / 70 / 44 |
| `texParameteri` (without filter) | 480 | 48 |
| draws, `bufferSubData` bytes | 41, 85,904 | 41, 85,904 |

20k spheres (with filter): 168.5 → **131** calls a frame. `bufferSubData`
stays at 2,321,399 B.

### Worker time, reference scene, three rounds each, base and branch alternated

| ms / frame | base (A) | branch (B) |
|---|---:|---:|
| `og_engine_frame` | 3.38 / 3.94 / 3.45 | **3.00 / 2.77 / 3.13** (−17 %) |
| tick | 1.25 / 1.19 / 1.13 | **0.73 / 0.68 / 0.84** |
| extract | 0.36 / 0.48 / 0.42 | 0.38 / 0.32 / 0.35 |
| render | 1.51 / 1.86 / 1.65 | 1.75 / 1.62 / 1.72 (flat) |
| hal submit | 0.74 / 0.92 / 0.85 | 0.84 / 0.79 / 0.78 (flat) |
| round-trip p50 | 3.7 / 3.9 / 3.6 | 3.4 / 3.2 / 3.4 |

Inside submit (B vs A): the JS filter's own time fell 0.18 → 0.09–0.13 ms
and wasm-to-JS glue 0.17 → 0.12 ms, while the shadow's wasm bookkeeping
rose 0.03 → 0.06 ms. Native GL time is the same, since the same ~730 calls
reach WebGL. On llvmpipe that nets to about −0.1 ms, inside the noise. On
a GPU, where native GL calls cost less, the crossings saved are a larger
share. Two branch rounds' mean "busy" was inflated by `og_web_warm`
compiling pipelines in a timer between frames (behind-the-gate warm-up
finishing late). That is outside the frame round trip, so the table above
excludes it.

### `perf_extract_spheres_20k` (coordinator's request)

`cargo test -p og-runtime --release --test extract_cost -- --ignored
perf_extract_spheres_20k --nocapture`, same orb session, three runs each
(each run's printed best; each run then fails for want of a baseline for
this CPU):

| | run 1 | run 2 | run 3 |
|---|---:|---:|---:|
| base 7ca4e251 | 2.345 | 2.325 | 2.251 |
| branch (deferred compose only, an intermediate) | 1.909 | 1.990 | 1.943 |
| branch (final) | **1.261** | **1.235** | **1.240** |

That is −46 %, close to the ~1.1 ms pre-S1 cost on an orb. What remains of
`before_tick` is `Hierarchy::refresh` comparing 20,000 transforms (~0.14 ms
here). No baseline recorded.

## Tests that fail on the old code

- `geometry::tests::the_raycast_record_composes_only_what_moved` (extended)
  asserts that a tick composes no world matrix (old code: 350 at the first
  tick). It also checks the record's rows against a fresh read after a mesh
  is hidden and another's geometry swapped (fails with the kept mesh rows
  never reread; mutation-checked).
- `codec::tests::a_kept_decode_is_redone_only_when_the_bytes_change`
  (`decode_kept` does not exist on base).
- `og-ecs` `systems_are_timed_with_the_world_clock` (extended):
  `system_ms` is empty without a clock and `[("idle", 1.5)]` with one.
- `feel::tests::the_time_payload_reads_back_as_the_formatted_numbers`
  checks equivalence, not a regression: 130,000 payloads (long runs at four
  scales, random bits, odd values) parse to the same bits as `{}`'s text. It
  caught zmij's f32 tie rounding during development.
- **The wgpu-hal patch has no committed test.** It needs a GL context.
  Evidence: the counts above and frame 90 at AE 0 (below). A browser-tier
  test asserting the unfiltered count (< 1,500) would hold it; that is the
  coordinator's to add, since it runs in the tiers agents do not run.

No new perf test, so no new M4 Pro baseline. The existing
`extract-spheres-20k.json` (M4 Pro 0.531 ms, already flagged at 0.798 ms)
should drop.

## What moves

- **Frames**: reference scene frame 90 on the WebGL2 tier, base vs branch,
  `magick compare -metric AE` = **0** with the JS filter and **0**
  without it (the patched hal alone), and the frame was inspected. Native
  tiers use Metal and Vulkan, whose wgpu-hal code is byte-identical to
  30.0.1, and og-render is untouched. The web-reference `--webgl` frames
  were not run here (AGENTS.md).
- **World hashes**: should not move. No simulation code changed, every
  time-payload field parses to the same number, and raycast answers are
  composed from the same tick-start transforms. Determinism and hashes were
  not run here; they are wave-end.
- **Wasm size**: 3D publish (single-threaded, `wasm` profile, wasm-opt
  -Oz, names stripped) brotli 3,336,549 → 3,341,791 B (**+5.2 kB**). The 3D
  lanes variant was 80.36 % of 4 MiB, so expect about 80.49 %. The
  coordinator's size gate measures it.

## Decisions for the coordinator

1. **Mac re-measure** (`threejs-compare --mode WebGL2`, reference and
   spheres). The orb shows −17 % of the frame's wasm work and −62 % of the
   calls leaving wasm, but whether that is the −15 % of sustained frame
   time 1.5× needs depends on how much of the Mac's 1.996 ms is GPU-bound.
2. **Merge the runtime commits (1–4) even if the hal patch needs another
   round.** They stand alone.
3. **The vendored wgpu-hal** (5–6): adopt it, or keep only the JS filter.
   It is 53 k lines of upstream under `third_party` plus a ~450-line patch,
   to be carried over at every wgpu move (ORIGIN.md). In the orb its frame
   gain is small because the JS filter already absorbs most calls; its
   value is on GPUs, where JS crossings dominate more.
4. **opt-level 3 for wgpu-core/wgpu-hal: not adopted.** +41 kB brotli
   against a lanes variant already over its 80 % target. Take it only if
   the Mac falls short after 1–6.
5. **The spheres scene**: every sphere moves every tick, and three's
   InstancedMesh also re-uploads all 20k matrices each frame
   (`needsUpdate = true`, 1.28 MB). "Upload only changed instances" would
   therefore not help this scene. The engine uploads 2.32 MB, ~116 B an
   instance against the 64 B `GpuTransform`, so something besides the
   transforms is uploaded per instance. That is the og-render lead to
   follow (not verified here).
6. The perf lane's `extract-spheres-20k` baseline can be re-recorded on the
   M4 Pro after merge.

## The coordinator's decisions (2026-10-25, Mac M4 Pro measurements)

Commits 1–4 merged as 9ff1a977 (now main). The vendored wgpu-hal and its
GLES shadow (5–6) are **not adopted**, and 7ebac12e's branch is kept only
for reference (`r4/hal-shadow` = 049d7937 carries the two commits on top
of 9ff1a977 for the isolated measurement). opt-level 3 stays out.

- `perf_extract_spheres_20k`, release, three serial runs after idle
  minutes on 9ff1a977: 0.440 / 0.435 / 0.447 ms, median 0.440, under the
  0.531 baseline (its tolerance limit 0.611). The S1 regression is closed;
  the baseline stays.
- `threejs-compare --mode WebGL2`, 600 frames, 60 warm, two rounds per
  mode, an idle minute before each, loads 3.1–5.8:

  | Revision | Reference engine | three | Ratio | Spheres engine | three | Ratio |
  |---|---:|---:|---:|---:|---:|---:|
  | 9ff1a977 (JS filter only) | 1.952 ms / 512 FPS | 2.500 / 400 | **1.281×** | 3.742 / 267 | 2.370 / 422 | 0.633× |
  | 049d7937 (filter + hal shadow) | 1.993 / 502 | 2.578 / 388 | 1.294× | 3.901 / 256 | 2.370 / 422 | 0.608× |

  The shadow does not make the engine faster on the Mac (reference +2.1 %,
  spheres +4.2 %, within two rounds' noise); the ratio's rise is the
  slower three control. No performance case for carrying a 53 k-line fork
  across wgpu moves.
- WebGPU control on 9ff1a977: 1.313 ms / 762 FPS against the same three
  reference, 1.904× (1.937× before the runtime commits).

Still owed: WebGL2 ≥ 1.5× on the reference scene (1.281×) and the spheres
gap (0.63×). The lead is decision 5 above, the ~116 B per instance the
engine uploads against the 64 B `GpuTransform`, which is og-render work,
not a GL-call count: at 732 calls a frame after the JS filter the call
count is no longer where the frame goes.
