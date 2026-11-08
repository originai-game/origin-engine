<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-reverse"></a>

### `og.ui.reverse`

Turn a play round where it is: an interrupted opening closes from where it got to.

```ts
og.ui.reverse(play: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `boolean`.

Binding symbol `og_ui_reverse` · publish builds: kept

```ogt
fn reverse {
  doc "Turn a play round where it is: an interrupted opening closes from where it got to."
  param play u32 "The play."
  returns bool
}
```

<sub>Module `ui` line 2339</sub>
