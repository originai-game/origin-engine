<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-track"></a>

### `og.story.track`

Show a quest in the HUD tracker, or stop showing it.

```ts
og.story.track(world: World, id: string, on: boolean): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |
| `on` | `boolean` | no | Tracked. |

**Returns** `boolean`: False for a quest not started.

Binding symbol `og_story_track` · publish builds: kept

```ogt
fn track {
  doc "Show a quest in the HUD tracker, or stop showing it."
  param world World "The world."
  param id String "The quest's id."
  param on bool "Tracked."
  returns bool "False for a quest not started."
}
```

<sub>Module `story` line 418</sub>
