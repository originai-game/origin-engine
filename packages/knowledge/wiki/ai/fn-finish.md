<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="fn-finish"></a>

### `og.ai.finish`

End an agent's running game actions (og.AiActionStarted): each answers success or failure at the agent's next think and sends og.AiActionStopped. With `action`, only the running ones of that name. World state, in saves.

```ts
og.ai.finish(world: World, entity: Entity, success: boolean, action?: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The agent. |
| `success` | `boolean` | no | Whether it worked: the node succeeds, or fails. |
| `action` | `string` | yes | The action's name; omitted: every game action it runs. |

**Returns** `boolean`: False when it runs no such action.

Binding symbol `og_ai_finish` · publish builds: kept

```ogt
fn finish {
  doc "End an agent's running game actions (og.AiActionStarted): each answers success or failure at the agent's next think and sends og.AiActionStopped. With `action`, only the running ones of that name. World state, in saves."
  param world World "The world."
  param entity Entity "The agent."
  param success bool "Whether it worked: the node succeeds, or fails."
  param action String "The action's name; omitted: every game action it runs." {
    optional true
  }
  returns bool "False when it runs no such action."
}
```

<sub>`api.ogt` line 14430</sub>
