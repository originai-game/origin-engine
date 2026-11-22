<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-capture"></a>

### `og.input.capture`

Where the capture is: active while it waits, done with the control it took, its binding and the conflicts applying it would meet, or cancelled.

```ts
og.input.capture(world: World): CaptureState
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `CaptureState`.

Binding symbol `og_input_capture` · publish builds: kept

```ogt
fn capture {
  doc "Where the capture is: active while it waits, done with the control it took, its binding and the conflicts applying it would meet, or cancelled."
  param world World "The world."
  returns CaptureState
}
```

<sub>Module `input` line 457</sub>
