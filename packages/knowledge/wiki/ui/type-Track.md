<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Track"></a>

### `Track` (union)

A grid track's size.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "px"` | [`TrackPx`](../ui/type-TrackPx.md#type-TrackPx) | A fixed size. |
| `kind: "fr"` | [`TrackFr`](../ui/type-TrackFr.md#type-TrackFr) | A share of the free space. |
| `kind: "auto"` | [`TrackAuto`](../ui/type-TrackAuto.md#type-TrackAuto) | Its largest single-cell item. |
| `kind: "minMax"` | [`TrackRange`](../ui/type-TrackRange.md#type-TrackRange) | Its content between min and max. |
| `kind: "percent"` | [`TrackPercent`](../ui/type-TrackPercent.md#type-TrackPercent) | A fraction of the grid. |

```ogt
union Track {
  doc "A grid track's size."
  tag kind
  case px TrackPx "A fixed size."
  case fr TrackFr "A share of the free space."
  case auto TrackAuto "Its largest single-cell item."
  case minMax TrackRange "Its content between min and max."
  case percent TrackPercent "A fraction of the grid."
}
```

<sub>Module `ui` line 2023</sub>
