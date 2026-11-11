<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-resetBindings"></a>

### `og.input.resetBindings`

Back to the defaults: one action's bindings and toggle, or (no action) every action's, and the pads' dead zones.

```ts
og.input.resetBindings(world: World, action?: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | yes | The action; omitted is every action. |

**Returns** `Result<void>`.

Binding symbol `og_input_reset_bindings` · publish builds: kept

```ogt
fn resetBindings {
  doc "Back to the defaults: one action's bindings and toggle, or (no action) every action's, and the pads' dead zones."
  param world World "The world."
  param action String "The action; omitted is every action." {
    optional true
  }
  returns Result.Void
}
```

<sub>Module `input` line 420</sub>
