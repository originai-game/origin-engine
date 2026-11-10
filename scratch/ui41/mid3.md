
### `LayoutCache` (`Ui.kept`, 44 fields) split

`kept` has 163 sites, written by tree (9), style (18), layout (21), paint (32) and docs (2). It is four stages' caches in one struct. It splits by the stage that writes each field (the per-file counts come from `rg 'kept\.<field>'`):

| New struct | Owner | Fields |
|---|---|---|
| `Invalidation` | Tree | `whole fresh full epoch prev_epoch forced rederive restored visit visiting` (10) |
| `LayoutCache` | Layout | `prev clean laid_out sig laid_mark laid resized relayout scrollers anchor_toggled prev_len` (11) |
| `Settlement` | Layout | `measurements query_roots unclean` (3) |
| `PaintCache` | Paint | `cmds last_cmds named ranges sigs repainted reused_all last_roots last_tail aligned aligned_kept shifted_out copied prev_cmds levels` (15) |
| `MotionCache` | Style | `motion_moving motion_groups widget_channels timelines timeline_restyle` (5) |

`prev_len` is written by paint and by `end_frame`. It is assigned after the code that uses it is read in batch 1. `Orders` (`ui/visit.rs`) splits the same way: `pre pre_pos roots feat` stay in Tree, and `draw ends pos` (the draw order, which paint writes in `update_draw_order` and `paint_reuse`) move to Paint. `WorldUi` (16 fields) splits into a declared half (`view poses emitters templates marks kept kept_mass mass_built mass_spawned kept_effects grounds`, written by tree) and a placed half (`placed frame pointed last_pointer state`, written by layout and particles) in batch 6.

## `impl Ui` blocks → target state

**Target** is where the block's methods go. *Ui (front end)*, *Ui (observers)* and *Ui (configuration API)* stay on the façade: they are the public surface and call through to the states. A tree-stage block is assigned to Interaction or Tree, whichever of the two it touches more. *Writes outside the target* are the field writes that need an explicit `&mut` to another state (`node.X` is a `Node` field of group X), or a restructuring step listed below. Every block writes `Out` (warnings, events), so those writes are left out.

Three blocks do not move whole:
- `ui/finish.rs:30` is split by region (batch 3). Its helpers go to Layout (`line_box`, `prepare_layout_node`, `apply_gutter`, `content_visibility`), Paint (`relevel`, `splice_levels`, `missing_glyphs`) and Interaction (`note_relevance`, `touch_min`). `end_frame` stays on `Ui` as the driver.
- `ui/configure.rs:11` keeps `new`, the setters and `begin` on `Ui`. `begin` becomes one `begin` per state (R12).
- `ui/commands.rs:3` moves to Paint, except `world_step`, `apply_world` and `plane_quads`, which go to Layout (they place; they do not paint).

