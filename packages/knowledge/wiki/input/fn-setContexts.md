<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-setContexts"></a>

### `og.input.setContexts`

The contexts whose actions are live, first the one whose bindings win a control another context also binds (`["vehicle", "gameplay"]`: E exits the car, not interacts). Empty (the default) is every context, none winning. Takes effect from the next tick's actions.

```ts
og.input.setContexts(world: World, contexts: readonly string[]): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `contexts` | `readonly string[]` | no | The live contexts, by priority. |

**Returns** `void`.

Binding symbol `og_input_set_contexts` · publish builds: kept

```ogt
fn setContexts {
  doc "The contexts whose actions are live, first the one whose bindings win a control another context also binds (`[\"vehicle\", \"gameplay\"]`: E exits the car, not interacts). Empty (the default) is every context, none winning. Takes effect from the next tick's actions."
  param world World "The world."
  param contexts List.String "The live contexts, by priority."
  returns Void
}
```

<sub>`api.ogt` line 2364</sub>
