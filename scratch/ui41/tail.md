
## Calls between stages

These are method calls from one stage's `impl Ui` methods to another's (`self.f(...)` / `ui.f(...)`), with rows as callers and columns as callees. After the move, each off-diagonal call either goes through an explicit borrow of the callee's state or is restructured.

| caller ↓ / callee → | tree | style | layout | paint | docs | front | env | observe |
|---|---|---|---|---|---|---|---|---|
| tree | 230 | 4 | 11 | 6 | 0 | 10 | 29 | 1 |
| style | 4 | 85 | 5 | 1 | 6 | 8 | 30 | 0 |
| layout | 15 | 1 | 36 | 3 | 0 | 3 | 4 | 0 |
| paint | 8 | 27 | 11 | 67 | 0 | 8 | 5 | 0 |
| docs | 51 | 42 | 6 | 5 | 203 | **134** | 39 | 3 |
| front | 113 | 10 | 1 | 8 | 0 | 609 | 121 | 2 |
| observe | 15 | 1 | 4 | 0 | 6 | 0 | 7 | 50 |

Calls into `env` are getters (`theme()`, `tick()`, `viewport()`) and become `env.x` reads. No core stage calls docs methods. Core depends on docs through **types**, not calls (next section), plus one function, `layers::build_without_host` (`end_frame:266`). Paint calls 6 tree methods (`hit path_of note_relevance note_semantics record_matches selected_ranges`); the last four are interaction bookkeeping done during paint, which batch 3 moves out to the interaction step.

## Cross-stage borrows that need restructuring

These are the writes that stop a block from becoming a method on one state with `&` borrows of the others. Each item says where it is fixed in the plan.

| # | Where | What crosses | Fix | Batch |
|---|---|---|---|---|
| R1 | `ui/commands.rs:487–511, 545–560` | Paint overwrites `theme.palette` with the accent, `Node.clip` with the draw clip and `Ui.drawing` with the node's map around `node_commands`, then restores them | Pass a `PaintCtx { clip, palette, map }` argument through `node_commands` and the 5 `node_draw` blocks. This removes the `drawing` field. | 1 |
| R2 | `Ui.kept` | Four stages' caches in one struct (163 sites) | Split into five structs (table above) | 1 |
| R3 | `Orders.draw/ends/pos` | Paint writes Tree's orders (14) | Draw order moves to Paint | 1 |
| R4 | `layout.rs apply_patch`, `layout/writing.rs`, `ui/settlement.rs swap`, `end_frame` (11), `line_box`, `apply_gutter` | Layout writes the authored element: `el` 45, `props` 8, `content` 9, `sampled` 15 (`write_values`). The `layout_mutated` / `authored_box` mechanism exists to undo these writes. | Layout keeps used values in `LayoutBox` and reads `el`. Container-query and anchor patches become Style outputs (`StyleNode`). | 6 |
| R5 | `run_motion` | Style writes `kept` motion fields (13) | Removed by `MotionCache` (R2) | 1 |
| R6 | `ui/commands.rs` | Paint commits `part_commits` (Style) after rendering | Paint returns the commits and the driver applies them | 1 |
| R7 | `ui/select.rs mark_selection`, `select_marks` | Paint writes the selection (`Ui.selected`, `State.selection`, `Node.selected`) | The marks are computed in the interaction step before paint; paint reads them | 3 |
| R8 | `State.motion` | Written by paint (3) and tree (2): ghost drawings and drags | Paint writes the ghost commands into Paint, and motion reads them on the next tick. Drags go through Interaction. | 4 |
| R9 | `Ui.world` | Written by tree (31, declarations), layout (21, placement) and paint (particles, 6) | Split `WorldUi` into a declared half and a placed half; particle simulation goes to Style (motion) | 6 |
| R10 | builders | Docs and front end write Tree's build cursor (`stack` 22, `floating`, `last_closed`, `owning`, `in_place`, `emitted`), `Node` authored fields, and Interaction requests (`request_focus` 7, `nav`, `virtual_lists` 7, `scroll`) | One builder borrow: `&mut Tree` plus a request queue into Interaction, which is applied by `interaction.update`. This is expected for builders and becomes part 2's interface. | 4, part 2 |
| R11 | `doc/build/**` | Docs call 55 front-end widget methods (134 sites) | Part 2 inverts this: controls become document element behaviours | part 2 |
| R12 | `ui/configure.rs begin` | `begin` resets fields of every state (Tree 44, Interaction 38, Env 36, …) | One `begin` per state, called in today's order | 5 |
| R13 | `end_frame:1034`, style blocks | The interaction step reads `docs.retained`; style reads `DocHost` (34 sites) and writes it (5, pseudo-element registrations in `generated.rs` and `animate.rs`) | `&Docs` parameters; the writes become return values | 4 |
| R14 | `motion/run.rs:1419` (`note_drag_and_drop`, `record_matches`) | Observer and interaction bookkeeping kept in motion files writes Interaction (6) | Moves to Interaction | 4 |

