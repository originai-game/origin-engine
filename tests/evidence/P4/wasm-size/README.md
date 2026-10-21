# Where the lanes wasm's bytes are, and what trimming them paid (wave 8)

Branch `wasm-size`, Apple M4 Pro, 2026-10-21, other streams building on
the machine. Every number is the shipping `wasm` profile (Cargo.toml),
wasm-bindgen 0.2.129, `scripts/wasm-opt.sh` (binaryen 133, `--threads` for
lanes) and `brotli -q 11`, as `scripts/check.sh --all` measures it.
"Before" is main at 161bad3b, "after" is this branch merged with it; the
two differ only in og-lanes and og-render.

## The three downloads

| Download | Before | of 4 MiB | After | of 4 MiB | Δ |
|---|---:|---:|---:|---:|---:|
| lanes (`og_host_web-lanes`, both profiles, atomics) | 3,498,640 | 83.4 % | 3,464,929 | 82.6 % | -33,711 |
| 3D publish | 3,078,397 | 73.4 % | 3,048,477 | 72.7 % | -29,920 |
| 2D publish | 2,910,417 | 69.4 % | 2,882,459 | 68.7 % | -27,958 |

Lanes stays above 80 % (3,355,443 B): the trims below are what this
module had that removes no behaviour, and they are 34 kB of the 143 kB
needed. What gets it under is a lanes module per physics profile (Options).

## Method

```
scripts/wasm-lanes.sh --profile wasm -p og-host-web --lib --features publish
scripts/wasm-size-report.sh [-n 30] [--keep DIR] target/wasm-mt/wasm32-unknown-unknown/wasm/og_host_web.wasm
```

