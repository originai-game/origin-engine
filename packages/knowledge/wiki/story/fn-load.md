<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.story.load`

A cooked dialogue or quest (an `.ogt` of `type dialogue` or `type quest`, validated by og cook). Loading again replaces it, so a live edit takes effect at the conversation's next step. The first load registers og.Story and the story's events, so call it in setup: inside a system it is refused until one load has run outside one.

```ts
og.story.load(world: World, path: string): Result<StoryAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The asset, such as `story/smith.ogt`. |

**Returns** `Result<StoryAsset>`: Refused for a path that is not a cooked dialogue or quest.

Binding symbol `og_story_load` · publish builds: kept

```ogt
fn load {
  doc "A cooked dialogue or quest (an `.ogt` of `type dialogue` or `type quest`, validated by og cook). Loading again replaces it, so a live edit takes effect at the conversation's next step. The first load registers og.Story and the story's events, so call it in setup: inside a system it is refused until one load has run outside one."
  param world World "The world."
  param path String "The asset, such as `story/smith.ogt`."
  returns Result.StoryAsset "Refused for a path that is not a cooked dialogue or quest."
}
```

<sub>`api.ogt` line 14647</sub>
