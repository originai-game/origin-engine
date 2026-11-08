<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-docEvents"></a>

### `og.ui.docEvents`

Take the events of documents built where code did not see them (layers built at the frame's end); og.ui.screen answers its own.

```ts
og.ui.docEvents(world: World): readonly UiDocEvent[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly UiDocEvent[]`.

Binding symbol `og_ui_doc_events` · publish builds: kept

```ogt
fn docEvents {
  doc "Take the events of documents built where code did not see them (layers built at the frame's end); og.ui.screen answers its own."
  param world World "The world."
  returns List.UiDocEvent
}
```

<sub>Module `ui` line 8169</sub>
