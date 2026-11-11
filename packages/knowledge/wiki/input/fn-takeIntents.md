<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-takeIntents"></a>

### `og.input.takeIntents`

Drain accepted intents in delivery order for recording. Call regularly; no presentation state is included.

```ts
og.input.takeIntents(world: World): readonly RecordedIntent[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The game world. |

**Returns** `readonly RecordedIntent[]`.

Binding symbol `og_input_take_intents` · publish builds: kept

```ogt
fn takeIntents {
  doc "Drain accepted intents in delivery order for recording. Call regularly; no presentation state is included."
  param world World "The game world."
  returns List.RecordedIntent
}
```

<sub>Module `input` line 28</sub>
