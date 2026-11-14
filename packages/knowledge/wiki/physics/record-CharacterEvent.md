<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-CharacterEvent"></a>

### `og.CharacterEvent` (event)

Something a CharacterController or PlatformerController did this step: the hooks for dust, squash, sounds and camera kicks. Read the next tick: the controllers move in the physics step, after the game's systems, so a jump pressed in tick T moves the character in T and its CHARACTER_JUMPED is read in T + 1.

Token `og.CharacterEvent`, registered under the schema name `og.CharacterEvent`; its value type is `CharacterEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The character. |
| `kind` | `number` | `0` | 0 … 6 | — | — | CHARACTER_JUMPED 1, LANDED 2, WALL_JUMPED 3, AIR_JUMPED 4, LEFT_GROUND 5, BUMPED 6. |
| `speed` | `number` | `0` | — | `mps` | — | The vertical speed at the moment (a landing's impact speed is negative). |

```ogt
event CharacterEvent {
  doc "Something a CharacterController or PlatformerController did this step: the hooks for dust, squash, sounds and camera kicks. Read the next tick: the controllers move in the physics step, after the game's systems, so a jump pressed in tick T moves the character in T and its CHARACTER_JUMPED is read in T + 1."
  field entity Entity "The character."
  field kind u32 "CHARACTER_JUMPED 1, LANDED 2, WALL_JUMPED 3, AIR_JUMPED 4, LEFT_GROUND 5, BUMPED 6." {
    min 0
    max 6
  }
  field speed f32 "The vertical speed at the moment (a landing's impact speed is negative)." {
    unit mps
  }
}
```

<sub>Module `physics` line 745</sub>
