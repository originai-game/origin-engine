# Round 4, size 3: every shipping variant under 80 % of 4 MiB, with margin

Branch `r4/size-3`, from `a347cca3` (origin/r4/candidate). Three code
commits, each measured on the 3D publish module; the four variants measured
at the base and at the head. Nothing here ran `scripts/check.sh`, `og test`,
web-reference, a reference capture or the determinism matrix (below). The
`wasm` profile's per-crate opt-levels are unchanged.

## How it was measured

As r4/size-2 (`../size-2/NOTES.md` §How it was measured): the shipping
`wasm` profile, `cargo build --target wasm32-unknown-unknown --profile wasm
-p og-host-web --lib --no-default-features --features ...` for 3D and 2D
publish, the same through `scripts/wasm-lanes.sh` for the lanes modules,
then `scripts/wasm-size-report.sh -n 60 --keep DIR` (wasm-bindgen 0.2.129,
binaryen 133, `brotli -q 11`; its first line is the size gate's number) and
`tests/evidence/debt2/size/twiggy-crates.ts` (twiggy 0.8.0) over
DIR/named.wasm. Orb: 16 cores, Linux x64, Rust 1.99.1.

Files: `<variant>-before.md` / `-after.md` (per-crate table and 60 largest
functions), `<variant>-table.md` (side by side, `../size-2/variant-table.py`),
`publish-cut{1,2,3}-table.md` (3D publish per commit), `generics.ts` (the
monomorphisation families), `btree.ts` (B-tree items grouped by key and
value type).

Budget 4,194,304 B; the target, 80 %, is 3,355,443 B.

## Result

| Variant | before raw after wasm-opt | before brotli | share | after raw after wasm-opt | after brotli | share | Δ brotli |
|---|---:|---:|---:|---:|---:|---:|---:|
| 3D publish | 11,799,681 | 3,330,189 | 79.4 % | 10,969,279 | **3,182,706** | **75.9 %** | −147,483 |
| 3D lanes | 12,055,848 | 3,370,010 | 80.3 % | 11,259,570 | **3,210,350** | **76.5 %** (the largest) | −159,660 |
| 2D publish | 10,892,604 | 3,086,957 | 73.6 % | 10,092,821 | **2,935,753** | **70.0 %** | −151,204 |
| 2D lanes | 11,154,983 | 3,124,590 | 74.5 % | 10,385,027 | **2,965,468** | **70.7 %** | −159,122 |

3D lanes is 145,093 B under 80 %. (This machine measured the base's 3D
lanes at 3,370,010 B, 729 B under the 3,370,739 the brief quotes.)

## The cuts, one commit each (3D publish, brotli -q 11)

| Commit | Cut | brotli | Δ |
|---|---|---:|---:|
| (base) | a347cca3 | 3,330,189 | |
| 0d3b552c | 1. `log` macros compiled out of the wasm module | 3,304,989 | −25,200 |
| 930982a0 | 2. a derived schema is static data read by one function | 3,279,149 | −25,840 |
| 09d579c8 | 3. per-entity maps share one B-tree (`og_core::IdMap`) | 3,182,706 | −96,443 |
| | **total** | | **−147,483** |

### What the table said first

Marginal brotli (the module's bodies and data compressed with and without
the family) at the base: `alloc::collections::btree` 202 kB (1,659 items,
730 kB shallow, 227 key/value pairs), drop glue 93 kB, `Debug` impls 57 kB,
sorts 55 kB (mostly rapier's and og-physics' `sort_unstable`, kept),
hashbrown 30 kB, serde_json 20 kB, derived `schema()` bodies 18 kB (86
types), bincode 15 kB. The B-tree was the largest family by far, and most
of it was per-entity state keyed by `u64`/`u32`/`usize`.

### 1. `log` off at compile time on wasm32 (0d3b552c)

- **Why.** Nothing in the repository installs a logger (`log::set_logger`
  appears nowhere), so on the web every `log::*!` of wgpu, naga and
  og-render already reads the run-time level Off and discards its record.
- **Change.** og-host-web's wasm32-only dependencies gain `log` with
  `max_level_off`: the same records dropped, their formatting (wgpu-core's
  and naga's `Debug` impls) not built in. Native builds are untouched
  (target-specific dependency, resolver 2).
- **Measured.** −25,200 B. naga −25.8 kB twiggy, wgpu −15.8 kB, wgpu-core
  −9.9 kB, data −28 kB; `Debug` impls 148,587 → 132,307 B
  (`publish-cut1-table.md`).

### 2. Derived schemas as static tables (930982a0)

- **Why.** `#[derive(Reflect)]` wrote each `schema()` as code: a
  `String::from` per name, doc, label and unit, `vec!`s of defaults and
  variants, one `FieldSchema` literal per field: 67 kB across 86 types.
- **Change.** The derive writes a `const FIELDS: &[og_core::reflect::FieldDecl]`
  (the same names, types, docs through `reflect_doc!`, labels, units,
  ranges, `tunable`, defaults, variants as `&'static`) and calls
  `Schema::declared` (`#[inline(never)]`, compiled once), which builds the
  same `Schema`.
- **Measured.** −25,840 B (og_host_web exports −64 kB twiggy; the tables
  move into data, +46 kB raw, +13 kB brotli alone).
- **Test.** og-core `tests/reflect.rs::derive_produces_the_schema_with_metadata`
  already compares a derived schema carrying every kind of metadata to a
  literal `Schema`; it and the publish-doc test pass unchanged.

### 3. `og_core::IdMap` (09d579c8)

- **Why.** `BTreeMap<K, V>` compiles the whole B-tree (search, insertion
  and splits, removal and merges/steals, iteration, drop) per key/value
  pair, about 3 kB a pair.
- **Change.** `IdMap<K, V>` keeps entries in a `Vec<(K, V)>` and orders
  them through one `BTreeMap<u64, u32>` from the key's bits to its slot,
  the same type for every map, so the tree is compiled once. Every
  observable is the `BTreeMap`'s: iteration (`iter`, `iter_mut`, `keys`,
  `values`, `range`, `retain`, `into_iter`, `Debug`, `Serialize`) in
  ascending key order, `insert` returns the old value, `collect`/`extend`
  and deserialisation keep a repeated key's last value, and it serialises
  as a map of its entries in key order (same bincode and JSON bytes). A
  removal swaps the last slot in, which no iteration observes. Keys are
  `u64`, `u32`, `usize` (`IdKey`: order of the bits is the key's order).
  Every `BTreeMap` with such a key in the crates the web host links is
  converted: og-physics, og-anim, og-ui, og-ai, og-nav, og-tween, og-audio,
  og-net, og-render, og-tilemap, og-cook (tilemap cook), og-runtime,
  og-host-web (which now depends on og-core directly). String-, tuple- and
  entity-keyed maps stay `BTreeMap`.
- **Measured.** −96,443 B. B-tree family 730,058 → 267,227 B shallow (227
  → 82 pairs; `u64`-keyed 244,146 → 14,919 B); drop glue 159,164 →
  142,062 B; og_physics −163 kB twiggy, og_runtime −142 kB, std −114 kB,
  og_host_web exports −98 kB, og_anim −57 kB, og_nav −45 kB, og_ui −40 kB;
  og_core +28 kB (`publish-cut3-table.md`).
- **Tests (fail if order or bytes change).** og-core
  `idmap::tests::behaves_as_the_btreemap_it_replaces`: 3,000 random
  inserts, removals, retains, entries, `get_mut`, `pop_first`/`pop_last`
  over dense, sparse and full-range keys, against a `BTreeMap`: every
  answer, both iteration directions, `iter_mut`, the order `retain` visits
  in, `Debug`, serde_json text and bincode bytes. `collects_and_reads_back_as_the_btreemap`:
  repeated keys in `collect` and in JSON, and a round trip into a
  `BTreeMap`.
- **Found on the way.** og-anim's flipbook state block boxed a `BTreeMap`
  in `decode` and downcast an `IdMap` in `apply`: a mismatch only the
  test (`a_snapshot_keeps_the_playback_and_the_transcript_is_the_same`)
  caught, fixed before the commit. Every other `downcast` in the engine
  crates was read: none crosses a converted type.

## What moves

Nothing expected: no world hash, frame or save byte. Cut 1 drops records
that were already dropped; cut 2 builds equal schemas; cut 3 keeps every
iteration order and serialised byte (tested). The physics, anim, AI, nav,
tween and audio suites, which include their determinism and
restore-mid-run tests, pass.

## Decided, and why

- **The B-tree before features.** As in size-2, the largest family was a
  shape of code, not a subsystem; nothing leaves a game.
- **`log` off only on wasm32 and only through og-host-web.** If the web
  host ever installs a logger (a console logger for wgpu errors), this
  feature must go with it, or it silently logs nothing.
- **IdMap's costs.** A lookup is the same tree search plus a vector index;
  an iteration reads slots in key order, one indirection more; a map holds
  12 bytes more per entry. No frame time was measured here.

## For the coordinator

1. **Frame times** of the published games on WebGPU and WebGL2 against the
   candidate (the E7 method): IdMap is on per-tick paths (physics'
   tracked bodies, anim states, AI memories, UI scroll state, extract's
   model maps).
2. **The determinism matrix** and the **Metal/GPU reference frames**
   (functional lane): none should move.
3. **og-render's integration tests** (gfx references) were not run here,
   only its lib tests; og-render's change is map types only.
4. **`wasm-gate` numbers** run 5–6 % above these; 3D lanes there should be
   about 3.38–3.40 MB, under the budget.

## Verification run here

- `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`
  clean; `cargo clippy -p og-core --features publish --all-targets -- -D
  warnings` clean; `cargo check -p og-host-web --target
  wasm32-unknown-unknown`; `cargo check -p og-net --features ws
  --all-targets`.
- nextest: og-core (45, default and `--features publish`), og-core-derive,
  og-physics (164 with og-core), og-anim, og-ui, og-ai, og-nav, og-tween,
  og-audio, og-net, og-tilemap (309), og-anim, og-runtime, og-host-web,
  og-cook (339), og-render lib.

## Not run

`scripts/check.sh` (any lane), `og test`, web-reference, reference
captures, the determinism matrix, browser frame times, og-render's GPU
integration tests. No reference frame was re-recorded.
