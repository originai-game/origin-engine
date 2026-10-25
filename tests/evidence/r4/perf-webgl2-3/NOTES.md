# r4/perf-webgl2-3: what the WebGL2 frame uploads per instance

Base: `origin/main` @ fdbbbad3 (the code path of 9ff1a977, where the Mac
measured WebGL2 reference 1.952 ms vs three 2.500 ms, 1.281×, and spheres
3.742 ms vs 2.370 ms, 0.633×). Goal: WebGL2 ≥ 1.5× three.js on the
reference scene.

**Measured in a CPU-only orb** (Chrome 156 headless, `--use-angle=gl-egl`,
Mesa llvmpipe, 16 cores). Calls and bytes are exact on any GPU. Times are
relative only, and on the spheres scene llvmpipe's rasterising competes
with the Worker for the CPU. This is not acceptance evidence; the Mac lane
is.

## Where the 116 bytes an instance came from

The per-instance record is 64 B, as documented (`instances::GpuTransform`:
the x, y, z of four columns, 48 B, then the layer lane and three look lanes,
16 B). Nothing else is in it. The extra bytes were whole records uploaded
twice:

- On the CPU culling path (WebGL2 always; WebGPU below
  `GPU_CULLING_MIN_INSTANCES` or with object motion) a pass draws a batch
  that keeps ≥ 90 % of its instances whole from the sorted records, and
  otherwise from copies of the visible ones appended after the sorted
  records (`Renderer::cull`).
- The spheres camera leaves about a fifth of the field out of view, so the
  one batch is drawn from 16,272 copies. The upload still sent all 20,000
  sorted records, which no draw read: 20,000 × 64 + 16,272 × 64 + the
  frame's uniforms = 2,321,399 B, 116 B an instance.

Commit 1 uploads only what a draw reads. The frame's CPU culls mark the
batches a list draws whole (`CullScratch::whole`). `DirtyUpload::write`
takes the byte ranges a draw reads and writes only their 4 KiB blocks that
differ from what the buffer holds, with a per-block "known" flag. A block
left out while no draw read it is therefore written whole once one does.
Object motion's previous transforms go through the same tracking and are
computed only for drawn records; they used to be rewritten whole every
frame.

Shrinking or splitting the record was considered and not done. The sphere
drawn now costs 64 B; three's InstancedMesh sends 64 B for every sphere
(1,280,000 B). Moving the 16 B of layer and look lanes into a static stream
would save 25 % of a moving record. The cost would be a second instance
vertex buffer on every draw (more `bindBuffer` and `vertexAttribPointer`
per draw on GL) and a change to every lit pipeline's vertex layout and
instance input (`instance.wgsl`, `pbr.wgsl`, hooks, lean, array, physical).
That is a frame risk across every tier for a gain the spheres scene no
longer needs to stay below three's byte count.

## Counts and bytes (`counts.json` beside this file)

WebGL2, with the JS filter, per frame:

| | reference base | reference branch | spheres base | spheres branch |
|---|---:|---:|---:|---:|
| WebGL calls | 761 | 761 | 133.1 | 133.3 |
| `bufferSubData` calls | 15 | 15 | 10 | 10 |
| `bufferSubData` bytes | 85,904 | 85,904 | **2,321,399** | **1,043,447** (−55 %) |
| bytes per instance submitted | — | — | 116 | 52 |
| `texSubImage2D` bytes | 16,384 | 16,384 | 0 | 0 |

The spheres' upload is now below three's 1,280,000 B. The reference scene's
bytes do not move: its main pass draws every batch whole, so the sorted
records are all read. Its largest write is 41,792 B (653 records of 64 B
against 501 sorted instances, consistent with culled copies for the shadow
passes; not traced to the buffer).

## The per-byte path, and the raycast record

The spheres profile (base, below) splits the upload's cost:
`DirtyUpload::write` 1.39 ms. Of that, wgpu's `write_buffer` takes 0.93 ms:
`StagingBuffer::new` 0.64 ms, which on WebGL2 is wgpu-hal GLES emulating a
MAP_WRITE buffer with a zeroed `vec![0; size]` per write, and the copy into
it about 0.3 ms. The dirty compare and the held copy take 0.45 ms. Then
`bufferSubData` in the hal submit takes 0.88 ms. All of it scales with
bytes, so commit 1 cut it about in half: `DirtyUpload::write` 0.52–0.57 ms.

