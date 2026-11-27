<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="fn-create"></a>

### `og.ai.create`

Compile a structured behaviour now. Uses file.behaviour's definition shape, except subtree source values are canonical cooked behaviour ID strings rather than author ref tokens. Uses the same native compiler and validation as cook; no text parser.

```ts
og.ai.create(world: World, definition: unknown): Result<BehaviourInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `definition` | `unknown` | no | {name, keys?, routes?, root}; root is {kind:'tree',node} or {kind:'machine',initial,states}. See file:behaviour for the generated definition schema. |

**Returns** `Result<BehaviourInfo>`: Refused for invalid definitions or missing cooked subtrees.

Binding symbol `og_ai_create` · publish builds: kept

```ogt
fn create {
  doc "Compile a structured behaviour now. Uses file.behaviour's definition shape, except subtree source values are canonical cooked behaviour ID strings rather than author ref tokens. Uses the same native compiler and validation as cook; no text parser."
  param world World "The world."
  param definition Json "{name, keys?, routes?, root}; root is {kind:'tree',node} or {kind:'machine',initial,states}. See file:behaviour for the generated definition schema."
  returns Result.BehaviourInfo "Refused for invalid definitions or missing cooked subtrees."
}
```

<sub>Module `ai` line 201</sub>
