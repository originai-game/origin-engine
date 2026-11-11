<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-reset"></a>

### `og.story.reset`

A new playthrough: the story's variables, seen nodes, conversations and quests are cleared. Lines read stay read (DialogueView.seen), so a second playthrough can skip them.

```ts
og.story.reset(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `void`.

Binding symbol `og_story_reset` · publish builds: kept

```ogt
fn reset {
  doc "A new playthrough: the story's variables, seen nodes, conversations and quests are cleared. Lines read stay read (DialogueView.seen), so a second playthrough can skip them."
  param world World "The world."
}
```

<sub>Module `story` line 381</sub>
