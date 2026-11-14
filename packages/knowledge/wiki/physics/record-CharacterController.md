<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-CharacterController"></a>

### `og.CharacterController` (component)

A kinematic character (3D, or top-down 2D): the entity's capsule (or any og.Collider) moved by `move` each tick, sliding along walls, climbing slopes up to `maxSlope` and steps up to `stepHeight`, snapped to the ground, carried by the moving platform it stands on and falling by gravity. It keeps `skin` off every surface. What blocks it is every solid collider its og.Collider's `layers` and `mask` interact with, as for bodies. It needs an og.Transform and an og.Collider; the step makes it a kinematic body that pushes dynamic bodies (`pushBodies`, `pushStrength`).

See `physics §6`.

Also known as: player controller, kinematic character, Unity CharacterController, Godot CharacterBody3D

Token `og.CharacterController`, registered under the schema name `og.CharacterController`; its value type is `CharacterController`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `move` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | — | The wanted velocity across the ground (m/s, world space; y is ignored unless gravityScale is 0, or it is in water with og.Buoyancy, where y is its stroke down or up). Set it every tick. |
| `jump` | `boolean` | `false` | — | — | — | Held to jump: a jump starts on the tick it becomes true while grounded (or in coyote time). |
| `jumpSpeed` | `number` | `6` | ≥ 0 | `mps` | tunable | The upward speed of a jump. |
| `gravityScale` | `number` | `1` | 0 … 10 | — | tunable | Multiplies og.Physics.gravity; 0 for top-down movement in any direction. |
| `maxSlope` | `number` | `45` | 0 … 89 | `deg` | tunable | The steepest slope it walks up. |
| `stepHeight` | `number` | `0.3` | ≥ 0 | `m` | tunable | The highest step it climbs without jumping: the ground beyond the step at most this over the ground it stands on (0: none). |
| `snapDistance` | `number` | `0.2` | ≥ 0 | `m` | tunable | How far below it the ground still holds it (stairs down, slopes): 0 lets it leave every edge. |
| `skin` | `number` | `0.02` | ≥ 0.001 | `m` | — | The gap kept between it and every surface. |
| `coyoteTime` | `number` | `0.1` | ≥ 0 | `s` | tunable | After leaving an edge, a jump still starts within this time. |
| `maxFallSpeed` | `number` | `40` | ≥ 0 | `mps` | tunable | The fastest fall. |
| `pushBodies` | `boolean` | `true` | — | — | — | Push dynamic bodies it walks into, with at most `pushStrength`. |
| `pushStrength` | `number` | `1000` | ≥ 0 | `newton` | tunable | The most force it pushes with: each tick the bodies it walks into share an impulse of at most this times the tick's length (mass times the speed it asks of them, capped). 0 leaves every body where it is, as `pushBodies` false does. See `physics §6`. |
| `waterSpeed` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | The share of `move` it makes fully under water, with og.Buoyancy beside it: wading and swimming slow it by how much of it is under. See `physics §8`. |
| `grounded` | `boolean` | `false` | — | — | read only | Written: standing on walkable ground after the step. |
| `sliding` | `boolean` | `false` | — | — | read only | Written: sliding down a slope steeper than maxSlope. |
| `velocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | read only | Written: the velocity it moved at this tick (the platform's carry included). |
| `groundNormal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | read only | Written: the ground's normal (zero in the air). |
| `coyote` | `number` | `0` | — | — | read only | Written: seconds left in which a jump may still start (coyote time). |
| `jumpWasHeld` | `boolean` | `false` | — | — | read only | Written: `jump` as the last step saw it (a jump starts on its rising edge). |
| `ground` | `Entity` | `0` | — | — | read only | Written: the entity it stands on (0 in the air); a moving one carries it. |

```ogt
component CharacterController {
  doc "A kinematic character (3D, or top-down 2D): the entity's capsule (or any og.Collider) moved by `move` each tick, sliding along walls, climbing slopes up to `maxSlope` and steps up to `stepHeight`, snapped to the ground, carried by the moving platform it stands on and falling by gravity. It keeps `skin` off every surface. What blocks it is every solid collider its og.Collider's `layers` and `mask` interact with, as for bodies. It needs an og.Transform and an og.Collider; the step makes it a kinematic body that pushes dynamic bodies (`pushBodies`, `pushStrength`)."
  guide "physics §6"
  aliases "player controller, kinematic character, Unity CharacterController, Godot CharacterBody3D"
  field move Vector3 "The wanted velocity across the ground (m/s, world space; y is ignored unless gravityScale is 0, or it is in water with og.Buoyancy, where y is its stroke down or up). Set it every tick." {
    unit mps
  }
  field jump bool "Held to jump: a jump starts on the tick it becomes true while grounded (or in coyote time)."
  field jumpSpeed f32 "The upward speed of a jump." {
    default 6
    min 0
    unit mps
    tunable true
  }
  field gravityScale f32 "Multiplies og.Physics.gravity; 0 for top-down movement in any direction." {
    default 1
    min 0
    max 10
    tunable true
  }
  field maxSlope f32 "The steepest slope it walks up." {
    default 45
    min 0
    max 89
    unit deg
    tunable true
  }
  field stepHeight f32 "The highest step it climbs without jumping: the ground beyond the step at most this over the ground it stands on (0: none)." {
    default 0.3
    min 0
    unit m
    tunable true
  }
  field snapDistance f32 "How far below it the ground still holds it (stairs down, slopes): 0 lets it leave every edge." {
    default 0.2
    min 0
    unit m
    tunable true
  }
  field skin f32 "The gap kept between it and every surface." {
    default 0.02
    min 0.001
    unit m
  }
  field coyoteTime f32 "After leaving an edge, a jump still starts within this time." {
    default 0.1
    min 0
    unit s
    tunable true
  }
  field maxFallSpeed f32 "The fastest fall." {
    default 40
    min 0
    unit mps
    tunable true
  }
  field pushBodies bool "Push dynamic bodies it walks into, with at most `pushStrength`." {
    default true
  }
  field pushStrength f32 "The most force it pushes with: each tick the bodies it walks into share an impulse of at most this times the tick's length (mass times the speed it asks of them, capped). 0 leaves every body where it is, as `pushBodies` false does." {
    guide "physics §6"
    default 1000
    min 0
    unit newton
    tunable true
  }
  field waterSpeed f32 "The share of `move` it makes fully under water, with og.Buoyancy beside it: wading and swimming slow it by how much of it is under." {
    guide "physics §8"
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
  field grounded bool "Written: standing on walkable ground after the step." {
    readonly true
  }
  field sliding bool "Written: sliding down a slope steeper than maxSlope." {
    readonly true
  }
  field velocity Vector3 "Written: the velocity it moved at this tick (the platform's carry included)." {
    unit mps
    readonly true
  }
  field groundNormal Vector3 "Written: the ground's normal (zero in the air)." {
    readonly true
  }
  field coyote f32 "Written: seconds left in which a jump may still start (coyote time)." {
    readonly true
  }
  field jumpWasHeld bool "Written: `jump` as the last step saw it (a jump starts on its rising edge)." {
    readonly true
  }
  field ground Entity "Written: the entity it stands on (0 in the air); a moving one carries it." {
    readonly true
  }
}
```

<sub>Module `physics` line 432</sub>
