<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-stop"></a>

### `og.ui.stop`

Stop a play: its elements go back to their own values (on their motion, if they have one).

```ts
og.ui.stop(play: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `boolean`.

Binding symbol `og_ui_stop` · publish builds: kept

```ogt
fn stop {
  doc "Stop a play: its elements go back to their own values (on their motion, if they have one)."
  param play u32 "The play."
  returns bool
}
```

<sub>Module `ui` line 5151</sub>
