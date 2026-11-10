# og-ui into stage states — field-to-stage table and commit plan (#41 step 3, Phase A)

**For approval by the coordinator.** Measured on `claude/zen-clarke-grqlh1` at `66e91368` (main `21fd3148` plus the branch). Nothing is committed. Phase B starts after this plan is approved and step 1 (the leaf crates) is on main. All counts get measured again at that point, because step 1 moves files this plan touches.

## Summary

- `Ui` has **91** fields (not 93), `State` 61 and `Node` 106. The compiler reports **7,554 / 1,559 / 5,761** access sites for them (from the lib and test builds, deduplicated to 3,894 / 796 / 2,894). There are **101** `impl Ui` blocks with 765 methods, about 39,200 lines.
- The target is a façade `Ui` holding ten states: **Env** (read-only configuration), **Tree** and **Interaction** (the tree-and-interaction stage), **Style** (motion included), **Layout**, **Paint**, **Docs**, **World**, **Out** (the frame's output queues) and **Front** (the immediate front end's own memory). `end_frame` becomes a driver that calls each stage with explicit borrows.
- Most fields have a single stage that writes them. The restructuring is concentrated in a few places:
  - paint temporarily mutates `theme.palette`, `Node.clip` and `drawing` around `node_commands`;
  - `LayoutCache` (44 fields) bundles the caches of four stages and is written by tree, style, layout, paint and docs;
  - layout writes the authored element (`Node.el` 45 times, `props` 8, `content` 9);
  - documents lower **through the immediate widgets**: 134 call sites into 55 widget methods, 36 of them in `build_resolved_node` alone.
- The last point is the centre of part 2. Today the immediate API is not just a front end over documents: it is how documents implement their controls. Inverting that dependency is what "the retained tree becomes the model" means here.
- Following the decision you confirmed, the immediate API stays engine-internal: the inspector, debug overlays and the engine's own callers use it. It is not in the IDL today (no builders in `crates/og-api/api/*.ogt`), and the plan keeps it that way.
- The plan has 9 batches. Every commit compiles and moves no frame, and moves are kept in separate commits from logic edits.

## Measurements before

| Measure | Value |
|---|---|
| `Ui` / `State` / `Node` fields | 91 / 61 / 106 |
| `impl Ui` blocks (methods, lines) | 101 (765, 39,224) — doc 29, ui 29, widgets 13, motion 5, element 3, rich 3, world 3, other 16 |
| Longest functions | `end_frame` 1,151 · `build_resolved_node` 1,134 · `run_motion` 906 · `svelte::element_node` 781 · `lower_decl` 767 · `edit` 708 · `resolve` 661 |
| og-ui functions ≥ 300 / ≥ 500 lines | 22 / 9 (of 3,843, tests excluded) |
| Files outside `src/doc` naming `crate::doc` | 30 (the issue said 29) |
| Docs → front-end calls | 134 sites, 55 widget methods |
| Immediate API callers outside og-ui | og-runtime inspector (`inspector/panel.rs`, `inspector/ui_debug.rs`), `timeline/evaluate.rs` (label), `modules/game_widgets.rs` (toast), `determinism.rs` (test), og-host-node `ui_check.rs`, og-tests-cooked scenes (`text_scenes.rs` 15, `materials_gfx.rs` 5, `ui.rs` 1), whose **reference frames must not move** |

## Target shape

```rust
pub struct Ui {
    env: Env,                 // configuration; &Env to every stage
    tree: Tree,               // node arena, build cursor, slots, index, preorder, invalidation
    interaction: Interaction, // input, persisted State (focus, caret, scroll, hits...), nav frame
    style: Style,             // part CSS, tokens, motion, timelines, view transition
    layout: Layout,           // text cache, layout reuse, settlement
    paint: Paint,             // paint reuse, part and decor draws, draw order
    docs: Docs,               // DocHost and the reactive session
    world: World,             // world-space UI (declared half / placed half)
    out: Out,                 // warnings, events, feedback, haptics, clipboard, UiFrame, stats
    front: Front,             // the immediate front end's memory (touched, kept texts, toasts...)
}
```

The frame is today's `end_frame` regions, called in the same order:

| Today's `end_frame` lines | Step | Borrows |
|---|---|---|
| 247–438 | `tree.finish_build` (retire, variants, part appearances, derive lists, gutters, content visibility, reuse check, `sync_tree`) | `&mut Tree`, `&Style`, `&Env` |
| 439–910 | `layout.run` (layout and settlement, popover arrows, orders, baselines, anchors) | `&mut Layout`, `&mut Tree` (until batch 6: `&Tree` + `&mut [LayoutBox]`), `&Env` |
| 911–955 | `style.animate` (sampling, RTL mirror, shakes, stagger, `run_motion`, bursts) | `&mut Style`, `&Tree`, `&mut Layout` (until batch 6) |
| 956–979 | `layout.place` (draw order, HUD, maps, planes, world) | `&mut Layout`, `&mut World`, `&Tree` |
| 980–1202 | `interaction.update` (scroll ranges, scopes, input specs, hits, focus, pads, taps, cursor) | `&mut Interaction`, `&Tree`, `&Docs` |
| 1203–1397 | `paint.record` (parts, decorations, selection marks, commands, levels, view transition, items) | `&mut Paint`, `&Tree`, `&Style`, `&Env`, `&mut Out` |

The **owner** of a field is the state that writes it during the frame. Other stages read it through an explicit `&` borrow. Every remaining cross-stage write is listed under [Cross-stage borrows](#cross-stage-borrows-that-need-restructuring) and is resolved before the methods move.

## Field-to-stage table

Stage codes in the columns: **T** tree and interaction · **S** style, motion included · **L** layout · **P** paint · **D** documents · **F** immediate front end (widgets, controls, `ui/build.rs` builders) · **E** configuration API and `begin` (`ui/configure.rs`) · **O** observers (snapshot, inspect, debug, probe) · **C** constructors · **X** tests. Example: `T12 L3` means 12 sites in tree code and 3 in layout code.

How the sites were found: every field of the struct was renamed, `cargo check -p og-ui --lib --tests` was run, and each E0609 / E0615 / E0560 error location was taken as one site. The stage of a site is the stage of the function that contains it. In `end_frame` it is the stage of the region that contains it (table above). An access counts as a write when it is an assignment, a compound assignment, `&mut`, or a mutating method call (`push`, `insert`, `take`, `retain`, …); otherwise it is a read. The read/write split is heuristic (it misses writes made through a `let x = &mut …` binding), but the site counts are exact.

### `Ui` (91 fields → 10 states)

Owners: Env 16 · Tree 24 · Interaction 16 · Style 8 · Layout 3 · Paint 7 · Docs 2 · World 1 · Out 8 · Front 6.

