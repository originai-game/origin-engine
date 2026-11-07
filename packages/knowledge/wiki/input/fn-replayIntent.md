<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-replayIntent"></a>

### `og.input.replayIntent`

Deliver recorded input between ticks without UI state. Refuse an unknown event, layout hash mismatch, invalid payload or wrong tick as an error value without changing the world.

```ts
og.input.replayIntent(world: World, intent: RecordedIntent): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The game world. |
| `intent` | `RecordedIntent` | no | The recorded input. |

**Returns** `Result<void>`.

Binding symbol `og_input_replay_intent` · publish builds: kept

```ogt
fn replayIntent {
  doc "Deliver recorded input between ticks without UI state. Refuse an unknown event, layout hash mismatch, invalid payload or wrong tick as an error value without changing the world."
  param world World "The game world."
  param intent RecordedIntent "The recorded input."
  returns Result.Void
}
```

<sub>Module `input` line 34</sub>
