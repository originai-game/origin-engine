<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-completions"></a>

### `og.load.completions`

The world's journal of uncooked loads put in place, oldest first: what a replay hands og.load.replayCompletions to put each in place at the same tick. og replay --record keeps it in the trace.

```ts
og.load.completions(world: World): readonly LoadCompletion[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly LoadCompletion[]`.

Binding symbol `og_load_completions` · publish builds: kept

```ogt
fn completions {
  doc "The world's journal of uncooked loads put in place, oldest first: what a replay hands og.load.replayCompletions to put each in place at the same tick. og replay --record keeps it in the trace."
  param world World "The world."
  returns List.LoadCompletion
}
```

<sub>Module `load` line 607</sub>
