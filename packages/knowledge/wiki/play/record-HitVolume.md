<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`play`](../play.md) · [index](../index.md)

<a id="record-HitVolume"></a>

### `og.HitVolume` (component)

What strikes: a shape at its entity's simulation pose (its og.Transform after its parents', a posed bone's frame with og.BoneAttachment; og.motion's drawn pose is never read). While `active` it is swept each tick from its last pose, along the turn and slide between the two, and strikes each og.HurtVolume of another team and owner, and each collider on `colliders`, once per owner's swing (og.HitEvent). Sizes are scaled by the entity's world scale.

Also known as: hitbox, attack box, weapon trace, melee hit detection, sword collider, active frames, Unreal weapon trace, Unity hitbox trigger, Godot Hitbox Area3D

Token `og.HitVolume`, registered under the schema name `og.HitVolume`; its value type is `HitVolume`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `VolumeShape` | `"sphere"` | — | — | — | Its shape. |
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | ≥ 0 | `m` | tunable | A box's full extents. |
| `radius` | `number` | `0.5` | ≥ 0 | `m` | tunable | A sphere's or capsule's radius. |
| `height` | `number` | `1` | ≥ 0 | `m` | tunable | A capsule's distance between its cap centres. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The shape's centre in its entity's frame. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The shape's rotation in its entity's frame; a zero quaternion is no rotation. |
| `active` | `boolean` | `false` | — | — | — | Whether it strikes this tick (an attack's active window). Swept from its last pose only when it was active at the last tick too; its first active tick tests where it is. |
| `team` | `number` | `0` | — | — | — | Its team: it never strikes an og.HurtVolume of the same team (as og.AiAgent.team). |
| `owner` | `Entity` | `0` | — | — | — | Whose swing it is: a target is struck once per owner and swing, whichever of the owner's volumes meets it, and never the owner's own hurt volumes; 0 is its own entity. |
| `swing` | `number` | `0` | — | — | — | Which swing it is part of: change it for the next swing, and each target can be struck again. A swing is forgotten once none of its owner's volumes holds it. |
| `colliders` | `number` | `0` | — | — | — | The collider layers (og.Collider.layers bits) it strikes besides hurt volumes: walls, props; 0 strikes no collider. |
| `lastPosition` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only, worldspace `world` | Written: where its shape's centre was at the last tick. |
| `lastQuaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | read only | Written: its shape's rotation at the last tick. |
| `wasActive` | `boolean` | `false` | — | — | read only | Written: whether it was active at the last tick (the sweep starts from the last pose). |

```ogt
component HitVolume {
  aliases "hitbox, attack box, weapon trace, melee hit detection, sword collider, active frames, Unreal weapon trace, Unity hitbox trigger, Godot Hitbox Area3D"
  doc "What strikes: a shape at its entity's simulation pose (its og.Transform after its parents', a posed bone's frame with og.BoneAttachment; og.motion's drawn pose is never read). While `active` it is swept each tick from its last pose, along the turn and slide between the two, and strikes each og.HurtVolume of another team and owner, and each collider on `colliders`, once per owner's swing (og.HitEvent). Sizes are scaled by the entity's world scale."
  field shape VolumeShape "Its shape."
  field size Vector3 "A box's full extents." {
    default 1 1 1
    min 0
    unit m
    tunable true
  }
  field radius f32 "A sphere's or capsule's radius." {
    default 0.5
    min 0
    unit m
    tunable true
  }
  field height f32 "A capsule's distance between its cap centres." {
    default 1
    min 0
    unit m
    tunable true
  }
  field offset Vector3 "The shape's centre in its entity's frame." {
    unit m
  }
  field quaternion Quaternion "The shape's rotation in its entity's frame; a zero quaternion is no rotation."
  field active bool "Whether it strikes this tick (an attack's active window). Swept from its last pose only when it was active at the last tick too; its first active tick tests where it is."
  field team u32 "Its team: it never strikes an og.HurtVolume of the same team (as og.AiAgent.team)."
  field owner Entity "Whose swing it is: a target is struck once per owner and swing, whichever of the owner's volumes meets it, and never the owner's own hurt volumes; 0 is its own entity."
  field swing u32 "Which swing it is part of: change it for the next swing, and each target can be struck again. A swing is forgotten once none of its owner's volumes holds it."
  field colliders u32 "The collider layers (og.Collider.layers bits) it strikes besides hurt volumes: walls, props; 0 strikes no collider."
  field lastPosition Vector3 "Written: where its shape's centre was at the last tick." {
    readonly true
    unit m
    worldspace world
  }
  field lastQuaternion Quaternion "Written: its shape's rotation at the last tick." {
    readonly true
  }
  field wasActive bool "Written: whether it was active at the last tick (the sweep starts from the last pose)." {
    readonly true
  }
}
```

<sub>Module `play` line 18</sub>
