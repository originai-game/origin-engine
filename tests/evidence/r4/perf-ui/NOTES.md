# r4/perf-ui: perf-games follow-ups (row 90) and Lantern Vale's render

Branch `r4/perf-ui`, from `origin/r4/candidate` at 753e8d9d. Six commits:

| Commit | What | From |
|---|---|---|
| 0e26a12c | Sprite, flipbook, afterimage and tile-layer extraction reads fields by offset (`codec::Slot`, `Key`) | perf-2 ae207b27, re-applied |
| cd87b574 | `og.ui.end` keeps a text's measurement while its string, font, size and width are unchanged; `Ui::text_measurements()` | perf-2 6f7f9f6c, re-applied |
| 66dfc6c5 | og-ui assigns painter levels through a grid, now sized by the command count | perf-2 a53bb596, re-applied and changed |
| 424aa29f | physics: soft bodies leave the lanes pool for a while when it does not pay (`lanes::Gauge`) | perf-2 4c7ac856, unchanged code, new message |
| 3e84966d | Model extraction reads og.Model rows in place by offset | new (Lantern profile) |
| 09fd772d | og-render keeps the transmission copy's view and mip chain with the copy | new (Lantern profile) |

Every number below was measured **on an orb**: 16 vCPU Intel Xeon @ 2.60 GHz,
CPU only, Vulkan on lavapipe (Mesa 25.0.7 from bookworm-backports), release
`og` and addon. The numbers are relative (old against new on the same orb, runs
interleaved) and **not comparable to the Metal numbers** in
`tests/evidence/r4/measure/GPU.md`. Lavapipe runs the GPU on the same CPU, so
render times here are mostly lavapipe's rasterising.

## What landed from perf-2 and what was left out

- **ae207b27 (fields by offset):** re-applied. In the conflicts, the
  candidate's `Row::field` refuses list fields (it keeps that, inside the
  generic `field`), and the sprite has new fields `layers` and `facing` (now
  slots too). codec's test reads every field of every engine record by slot as
  well as by name, against the JSON.
- **6f7f9f6c (text-measure cache):** re-applied. The candidate's new third
  `layout` call (the anchored groups) takes the cache too. Added
  `Ui::text_measurements()` and the test
  `layout::text_cache::a_frame_measures_only_the_texts_that_changed`.
- **a53bb596 (painter-level grid):** re-applied, then changed. perf-2's grid cut
  the frame's bounds into 32 px cells (920 at 1280×720) whatever the command
  count, and a full-frame backdrop is filed in every cell. That cost 44 to 54 µs
  per frame, where the old pair-by-pair test of a 40-command HUD costs 1.4 µs.
  On the orb it made Kestrel's `og.ui.end` *slower* (0.081 → 0.090 ms). The
  grid now has about one cell per four commands (at most 64×64, at least
  32 px). Release, a full-frame backdrop plus n 40×24 px rectangles, mean of
  2,000 runs:

  | n | perf-2 grid µs | sized grid µs | quadratic (old) µs |
  |---:|---:|---:|---:|
  | 10 | 53.6 | 0.9 | 0.3–0.5 |
  | 40 | 44.0 | 3.7 | 1.4 |
  | 160 | 60.0 | 16.8 | 13.9 |
  | 320 | 76.7 | 32.7 | 54.2 |
  | 1,620 (full dissolve) | 315.8 | 309.8 | 1,506 |

  The levels do not depend on how the grid is cut, because a rectangle is
  filed in every cell it touches. The existing test against the quadratic
  levels passes unchanged.
- **4c7ac856 ("the terrain streaming test in progress"):** the message is
  wrong. The commit is the lanes `Gauge` for soft bodies (row 90's first
  follow-up, "the lanes on a loaded machine"). It applies cleanly and its unit
  test passes, so it landed with a message that describes it. Kestrel, no
  renderer, `og.physics` mean of 600 ticks, five interleaved rounds:
  - At load average 21–23: 3.00 → 2.07 ms (median; ranges 1.93–3.23 →
    1.69–2.29).
  - Idle: 0.815 → 0.817 ms.

  The hash was `121254f596361402` in every run.
