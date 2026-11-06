<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="record-PrefabLink"></a>

### `og.PrefabLink` (component)

Where an entity came from when it was placed from a prefab: a scene's `prefab` line (flattened by the cook) or world.instantiate of og.load.prefab's asset. The engine writes it as the entity spawns and never changes it; it is world state like any component (snapshots, restores, the hash). A world that never placed a prefab does not register it, so its saves and hash are as before.

Token `og.PrefabLink`, registered under the schema name `og.PrefabLink`; its value type is `PrefabLink`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `prefab` | `Asset` | `0` | — | — | read only | The prefab placed (a prefab nested in another links to the outermost one placed). Loaded as og.load.prefab loads it. |
| `node` | `number` | `0` | — | — | read only | The entity's node: its index in the prefab's own entity table, depth first with parents first (0 is the prefab's root). |
| `root` | `Entity` | `0` | — | — | read only | The instance's root entity (the root's own, for the root). |

```ogt
component PrefabLink {
  doc "Where an entity came from when it was placed from a prefab: a scene's `prefab` line (flattened by the cook) or world.instantiate of og.load.prefab's asset. The engine writes it as the entity spawns and never changes it; it is world state like any component (snapshots, restores, the hash). A world that never placed a prefab does not register it, so its saves and hash are as before."
  field prefab Asset "The prefab placed (a prefab nested in another links to the outermost one placed). Loaded as og.load.prefab loads it." {
    readonly true
  }
  field node u32 "The entity's node: its index in the prefab's own entity table, depth first with parents first (0 is the prefab's root)." {
    readonly true
  }
  field root Entity "The instance's root entity (the root's own, for the root)." {
    readonly true
  }
}
```

<sub>Module `load` line 216</sub>