The same profile showed lead 3's per-frame runtime work in the raycast
record, not in extract's codec. The probe's tick start re-read every
transform into the record's `Hierarchy` per entity, 1.4–2.0 ms for 20,000
moving spheres, though most ticks never raycast. Commit 2 splits
`Hierarchy::refresh` into `capture` (copy the transform columns and
parents, compare with the last read) and `apply` (bring the slots to that
read). The record's tick start only captures; the first `world_of` (a
raycast) applies. Captures without a question between them add up: an
archetype is still only when every capture since the last apply found it
still, and a reparenting at any of them recomposes everything. Extraction
still refreshes whole. The record composes from the same tick-start bytes,
so its answers are unchanged bit for bit (the record test compares every
target with a fresh read).

## Worker time, `og_engine_frame` and its parts (ms a frame, orb, V8 sampling)

`webgl2-trace.ts --times-only --gap 300 --profile`, builds with names kept
(`wasm-opt -g`, same flags), base and branch alternated.

Spheres (20 frames after 20 warm; three rounds each):

| | frame | render | upload | hal submit | tick | record | extract |
|---|---:|---:|---:|---:|---:|---:|---:|
| base | 10.96 / 12.28 / 12.41 | 4.65 / 5.47 / 5.71 | 1.39 / 1.66 / 1.50 | 1.22 / 1.53 / 1.53 | 2.79 / 3.16 / 2.67 | 1.44 / 2.02 / 1.40 | 3.28 / 3.44 / 3.75 |
| commit 1 only | 9.82 | 3.68 | 0.53 | 0.84 | 2.41 | 1.31 | 3.56 |
| branch | **9.06 / 10.57 / 10.43** | 3.66 / 4.72 / 4.02 | 0.55 / 0.57 / 0.52 | 0.85 / 1.15 / 0.87 | 1.64 / 1.76 / 2.28 | **0.50 / 0.53 / 0.55** | 3.58 / 3.78 / 3.85 |

That is −14 to −17 % of the spheres frame's Worker work.

Reference (90 frames after 60 warm; three rounds each):

| | frame | render | hal submit | finish | tick | record | extract | round trip p50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| base | 3.15 / 3.18 / 3.40 | 1.66 / 1.56 / 1.76 | 0.77 / 0.67 / 0.91 | 0.38 / 0.33 / 0.30 | 0.85 / 0.99 / 0.95 | 0.080 / 0.075 / 0.078 | 0.39 / 0.33 / 0.37 | 3.6 / 3.5 / 3.6 |
| branch | 2.92 / 3.05 / 3.13 | 1.59 / 1.59 / 1.66 | 0.70 / 0.75 / 0.81 | 0.33 / 0.34 / 0.35 | 0.73 / 0.85 / 0.84 | 0.052 / 0.075 / 0.089 | 0.34 / 0.37 / 0.34 | 3.5 / 3.5 / 3.6 |

The reference frame does not move beyond the noise. `World::step`, which
neither commit touches, moved by as much (0.76–0.91 vs 0.67–0.77).
**This stream does not bring the reference scene to 1.5×.** Its instance
upload was already small and fully read, and its record cost 0.08 ms.

What the reference frame's Worker time is (base, orb): render 1.6 ms, of
which the hal GLES submit (GL calls through the filter) is 0.7–0.9 and
wgpu-core's `CommandEncoder::finish` 0.3–0.4. The tick is 0.85–1.0 ms:
`World::step` 0.76–0.91, of which og-anim 0.18, the TS system's dispatch
0.18, and og-physics' step 0.17 in a scene with no physics body (0.095 ms
self plus `is_sorted` 0.034). Extract is 0.35 ms, spread thin: no codec
line above 0.05 ms.

## Tests that fail on the old code

- og-tests `tests/upload.rs`, `only_the_records_a_draw_reads_are_uploaded`
  (new): spheres-20k with the camera turned past the field's edge and no
  shadows. The first frame uploads between the drawn records' bytes and
  that rounded up to blocks plus one. On the old code: 13,138 spheres drawn
  wrote 2,120,832 B, 161 B a sphere drawn, against at most 847,872. The
  first-frame bound of the existing test changed from "every instance" to
  "every instance in view". The partial-upload test (a frame after a
  partial upload equals a fresh renderer's, pixel for pixel) still passes.
  Ran on lavapipe in the orb.
- og-render `upload::tests::blocks_no_draw_reads_are_not_written` (new):
  undrawn blocks are not written, two ranges meeting in a block write it
  once, and a block first drawn later is written whole.
- og-runtime `geometry::tests::the_raycast_record_composes_only_what_moved`
  (extended): the tick leaves the record's read pending (no slot work
  before a raycast). It also covers ticks with no raycast between them: a
  move a tick before a still one (composes 1), a move and its undo (0), and
  a reparenting a tick before a still one (347, everything). Mutation
  checks: dropping either accumulation (the still flags or the reparented
  flag) fails it with "kept a stale world matrix".

