<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-seek"></a>

### `og.ui.seek`

Move a play to a time; it fires no events on the way. Returns false for an unknown play.

```ts
og.ui.seek(play: number, ms: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |
| `ms` | `number` | no | The time, ms. |

**Returns** `boolean`.

Binding symbol `og_ui_seek` · publish builds: kept

```ogt
fn seek {
  doc "Move a play to a time; it fires no events on the way. Returns false for an unknown play."
  param play u32 "The play."
  param ms f32 "The time, ms."
  returns bool
}
```

<sub>Module `ui` line 2724</sub>
