<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-cancelCapture"></a>

### `og.input.cancelCapture`

Stop a capture without taking anything.

```ts
og.input.cancelCapture(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `void`.

Binding symbol `og_input_cancel_capture` · publish builds: kept

```ogt
fn cancelCapture {
  doc "Stop a capture without taking anything."
  param world World "The world."
  returns Void
}
```

<sub>`api.ogt` line 2435</sub>
