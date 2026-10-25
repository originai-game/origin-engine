# Round 4, size 2: every shipping variant under 80 % of 4 MiB (the engine's side of row 99)

Branch `r4/size-2`, from `1bb341e5` (origin/main, 0.4.0). One commit per
cut, each measured on the module that ships. Nothing here ran
`scripts/check.sh`, `og test`, web-reference, a reference capture or the
determinism matrix: those are the coordinator's (below).

## How it was measured

- The four shipping variants, built as `packages/engine` ships them: the
  shipping `wasm` profile, `cargo build --target wasm32-unknown-unknown
  --profile wasm -p og-host-web --lib --no-default-features --features
  og-host-web/publish,og-host-web/physics3d,og-host-web/shaping` (3D
  publish), `...physics2d,profile2d...` (2D publish), and the same two
  through `scripts/wasm-lanes.sh` (3D and 2D lanes).
- Each module through `scripts/wasm-size-report.sh -n 60 --keep DIR`
  (wasm-bindgen 0.2.129, binaryen 133 with `scripts/wasm-opt.sh`'s flags,
  `brotli -q 11`): its first line is the size gate's number. The per-crate
  twiggy table is `tests/evidence/debt2/size/twiggy-crates.ts` (twiggy
  0.8.0) over DIR/named.wasm, as the debt2 stream did.
- `<variant>-before.md` and `<variant>-after.md` here are each variant's
  per-crate table (raw, brotli alone, twiggy shallow bytes) and its 60
  largest functions; `<variant>-table.md` puts before and after side by
  side (`variant-table.py`). `generics.ts` sums the monomorphisation
  families (sorts, drop glue, serde, fmt), `sorts.ts` the sorts per crate.
- Budget: 4,194,304 B; the target is 80 % of it, 3,355,443 B.
- The orb: 16 cores, Linux x64, Rust 1.99.1. The 0.4.0 numbers below are
  this machine's; the round 3 closeout's 3,787,011 B was the gate's
  `wasm-gate` profile (codegen units 16, 5 to 6 % above the shipping one,
  docs/ci.md §Lanes).

## Result

| Variant | before (1bb341e5) raw after wasm-opt | before brotli | share | after raw after wasm-opt | after brotli | share |
|---|---:|---:|---:|---:|---:|---:|
| 3D publish | 13,157,392 | 3,561,039 | 84.9 % | 11,604,499 | **3,283,141** | **78.3 %** |
| 3D lanes | 13,489,824 | 3,591,631 | 85.6 % | 11,850,400 | **3,320,422** | **79.2 %** (the largest) |
| 2D publish | 12,159,793 | 3,291,397 | 78.5 % | 10,700,674 | **3,034,243** | **72.3 %** |
| 2D lanes | 12,484,766 | 3,327,994 | 79.3 % | 10,952,306 | **3,072,212** | **73.2 %** |

No feature left any game, template, archetype or example: every cut
below keeps the same behaviour (the sorts give the same order, the maps
the same contents, the save the same bytes) or removes text nothing in a
publish build reads (field docs), or is a compiler setting.

## The cuts, one at a time

