<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.story.load`

A cooked `.yarn` dialogue or `type quest` asset. Loading again replaces it; stable ids preserve the conversation at its next step. The first load registers og.Story and its events, so call it in setup: inside a system it is refused until one load has run outside one.

```ts
og.story.load(world: World, path: string): Result<StoryAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The asset, such as `story/smith.yarn`. |

**Returns** `Result<StoryAsset>`: Refused for a path that is not a cooked dialogue or quest.

Binding symbol `og_story_load` · publish builds: kept

```ogt
fn load {
  doc "A cooked `.yarn` dialogue or `type quest` asset. Loading again replaces it; stable ids preserve the conversation at its next step. The first load registers og.Story and its events, so call it in setup: inside a system it is refused until one load has run outside one."
  param world World "The world."
  param path String "The asset, such as `story/smith.yarn`."
  returns Result.StoryAsset "Refused for a path that is not a cooked dialogue or quest."
}
```

<sub>Module `story` line 259</sub>
