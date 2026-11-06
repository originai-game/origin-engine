<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="fn-setBudget"></a>

### `og.stream.setBudget`

Set the memory budget from the next tick boundary; a lower one evicts the lowest-priority groups until the rest fit.

```ts
og.stream.setBudget(world: World, bytes: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `bytes` | `number` | no | The bytes resident groups may take. |

**Returns** `Result<void>`: Refused for a negative or non-finite size.

Binding symbol `og_stream_set_budget` · publish builds: kept

```ogt
fn setBudget {
  doc "Set the memory budget from the next tick boundary; a lower one evicts the lowest-priority groups until the rest fit."
  param world World "The world."
  param bytes f64 "The bytes resident groups may take."
  returns Result.Void "Refused for a negative or non-finite size."
}
```

<sub>`api.ogt` line 25639</sub>
