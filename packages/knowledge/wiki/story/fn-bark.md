<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-bark"></a>

### `og.story.bark`

Ask an entity to bark a line of a topic: at the og.story system's next run, of that topic's barks whose `when` holds (it may read the world: `self.Health.hp`, `Game.alarm`) and whose cooldowns are over, the highest priority is said, ties drawn by weight on the story's stream; og.BarkSaid follows. Nothing is said when none qualifies, or the entity is saying a bark of equal or higher priority.

```ts
og.story.bark(world: World, desc?: BarkDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `BarkDesc` | yes | Who and which topic. |

**Returns** `Result<void>`: Refused for a topic no loaded dialogue has.

Binding symbol `og_story_bark` · publish builds: kept

```ogt
fn bark {
  doc "Ask an entity to bark a line of a topic: at the og.story system's next run, of that topic's barks whose `when` holds (it may read the world: `self.Health.hp`, `Game.alarm`) and whose cooldowns are over, the highest priority is said, ties drawn by weight on the story's stream; og.BarkSaid follows. Nothing is said when none qualifies, or the entity is saying a bark of equal or higher priority."
  param world World "The world."
  param desc BarkDesc "Who and which topic."
  returns Result.Void "Refused for a topic no loaded dialogue has."
}
```

<sub>`api.ogt` line 14779</sub>
