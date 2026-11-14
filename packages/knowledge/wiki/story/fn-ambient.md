<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-ambient"></a>

### `og.story.ambient`

Let an entity bark a topic on its own, every `everyTicks` ticks (and up to `jitterTicks` more), as og.story.bark would; `everyTicks` 0 stops it. In saves; a despawned entity stops.

Also known as: idle barks, ambient dialogue, chatter, random lines, background banter

```ts
og.story.ambient(world: World, desc?: AmbientDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `AmbientDesc` | yes | Who, which topic and how often. |

**Returns** `Result<void>`: Refused for a topic no loaded dialogue has.

Binding symbol `og_story_ambient` · publish builds: kept

```ogt
fn ambient {
  doc "Let an entity bark a topic on its own, every `everyTicks` ticks (and up to `jitterTicks` more), as og.story.bark would; `everyTicks` 0 stops it. In saves; a despawned entity stops."
  aliases "idle barks, ambient dialogue, chatter, random lines, background banter"
  param world World "The world."
  param desc AmbientDesc "Who, which topic and how often."
  returns Result.Void "Refused for a topic no loaded dialogue has."
}
```

<sub>Module `story` line 352</sub>
