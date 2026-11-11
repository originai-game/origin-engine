<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Joint"></a>

### `og.Joint` (component)

A joint between two bodies (or a body and a point of the world). The component may sit on one of the bodies (`body` 0) or on an entity of its own, so one body can have several joints. Anchors are in each body's frame (rotated and moved with it, not scaled); without `other` the second anchor is a world point. A joint whose bodies are not both `og.RigidBody` entities does nothing until they are.

See `physics §1`.

Token `og.Joint`, registered under the schema name `og.Joint`; its value type is `Joint`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `number` | `0` | 0 … 5 | — | label “Kind” | JOINT_FIXED 0, JOINT_REVOLUTE 1 (hinge), JOINT_PRISMATIC 2 (slider), JOINT_SPRING 3, JOINT_ROPE 4, JOINT_BALL 5. |
| `body` | `Entity` | `0` | — | — | — | The first body; 0 is this entity. |
| `other` | `Entity` | `0` | — | — | — | The second body; 0 joins the first one to the world at `otherAnchor`. |
| `anchor` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | Where the joint sits on the first body, in its frame. |
| `otherAnchor` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `hook` | Where it sits on the second body, in its frame (a world point without `other`). |
| `axis` | `Vector3` | `{ x: 1, y: 0, z: 0 }` | — | — | — | The hinge or slide axis in the first body's frame (a hinge in 2D turns about z). |
| `limits` | `boolean` | `false` | — | — | — | Whether `min`..`max` bound the hinge angle (rad) or the slide (m). |
| `min` | `number` | `0` | — | — | — | The lower limit: radians for a hinge, metres for a slider. |
| `max` | `number` | `0` | — | — | — | The upper limit. |
| `motor` | `number` | `0` | 0 … 2 | — | — | MOTOR_NONE 0, MOTOR_VELOCITY 1 (drive at motorTarget speed), MOTOR_POSITION 2 (spring to motorTarget). |
| `motorTarget` | `number` | `0` | — | — | tunable | The motor's target speed (rad/s, m/s) or angle or position (rad, m). |
| `motorStiffness` | `number` | `50` | ≥ 0 | — | tunable | A position motor's stiffness; a velocity motor's strength. |
| `motorDamping` | `number` | `5` | ≥ 0 | — | tunable | A position motor's damping. |
| `motorMaxForce` | `number` | `0` | ≥ 0 | — | tunable | The most force (or torque) the motor applies; 0 is unbounded. |
| `restLength` | `number` | `1` | ≥ 0 | `m` | tunable | A spring's rest length. |
| `stiffness` | `number` | `40` | ≥ 0 | — | tunable | A spring's stiffness (N/m). |
| `damping` | `number` | `2` | ≥ 0 | — | tunable | A spring's damping (N·s/m). |
| `length` | `number` | `2` | ≥ 0 | `m` | tunable | A rope's longest distance between the anchors. |
| `collide` | `boolean` | `false` | — | — | — | Whether the two bodies still collide with each other. |
| `value` | `number` | `0` | — | — | read only | Written: the hinge angle (rad) or the slide (m) after the step; the anchors' distance for springs and ropes. |
| `breakForce` | `number` | `0` | ≥ 0 | `newton` | tunable | The force the joint holds before it breaks (JointBroken, `broken`): the solver's load on it through the step, its weight, gravity and what rests or presses on it; a lone impulse (applyImpulse, applyForce) is absorbed before it counts. 0 never breaks. |
| `breakTorque` | `number` | `0` | ≥ 0 | `newtonMetre` | tunable | The torque about its anchor it holds before it breaks, as breakForce (a shelf bracket, a cantilevered beam); 0 never breaks. |
| `broken` | `boolean` | `false` | — | — | — | Written: true once the joint broke; a broken joint holds nothing. Write false to mend it, rebuilt where its bodies are now. |

```ogt
component Joint {
  doc "A joint between two bodies (or a body and a point of the world). The component may sit on one of the bodies (`body` 0) or on an entity of its own, so one body can have several joints. Anchors are in each body's frame (rotated and moved with it, not scaled); without `other` the second anchor is a world point. A joint whose bodies are not both `og.RigidBody` entities does nothing until they are."
  guide "physics §1"
  field kind u32 "JOINT_FIXED 0, JOINT_REVOLUTE 1 (hinge), JOINT_PRISMATIC 2 (slider), JOINT_SPRING 3, JOINT_ROPE 4, JOINT_BALL 5." {
    min 0
    max 5
    label "Kind"
  }
  field body Entity "The first body; 0 is this entity."
  field other Entity "The second body; 0 joins the first one to the world at `otherAnchor`."
  field anchor Vector3 "Where the joint sits on the first body, in its frame." {
    worldspace none
    unit m
  }
  field otherAnchor Vector3 "Where it sits on the second body, in its frame (a world point without `other`)." {
    worldspace hook
    unit m
  }
  field axis Vector3 "The hinge or slide axis in the first body's frame (a hinge in 2D turns about z)." {
    default 1 0 0
  }
  field limits bool "Whether `min`..`max` bound the hinge angle (rad) or the slide (m)."
  field min f32 "The lower limit: radians for a hinge, metres for a slider."
  field max f32 "The upper limit."
  field motor u32 "MOTOR_NONE 0, MOTOR_VELOCITY 1 (drive at motorTarget speed), MOTOR_POSITION 2 (spring to motorTarget)." {
    min 0
    max 2
  }
  field motorTarget f32 "The motor's target speed (rad/s, m/s) or angle or position (rad, m)." {
    tunable true
  }
  field motorStiffness f32 "A position motor's stiffness; a velocity motor's strength." {
    default 50
    min 0
    tunable true
  }
  field motorDamping f32 "A position motor's damping." {
    default 5
    min 0
    tunable true
  }
  field motorMaxForce f32 "The most force (or torque) the motor applies; 0 is unbounded." {
    min 0
    tunable true
  }
  field restLength f32 "A spring's rest length." {
    default 1
    min 0
    unit m
    tunable true
  }
  field stiffness f32 "A spring's stiffness (N/m)." {
    default 40
    min 0
    tunable true
  }
  field damping f32 "A spring's damping (N·s/m)." {
    default 2
    min 0
    tunable true
  }
  field length f32 "A rope's longest distance between the anchors." {
    default 2
    min 0
    unit m
    tunable true
  }
  field collide bool "Whether the two bodies still collide with each other."
  field value f32 "Written: the hinge angle (rad) or the slide (m) after the step; the anchors' distance for springs and ropes." {
    readonly true
  }
  field breakForce f32 "The force the joint holds before it breaks (JointBroken, `broken`): the solver's load on it through the step, its weight, gravity and what rests or presses on it; a lone impulse (applyImpulse, applyForce) is absorbed before it counts. 0 never breaks." {
    min 0
    unit newton
    tunable true
  }
  field breakTorque f32 "The torque about its anchor it holds before it breaks, as breakForce (a shelf bracket, a cantilevered beam); 0 never breaks." {
    min 0
    unit newtonMetre
    tunable true
  }
  field broken bool "Written: true once the joint broke; a broken joint holds nothing. Write false to mend it, rebuilt where its bodies are now."
}
```

<sub>Module `physics` line 309</sub>
