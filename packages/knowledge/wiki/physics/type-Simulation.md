<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-Simulation"></a>

### `Simulation` (enum)

Where a cloth, a garment, a soft body or a 3D fluid is simulated: Cloth.simulation, Garment.simulation, SoftBody.simulation and Fluid3D.simulation.

See `physics §8`.

A string: one of the variants below. The zero value is `"gameplay"`.

| Variant | Doc |
|---|---|
| `"gameplay"` | In the physics step on the CPU, deterministic: its points are world state that snapshots, replays, lockstep and the hash carry, and that og.physics.clothPoints, softBodyPoints and garmentPoints read. The default. |
| `"visual"` | By the renderer on the GPU for looks only, never read back. The tick's points are the rest shape carried rigidly by the entity (a garment's: the skinned pose). Without RenderCapabilities.visualSimulation it draws that shape. |

```ogt
enum Simulation {
  doc "Where a cloth, a garment, a soft body or a 3D fluid is simulated: Cloth.simulation, Garment.simulation, SoftBody.simulation and Fluid3D.simulation."
  guide "physics §8"
  variant gameplay "In the physics step on the CPU, deterministic: its points are world state that snapshots, replays, lockstep and the hash carry, and that og.physics.clothPoints, softBodyPoints and garmentPoints read. The default."
  variant visual "By the renderer on the GPU for looks only, never read back. The tick's points are the rest shape carried rigidly by the entity (a garment's: the skinned pose). Without RenderCapabilities.visualSimulation it draws that shape."
}
```

<sub>Module `physics` line 1107</sub>