3D publish was measured after every commit (before clippy's fixups were
folded in: a test's sort call, a test assertion and three redundant
`.into_iter()` calls, none of which changes the module's code); the other three at the base,
after the four code commits together, and after the profile.

| Commit | Cut | 3D publish brotli | Δ |
|---|---|---:|---:|
| (base) | 1bb341e5 | 3,561,039 | |
| 47786ede | 1. one stable sort (`og_core::sort`) | 3,472,503 | −88,536 |
| 663a1b0d | 2. reflected and record field docs out of publish | 3,412,596 | −59,907 |
| cf651a49 | 3. one bincode encoder for the physics save | 3,402,371 | −10,225 |
| 201617c9 | 4. BTreeMap/BTreeSet built by insertion | 3,375,547 | −26,824 |
| 1b226223 | 5. the `wasm` profile at opt-level s, per-frame crates at 3 | 3,283,141 | −92,406 |
| | **total** | | **−277,898** |

| Variant | base | after cuts 1–4 | Δ | after cut 5 | Δ |
|---|---:|---:|---:|---:|---:|
| 3D publish | 3,561,039 | 3,375,547 | −185,492 | 3,283,141 | −92,406 |
| 3D lanes | 3,591,631 | 3,410,773 | −180,858 | 3,320,422 | −90,351 |
| 2D publish | 3,291,397 | 3,126,084 | −165,313 | 3,034,243 | −91,841 |
| 2D lanes | 3,327,994 | 3,163,090 | −164,904 | 3,072,212 | −90,878 |

### What the table said first

The debt2 stream had already taken the publish-only members (GPU timing,
the probe's tool members, the 2D host's 3D world). So the first thing to
look at was not a feature but the shape of the code. Grouping the
3D publish module's twiggy items by family (`generics.ts`, base) showed:

| Family | items | shallow B |
|---|---:|---:|
| `core::slice::sort::stable` (driftsort) | 280 | 654,669 |
| `core::slice::sort::unstable` | 38 | 154,745 |
| `core::slice::sort::shared` (small sorts, pivots) | 290 | 123,810 |
| drop glue | 801 | 274,623 |
| serde `Deserialize` | 124 | 277,729 |
| `Debug` impls | 1,355 | 139,566 |
| `Display` impls | 193 | 73,578 |
| `Clone` impls | 128 | 95,382 |

The sorts were 933 kB, 7 % of the module: one driftsort per element
type *and* per comparator closure, 2 to 18 kB each. og-render alone had
72 of them (212 kB: the sprite, overlay, ribbon and light orders), then
og-runtime, og-timeline (50 kB: one per key type), og-ecs, serde_json's
map (`BTreeMap::from_iter` sorts first), og-geom.

The data section (1.52 MB, 373 kB brotli alone) had the panic and
assertion messages (`.rodata.1`, 199 kB), the WGSL (stripped of comments
already), wgpu's and naga's error strings, and the IDL's and the derives'
field docs (about 141 kB of text in total, below).

### 1. One stable sort for the engine's slices (47786ede)

- **Change.** `og_core::sort::{sort, sort_by, sort_by_key}`: a bottom-up
  merge sort of `u32` positions (insertion runs of 16; a run pair already
  in order is copied after one comparison), not generic, compiled once;
  each call site adds only its comparator and the permutation's
  application (cycle by cycle, swaps). Stable, so the result is the
  standard library's for every comparator that is a total order: frames
  and hashes cannot move. Every stable sort (about 65 call sites) of the crates the web host
  links (og-render, og-runtime, og-ecs, og-timeline, og-geom, og-ui,
  og-anim, og-physics, og-audio, og-tilemap, og-i18n, og-probe, og-cook's
  runtime) call it; og-geom gains og-core as a dependency.
- **Kept.** `sort_unstable*` (physics' per-step integer sorts, nav tiles,
  audio): what the standard unstable sort does with ties is its own, and
  those are hot integer sorts.
- **Measured.** 3D publish −88,536 B brotli (13,157,392 → 12,659,761 B
  after wasm-opt). Stable-sort items 654,669 → 224,184 B, shared
  123,810 → 48,294 B.
- **Time** (native, scratch bench, 64-byte elements, a three-key
  comparator): 0.71× driftsort's time at 100 random, 1.66× at 1,000,
  1.30× at 10,000; presorted 1.79× at 100 and 0.44× at 10,000. Microseconds
  in all of them; no engine sort orders more than a frame's draw list.
- **Test.** og-core `sort::tests::sorts_as_the_standard_stable_sort_does`:
  every length 0 to 79 and across the merge widths (127 to 4,099), random
  with 2, 7 and 10⁶ keys, ascending and descending runs, element for
  element against `slice::sort_by_key`, each element carrying its
  original index, so a tie out of order fails. Every edited crate's tests
  pass (og-core, og-ecs, og-geom, og-timeline, og-anim, og-ui, og-probe,
  og-tilemap, og-audio, og-i18n, og-physics, og-render's lib, og-runtime,
  og-cook's).

### 2. Reflected and record field docs out of publish (663a1b0d)

- **Why.** og-api's `RECORDS` carried 1,140 field docs (84 kB of text),
  `#[derive(Reflect)]` copied every doc comment into its `Schema` (about
  57 kB across og-physics, og-audio, og-anim, og-render, og-input...), and
  the world registers both. In publish nothing reads a doc: the probe's
  schema members, the world dump, the inspector and Tune are not built in;
  `Schema::layout_hash` and every schema comparison use names, types and
  variants only.
- **Change.** og-core and og-api gain `publish` (og-runtime's `publish`
  turns both on). The derive writes `og_core::reflect_doc!(#doc)` and
  og-idl's generator writes `doc!("...")`; each is the literal or `""`.
  `og api gen` regenerated `crates/og-api/src/generated/host.rs` (the
  `doc!` wrapping and the field's doc comment only). Labels, units,
  ranges, defaults and variants stay.
- **Measured.** 3D publish −59,907 B brotli (12,659,761 → 12,452,229).
- **Test.** og-core `tests/reflect.rs::a_publish_build_leaves_the_docs_out`
  (`--features publish`): no doc in a derived schema, label and unit kept,
  the layout hash unchanged; the default build's metadata test is gated
  to the default build.

### 3. One bincode encoder for the physics save (cf651a49)

- **Why.** `bincode::serialize` is a size pass and then a writer of
  another options type, the colliders' offset took a third
  (`serialized_size`) and the restore wrote rebuilt shapes with a fourth:
  the whole rapier world's `Serialize` four times over.
- **Change.** `og_physics::encode` (the block's codec, `serialize_into` a
  `&mut Vec<u8>`) is the only encoder; the offset is measured by writing
  the fields before the colliders again as a tuple of references, which
  bincode writes field after field as it does the struct. Same bytes.
- **Measured.** 3D publish −10,225 B brotli.
- **Test.** og-physics `tests/saves.rs` (mesh colliders carried by their
  asset and restored to the same world) and the crate's other suites,
  unchanged.

### 4. BTreeMap and BTreeSet by insertion (201617c9)

- **Why.** After cut 1, 100 sort items (137 kB) remained from
  `BTreeMap`/`BTreeSet`'s `FromIterator`, which sorts the pairs first.
- **Change.** `og_core::sort::{map, set}` (extend an empty map: the same
  map, a later equal key's value wins as with `collect`); 41 engine
  collect sites call them (the commit message's "27" undercounts); two `serde_json::Map`s built from an iterator
  extend an empty one.
- **Measured.** 3D publish −26,824 B brotli.
- **Test.** og-core `sort::tests::maps_as_collect_does`.

### 5. The `wasm` profile at opt-level s, the per-frame crates at 3 (1b226223)

- **Change.** `[profile.wasm]` `opt-level = "s"`, with the crates whose
  loops run per tick or per frame named at 3: rapier2d/3d, parry2d/3d,
  og-physics, glam, nalgebra, simba, og-ecs, og-render, og-anim, og-lanes,
  og-ui, wgpu (the WebGPU front end), og-nav, og-ai. The load-time crates
  stay at z. To s: og-runtime, wgpu-core and wgpu-hal (wgpu's WebGL2
  path), serde_json, og-cook's runtime, og-core, og-host-web and the rest.
  `wasm-gate` inherits it. Native builds never use the profile; the
  determinism tool has its own release profile.
- **Measured.** 3D publish −92,406 B, 3D lanes −90,351 B, 2D publish
  −91,841 B, 2D lanes −90,878 B (table above).
- **Alternatives measured** (3D publish, the code of cut 4):

| Configuration | brotli | share |
|---|---:|---:|
| the old profile (3, load-time crates z) | 3,375,547 | 80.5 % |
| s, with every crate E7 named per-frame at 3 (adds og-runtime, wgpu-core, wgpu-hal, wgpu-types) | 3,396,327 | 81.0 % |
| s, all of those but og-runtime at 3 | 3,319,736 | 79.1 % (3D lanes would be ~80 %) |
| **s, og-runtime, wgpu-core, wgpu-hal at s (chosen)** | **3,283,141** | **78.3 %** |
| s, also og-render, og-ui, og-anim, og-lanes, wgpu, og-nav, og-ai at s | 3,255,847 | 77.6 % |
| s everywhere (z crates kept) | 3,107,455 | 74.1 % |
| `-C lto=fat` (old profile) | 3,559,292 | −1,747 |
| wasm-opt `-Oz --converge` / `--gufa -Oz --converge` | 3,544,382 / 3,543,294 | −17 kB, 3× the wasm-opt time |

  The first line of the s rows is larger than the old profile: s with the
  E7 set at 3 inlines less across the s/3 boundary. The saving is
  og-runtime (77 kB) and the WebGL2 path (part of 64 kB), so those are
  the two judgment calls.

## Decided, and why

- **Code shape before features.** The table's largest family was the
  sort, not a subsystem; the cuts 1, 3 and 4 remove copies, not
  behaviour, and need no capability or refusal.
- **Docs are not a feature in publish.** No member a publish build
  serves reads them; Tune and the inspector need the dev host.
- **og-runtime at s.** It is the only per-frame crate moved to s. Its
  per-frame work (extract, the bindings) is copying rows into og-render's
  scene; og-render and og-ecs, which do the per-entity loops, stay at 3.
  It is the cut most worth a frame-time check (below).
- **wgpu-core and wgpu-hal at s.** Used only by the WebGL2 tier (the
  WebGPU tier goes through `wgpu`'s front end, at 3), where a frame's cost
  is the GL calls across the JS boundary.

## Left out, and why

- **Panic and assertion strings** (`.rodata.1`, 199 kB) and `fmt`'s
  `Debug`/`Display` impls (213 kB): `panic_immediate_abort` needs nightly
  `-Z build-std` for the single-threaded modules too (the lanes modules
  already rebuild std with `RUSTC_BOOTSTRAP`), and the panic hook's report
  (`og_web_panic`: kind, message, location) is a crash report the engine
  Worker reads. Not a cut of this stream; it would be the next lever (not
  measured).
- **WGSL as text.** The comments are already stripped (build.rs); the
  remaining 270 kB is ~48 kB brotli alone and compresses well inside the
  module. Compressing at build and inflating at module creation would trade
  a few kB for a decoder at startup; not worth it at this margin.
- **serde_json in publish.** It is the bindings' wire format (every
  `og_*` export that takes or returns JSON), 83 kB raw after the cuts;
  replacing it is an API change, not a size cut.
- **og-geom's predicates and CSG** (`robust`, `spade`, 52 kB raw): reached
  at run time by `og.geometry.csg` and the shape generators in a shipped
  game, so they stay.
- **Drop glue** (157 kB after): spread over 900 types, no one cause.

## For the coordinator to re-check

1. **Chrome frame times and first frames** of the published games at
   this head against main, on WebGPU and WebGL2 (the E7 method,
   `tests/evidence/E7/wasm-size.md`): og-runtime, wgpu-core and wgpu-hal
   are at s. If a frame moves, put og-runtime back to 3 first (3D publish
   +~77 kB, 3D lanes then near 80 %).
2. **The determinism matrix** (`tools/wasm-determinism/run.sh`) and the
   **Metal reference frames**: none should move (stable sorts with the
   same order, maps with the same contents, the same save bytes, no
   fast-math at any opt-level).
3. **`og api check`** (drift): host.rs was regenerated by og-idl's
   `generate` and matches; the TypeScript, schema and wiki outputs did not
   change.
4. **The gate's numbers.** `wasm-gate` runs 5 to 6 % above these
   (3D lanes about 3.5 MB, 84 %), under the budget; the 80 % is the
   shipping profile's, as the row asks.
5. **Clippy** was run here, clean: `--all-targets -D warnings` on every
   edited crate, og-host-web and og-host-node, and on og-core and og-api
   with `--features publish`.
