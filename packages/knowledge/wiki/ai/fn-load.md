<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.ai.load`

Load native behaviour fields compiled from a checked file.behaviour declaration. Subtrees are already inlined; loading never reads author files.

Also known as: load a behaviour tree, behaviour tree asset, state machine asset

```ts
og.ai.load(world: World, path: string): Result<BehaviourInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | Canonical behaviour content ID, such as 'base/ai/slime'. |

**Returns** `Result<BehaviourInfo>`: Refused for a path that is not a cooked behaviour.

Binding symbol `og_ai_load` · publish builds: kept

```ogt
fn load {
  aliases "load a behaviour tree, behaviour tree asset, state machine asset"
  doc "Load native behaviour fields compiled from a checked file.behaviour declaration. Subtrees are already inlined; loading never reads author files."
  param world World "The world."
  param path String "Canonical behaviour content ID, such as 'base/ai/slime'."
  returns Result.BehaviourInfo "Refused for a path that is not a cooked behaviour."
}
```

<sub>Module `ai` line 193</sub>
