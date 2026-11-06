<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-LookAt"></a>

### `og.LookAt` (component)

An aim constraint on a skeleton, applied after its clips, graph and IK chains: `joint` turns so its `axis` points at `target` and its `upAxis` leans towards `up` (three.js Object3D.lookAt: +Z at the target, +Y up). The direction is held within maxYaw and maxPitch of the animated forward; with `bones` above 1 the joint's parents share the turn. Put it on its own entity (a head, the eyes, a turret's barrel); constraints apply in query order, and og.JointLimit clamps the joints it turns.

Also known as: head look, aim constraint, head tracking, Unity LookAt constraint, Multi-Aim constraint, Unreal Look At node

Token `og.LookAt`, registered under the schema name `og.LookAt`; its value type is `LookAt`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton this constraint turns. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The joint that aims (an index into the skin's joints): the head, an eye. |
| `bones` | `number` | `1` | 1 … 8 | — | — | How many joints share the turn: `joint` and its parents (3 spreads a look over neck, chest and spine), each parent taking an even share and `joint` the rest. |
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `hook` | The point to look at, in `space`. |
| `space` | `IkSpace` | `"world"` | — | — | — | What `target` and `up` are relative to: world (the frame the skeleton entity's og.Transform is in) or model. |
| `axis` | `Vector3` | `{ x: 0, y: 0, z: 1 }` | — | — | — | The joint's forward, in its own frame: the axis turned onto the target. |
| `upAxis` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | The joint's up, in its own frame: yaw turns about it, pitch towards it, and it rolls towards `up`. |
| `up` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | The up hint, a direction in `space` the joint's `upAxis` leans to (three.js Object3D.up); the zero vector keeps the animated roll. |
| `maxYaw` | `number` | `3.1416` | 0 … 3.1416 | `rad` | tunable | The largest turn about `upAxis`, left or right of the animated forward. |
| `maxPitch` | `number` | `1.5708` | 0 … 1.5708 | `rad` | tunable | The largest turn towards or away from `upAxis`, from the animated forward. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the turn is applied (0 = the animated pose, 1 = aimed). |
| `clamped` | `boolean` | `false` | — | — | read only | Written: the target lay outside maxYaw or maxPitch, so the joint aims at the nearest direction inside them. |

```ogt
component LookAt {
  doc "An aim constraint on a skeleton, applied after its clips, graph and IK chains: `joint` turns so its `axis` points at `target` and its `upAxis` leans towards `up` (three.js Object3D.lookAt: +Z at the target, +Y up). The direction is held within maxYaw and maxPitch of the animated forward; with `bones` above 1 the joint's parents share the turn. Put it on its own entity (a head, the eyes, a turret's barrel); constraints apply in query order, and og.JointLimit clamps the joints it turns."
  aliases "head look, aim constraint, head tracking, Unity LookAt constraint, Multi-Aim constraint, Unreal Look At node"
  field skeleton Entity "The entity with the og.Skeleton this constraint turns."
  field joint u32 "The joint that aims (an index into the skin's joints): the head, an eye." {
    min 0
  }
  field bones u32 "How many joints share the turn: `joint` and its parents (3 spreads a look over neck, chest and spine), each parent taking an even share and `joint` the rest." {
    default 1
    min 1
    max 8
  }
  field target Vector3 "The point to look at, in `space`." {
    worldspace hook
    unit m
  }
  field space IkSpace "What `target` and `up` are relative to: world (the frame the skeleton entity's og.Transform is in) or model."
  field axis Vector3 "The joint's forward, in its own frame: the axis turned onto the target." {
    default 0 0 1
  }
  field upAxis Vector3 "The joint's up, in its own frame: yaw turns about it, pitch towards it, and it rolls towards `up`." {
    default 0 1 0
  }
  field up Vector3 "The up hint, a direction in `space` the joint's `upAxis` leans to (three.js Object3D.up); the zero vector keeps the animated roll." {
    default 0 1 0
  }
  field maxYaw f32 "The largest turn about `upAxis`, left or right of the animated forward." {
    default 3.1416
    min 0
    max 3.1416
    unit rad
    tunable true
  }
  field maxPitch f32 "The largest turn towards or away from `upAxis`, from the animated forward." {
    default 1.5708
    min 0
    max 1.5708
    unit rad
    tunable true
  }
  field weight f32 "How much of the turn is applied (0 = the animated pose, 1 = aimed)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field clamped bool "Written: the target lay outside maxYaw or maxPitch, so the joint aims at the nearest direction inside them." {
    readonly true
  }
}
```

<sub>`api.ogt` line 11779</sub>
