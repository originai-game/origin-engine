<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-contexts"></a>

### `og.input.contexts`

The live contexts as og.input.setContexts set them (empty: every context).

```ts
og.input.contexts(world: World): readonly string[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly string[]`.

Binding symbol `og_input_contexts` · publish builds: kept

```ogt
fn contexts {
  doc "The live contexts as og.input.setContexts set them (empty: every context)."
  param world World "The world."
  returns List.String
}
```

<sub>`api.ogt` line 2359</sub>
