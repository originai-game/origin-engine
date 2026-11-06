<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.ai.load`

A cooked behaviour (an `.ogt` of `type behaviour`, validated by og cook); its `use` subtrees are read from their own cooked assets.

```ts
og.ai.load(world: World, path: string): Result<BehaviourInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The behaviour asset, such as `ai/slime.ogt`. |

**Returns** `Result<BehaviourInfo>`: Refused for a path that is not a cooked behaviour.

Binding symbol `og_ai_load` · publish builds: kept

```ogt
fn load {
  doc "A cooked behaviour (an `.ogt` of `type behaviour`, validated by og cook); its `use` subtrees are read from their own cooked assets."
  param world World "The world."
  param path String "The behaviour asset, such as `ai/slime.ogt`."
  returns Result.BehaviourInfo "Refused for a path that is not a cooked behaviour."
}
```

<sub>`api.ogt` line 14323</sub>
