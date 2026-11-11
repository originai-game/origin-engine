<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-CollisionEnded"></a>

### `og.CollisionEnded` (event)

Two solid colliders stopped touching, or one of them went away.

Token `og.CollisionEnded`, registered under the schema name `og.CollisionEnded`; its value type is `CollisionEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `a` | `Entity` | `0` | — | — | — | The entity with the lower id. |
| `b` | `Entity` | `0` | — | — | — | The entity with the higher id. |

```ogt
event CollisionEnded {
  doc "Two solid colliders stopped touching, or one of them went away."
  field a Entity "The entity with the lower id."
  field b Entity "The entity with the higher id."
}
```

<sub>Module `physics` line 228</sub>
