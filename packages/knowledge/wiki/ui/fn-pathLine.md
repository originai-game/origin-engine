<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-pathLine"></a>

### `og.ui.pathLine`

A path line to the bar of Baldur's Gate 3, XCOM 2 and RTS move orders: the route on the ground (draped on the surface with desc.ground) as a line with chevrons marching toward its end, the part past this turn's reach in danger, an arrow head or a destination ring at its end with a label (the distance, the cost), a cross when it cannot be taken.

Parts: PathLine.line[reach|beyond][invalid], PathLine.chevron, PathLine.end, PathLine.label. Accessibility: text named the label. Probe ids: <id>, <id>Label (the label, a tag of its own: it declutters with the other tags and stays in the safe area).

```ts
og.ui.pathLine(world: World, id: string, desc?: PathLineDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world: its og.Camera projects the ground, so the calling system declares og.Camera among its reads. |
| `id` | `string` | no | Its id. |
| `desc` | `PathLineDesc` | yes | The route. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn pathLine {
  doc "A path line to the bar of Baldur's Gate 3, XCOM 2 and RTS move orders: the route on the ground (draped on the surface with desc.ground) as a line with chevrons marching toward its end, the part past this turn's reach in danger, an arrow head or a destination ring at its end with a label (the distance, the cost), a cross when it cannot be taken."
  doc "Parts: PathLine.line[reach|beyond][invalid], PathLine.chevron, PathLine.end, PathLine.label. Accessibility: text named the label. Probe ids: <id>, <id>Label (the label, a tag of its own: it declutters with the other tags and stays in the safe area)."
  host false
  param world World "The world: its og.Camera projects the ground, so the calling system declares og.Camera among its reads."
  param id String "Its id."
  param desc PathLineDesc "The route."
  returns Void
}
```

<sub>Module `ui` line 6765</sub>
