<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-applyCapture"></a>

### `og.input.applyCapture`

Apply what the capture took: og.input.rebind with its slot and binding.

```ts
og.input.applyCapture(world: World, policy?: ConflictPolicy): Result<RebindAnswer>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `policy` | `ConflictPolicy` | yes | What to do with a conflict; omitted is refuse. |

**Returns** `Result<RebindAnswer>`: Refused when no capture is done.

Binding symbol `og_input_apply_capture` · publish builds: kept

```ogt
fn applyCapture {
  doc "Apply what the capture took: og.input.rebind with its slot and binding."
  param world World "The world."
  param policy ConflictPolicy "What to do with a conflict; omitted is refuse." {
    optional true
  }
  returns Result.RebindAnswer "Refused when no capture is done."
}
```

<sub>Module `input` line 427</sub>