`scripts/wasm-size-report.sh` binds the module with wasm-bindgen
(`origin-engine-lanes` and `--threads` when it has atomics), optimises it
twice side by side, with the shipping flags (the brotli it prints first is
the size gate's) and with `-g` (names kept, the code within 0.1 %), and
hands the named one to `scripts/wasm-size-report.ts` (bun): the per-crate
table and the largest functions, each with its brotli alone. Any module
works: `target/og-wasm/raw/*.wasm` after a gate's wasm-build, the publish
variants built with `--no-default-features --features
og-host-web/publish,og-host-web/physics3d,og-host-web/shaping` (or
`physics2d`). Two minutes for the lanes module.

A function is charged to the first crate its demangled name mentions
other than core, alloc, std and hashbrown, so a std generic instantiated
for a type (drop glue, a BTreeMap, a sort) is its type's crate, while a
rayon job run for some crate's closure is rayon's (rayon is named first). `og_host_web (exports)` is the binding table's exports, into which
wasm-opt inlined their bodies (mostly og-runtime's host functions);
`[unnamed]` is wasm-opt's merged functions.

"Brotli alone" ranks by what a player downloads. Raw bytes do not: the
`wasm-gate` profile's lanes module carried 482 kB of rayon_core
instantiations that compressed to 35 kB, and the og-lanes change below
removed 230 kB raw (wasm-gate) for 12 kB of brotli. The `wasm-gate`
profile is also a poor proxy for where bytes are: with 16 codegen units
its lanes module is 6.6 % larger and its data section held the IDL's
record table three times (one copy per codegen unit that used the `const`);
the shipping profile has one, so turning `og_api::host::RECORDS` into a
`static` gains nothing there and was left alone.

## Per crate, lanes, after

Code bodies and the data section, 13,056,640 B raw after wasm-opt; the
groups' brotli alone sums to 3,729,020 B (the module, 3,464,929 B,
compresses them against each other).

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 509 | 1,240,783 | 9.50 % | 362,472 | 9.72 % |
| rapier3d | 382 | 989,690 | 7.58 % | 327,007 | 8.77 % |
| [data] | 1 | 1,456,798 | 11.16 % | 324,237 | 8.69 % |
| rapier2d | 295 | 722,489 | 5.53 % | 245,716 | 6.59 % |
| og_physics | 317 | 900,214 | 6.89 % | 231,617 | 6.21 % |
| og_render | 395 | 747,633 | 5.73 % | 231,609 | 6.21 % |
| parry3d | 949 | 753,280 | 5.77 % | 202,725 | 5.44 % |
| og_host_web (exports) | 288 | 678,976 | 5.20 % | 198,727 | 5.33 % |
| naga | 1,322 | 646,794 | 4.95 % | 192,220 | 5.15 % |
| wgpu | 318 | 404,072 | 3.09 % | 140,293 | 3.76 % |
| parry2d | 708 | 453,485 | 3.47 % | 128,231 | 3.44 % |
| og_ui | 223 | 331,441 | 2.54 % | 108,903 | 2.92 % |
| wgpu_core | 751 | 362,414 | 2.78 % | 95,009 | 2.55 % |
| std | 1,008 | 344,841 | 2.64 % | 82,608 | 2.22 % |
| wgpu_hal | 309 | 187,812 | 1.44 % | 65,124 | 1.75 % |
| og_ecs | 355 | 203,132 | 1.56 % | 54,410 | 1.46 % |
| og_nav | 82 | 150,441 | 1.15 % | 50,178 | 1.35 % |
| og_anim | 152 | 147,206 | 1.13 % | 46,639 | 1.25 % |
| ttf_parser | 294 | 129,836 | 0.99 % | 42,855 | 1.15 % |
| [unnamed] | 405 | 152,340 | 1.17 % | 42,394 | 1.14 % |
| rayon | 68 | 170,860 | 1.31 % | 41,991 | 1.13 % |
| rustybuzz | 296 | 109,083 | 0.84 % | 40,509 | 1.09 % |
| serde_json | 185 | 188,650 | 1.44 % | 36,028 | 0.97 % |
| og_probe | 59 | 102,194 | 0.78 % | 35,502 | 0.95 % |
| og_cook | 223 | 108,055 | 0.83 % | 34,483 | 0.92 % |
| og_audio | 82 | 86,625 | 0.66 % | 31,387 | 0.84 % |
| og_core | 309 | 132,871 | 1.02 % | 28,436 | 0.76 % |
| og_ai | 64 | 83,535 | 0.64 % | 28,122 | 0.75 % |
| js_sys | 68 | 76,182 | 0.58 % | 25,681 | 0.69 % |
| bincode | 126 | 102,371 | 0.78 % | 25,178 | 0.68 % |
| rayon_core | 474 | 243,223 | 1.86 % | 24,211 | 0.65 % |
| og_host_web | 65 | 75,451 | 0.58 % | 21,907 | 0.59 % |
| og_lanes | 104 | 70,437 | 0.54 % | 19,492 | 0.52 % |

(The script prints every crate; the rest are below 17 kB alone.) The two
physics profiles together are rapier2d, rapier3d, parry2d, parry3d and
og_physics: 935 kB alone, the largest part of the module by far; the 2D
half (rapier2d, parry2d and og_physics' dim2) is what a 3D game's lanes
page downloads for nothing.

The data section, 1,456,798 B raw (324 kB alone): WGSL 257 kB raw (45.5 kB
brotli, after the comment trim; 72 kB before), panic locations 81 kB raw
(5.5 kB: absolute registry paths, which compress to almost nothing), other
text 439 kB raw (112 kB: the IDL's record table with its field docs,
the same docs again in og-physics' Rust-declared schemas, wgpu's and
naga's error messages, naga's keyword tables), and 687 kB of binary tables (about 160 kB: the
record and schema tables' pointers and defaults, the unicode and shaping
tables, constants).

## Top 30 functions, lanes, after

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 398,979 | 138,041 | og_render | `<og_render::Renderer>::render` |
| 199,134 | 53,777 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::decode` |
| 182,909 | 57,977 | og_runtime | `og_runtime::extract::extract` |
| 182,539 | 49,824 | og_runtime | `og_runtime::engine::assemble` |
| 171,851 | 62,023 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 157,480 | 57,493 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 124,131 | 44,741 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 101,553 | 35,821 | og_physics | `og_physics::dim3::sim::step_here` |
| 99,621 | 33,532 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 85,110 | 28,735 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 76,751 | 26,911 | og_physics | `og_physics::dim2::sim::step_here` |
| 70,914 | 10,734 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 68,804 | 22,884 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 61,620 | 20,055 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 55,572 | 21,967 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 51,175 | 17,604 | og_runtime | `og_runtime::engine::answer_call` |
| 49,248 | 18,509 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 46,450 | 8,958 | og_physics | `og_physics::gather::gather` |
| 39,382 | 15,031 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 38,727 | 14,829 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 36,132 | 12,983 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 32,588 | 12,458 | og_probe | `<og_probe::probe::Probe>::execute` |
| 31,430 | 12,152 | naga | `<naga::valid::Validator>::validate_impl` |
| 30,666 | 11,038 | og_host_web (exports) | `og_load_animation multivalue shim` |
| 30,125 | 7,070 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 29,601 | 10,338 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 29,498 | 7,165 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 29,021 | 12,311 | rapier3d | `<rapier3d::geometry::narrow_phase::NarrowPhase>::compute_contacts::{closure#3}` |
| 28,692 | 11,036 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 28,097 | 8,991 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |

## What was trimmed (one commit each; lanes, `wasm` profile)

| Commit | Change | Lanes brotli | Δ |
|---|---|---:|---:|
| 6d3f45a4 | og-lanes: `map`, `for_each_mut` and `for_each_indexed` run through one non-generic `par::ranges(len, grain, &dyn Fn(start, end))` instead of rayon's par_iter per item and result type (about 200 instantiations of `og_lanes::par::map` before; og_lanes 30.4 kB alone to 19.5 kB, rayon_core 93 kB and rayon 49 kB raw less) | 3,498,258 → 3,489,535 | -8,723 |
| c51e8d05 | og-render: build.rs copies `shaders/` with its `//` comments removed and the `wgsl!` macro includes the copy (every token and line kept; the shaders untouched); `terrain_lit_array_source` anchors on `fn band(` instead of the comment above it | 3,489,535 → 3,466,050 | -23,485 |

(Measured at 5e50479f, the branch's base; the table above is the same two
commits on main at 161bad3b.) The single-threaded modules gain the WGSL
trim too (the og-lanes helpers run serially there): 3D publish -29.9 kB,
2D publish -28.0 kB. og-lanes' results are item for item what they were
(og-lanes' tests, og-physics' lanes tests, which compare worlds split and
serial, og-anim's and og-runtime's extraction tests pass natively);
og-render's shader tests compile what ships on both tiers and check the
built copies line for line against the sources.

## What did not pay

| Tried | Lanes brotli | Δ | Why not |
|---|---:|---:|---|
| `[profile.wasm.package.og-host-web] opt-level = "z"` (the exports' glue) | 3,471,914 | +5,864 | larger: z stops inlining into the exports, and the bodies they called stay at 3 |
| `og_api::host::RECORDS` and `SYMBOLS` as `static` instead of `const` | | 0 | the three copies of the record table were `wasm-gate`'s 16 codegen units; `wasm` has one |
| per-type rayon instantiations, judged by raw size | | | 230 kB raw were 9 kB brotli (the og-lanes commit kept, for what it pays) |
| debug-only code in the probe behind `publish` | | | `Probe::execute` and `pump` (52 and 69 kB raw retained) run og.ai's game commands in a published game too (e6.rs: the probe's inbox), so they are not debug-only |
| cook-only code reached through og-cook | | | none: og-cook's 34 kB alone in the lanes module are the runtime loaders (`Bundle`, KTX2, meshopt, the animation codec) |
| `Debug`/`format!` in shipped og crates | | | 41 `{:?}` sites in the og crates the web host links (og-cook's 22 more are mostly the cook's), 27 of them og-render's labels and messages; wgpu's and naga's Debug and Display impls are their error messages |

`panic = "abort"` is the wasm32 target's already, and the wasm profiles
carry no debuginfo; wasm-opt drops the name section. Float formatting
(flt2dec, about 24 kB raw) is reached by JSON output and messages that print
numbers.

## Options not taken

| Option | Lanes brotli | of 4 MiB | What it costs |
|---|---:|---:|---|
| A lanes module per physics profile, as the publish variants are (`--no-default-features --features publish,physics3d,shaping` with atomics; 2D likewise), each in its profile's `dist/web-publish*` (taken in wave 10, owed row 71: below) | 3D 3,088,792, 2D 2,919,700 | 73.6 %, 69.6 % | no behaviour or feature: a page already downloads its profile's publish module. It changes what ships: scripts/wasm-crates.txt (a lanes variant per profile), check.sh's wasm-build, wasm-opt and opt.txt naming, size-gate's downloads, wasm-lanes.sh's target dir per variant (one fingerprint per unit), packages/engine build.sh, pack.sh, web-build.ts (lanes files from the profile's directory), doctor.ts, the determinism and browser tests that run 2D worlds on the lanes module, docs/web-export.md §Variants and §Lanes; one more lanes build per full gate. Measured at 5e50479f with this branch's two trims |
| `-Zlocation-detail=none` in scripts/wasm-lanes.sh (it already builds std with RUSTC_BOOTSTRAP) | 3,397,518 | 81.0 % | a lane's panic report and every crash in a lanes page lose their file, line and column (PanicReport.location `<redacted>:0:0`); -68.5 kB, mostly from panic sites wasm-opt can then merge. `-Zlocation-detail=file,line` (no column) is -12.3 kB |
| field docs out of the publish build's record table | not measured | | changes what the reflection serves in a published game (the docs reach og-core's schemas, the probe's `help` and Tune) |
| og-runtime at opt-level z | -71.7 kB (E7) | | its per-tick systems and the frame's dispatch: frame time (tests/evidence/E7/wasm-size.md) |
| `-Z build-std-features=panic_immediate_abort` | | | no panic hook: a lane's panic would no longer wake the engine thread with its message (og-lanes' crash design) |
| `-Z build-std-features=optimize_for_size` | | | std's unstable sorts become another algorithm, so equal keys can land in another order (rapier sorts with `sort_unstable_by_key`): a hash could change |
| GLSL precompiled for the GL tier | | | naga's WGSL front end and GLSL writer (192 kB alone) would leave the module, but every engine and game material shader would ship translated per tier: a cook and loader change, not a trim |

## Wave 10: a lanes module per physics profile (owed row 71)

Branch `lanes-per-profile`, Apple M4 Pro, 2026-10-21, at main 9d220790
(after the ci-green merge), other streams building beside it (load average
8 to 20). The option above, taken: `scripts/wasm-crates.txt` now has a
`lanes` line per profile, `lanes2d` (`publish,physics2d,shaping`) and
`lanes` (`publish,physics3d,shaping`), both through `scripts/wasm-lanes.sh`
into `target/wasm-mt`, and `og build --publish` takes the lanes files from
its profile's directory (`dist/web-publish` or `dist/web-publish-2d`) as it
takes the single-threaded module. Measured with
`OG_WASM_PROFILE=wasm scripts/check.sh --only size-gate` before and after
(the shipping profile, brotli -q 11, the size gate's own numbers), and with
`OG_WASM_PROFILE=wasm-gate` for the path-aware lane's:

| Download | Before (`wasm`) | of 4 MiB | After (`wasm`) | of 4 MiB | Δ | After (`wasm-gate`) |
|---|---:|---:|---:|---:|---:|---:|
| lanes, both profiles (gone) | 3,472,308 | 82.8 % | | | | |
| 3D lanes (`og_host_web-lanes`) | | | 3,095,052 | 73.8 % | -377,256 against the one lanes module | 3,278,998 |
| 2D lanes (`og_host_web-lanes2d`) | | | 2,928,498 | 69.8 % | -543,810 against the one lanes module | 3,106,406 |
| 3D publish | 3,055,713 | 72.9 % | 3,055,713 | 72.9 % | 0 | 3,218,005 |
| 2D publish | 2,894,396 | 69.0 % | 2,894,396 | 69.0 % | 0 | 3,045,414 |
| the largest download | 3,472,308 (lanes) | 82.8 % | 3,095,052 (3D lanes) | 73.8 % | -377,256 | 3,278,998 |

Raw after wasm-opt: lanes 13,186,979 B before; 3D lanes 11,401,997 B,
2D lanes 10,800,500 B after. Each lanes module is now its profile's
publish module plus 34 to 39 kB of brotli (rayon, wasm-bindgen-rayon, the
relay, rapier's `parallel` paths and the thread setup), where the one
lanes module was the 3D publish module plus 417 kB, most of it rapier2d,
parry2d and og-physics' dim2 (tables above). The single-threaded modules
are byte for byte what they were (no source changed).

What the extra lanes build costs (docs/ci.md §The lanes wasm): cold, the
both-profile lanes build took 3 min 20 s in `wasm`; after it the 2D one
took 1 min 56 s and the 3D one 1 min 46 s (std and the dependencies
shared in `target/wasm-mt`). In `wasm-gate`, cold, 63 s then 39 s; after
an og-physics edit 14 s each, after an og-runtime edit 8.5 s each; in
`wasm` 64 and 83 s after an og-physics edit. With nothing changed each
relinks og-host-web (6 s), as the publish variants do in their shared
directory. wasm-opt and brotli of the extra module run beside the others.

Determinism: `tools/wasm-determinism/run.sh` builds the tool twice more in
the lanes configuration (`scripts/wasm-lanes.sh`, og-physics with one
profile each), binds each with wasm-bindgen and runs it under node on one
lane; every build agrees, the 2D world hash on the 2D lanes build and the
3D ones on the 3D (lanes-per-profile-determinism.md; 23 s per lanes build
after an og-physics change). `packages/engine/tests/hosts.test.ts` loads
each profile's lanes module under bun: its profile's physics scene
(64 bodies on the engine thread, and 1,024 on a wasm-bindgen-rayon pool of
four Workers, where all 600 steps went to the pool) ends in the native
host's snapshot, which restored natively has the native world hash
(2D 967187ae3438d30b at 64 bodies), and a world of the other profile is
refused ("built without the physics3d feature" on the 2D module).
