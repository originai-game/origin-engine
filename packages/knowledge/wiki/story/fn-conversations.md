<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-conversations"></a>

### `og.story.conversations`

The conversations that exist (started and not stopped), oldest first.

```ts
og.story.conversations(world: World): readonly number[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly number[]`.

Binding symbol `og_story_conversations` · publish builds: kept

```ogt
fn conversations {
  doc "The conversations that exist (started and not stopped), oldest first."
  param world World "The world."
  returns List.u32
}
```

<sub>Module `story` line 317</sub>