- **01123635 (terrain-stream test): left out.** The candidate's 195cd595
  ("test(terrain): require tiles after the first frame") covers the same
  scenario. It moves the camera from the coarse tile into the valley after the
  first settle, and requires decoded and resident tiles to grow while all 600
  hashes match resident mode. That is why the file conflicted. Landing perf-2's
  version would replace a test that already does the job with a second runner
  (setup steps, timed moves).

## Lantern Vale's render: what it submits above the other two

Method: scratch timers (`instrumentation.patch`, the profiling build's diff;
its `perfui_mark` lines are not on the branch) around extract and each phase
of `Renderer::render`, plus `FrameStats` per frame. All three games ran the
perf-games scripts (`scripts/run.sh`, unpaced, 600 ticks, 1280×720).

| Per frame (median) | Hollowbell | Kestrel | Lantern |
|---|---:|---:|---:|
| draw calls | 33 | 70 | 62 |
| graph passes / transient textures | 13 / 9 | 20 / 13 | 24 / 17 |
| main-pass triangles | 0 | 2,132 | 28,296 (+20,480 terrain) |
| shadow passes / terrain shadow draws | 0 / 0 | 3 / 0 | 3 / 9 |
| extract, ms (CPU) | 0.44 | n/m | 0.55 |
| encode, ms (CPU) | 0.07 | n/m | 0.23 |

What Lantern has that the others do not:

- **Transmission.** The slimes' jelly is `KHR_materials_transmission`. From the
  first frame with a slime in view (tick ≈126 in this script), every frame
  copies the HDR scene and fills an 11-level mip chain. The copy texture was
  kept, but its view, and the chain's 20 level views and 10 bind groups, were
  made again every frame. On lavapipe, making a view of the copy while the GPU
  still read it waited for the frame in flight: about 480 ms on every other
  frame, in `create_view`.
- **The model rows.** Extract decoded each og.Model row to a JSON map to read
  nine fields: 0.167 ms of the 0.55 ms (p95 0.294). That is the cost sprites
  had before ae207b27, and Hollowbell has no models.
- **Terrain and a longer post chain**, which are its content: 20k terrain
  triangles plus 9 shadow draws; bloom (10 passes), auto exposure (meter,
  reduces, adapt), TAA, outline, ink lines, prepass. Nothing is wasted there
  on the CPU side. Each of those phases measured 0.001–0.05 ms.

Cuts:

- **3e84966d:** model rows are read in place by offset.
  - Lantern: 0.167 → 0.039 ms per frame (p95 0.294 → 0.070).
  - The perf lane's `perf_extract_models_20k` (20,000 rows, render time of a
    frame): old code 33.0 ms, new code 5.02 ms.
- **09fd772d:** the copy's view and mip chain (`mips::MipChain`) are made with
  the copy.
  - Encode of a transmissive frame: 0.263 → 0.146 ms (median).
  - The per-frame `create_view` is gone.
  - On lavapipe the stall moves to the next device call (`gpu_timings`' poll)
    and the orb's render times do not improve. Lavapipe serialises the device
    with its queue, so its render time cannot show what this does on Metal.

**The target is not proven.** The miss was unpaced release render p95 8.03 ms
on Metal, with the paced p95 at 2.81 ms. That points at waiting on the GPU,
which this orb cannot measure. On the CPU the branch takes about 0.13 ms (model
rows) off every frame and 0.12 ms (encode) off every transmissive frame, and
removes a resource re-creation per frame. Against a 0.03 ms miss that is
plausible, but it needs a Metal run (see the decisions below).

## Row 90's other items

- **Text re-measurement:** fixed (cd87b574). `og.ui.end` mean, release, no
  renderer, 600 ticks, median of three runs, branch as landed:

  | Game | old ms | new ms |
  |---|---:|---:|
  | Lantern | 0.124 | 0.087 |
  | Hollowbell | 0.076 | 0.074 |
  | Kestrel | 0.079 | 0.077 |

  Lantern's step p95 is 2.45 → 2.05 ms.
- **Sprite fields by name:** fixed (0e26a12c). `perf_extract_sprites_20k`:
  old code 58.8 ms, new code 11.9 ms.
- **Lantern's dev step spikes (screen-transition dissolve, `assign_levels`
  quadratic):** fixed by 66dfc6c5. The 1,620-cell dissolve is 1,506 → 310 µs
  in release. Not re-measured in dev on the orb. perf-2's message has the dev
  numbers on macOS: step p95 6.9–12.3 → 3.0–3.8 ms.
