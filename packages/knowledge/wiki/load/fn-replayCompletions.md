<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-replayCompletions"></a>

### `og.load.replayCompletions`

Finish this world's host-timed loads at the ticks a recorded run did (its og.load.completions), not when this host finishes them: a load is held back until its tick, and at its tick the engine waits for it; a bundle the run opened at the call opens at the call. Call it before the first tick; og replay --compare does. A load the journal names as failed fails at its tick whatever this host made of it; one it never finished never finishes.

```ts
og.load.replayCompletions(world: World, completions: readonly LoadCompletion[]): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `completions` | `readonly LoadCompletion[]` | no | A recorded journal. |

**Returns** `Result<void>`: Refused once a tick has run.

Binding symbol `og_load_replay_completions` · publish builds: kept

```ogt
fn replayCompletions {
  doc "Finish this world's host-timed loads at the ticks a recorded run did (its og.load.completions), not when this host finishes them: a load is held back until its tick, and at its tick the engine waits for it; a bundle the run opened at the call opens at the call. Call it before the first tick; og replay --compare does. A load the journal names as failed fails at its tick whatever this host made of it; one it never finished never finishes."
  param world World "The world."
  param completions List.LoadCompletion "A recorded journal."
  returns Result.Void "Refused once a tick has run."
}
```

<sub>Module `load` line 714</sub>
