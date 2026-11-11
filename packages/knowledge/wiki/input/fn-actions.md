<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-actions"></a>

### `og.input.actions`

Every action, in definition order, with its bindings now and its defaults: what a controls screen lists.

```ts
og.input.actions(world: World): readonly ActionInfo[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly ActionInfo[]`.

Binding symbol `og_input_actions` · publish builds: kept

```ogt
fn actions {
  doc "Every action, in definition order, with its bindings now and its defaults: what a controls screen lists."
  param world World "The world."
  returns List.ActionInfo
}
```

<sub>Module `input` line 336</sub>