## `crate::doc` back-edges (30 files outside `src/doc`)

For the crate split, each back-edge goes one of four ways:
- **(a) style types:** CSS computed-value types that live under `doc` move down to the style stage.
- **(b) part names:** the part vocabulary `schema::parts::*` moves to style.
- **(c) document modules:** modules that belong to documents but live outside `doc` move into it.
- **(d) top-level callers:** the façade and the front end may depend on docs, so these edges stay.

| File | Names | Resolution |
|---|---|---|
| `actions.rs` | `UI_STORE`, `InputMode::Game` | (c) the constant and `InputMode` move to Interaction |
| `css_color.rs` | `css::camel`, `css` | step 1 moves colour out; its helpers go with it |
| `decor.rs` | `paint_css::{LayerFrame, CssPaint, Attach}` | (a) |
| `dress.rs` | `matching::Facts`, `css::kebab`, `selectors` | (a); `dress` is style |
| `layers.rs` | `view_transition`, `build`, `UiDocument` | (c) the layer stack is a document module |
| `layout/flex/position.rs` | `media::scroll` | (a) the sticky-edge bits |
| `layout/float.rs` | `paint_css::{GeoBox, Pos, CssClip}` | (a) |
| `layout/node.rs` | `build::PartHost` | (a) `PartHost` goes to style |
| `motion/css.rs` | `css::css_name`, `boxes::longhands`, `UiDocument` | (a); keyframes are passed in instead of `UiDocument` |
| `probe.rs` | `schema::parts::to_json`, `build::{slot_json, Built}` | (d) |
| `reactive/bindings.rs` | `UiDocument` | (c) `reactive` belongs to documents |
| `rich.rs`, `rich/spans.rs` | `text_css::{Decoration, DecorationStyle}` | (a) |
| `scroll.rs` | `media::scroll`, `scroll_css` | (a) |
| `slot.rs` | `UiDocument` | (c) |
| `style_file/art.rs` | `schema::parts::BUTTON_BASE` | (b) |
| `style_pack.rs` | `svelte::parse_with_pack`, `stylesheet::{part_rules_from, pack_extensions}` | (c) loading a pack parses documents |
| `svg.rs` | doc comment only | none |
| `svg/filter.rs` | `paint_css::blend` | (a) |
| `ui.rs`, `ui/configure.rs` | `StyleRule`, `at_rules::Sheet`, `build::{StyleCommit, DocHost}`, `selectors::Simple` | (a) for the rule and commit types; (d) for `DocHost` |
| `ui/controls.rs`, `widgets/controls/choices.rs`, `widgets/layers.rs`, `widgets/presentation.rs` | `schema::parts::*` | (b) |
| `ui/css_tokens.rs` | `css::kebab`, `text_css::family_list`, `calc::reduce` | (a) and step 1 (`calc`) |
| `ui/dressing.rs`, `ui/finish.rs` | `paint_css::LayerFrame` | (a) |
| `ui/text_controls.rs` | `build::Resolved` | removed in part 2 (front end over documents) |
| `view_transition.rs` | `doc::view_transition` | (a) the two modules join in style |

## Commit plan (Phase B)

### Invariants for every commit

