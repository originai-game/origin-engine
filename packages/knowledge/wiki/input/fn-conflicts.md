<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-conflicts"></a>

### `og.input.conflicts`

What a rebinding would share a control with, changing nothing.

```ts
og.input.conflicts(world: World, desc?: RebindDesc): Result<readonly BindingConflict[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RebindDesc` | yes | The rebinding. |

**Returns** `Result<readonly BindingConflict[]>`.

Binding symbol `og_input_conflicts` · publish builds: kept

```ogt
fn conflicts {
  doc "What a rebinding would share a control with, changing nothing."
  param world World "The world."
  param desc RebindDesc "The rebinding."
  returns Result.List.BindingConflict
}
```

<sub>Module `input` line 410</sub>
