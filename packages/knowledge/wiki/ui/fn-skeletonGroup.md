<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-skeletonGroup"></a>

### `og.ui.skeletonGroup`

A skeleton: shapes of the neutral fill where content will arrive, swept by one sheen across the whole group (not each piece on its own clock), cornered as the parts they stand for. The sheen stops under reduced motion and the shapes breathe their opacity instead.

Parts: Skeleton.piece[<shape>], Skeleton.sheen. Accessibility: one progressbar, busy, named by name. Probe ids: <id>, <id>/Piece#i.

```ts
og.ui.skeletonGroup(id: string, desc?: SkeletonGroupDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SkeletonGroupDesc` | yes | What it stands in for. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn skeletonGroup {
  doc "A skeleton: shapes of the neutral fill where content will arrive, swept by one sheen across the whole group (not each piece on its own clock), cornered as the parts they stand for. The sheen stops under reduced motion and the shapes breathe their opacity instead."
  doc "Parts: Skeleton.piece[<shape>], Skeleton.sheen. Accessibility: one progressbar, busy, named by name. Probe ids: <id>, <id>/Piece#i."
  host false
  param id String "Its id."
  param desc SkeletonGroupDesc "What it stands in for."
  returns Void
}
```

<sub>`api.ogt` line 21262</sub>
