<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-set"></a>

### `og.story.set`

Set a story variable; quests whose conditions now hold advance.

```ts
og.story.set(world: World, name: string, value: unknown): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The variable. |
| `value` | `unknown` | no | A number, a boolean or a string. |

**Returns** `Result<void>`: Refused for another kind of value or a world without a loaded story.

Binding symbol `og_story_set` · publish builds: kept

```ogt
fn set {
  doc "Set a story variable; quests whose conditions now hold advance."
  param world World "The world."
  param name String "The variable."
  param value Json "A number, a boolean or a string."
  returns Result.Void "Refused for another kind of value or a world without a loaded story."
}
```

<sub>`api.ogt` line 14808</sub>
