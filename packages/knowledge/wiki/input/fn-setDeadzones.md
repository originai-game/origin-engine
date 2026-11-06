<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-setDeadzones"></a>

### `og.input.setDeadzones`

A pad's dead zones (or every pad's): the player's setting, kept by og.input.saveBindings.

```ts
og.input.setDeadzones(world: World, desc?: DeadzoneDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `DeadzoneDesc` | yes | The pad and its dead zones. |

**Returns** `Result<void>`.

Binding symbol `og_input_set_deadzones` · publish builds: kept

```ogt
fn setDeadzones {
  doc "A pad's dead zones (or every pad's): the player's setting, kept by og.input.saveBindings."
  param world World "The world."
  param desc DeadzoneDesc "The pad and its dead zones."
  returns Result.Void
}
```

<sub>Module `input` line 478</sub>
