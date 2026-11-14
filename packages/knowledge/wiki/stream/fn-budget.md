<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="fn-budget"></a>

### `og.stream.budget`

The memory budget, what the groups take now, the peak and every group.

```ts
og.stream.budget(world: World): Result<StreamBudget>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Result<StreamBudget>`: The budget and the groups; refused only for a world that does not exist.

Binding symbol `og_stream_budget` · publish builds: kept

```ogt
fn budget {
  doc "The memory budget, what the groups take now, the peak and every group."
  param world World "The world."
  returns Result.StreamBudget "The budget and the groups; refused only for a world that does not exist."
}
```

<sub>Module `stream` line 140</sub>
