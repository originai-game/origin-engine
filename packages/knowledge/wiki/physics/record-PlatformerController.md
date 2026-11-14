<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-PlatformerController"></a>

### `og.PlatformerController` (component)

A 2D platformer character (the 2D profile): run, jump and fall with designer units (jump height and time to apex, not gravity), coyote time, jump buffering, variable jump height, apex hang, air jumps, wall slide and wall jump, one-way platforms (`oneWayMask`; hold `down` to drop through), moving platforms and corner correction. What blocks it is every solid collider its og.Collider's `layers` and `mask` interact with, as for bodies. It needs an og.Transform and an og.Collider (a box or capsule); it becomes a kinematic body that pushes dynamic bodies (`pushBodies`, `pushStrength`).

See `physics §6`.

Token `og.PlatformerController`, registered under the schema name `og.PlatformerController`; its value type is `PlatformerController`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `run` | `number` | `0` | -1 … 1 | — | — | The run input, -1 (left) to 1 (right). Set it every tick. |
| `jump` | `boolean` | `false` | — | — | — | Held to jump: a press is buffered for jumpBuffer; releasing while rising cuts the jump. |
| `down` | `boolean` | `false` | — | — | — | Held to drop through a one-way platform. |
| `addVelocity` | `Vector2` | `{ x: 0, y: 0 }` | — | `mps` | — | A velocity change to take on the next step (a knock-back, a spring, a recoil): added to `velocity`, then set back to zero. One with an upward y leaves the ground; the run's acceleration and gravity act on the result from then on. See `physics §6`. |
| `maxSpeed` | `number` | `8` | ≥ 0 | `mps` | tunable | The top running speed. |
| `acceleration` | `number` | `70` | ≥ 0 | `mps2` | tunable | How fast it reaches maxSpeed on the ground. |
| `deceleration` | `number` | `80` | ≥ 0 | `mps2` | tunable | How fast it stops on the ground without input. |
| `airAcceleration` | `number` | `45` | ≥ 0 | `mps2` | tunable | Acceleration in the air. |
| `airDeceleration` | `number` | `25` | ≥ 0 | `mps2` | tunable | Deceleration in the air without input. |
| `jumpHeight` | `number` | `3.2` | ≥ 0 | `m` | tunable | How high a full jump goes. |
| `timeToApex` | `number` | `0.4` | ≥ 0.05 | `s` | tunable | How long a full jump takes to reach its height (sets gravity and the jump speed). |
| `fallGravity` | `number` | `1.8` | ≥ 0 | — | tunable | Gravity multiplier while falling (snappier landings above 1). |
| `jumpCut` | `number` | `2.5` | ≥ 1 | — | tunable | Gravity multiplier while rising with jump released (variable jump height; 1 disables it). |
| `apexGravity` | `number` | `0.6` | 0 … 1 | — | tunable | Gravity multiplier near the apex while jump is held (hang time; 1 disables it). |
| `apexThreshold` | `number` | `1.5` | ≥ 0 | `mps` | tunable | The vertical speed below which the apex multiplier applies. |
| `maxFallSpeed` | `number` | `20` | ≥ 0 | `mps` | tunable | The fastest fall. |
| `coyoteTime` | `number` | `0.1` | ≥ 0 | `s` | tunable | After running off an edge, a jump still starts within this time. |
| `jumpBuffer` | `number` | `0.12` | ≥ 0 | `s` | tunable | A jump pressed this long before landing starts on landing. |
| `airJumps` | `number` | `0` | 0 … 8 | — | — | Extra jumps in the air (1: a double jump). |
| `wallJumps` | `boolean` | `true` | — | — | — | Slide down walls it pushes against (and jump off them). |
| `wallSlideSpeed` | `number` | `3` | ≥ 0 | `mps` | tunable | The fastest slide down a wall. |
| `wallJump` | `Vector2` | `{ x: 8, y: 12 }` | — | `mps` | tunable | A wall jump's velocity: x away from the wall, y up. |
| `wallJumpLock` | `number` | `0.14` | ≥ 0 | `s` | tunable | After a wall jump, the run input is ignored for this long (so it leaves the wall). |
| `oneWayMask` | `number` | `0` | — | — | — | The collision layers that are one-way platforms (solid from above only), as bits. |
| `cornerCorrection` | `number` | `0.15` | ≥ 0 | `m` | tunable | How far a rising jump that clips a ceiling corner is nudged sideways past it (0: never). |
| `maxSlope` | `number` | `50` | 0 … 89 | `deg` | tunable | The steepest slope it runs up. |
| `snapDistance` | `number` | `0.1` | ≥ 0 | `m` | tunable | How far below it the ground still holds it (slopes, small drops). |
| `skin` | `number` | `0.01` | ≥ 0.001 | `m` | — | The gap kept between it and every surface. |
| `pushBodies` | `boolean` | `true` | — | — | — | Push dynamic bodies it runs into, with at most `pushStrength`. |
| `pushStrength` | `number` | `1000` | ≥ 0 | `newton` | tunable | The most force it pushes with: each tick the bodies it runs into share an impulse of at most this times the tick's length (mass times the speed it asks of them, capped). 0 leaves every body where it is, as `pushBodies` false does. See `physics §6`. |
| `grounded` | `boolean` | `false` | — | — | read only | Written: standing on ground after the step. |
| `onWall` | `number` | `0` | -1 … 1 | — | read only | Written: -1 sliding on a wall to its left, 1 to its right, 0 none. |
| `facing` | `number` | `1` | -1 … 1 | — | read only | Written: -1 facing left, 1 right (the last run direction). |
| `velocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | read only | Written: the velocity after the step (the platform's carry excluded). |
| `jumping` | `boolean` | `false` | — | — | read only | Written: rising from a jump (until the apex or a release cut). |
| `airJumpsLeft` | `number` | `0` | — | — | read only | Written: air jumps left before landing. |
| `coyote` | `number` | `0` | — | — | read only | Written: seconds of coyote time left. |
| `buffer` | `number` | `0` | — | — | read only | Written: seconds a buffered jump press stays valid. |
| `wallLock` | `number` | `0` | — | — | read only | Written: seconds the run input stays locked after a wall jump. |
| `dropTime` | `number` | `0` | — | — | read only | Written: seconds one-way platforms stay passable after dropping through. |
| `jumpWasHeld` | `boolean` | `false` | — | — | read only | Written: `jump` as the last step saw it. |
| `ground` | `Entity` | `0` | — | — | read only | Written: the entity it stands on (0 in the air); a moving one carries it. |

```ogt
component PlatformerController {
  doc "A 2D platformer character (the 2D profile): run, jump and fall with designer units (jump height and time to apex, not gravity), coyote time, jump buffering, variable jump height, apex hang, air jumps, wall slide and wall jump, one-way platforms (`oneWayMask`; hold `down` to drop through), moving platforms and corner correction. What blocks it is every solid collider its og.Collider's `layers` and `mask` interact with, as for bodies. It needs an og.Transform and an og.Collider (a box or capsule); it becomes a kinematic body that pushes dynamic bodies (`pushBodies`, `pushStrength`)."
  guide "physics §6"
  field run f32 "The run input, -1 (left) to 1 (right). Set it every tick." {
    min -1
    max 1
  }
  field jump bool "Held to jump: a press is buffered for jumpBuffer; releasing while rising cuts the jump."
  field down bool "Held to drop through a one-way platform."
  field addVelocity Vector2 "A velocity change to take on the next step (a knock-back, a spring, a recoil): added to `velocity`, then set back to zero. One with an upward y leaves the ground; the run's acceleration and gravity act on the result from then on." {
    guide "physics §6"
    unit mps
  }
  field maxSpeed f32 "The top running speed." {
    default 8
    min 0
    unit mps
    tunable true
  }
  field acceleration f32 "How fast it reaches maxSpeed on the ground." {
    default 70
    min 0
    unit mps2
    tunable true
  }
  field deceleration f32 "How fast it stops on the ground without input." {
    default 80
    min 0
    unit mps2
    tunable true
  }
  field airAcceleration f32 "Acceleration in the air." {
    default 45
    min 0
    unit mps2
    tunable true
  }
  field airDeceleration f32 "Deceleration in the air without input." {
    default 25
    min 0
    unit mps2
    tunable true
  }
  field jumpHeight f32 "How high a full jump goes." {
    default 3.2
    min 0
    unit m
    tunable true
  }
  field timeToApex f32 "How long a full jump takes to reach its height (sets gravity and the jump speed)." {
    default 0.4
    min 0.05
    unit s
    tunable true
  }
  field fallGravity f32 "Gravity multiplier while falling (snappier landings above 1)." {
    default 1.8
    min 0
    tunable true
  }
  field jumpCut f32 "Gravity multiplier while rising with jump released (variable jump height; 1 disables it)." {
    default 2.5
    min 1
    tunable true
  }
  field apexGravity f32 "Gravity multiplier near the apex while jump is held (hang time; 1 disables it)." {
    default 0.6
    min 0
    max 1
    tunable true
  }
  field apexThreshold f32 "The vertical speed below which the apex multiplier applies." {
    default 1.5
    min 0
    unit mps
    tunable true
  }
  field maxFallSpeed f32 "The fastest fall." {
    default 20
    min 0
    unit mps
    tunable true
  }
  field coyoteTime f32 "After running off an edge, a jump still starts within this time." {
    default 0.1
    min 0
    unit s
    tunable true
  }
  field jumpBuffer f32 "A jump pressed this long before landing starts on landing." {
    default 0.12
    min 0
    unit s
    tunable true
  }
  field airJumps u32 "Extra jumps in the air (1: a double jump)." {
    min 0
    max 8
  }
  field wallJumps bool "Slide down walls it pushes against (and jump off them)." {
    default true
  }
  field wallSlideSpeed f32 "The fastest slide down a wall." {
    default 3
    min 0
    unit mps
    tunable true
  }
  field wallJump Vector2 "A wall jump's velocity: x away from the wall, y up." {
    default 8 12
    unit mps
    tunable true
  }
  field wallJumpLock f32 "After a wall jump, the run input is ignored for this long (so it leaves the wall)." {
    default 0.14
    min 0
    unit s
    tunable true
  }
  field oneWayMask u32 "The collision layers that are one-way platforms (solid from above only), as bits."
  field cornerCorrection f32 "How far a rising jump that clips a ceiling corner is nudged sideways past it (0: never)." {
    default 0.15
    min 0
    unit m
    tunable true
  }
  field maxSlope f32 "The steepest slope it runs up." {
    default 50
    min 0
    max 89
    unit deg
    tunable true
  }
  field snapDistance f32 "How far below it the ground still holds it (slopes, small drops)." {
    default 0.1
    min 0
    unit m
    tunable true
  }
  field skin f32 "The gap kept between it and every surface." {
    default 0.01
    min 0.001
    unit m
  }
  field pushBodies bool "Push dynamic bodies it runs into, with at most `pushStrength`." {
    default true
  }
  field pushStrength f32 "The most force it pushes with: each tick the bodies it runs into share an impulse of at most this times the tick's length (mass times the speed it asks of them, capped). 0 leaves every body where it is, as `pushBodies` false does." {
    guide "physics §6"
    default 1000
    min 0
    unit newton
    tunable true
  }
  field grounded bool "Written: standing on ground after the step." {
    readonly true
  }
  field onWall i32 "Written: -1 sliding on a wall to its left, 1 to its right, 0 none." {
    min -1
    max 1
    readonly true
  }
  field facing i32 "Written: -1 facing left, 1 right (the last run direction)." {
    default 1
    min -1
    max 1
    readonly true
  }
  field velocity Vector3 "Written: the velocity after the step (the platform's carry excluded)." {
    unit mps
    readonly true
  }
  field jumping bool "Written: rising from a jump (until the apex or a release cut)." {
    readonly true
  }
  field airJumpsLeft u32 "Written: air jumps left before landing." {
    readonly true
  }
  field coyote f32 "Written: seconds of coyote time left." {
    readonly true
  }
  field buffer f32 "Written: seconds a buffered jump press stays valid." {
    readonly true
  }
  field wallLock f32 "Written: seconds the run input stays locked after a wall jump." {
    readonly true
  }
  field dropTime f32 "Written: seconds one-way platforms stay passable after dropping through." {
    readonly true
  }
  field jumpWasHeld bool "Written: `jump` as the last step saw it." {
    readonly true
  }
  field ground Entity "Written: the entity it stands on (0 in the air); a moving one carries it." {
    readonly true
  }
}
```

<sub>Module `physics` line 540</sub>