- **It compiles.** `cargo check -p og-ui --all-targets`, workspace clippy `-D warnings`, `cargo fmt --check`.
- **Moves and logic edits are separate commits.** A move commit is checked with `scripts/verify-move.py`; a regrouping commit with `verify-move.py --rename` (batch 0). A logic commit is small, and its message says why the output cannot change.
- **No frame moves.** Each batch is checked in three steps:
  1. **CPU identity check** (scratch, not committed): for every og-ui test document and every UI-pack fixture, hash `UiFrame.items`, the `save_state` bytes and `dump_ui` at the base and at the head, and require them equal.
  2. **Metal compatibility:** og-ui's GPU tests on macrunner.
  3. **Gate:** the L4 run by T-01a1c5e9.
- **Per batch:** og-ui's tests and `scripts/check.sh quick --only fmt clippy rust-cpu drift bun-quick` in lane-13. The batch is pushed as a branch `ui-stages-N` and sent to T-01a1c5e9 to gate and land. `land.sh` is never run on the Mac.
- **No changelog fragment:** nothing a game sees changes. If a public `og_ui::` path used by another crate changes, that crate is updated in the same commit.

### Part 1 — `Ui` into stage states

| Batch | Commits (in order) | Kind | Result |
|---|---|---|---|
| **0** tool | 0.1 `verify-move.py --rename OLD=NEW…`: rename tokens on the old side, then compare multisets as today | tool | Regrouping commits are checked mechanically |
| **1** make the borrows possible | 1.1 `PaintCtx` replaces the palette, clip and map swaps (R1) · 1.2–1.5 split `LayoutCache` into `Invalidation`, `Settlement`, `PaintCache` and `MotionCache` (one struct per commit; R2, R5) · 1.6 split `Orders` (R3) · 1.7 paint returns `part_commits` (R6) | logic (1.1, 1.7) · rename (1.2–1.6) | No stage writes another stage's cache or Env |
| **2** group the fields | one commit per state, in this order: Env (16 + `State.tick`) · Out (8 + `voices feedback_rng feedback_last`) · Docs (2 + `rive`) · Style (8 + 4 State fields + `MotionCache`) · Paint (6 + `PaintCache`, draw order) · Layout (`texts`, `LayoutCache`, `Settlement`, `layout_mutated`) · World · Interaction (16 + `State`) · Tree (24 + `Invalidation`; `TreeIndex` renamed `index`) · Front (6 + `toasts next_toast touch_owners`) | rename only, `--rename`-verified | **`Ui` 91 → 10 fields**; snapshot bytes unchanged |
| **3** split the giant functions (still on `Ui`) | 3.1 `end_frame` → the six steps above, then the layout step (414 lines) into its settlement loop and its passes; driver ≤ 100 lines · 3.2 `run_motion` → its 9 phases (pending plays, plays, gestures, per-element states, channels, entrance and FLIP, shared elements, leaving, retention, CSS ends), each ≤ 200 · 3.3 `build_resolved_node` → the container half and one function per control kind (25 kinds: `label button art image icon meter select progress slider toggle color media file datetime radio tabs field badge kbd spacer divider joystick`, …), ≤ 250 · 3.4 `lower_decl` → family lowerers on its existing `<family>::lower(name, value)` pattern (`tracks`, `position_css`, …), ≤ 200 · 3.5 selection marks before paint (R7) | move (extractions: only the new fn headers and call lines differ) · 3.5 logic | The four functions over 750 lines are gone |
| **4** methods onto states | per stage, in this order: Layout (8 blocks) · Paint (13) · Style (14) · Interaction (10) · Tree (9) · Docs (22). Each block takes two commits: (a) in place, `impl Ui` → `impl <State>` with explicit parameters (`self.tree.x` → `tree.x`, `self.layout.x` → `self.x`), `--rename`-verified, with the call sites updated; (b) a verbatim move into the state's module. R8, R10 (builder borrow), R13 and R14 are fixed here. | rename + move | 76 blocks leave `Ui`; 25 stay as the front end, observers and configuration |
| **5** the pipeline | 5.1 `begin` per state (R12) · 5.2 `end_frame` destructures `Ui` and calls `tree.finish_build`, `layout.run`, `style.animate`, `layout.place`, `interaction.update`, `paint.record` | logic (call order unchanged) | The stages are steps with data between them |
| **6** `Node` by stage | 6.1 `Placed` → `Vec<Placed>` · 6.2 layout stops writing the authored element (R4), which removes `layout_mutated` and `authored_box` · 6.3 `LayoutBox` → `Vec<LayoutBox>` · 6.4 `StyleNode` · 6.5 `World` declared/placed (R9) | logic, one group per commit, each through the L4 | Layout takes `(&[Node], &mut [LayoutBox])`; `Node` 106 → about 43 fields |

