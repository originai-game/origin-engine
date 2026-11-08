<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-remember"></a>

### `og.ui.remember`

Set a number memory (see memory).

```ts
og.ui.remember(id: string, value: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The id. |
| `value` | `number` | no | The value. |

**Returns** `void`.

Binding symbol `og_ui_remember` · publish builds: kept

```ogt
fn remember {
  doc "Set a number memory (see memory)."
  param id String "The id."
  param value f64 "The value."
  returns Void
}
```

<sub>Module `ui` line 1690</sub>
