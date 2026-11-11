<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="fn-parse"></a>

### `og.ai.parse`

A behaviour from text, now (tests and generated behaviours; assets go through og cook). Its `use` subtrees are read from cooked assets.

```ts
og.ai.parse(world: World, text: string): Result<BehaviourInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `text` | `string` | no | The behaviour's `.ogt` text. |

**Returns** `Result<BehaviourInfo>`: Refused with the first error's line and the fix.

Binding symbol `og_ai_parse` · publish builds: kept

```ogt
fn parse {
  doc "A behaviour from text, now (tests and generated behaviours; assets go through og cook). Its `use` subtrees are read from cooked assets."
  param world World "The world."
  param text String "The behaviour's `.ogt` text."
  returns Result.BehaviourInfo "Refused with the first error's line and the fix."
}
```

<sub>Module `ai` line 201</sub>
