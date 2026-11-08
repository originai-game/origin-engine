<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-pause"></a>

### `og.ui.pause`

Hold a play where it is.

```ts
og.ui.pause(play: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `boolean`.

Binding symbol `og_ui_pause` · publish builds: kept

```ogt
fn pause {
  doc "Hold a play where it is."
  param play u32 "The play."
  returns bool
}
```

<sub>Module `ui` line 4434</sub>