Batches 1–2 are mechanical and low-risk, and can land within a few days of the start. Batch 4 is the longest (about 150 commits in 6 stage branches). Batch 6 is the riskiest: R4 changes how settlement produces its result, so it runs through the L4 after every commit.

### Part 2 — the retained tree becomes the model, the immediate API a front end

This part starts after part 1 lands, and its counts are measured again first.

1. **Controls as document elements.** Each control kind that `build_resolved_node` lowers through a widget becomes a behaviour of the document element on the retained tree. Its internal nodes are created on mount and patched on change. The widget bodies move verbatim into `doc/controls/<kind>.rs`, then are adapted in separate commits. This removes R11, one control kind per commit, with the L4 after each.
2. **The slot machinery goes.** `render_slots free_slots retiring slot_present persistent owning in_place built anonymous`, the persistent-slot ownership patches, are replaced by the retained tree's own node lifetimes.
3. **The immediate front end.** The builders (`label`, `button`, `row`, `text_field`, …) move to `og_ui::immediate`. They build a per-frame immediate document keyed by path and diffed against the last frame, so the same retained pipeline draws them. Their callers move with them: the inspector, debug overlays, the timeline label, the toast, `ui_check`, and the og-tests-cooked scenes, whose frames are the check. The API is not re-exported at the crate root and not added to the IDL.

### Part 3 — crates follow the stages

Crates, after step 1's leaves:
- **og-ui-tree:** the element model, the `Node` arena and `Env`.
- **og-ui-style:** CSS and computed values, the part vocabulary, motion, `Style`.
- **og-ui-layout:** `Layout`, text layout.
- **og-ui-paint:** `Paint`, draw and decorations.
- **og-ui-doc:** documents, Svelte, reactive, the layer stack, `Docs`.
- **og-ui:** the façade, `Interaction`, `World`, the front end and the pipeline.

The dependency order is tree ← style ← layout ← paint, and tree, style ← doc. Each crate move is a verbatim move after the back-edges above are resolved.

## Risks and open questions

- **Overlap with step 1.** Step 1 moves the CSS lexer, `calc`, colour, selectors and text out of og-ui core. Batch 2 renames sites in text and layout files. Phase B therefore starts from main after step 1 lands, and this table is regenerated with the scripts below before the first commit.
- **R4 is the only part that is not mechanical.** Settlement and writing modes currently rewrite authored boxes and undo the rewrite on the next frame. Moving that to used values must give the same boxes. The CPU identity check catches a difference before the L4 does.
- **Batch 4 review load.** About 150 small commits. I propose landing them as six branches, one per stage, each gated once on the L4, rather than one branch per block.
- **Coordinator decision:** whether part 2 step 3 also moves the og-tests-cooked scenes that use the immediate API to documents. That is not required: they keep working through the front end, and their frames are the regression check, so I propose they stay.

## How this was measured

The scripts are in `.amp/in/ui41/` on macrunner (not committed) and are rerun before Phase B:
- `collect.py` renames a struct's fields, runs `cargo check`, and collects the error sites.
- `index.py` brace-matches fns and impls.
- `join.py` maps sites to fns and classifies reads and writes.
- `stage.py` maps files, fns and `end_frame` regions to stages.
- `owners.py` holds the owner table.
- `blocks.py` and `calls.py` produce the block table and the call matrix; `gen.py` writes the tables.
