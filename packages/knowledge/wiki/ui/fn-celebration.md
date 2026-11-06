<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-celebration"></a>

### `og.ui.celebration`

A celebration: confetti over the whole screen, a burst from the container it is built in, a level-up flourish centred on the screen, or fireworks. Particles are the UI's own (seeded, so a replay throws the same), drawn over the HUD and under dialogs; under reduced motion only the level-up's text and a soft glow show. Plays reward (levelUp, fireworks) or success (confetti, burst) when it fires.

Parts: Celebration.ring, Celebration.rays, Celebration.title, Celebration.value (level up). States: group play (idle, playing). Accessibility: a level up is a status named by its title and value. Probe ids: <id>, <id>/Title, <id>/Value.

```ts
og.ui.celebration(id: string, desc?: CelebrationDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `CelebrationDesc` | yes | Which one and when. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn celebration {
  doc "A celebration: confetti over the whole screen, a burst from the container it is built in, a level-up flourish centred on the screen, or fireworks. Particles are the UI's own (seeded, so a replay throws the same), drawn over the HUD and under dialogs; under reduced motion only the level-up's text and a soft glow show. Plays reward (levelUp, fireworks) or success (confetti, burst) when it fires."
  doc "Parts: Celebration.ring, Celebration.rays, Celebration.title, Celebration.value (level up). States: group play (idle, playing). Accessibility: a level up is a status named by its title and value. Probe ids: <id>, <id>/Title, <id>/Value."
  host false
  param id String "Its id."
  param desc CelebrationDesc "Which one and when."
  returns Void
}
```

<sub>`api.ogt` line 21516</sub>
