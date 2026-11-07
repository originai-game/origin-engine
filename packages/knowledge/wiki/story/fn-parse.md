<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-parse"></a>

### `og.story.parse`

A dialogue or quest from text, now (tests and generated content; assets go through og cook). Registers og.Story as load does.

```ts
og.story.parse(world: World, text: string): Result<StoryAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `text` | `string` | no | Yarn text with its profile and name tags, or a `type quest` .ogt. |

**Returns** `Result<StoryAsset>`: Refused with the first error's line and the fix.

Binding symbol `og_story_parse` · publish builds: kept

```ogt
fn parse {
  doc "A dialogue or quest from text, now (tests and generated content; assets go through og cook). Registers og.Story as load does."
  param world World "The world."
  param text String "Yarn text with its profile and name tags, or a `type quest` .ogt."
  returns Result.StoryAsset "Refused with the first error's line and the fix."
}
```

<sub>Module `story` line 287</sub>