- **Lanes on a loaded machine:** 424aa29f, numbers above.

## Tests that fail on the old code

- `og-ui layout::text_cache::a_frame_measures_only_the_texts_that_changed`:
  run on 753e8d9d with a counter in `size_w`/`distribute_w`, an unchanged
  frame counts 6 measurements against 0 expected.
- `og-runtime extract_cost::perf_extract_sprites_20k` (perf lane): old 58.8 ms
  against a baseline of 11.9 ms (limit 13.7).
- `og-runtime extract_cost::perf_extract_models_20k` (perf lane): old 33.0 ms
  against a baseline of 5.02 ms (limit 5.77).
- `og-tests perf::perf_materials_physical` (perf lane, five transmissive
  objects) is the baseline for 09fd772d. Not run here: og-tests' GPU lanes are
  the coordinator's, and lavapipe times are not a baseline.
- `og-physics lanes::tests::a_pool_that_does_not_pay_…` is from perf-2 and
  walks the backoff. There is no old version of this code to fail it against.
- The grid's level test (perf-2) checks equality with the quadratic levels. It
  holds the behaviour, not the cost; the cost numbers are in the table above.

The orb's baselines are in `og-runtime-perf/intel-r-xeon-r-processor-2-60ghz/`
(evidence only, not under `crates/og-runtime/perf/`).

## What moves: nothing

- **World hashes, release, no renderer, 600 ticks, three rounds per build:**
  Hollowbell `48fdc8019bcc8625`, Kestrel `121254f596361402`, Lantern
  `b872d3f66a627512`, identical across builds. The orb's Lantern and Kestrel
  hashes differ from GPU.md's Metal ones (measured at e4b40fe2, an older
  revision). The base 753e8d9d build and the branch agree with each other.
- **Frames:** `og run` screenshots, base and branch, vulkan-cpu.
  - Four frames each of the three perf games' scripts (`scripts/shots.sh`):
    12 PNGs, byte-identical.
  - Lantern's frames playtest shots (title, start, lantern, slime with the
    translucent jelly in view, far, settings-zh; `scripts/frames-lv.sh`): 6
    PNGs, byte-identical.
  - Hash at tick 426: `41c99ea108b0e8f3` on both.
- **Dumps:** `dump_world` from each of those runs is identical apart from the
  systems' wall-clock fields (`scripts/dumpcmp.py`).
- **Tests run here, one crate at a time:**
  - og-ui: 121 passed, 2 skipped
  - og-runtime: 73 passed, 5 skipped (the ignored perf tests)
  - og-render: 95 passed
  - og-physics: 119 passed
- **Static checks:** `cargo fmt --check` is clean;
  `cargo clippy --all-targets -- -D warnings` is clean. `crates/og-api/api.ogt`
  is untouched, so there is no generated-file change.

## Decisions for the coordinator

1. **Record the M4 Pro baselines** for the three new perf tests, or the perf
   lane fails with "no baseline":
   - `crates/og-runtime/perf/apple-m4-pro/extract-sprites-20k.json`
   - `crates/og-runtime/perf/apple-m4-pro/extract-models-20k.json`
     (`OG_PERF_RECORD=1 cargo test -p og-runtime --release -- --ignored perf`)
   - `crates/og-tests/perf/metal-apple-m4-pro/materials-physical.json`
2. **Re-run perf-games release unpaced on Metal** to settle Lantern's render
   p95 < 8 ms. This orb cannot measure it.
3. **Functional lane:** the og-ui grid and text cache touch every UI frame, and
   og-render's transmission path touches `materials-physical` and the stylised
   physical scenes. They are expected unmoved (byte-identical here on
   vulkan-cpu).
4. Row 90 in `docs/phase-1.md` is marked closed on this branch pending items 1
   and 2. Revert that line if Metal disagrees.
5. 01123635 is dropped as superseded by 195cd595. perf-2 can be retired once
   this merges.
