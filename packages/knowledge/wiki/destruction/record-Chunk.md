<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="record-Chunk"></a>

### `og.Chunk` (component)

One chunk of an og.Destructible, written by og.destruction when the prop fractures. A loose chunk is a dynamic body (with og.RigidBody and og.Velocity, or a child collider of its island's body); an anchored one a static collider. Drawn with its part of the destructible's model. Despawn it to clear it away.

Token `og.Chunk`, registered under the schema name `og.Chunk`; its value type is `Chunk`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `destructible` | `Entity` | `0` | — | — | read only | The og.Destructible it belongs to. |
| `chunk` | `number` | `0` | — | — | read only | Its index in the fracture asset. |
| `level` | `number` | `0` | — | — | read only | 1 a support chunk, 2 a chunk's child. |
| `body` | `Entity` | `0` | — | — | read only | The entity whose og.RigidBody moves it: its own, its island's body, or 0 while anchored. |
| `looseTick` | `number` | `4294967295` | — | — | read only | The tick it broke loose; 4294967295 while anchored. |

```ogt
component Chunk {
  doc "One chunk of an og.Destructible, written by og.destruction when the prop fractures. A loose chunk is a dynamic body (with og.RigidBody and og.Velocity, or a child collider of its island's body); an anchored one a static collider. Drawn with its part of the destructible's model. Despawn it to clear it away."
  field destructible Entity "The og.Destructible it belongs to." {
    readonly true
  }
  field chunk u32 "Its index in the fracture asset." {
    readonly true
  }
  field level u32 "1 a support chunk, 2 a chunk's child." {
    readonly true
  }
  field body Entity "The entity whose og.RigidBody moves it: its own, its island's body, or 0 while anchored." {
    readonly true
  }
  field looseTick u32 "The tick it broke loose; 4294967295 while anchored." {
    default 4294967295
    readonly true
  }
}
```

<sub>Module `destruction` line 89</sub>
