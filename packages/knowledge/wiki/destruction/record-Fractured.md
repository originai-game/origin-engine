<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="record-Fractured"></a>

### `og.Fractured` (event)

An island of chunks broke loose from a destructible during the tick (or the destructible shattered): read the tick after, for its sound, its camera shake, its score. Sent in the order the islands broke.

Token `og.Fractured`, registered under the schema name `og.Fractured`; its value type is `Fractured`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Destructible. |
| `body` | `Entity` | `0` | — | — | — | The island's body: the entity with its og.RigidBody. |
| `chunks` | `number` | `0` | — | — | — | Its chunks. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where it broke: the centre of its broken bonds. |
| `volume` | `number` | `0` | — | `m3` | — | Its volume. |
| `mass` | `number` | `0` | — | `kg` | — | Its mass. |
| `cause` | `number` | `0` | — | — | — | What broke it: one of the CAUSE_* codes. |

```ogt
event Fractured {
  doc "An island of chunks broke loose from a destructible during the tick (or the destructible shattered): read the tick after, for its sound, its camera shake, its score. Sent in the order the islands broke."
  field entity Entity "The og.Destructible."
  field body Entity "The island's body: the entity with its og.RigidBody."
  field chunks u32 "Its chunks."
  field point Vector3 "Where it broke: the centre of its broken bonds." {
    unit m
    worldspace world
  }
  field volume f32 "Its volume." {
    unit m3
  }
  field mass f32 "Its mass." {
    unit kg
  }
  field cause u32 "What broke it: one of the CAUSE_* codes."
}
```

<sub>Module `destruction` line 102</sub>
