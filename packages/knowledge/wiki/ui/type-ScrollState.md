<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ScrollState"></a>

### `ScrollState` (struct)

A scroll container as the last frame laid it out (og.ui.scrollState).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offsetX` | `number` | `0` | — | `cssPx` | — | How far its content is scrolled right. |
| `offsetY` | `number` | `0` | — | `cssPx` | — | How far its content is scrolled down. |
| `viewWidth` | `number` | `0` | — | `cssPx` | — | Its view's width, inside its padding. |
| `viewHeight` | `number` | `0` | — | `cssPx` | — | Its view's height, inside its padding. |
| `contentWidth` | `number` | `0` | — | `cssPx` | — | Its whole content's width, unclipped (the view's on an axis it does not scroll). |
| `contentHeight` | `number` | `0` | — | `cssPx` | — | Its whole content's height, unclipped. |

```ogt
struct ScrollState {
  doc "A scroll container as the last frame laid it out (og.ui.scrollState)."
  field offsetX f32 "How far its content is scrolled right." {
    unit cssPx
  }
  field offsetY f32 "How far its content is scrolled down." {
    unit cssPx
  }
  field viewWidth f32 "Its view's width, inside its padding." {
    unit cssPx
  }
  field viewHeight f32 "Its view's height, inside its padding." {
    unit cssPx
  }
  field contentWidth f32 "Its whole content's width, unclipped (the view's on an axis it does not scroll)." {
    unit cssPx
  }
  field contentHeight f32 "Its whole content's height, unclipped." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 17061</sub>
