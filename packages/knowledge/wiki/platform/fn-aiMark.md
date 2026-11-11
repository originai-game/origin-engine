<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-aiMark"></a>

### `og.platform.aiMark`

Disclose AI-generated content the game shows.

```ts
og.platform.aiMark(world: World, desc?: AiMarkDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `AiMarkDesc` | yes | What was generated. |

**Returns** `Result<number>`: Refused for an empty kind or label. The answer's data is `{ marked: true }`.

Binding symbol `og_platform_ai_mark` · publish builds: kept

```ogt
fn aiMark {
  doc "Disclose AI-generated content the game shows."
  param world World "The world."
  param desc AiMarkDesc "What was generated."
  returns Result.u32 "Refused for an empty kind or label. The answer's data is `{ marked: true }`."
}
```

<sub>Module `platform` line 181</sub>
