<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-setSaliency"></a>

### `og.story.setSaliency`

Select this world's Yarn content strategy after loading its story. The strategy and view counts are saved with the story.

```ts
og.story.setSaliency(world: World, strategy: StorySaliency): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `strategy` | `StorySaliency` | no | The selection strategy. |

**Returns** `Result<void>`: Refused before a story is loaded or for an unknown strategy.

Binding symbol `og_story_set_saliency` · publish builds: kept

```ogt
fn setSaliency {
  doc "Select this world's Yarn content strategy after loading its story. The strategy and view counts are saved with the story."
  param world World "The world."
  param strategy StorySaliency "The selection strategy."
  returns Result.Void "Refused before a story is loaded or for an unknown strategy."
}
```

<sub>Module `story` line 15</sub>
