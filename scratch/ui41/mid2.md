
### `Node` (106 fields → 6 groups)

In batches 2–5, `Node` stays one struct in `Tree`'s arena. Batch 6 splits it into parallel arrays indexed by slot, one per group: `Tree` structure and the `Authored` element stay as `Node`; `StyleNode`, `LayoutBox` and `Placed` become arrays owned by Style and Layout. After the split, a layout function takes `(&[Node], &mut [LayoutBox])` instead of `&mut [Node]`. The `layout 1` / `tree 1` writes on many rows below are `Node::new` and `inherit_layout`, which are counted as constructors.

Groups: Layout 47 · Authored 31 · Tree 12 · Placed 9 · Style 4 · Interaction 3.