Also run: `cargo test -p og-render --lib` (84 passed), `cargo test -p
og-runtime` (every lib and integration test), and `cargo clippy
--all-targets -D warnings` and `cargo fmt --check` on og-render, og-tests
and og-runtime.

og-runtime's ignored perf test `raycast_record_cost` was run in release
three times each, branch and then base (og-runtime sources only), in the
orb with no baseline for its CPU. It times `og.render.raycast` between
ticks, which captures and applies in one call:

| best ms a read, 20,000 meshes | base | branch |
|---|---:|---:|
| `raycast-record-20k` (still) | 1.849 / 1.560 / 1.823 | 0.869 / 0.837 / 0.849 |
| `raycast-record-20k-moving` | 3.282 / 3.103 / 2.951 | 2.123 / 2.130 / 2.132 |

That read does the same work as before (capture, then apply on the first
compose). The native drop is not attributed: the wasm profile shows no
such gain for extraction's refresh (base 0.86–1.0 ms, branch capture 0.23
plus apply 0.74). The Mac's baselines for these two may need re-recording
after merge.

## What moves

- **Frames: none should.** Only bytes no draw reads are left unwritten, and
  the record composes from the same bytes. WebGL2 frame 90 of the reference
  scene, base vs commit 1 and base vs branch: `magick compare -metric AE` =
  **0**, inspected as correct. Spheres frame 30, base vs commit 1 (two
  captures) and base vs branch: **0**. One early base capture of the
  spheres differed by 402,264 px from a second base capture, which equals
  commit 1's and the branch's exactly. That is the `--shot` path's first
  run after building a dist, not this change. The native tiers' reference
  frames and web-reference were not run (AGENTS.md); commit 1 runs on
  every tier's CPU culling path.
- **World hashes**: no simulation code changed. The record's answers are
  bit for bit a fresh read's (tested).
- **`FrameStats::instance_upload_bytes`** now also counts object motion's
  previous transforms (written through the same tracking). Only
  `tests/upload.rs` reads it.
- **Wasm size** (built in the orb: the `wasm` profile, wasm-bindgen,
  `scripts/wasm-opt.sh`, brotli -q 11):

  | variant | base | branch | delta |
  |---|---:|---:|---:|
  | 3D publish, single-threaded | 3,188,790 | 3,186,924 | −1,866 B |
  | 3D lanes | 3,214,921 (76.65 % of 4 MiB) | 3,216,659 (76.69 %) | **+1,738 B** |

  The size gate's own measurement is the coordinator's.

## For the Mac to re-measure

1. `threejs-compare --mode WebGL2`, reference and spheres. Expect the
   spheres engine time to fall (−55 % upload bytes, −14 to −17 % Worker
   work in the orb). Expect the reference to stay within its noise of
   1.952 ms.
2. `perf_extract_spheres_20k` (it times the tick with the record): the
   record's tick start should fall. Its baseline (0.531 ms) can be lowered
   after merge if it does. Also `perf_raycast_record_20k` and its moving
   twin, which fell in the orb (table above).
3. The WebGPU control (reference 1.313 ms) should not move. Its GPU culling
   path uploads everything as before; its CPU path gets commit 1.

## Decisions for the coordinator

1. **Merge both commits.** Neither moves a frame, and each has a test that
   fails without it.
2. **The reference scene's 1.5× is not in the per-instance upload.** At
   85,904 B a frame, all of it read, the remaining WebGL2-specific Worker
   cost is the hal GLES submit and wgpu-core's encoder finish (about a
   third of the reference frame in the orb), which the call-count work
   already took as far as it went. Candidates for a further stream,
   none measured on a GPU:
   - og-physics stepping a world with no bodies (0.17 ms of 3.1 in the
     orb; its state block and hashes need care);
   - a persistent mapped staging ring for the instance buffer, which drops
     wgpu-hal's per-write zeroed allocation (0.18 ms of the spheres frame,
     negligible on the reference);
   - fewer render passes on WebGL2. Each pass is a framebuffer switch and
     an ANGLE encoder: there are ~22, ten of them the bloom chain.
   Whether the Mac's 1.952 ms is CPU- or GPU-bound decides which of these
   matters; the orb cannot say.
3. **Splitting the instance record** (48 B per frame, 16 B static) was not
   done, for the reasons above. Revisit it only if the spheres scene gets a
   budget.
