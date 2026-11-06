<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-loadBindings"></a>

### `og.input.loadBindings`

Take what og.input.saveBindings wrote, in place of the player's settings now. Bindings for an action not defined yet are kept until it is (load at start, before or after defining).

```ts
og.input.loadBindings(world: World, text: string): Result<readonly string[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `text` | `string` | no | The saved text. |

**Returns** `Result<readonly string[]>`: A warning for each entry it could not take (that action keeps its defaults); refused for text that is not saved bindings.

Binding symbol `og_input_load_bindings` · publish builds: kept

```ogt
fn loadBindings {
  doc "Take what og.input.saveBindings wrote, in place of the player's settings now. Bindings for an action not defined yet are kept until it is (load at start, before or after defining)."
  param world World "The world."
  param text String "The saved text."
  returns Result.List.String "A warning for each entry it could not take (that action keeps its defaults); refused for text that is not saved bindings."
}
```

<sub>`api.ogt` line 2415</sub>
