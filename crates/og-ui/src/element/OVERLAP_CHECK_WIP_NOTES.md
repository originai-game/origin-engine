# WIP: og ui check skips the base screen layer (and the attempted fix is too wide)

Branch `wip/ui-check-base-layer`. **Not for landing.** Diagnosis + a first fix
attempt + a regression test, for UI Sol 2 to finish. Delete this file in the
real head.

## The defect

`og ui check` / `og.ui.overlaps` reports **0 overlaps for almost every
screen**, so it never catches a real content-on-content overlap inside a
screen (first found on `film-flowchart` at phone 150 %: a NodeGraph caption
landing on the node in the row below — since fixed in the node-graph document,
but the check should have caught it).

## Root cause (pinned with addon vs in-crate dumps)

A screen's whole content is mounted on one `Float::Modal` layer. The default
screen layer kind is `Menu`, which maps to `Float::Modal`; ~30 of the 34
screens in the sheet are Menu / Modal / default / popover, i.e. non-anchored
floating layers.

`find_overlaps` (this file) builds `over_layers` to skip layers drawn **over**
the rest (dialogs, menus, tips, toasts): the preorder of every non-anchored
floating layer. Both overlap loops exclude `over_layers`. So the **base screen
layer itself** is classified as an over-layer and the whole screen is skipped.

Decisive dumps (temporary `OGTRACE`/`OGDUMP` eprintln in `find_overlaps`),
old checker, `film-flowchart:story` at phone 150 %:

```
# addon render (og ui review) — the whole screen is one Modal layer, loops empty:
OGTRACE order_len=50 nodes_len=50 node0_children=0 floating=[1] hall_in_order=Some(true)
OGTRACE floating node 1 path=film-flowchart float=Modal
OGTRACE placed_count=0 others_count=0        # <-- the check is a no-op

# same component mounted in-crate under a plain <div> host (not a screen layer):
OGTRACE order_len=13 nodes_len=13 node0_children=1 floating=[]
OGTRACE placed_count=6 others_count=7        # <-- here it DOES report the overlap
```

Every field find_overlaps receives for the Hall caption and Wait node is
identical between the two (drawn rect, shown rect, out-of-flow, painted, seen,
decls, clip chain) — the only difference is that in the addon the content sits
on a `Float::Modal` base layer that `over_layers` swallows.

## The fix attempt in this branch (frame.rs)

Never put the **base layer** — the lowest drawn one — into `over_layers`, so
its own content is checked; a real dialog over a game page is unchanged
(node 0's in-tree content still draws beneath it, so the dialog stays an
over-layer). The regression test in `control_props.rs`
(`a_full_screen_layer_is_checked_for_its_own_overlaps`) fails on the old
checker (`overlaps=[]`) and passes on this fix, and the fix does catch the
content-on-content overlap.

## Why this fix is TOO WIDE (full-suite render, phone 150 %)

A `Float::Modal` is by definition "a full-surface backdrop over the tree". The
screens draw an absolute `inset: 0` scrim (`Veil` / `Ground` / `Shade` — e.g.
`film-flowchart.shade`, `credits.shade`, calibration `#Ground`) that
intentionally sits **under** all content. Un-skipping the base layer makes the
checker report that backdrop as overlapping every element on top of it:

- **139 overlaps across the 14 screens rendered** (~20–30 per screen), all of
  the form `Veil`/`Ground`/`Shade` **X** content. On the old checker these
  were 0 (whole layer skipped).

So the fix flags intentional under-layers — too wide.

## What a correct fix needs

Check content-on-content **within** the base layer while keeping a full-screen
backdrop exempt as an intentional **under**-layer — symmetric to how dialogs
are intentional over-layers. The hard part is robustly identifying the backdrop
(an absolute, viewport-covering element drawn lowest in the base layer) without
also exempting real content, across every screen and pack, then re-validating
the full component + screen suite finding counts (a real new finding routes to
the component's author; a false one means the exemption is still wrong).
