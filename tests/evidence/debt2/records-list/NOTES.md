# Debt round 2 `records-list`: evidence

Branch `debt2/records-list`, cut from main a3bf3dcc. It pays off item 4 of
"Decision: no list field in records now" in
`tests/evidence/debt/render-targets/NOTES.md`. Records now have a list
column, and `PostEffects.custom` is one list of `Post` handles.
`custom1..custom8` are deleted, with no compatibility layer.

As AGENTS.md requires, this stream did not run the gate, `og test`,
web-reference, or a wasm or release build. These are the checks it ran:
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt`;
- the tests of every edited crate: og-core, og-ecs, og-net, og-probe,
  og-cook, og-idl, og-runtime `--lib`, and og-tests `--test targets` (12
  pass);
- `og api gen`, `og api gen --wiki` and `og api check`;
- the engine typecheck, plus the bun tests `api-surface`, `runtime`,
  `look`, `strings`, `resource-writes` and `lists` on the native host;
- the knowledge package's tests (42 pass);
- `og run --commands` of the fx-showcase shaders station.

## What landed, one commit per item

### 1. reflect: `FieldType::List(Scalar, N)` (2ba124ae)

- A field can now be a fixed-capacity list of one number type. The schema
  JSON names it `S[N]`, for example `u64[8]`, the way `str(N)` names its
  length. N is between 1 and `LIST_MAX_CAPACITY` (1024).
- The field is laid out as N + 1 lanes of the element type. Lane 0 is the
  count, so `[]` and `[0]` are different values. Lanes past the length are
  zero, which keeps a packed list canonical, so hashes and equality hold.
- Its size, alignment and layout hash follow from the element type and the
  capacity.
- There is a `List<T, N>` Rust type, plus `pack_list`, `list_len` and
  `list_elements`.
- `check_packed` refuses a count over the capacity and stale lanes.
  `validate` refuses a default, a range or `tunable` on a list.
- The module rule ("A field holds one value: there is no array or list
  column") is rewritten in reflect.rs, the og-core README, the api.ogt
  reflect text and the og-core skill.

### 2. og-ecs: snapshot, restore and migration (9c8894b0)

A list round-trips through snapshot and restore. Restoring by field into
another capacity keeps a list that fits, and empties a longer one with a
warning. This is the same rule as a `str(N)` that no longer fits. Test:
`field_types.rs` `a_list_column_round_trips_and_migrates_to_another_capacity`.

### 3. og-net, og-probe: lists as JSON arrays (23162d9a)

- og-net state replication sends a list as a JSON array, with a unit test.
- og-probe `get:` and dumps answer a JSON array. `set:` takes
  `[3,5]`, `3,5`, or nothing for `[]`, and refuses a list longer than the
  capacity.
- `patch` refuses a list, which is never tunable.
- Test: `crates/og-probe/tests/lists.rs`.

### 4. og-cook: scene text (18077b2c)

A list field in scene text is one line: `custom 3 5`. The key alone is
`[]`. `scene_text` writes the key alone for an empty list, and the text
round-trips. Test: `crates/og-cook/tests/scene_list.rs`.

### 5. og-idl: `List.X.N`, for record fields only (3974510c)

- `List.X.N` is a field type of a record (a `field` of a `component` or
  `resource`) with a number or handle element. It is refused anywhere else
  (`list-*.ogt` fixtures).
- `column list { list true }` declares the TS factory.
- `FieldDecl` gains `list`, with an element kind and capacity, for the
  host. Bindings gain a `list` entry.
- The d.ts gains `ListSpec`/`ListKind` and the `FieldValues` mapping to
  `readonly T[]`, with a `maxItems` in the schema JSON.
- The wiki notes the capacity.
- Tests: five broken fixtures (`list-*.ogt`), a `Chain` resource in
  `valid.ogt`, and `idl.rs`.

### 6. api.ogt and the runtime readers: `PostEffects.custom List.Post.8` (e20a55fa)

- In og-runtime:
  - `codec.rs` maps a list to `Kind::List` and encodes and decodes JSON
    arrays.
  - `engine.rs` `resource_layout` gives `kind: "list"`, `element` and
    `capacity`.
  - `assets.rs` and `scene_save.rs` read and write list rows.
  - `look.rs` resolves a handle per element.
  - `views.rs` fills og-render's `custom_post` in list order, up to
    `POST_SLOTS` (= `MAX_CUSTOM_POSTS`, 8).
- og-cook's look cook checks each element's path and accepts
  `custom "post/a.ogt" "post/b.ogt"` on one line. A key alone is allowed
  only for a list.
- `og-cli cook.rs` handles list fields of engine schemas.
- og-render's post chain already ran an ordered `Vec<PostHandle>`, so it
  needs no change.
- `RenderCapabilities.maxCustomPosts` stays the bound.
- Tests: `an_engine_record_list_field_takes_the_idl_capacity` and
  `post_effects_custom_is_a_list_of_every_pass_og_render_runs`, plus the
  look cook tests.

### 7. TS runtime: a frozen read, a whole write (989ba226)

- `og.list(of, capacity, options)` declares a list column of a game
  component.
- Reading a list field answers an `Object.freeze`d copy, so
  `post.custom.push(x)` throws a TypeError instead of silently doing
  nothing.
- Writing takes a whole array. A write that is longer than the capacity,
  is not an array, or has an element of the wrong kind is refused.
- The generated .d.ts doc text says so.
- Tests:
  - `packages/engine/tests/lists.test.ts` covers the frozen read,
    whole-array write, refusals, snapshot and restore, and a hash fixture
    (`tests/support/lists-hash.ts`).
  - `api-surface.test.ts` declares a list.

### 8. fx-showcase, og-game skill, design (79bdbb81)

- The shaders station sets `post.custom = [h.tint, h.crt]` (`e5.ts`) and
  clears with `custom: []` (`main.ts`).
- `tests/shaders.og.ts` keeps its checks; only its comments and messages
  changed.
- The post assets' comments name the list.
- docs/design.md §2.6 and the og-game skill (the custom post section,
  §23 in the file) describe the list.

## Frames read (lavapipe, tier vulkan-cpu, debug build)

From `templates/fx-showcase`, the frame is copied here as
`station-shaders.png`:

```
og run . --commands "0:station:shaders;1:screenshot:before.png;1.5:screenshot:station-shaders.png;1.5:dump_entity:stage:stage.json" --ticks 95 --out /tmp/frames --size 1280x720
```

The frame shows the warm amber station: the tint pass, then the CRT's
curved, scanlined screen with black corners. The pass order is unchanged.
I measured regions with `magick … -crop … -format "%[fx:mean.r*255]…"`
and compared them against the previous lavapipe evidence frame
`tests/evidence/debt/render-targets/station-shaders.png` (custom1/custom2):

| region | this branch (R G B) | previous frame (R G B) |
|---|---|---|
| ground 200×100 at (900,280) | 170.1 150.3 110.2 | 170.1 150.3 110.2 |
| sky 300×80 at (400,120) | 214.9 208.8 196.3 | 215.0 208.8 196.3 |
| four 6×6 corners | 0 0 0 | 0 0 0 |

- The ground's red is over its blue by 60, against the playtest's
  threshold of 35.
- The whole-frame differences are on the animated sphere's dissolve and
  the grass edges only.
- I tried a same-run comparison at a3bf3dc, but that checkout ran the
  branch's ignored generated outputs and addon. The previous evidence
  frame is the old-code reference.
- og-tests `--test targets`, which includes the custom-post references,
  passes with every reference unmoved.

No Metal references are owed: the chain's behaviour and every reference
frame are unchanged.

## Left out

- The web cross-host half of `lists.test.ts` needs the wasm, which this
  stream does not build. The coordinator runs it at the wave's end with the
  browser tiers. The native half passes.
- The fx-showcase playtest (`tests/shaders.og.ts`) runs under `og test`,
  which this stream does not run. Its checks are the ones measured above.
- By design, a list holds one number or handle type. There are no lists of
  strings, vectors or structs, and no nested lists. A list is never tunable
  and has no default or range: it starts empty.
- No CHANGELOG line; the coordinator writes it at the merge.

## Shared files touched

- `crates/og-api/api.ogt` and every generated output:
  - `crates/og-api/src/generated/host.rs`;
  - `packages/engine/types/index.d.ts` and `bindings.json`;
  - `packages/engine/schema/api.schema.json`;
  - `packages/engine/src/generated/abi.ts`;
  - the generated wiki pages under `packages/knowledge/wiki/`. **The wiki's
    `api.ogt` line numbers moved on most pages**, because the reflect text,
    the `list` column and PostEffects changed near the top of api.ogt.
    Another branch that edits api.ogt will conflict there: regenerate with
    `og api gen --wiki` after the merge rather than merging by hand.
- `crates/og-core/src/reflect.rs`, the `FieldType` variant every reader
  matches on. A branch adding a `FieldType` arm elsewhere must add the
  `List` arm.
- `crates/og-runtime/src/codec.rs`, `engine.rs`, `views.rs`, `look.rs`,
  `assets.rs` and `scene_save.rs`.
- `crates/og-idl` (`model.rs`, `load.rs`, `validate.rs`, and gen `ts`,
  `tables`, `wiki` and `host`).
- `packages/engine/src/runtime.ts` and `index.ts`.
- `packages/knowledge/skills/og-core/SKILL.md`, `og-game/SKILL.md`, and
  `docs/design.md` §2.6.
