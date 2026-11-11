<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Vehicle"></a>

### `og.Vehicle` (component)

A car on a dynamic chassis (the entity's og.RigidBody and og.Collider). 3D: a raycast car with four wheels (suspension, grip, steering on the front axle), or with the og.VehicleWheel children it has; its forward is the chassis's local +z. 2D: two wheel bodies on sprung wheel joints (front at +x), driven by a motor. og.physics.vehicleWheels gives the wheels' poses and slip.

The defaults are a road car of about 1,000 kg (og.RigidBody.mass 1000). The suspension and the grip scale with the chassis's mass, the engine and the brakes do not.

The wheel rays meet what the chassis's collider would.

See `physics §7`.

Token `og.Vehicle`, registered under the schema name `og.Vehicle`; its value type is `Vehicle`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `throttle` | `number` | `0` | -1 … 1 | — | — | The accelerator, -1 (reverse) to 1. Set it every tick. |
| `steer` | `number` | `0` | -1 … 1 | — | — | The steering, -1 (left) to 1 (right); 3D only. |
| `brake` | `number` | `0` | 0 … 1 | — | — | The brake, 0 to 1. |
| `wheelBase` | `number` | `2.6` | ≥ 0 | `m` | tunable | The distance between the front and rear axles. |
| `track` | `number` | `1.6` | ≥ 0 | `m` | tunable | The distance between the left and right wheels (3D). |
| `wheelRadius` | `number` | `0.4` | ≥ 0.01 | `m` | tunable | The wheels' radius. |
| `mountHeight` | `number` | `0.2` | — | `m` | tunable | The height of the wheel mounts below the chassis centre. |
| `suspensionRest` | `number` | `0.35` | ≥ 0 | `m` | tunable | The suspension's length at rest. |
| `suspensionTravel` | `number` | `0.25` | ≥ 0 | `m` | tunable | How far the suspension compresses or extends from rest. |
| `suspensionStiffness` | `number` | `30` | ≥ 0 | — | tunable | The suspension's stiffness. |
| `suspensionDamping` | `number` | `4` | ≥ 0 | — | tunable | The suspension's damping. |
| `grip` | `number` | `2` | ≥ 0 | — | tunable | The tyres' grip (3D: the friction slip, how much of its load a tyre holds before it slides; 2D: the wheels' friction). See `physics §7`. |
| `engineForce` | `number` | `2500` | ≥ 0 | — | tunable | The engine's force at full throttle (3D: per driven wheel, in newtons; 2D: the motor's torque, N·m). See `physics §7`. |
| `maxSpeed` | `number` | `40` | ≥ 0 | `mps` | tunable | The top speed: the engine's force fades out towards it (throttle against the motion brakes first). |
| `brakeForce` | `number` | `2000` | ≥ 0 | — | tunable | The brakes' force at full brake (3D: per wheel, in newtons, every wheel braking; 2D: the wheels' braking torque). See `physics §7`. |
| `maxSteer` | `number` | `0.55` | 0 … 1.2 | `rad` | tunable | The front wheels' largest steering angle (3D). |
| `drive` | `number` | `0` | 0 … 2 | — | — | DRIVE_REAR 0, DRIVE_FRONT 1, DRIVE_ALL 2. |
| `wheelMass` | `number` | `20` | ≥ 0.01 | `kg` | tunable | The wheels' mass (2D: the wheel bodies; 3D ignores it). |
| `rollInfluence` | `number` | `0` | 0 … 1 | — | tunable | How much the tyres' sideways grip rolls the chassis in a bend (3D): 0 pushes at the centre of mass's height, so the car corners flat and never tips over from steering; towards 1 at the road, the roll that tips a tall car over. See `physics §7`. |
| `speed` | `number` | `0` | — | `mps` | read only | Written: the forward speed (m/s; negative reversing). |
| `contacts` | `number` | `0` | — | — | read only | Written: how many wheels touch the ground. |
| `slip` | `number` | `0` | 0 … 1 | — | read only | Written: the largest slip of its wheels (og.physics.vehicleWheels' `slip`): 0 every tyre grips, towards 1 one slides; for a squeal's volume or smoke without reading the wheels (3D; 0 in 2D). |
| `model` | `number` | `0` | 0 … 1 | — | — | VEHICLE_ARCADE 0 (engineForce, faded towards maxSpeed) or VEHICLE_DRIVETRAIN 1 (the Drivetrain fields; engineForce and maxSpeed unused). The driven wheels' torque is torqueCurve's at the engine's speed × throttle × the gear's ratio × finalDrive; engine speed follows the wheels while a gear is in. |
| `handbrake` | `number` | `0` | 0 … 1 | — | — | The handbrake, 0 to 1: it brakes the handbrake wheels (3D: the rear pair, or the og.VehicleWheel children flagged `handbrake`; 2D: the rear wheel) with that share of brakeForce, takes their drive and loosens their grip, sideways and in all, towards handbrakeGrip. See `physics §7`. |
| `handbrakeGrip` | `number` | `0.2` | 0 … 1 | — | tunable | The handbrake wheels' grip at full handbrake, sideways and in all, as a share of their own (3D); 1 keeps it. |
| `driftGrip` | `number` | `1` | 0 … 1 | — | tunable | Drift tuning: the handbrake wheels' sideways grip while the car drifts (the handbrake is on, or one of them slid last step), as a share of their own (3D). 1 is off. |
| `gears` | `readonly number[]` | `[]` | — | — | a list of at most 8 `f32` (`List.f32.8`): reads a frozen copy, writes a whole array | Drivetrain: the forward gears' ratios, first gear first; empty uses 3.4, 2.2, 1.6, 1.25, 1.0, 0.82. |
| `reverseRatio` | `number` | `3.2` | ≥ 0 | — | tunable | Drivetrain: the reverse gear's ratio. |
| `finalDrive` | `number` | `3.6` | ≥ 0 | — | tunable | Drivetrain: the final drive's ratio, after every gear. |
| `idleRpm` | `number` | `900` | ≥ 0 | `rpm` | tunable | Drivetrain: the engine's speed at rest. |
| `maxRpm` | `number` | `6800` | ≥ 1 | `rpm` | tunable | Drivetrain: the rev limit: the engine gives no torque at or above it. |
| `torqueCurve` | `readonly number[]` | `[]` | — | — | a list of at most 8 `f32` (`List.f32.8`): reads a frozen copy, writes a whole array | Drivetrain: the engine's torque (N·m) at full throttle, at evenly spaced speeds from 0 to maxRpm, interpolated between them; empty uses 140, 170, 195, 210, 215, 205, 185, 150 (a 1,000 kg road car's). |
| `automatic` | `boolean` | `true` | — | — | — | Drivetrain: the gearbox shifts by itself, up at shiftUpRpm and down at shiftDownRpm, into reverse when throttle is negative at a standstill and back into first when it is positive. False leaves `gear` to the game (a manual gearbox). |
| `shiftUpRpm` | `number` | `6000` | ≥ 0 | `rpm` | tunable | Drivetrain, automatic: the engine speed it shifts up at. |
| `shiftDownRpm` | `number` | `2800` | ≥ 0 | `rpm` | tunable | Drivetrain, automatic: the engine speed it shifts down at. |
| `clutchTime` | `number` | `0.25` | ≥ 0 | `s` | tunable | Drivetrain: how long a gear change takes; the engine drives nothing meanwhile. |
| `gear` | `number` | `0` | -1 … 8 | — | — | Drivetrain: the gear, -1 reverse, 0 neutral, 1 and up forward. An automatic gearbox writes it; for a manual one the game sets it, and each change takes clutchTime. |
| `rpm` | `number` | `0` | — | `rpm` | read only | Written, drivetrain: the engine's speed, for the engine's sound and the tachometer. |

```ogt
component Vehicle {
  doc "A car on a dynamic chassis (the entity's og.RigidBody and og.Collider). 3D: a raycast car with four wheels (suspension, grip, steering on the front axle), or with the og.VehicleWheel children it has; its forward is the chassis's local +z. 2D: two wheel bodies on sprung wheel joints (front at +x), driven by a motor. og.physics.vehicleWheels gives the wheels' poses and slip."
  doc "The defaults are a road car of about 1,000 kg (og.RigidBody.mass 1000). The suspension and the grip scale with the chassis's mass, the engine and the brakes do not."
  doc "The wheel rays meet what the chassis's collider would."
  guide "physics §7"
  field throttle f32 "The accelerator, -1 (reverse) to 1. Set it every tick." {
    min -1
    max 1
  }
  field steer f32 "The steering, -1 (left) to 1 (right); 3D only." {
    min -1
    max 1
  }
  field brake f32 "The brake, 0 to 1." {
    min 0
    max 1
  }
  field wheelBase f32 "The distance between the front and rear axles." {
    default 2.6
    min 0
    unit m
    tunable true
  }
  field track f32 "The distance between the left and right wheels (3D)." {
    default 1.6
    min 0
    unit m
    tunable true
  }
  field wheelRadius f32 "The wheels' radius." {
    default 0.4
    min 0.01
    unit m
    tunable true
  }
  field mountHeight f32 "The height of the wheel mounts below the chassis centre." {
    default 0.2
    unit m
    tunable true
  }
  field suspensionRest f32 "The suspension's length at rest." {
    default 0.35
    min 0
    unit m
    tunable true
  }
  field suspensionTravel f32 "How far the suspension compresses or extends from rest." {
    default 0.25
    min 0
    unit m
    tunable true
  }
  field suspensionStiffness f32 "The suspension's stiffness." {
    default 30
    min 0
    tunable true
  }
  field suspensionDamping f32 "The suspension's damping." {
    default 4
    min 0
    tunable true
  }
  field grip f32 "The tyres' grip (3D: the friction slip, how much of its load a tyre holds before it slides; 2D: the wheels' friction)." {
    guide "physics §7"
    default 2
    min 0
    tunable true
  }
  field engineForce f32 "The engine's force at full throttle (3D: per driven wheel, in newtons; 2D: the motor's torque, N·m)." {
    guide "physics §7"
    default 2500
    min 0
    tunable true
  }
  field maxSpeed f32 "The top speed: the engine's force fades out towards it (throttle against the motion brakes first)." {
    default 40
    min 0
    unit mps
    tunable true
  }
  field brakeForce f32 "The brakes' force at full brake (3D: per wheel, in newtons, every wheel braking; 2D: the wheels' braking torque)." {
    guide "physics §7"
    default 2000
    min 0
    tunable true
  }
  field maxSteer f32 "The front wheels' largest steering angle (3D)." {
    default 0.55
    min 0
    max 1.2
    unit rad
    tunable true
  }
  field drive u32 "DRIVE_REAR 0, DRIVE_FRONT 1, DRIVE_ALL 2." {
    min 0
    max 2
  }
  field wheelMass f32 "The wheels' mass (2D: the wheel bodies; 3D ignores it)." {
    default 20
    min 0.01
    unit kg
    tunable true
  }
  field rollInfluence f32 "How much the tyres' sideways grip rolls the chassis in a bend (3D): 0 pushes at the centre of mass's height, so the car corners flat and never tips over from steering; towards 1 at the road, the roll that tips a tall car over." {
    guide "physics §7"
    min 0
    max 1
    tunable true
  }
  field speed f32 "Written: the forward speed (m/s; negative reversing)." {
    unit mps
    readonly true
  }
  field contacts u32 "Written: how many wheels touch the ground." {
    readonly true
  }
  field slip f32 "Written: the largest slip of its wheels (og.physics.vehicleWheels' `slip`): 0 every tyre grips, towards 1 one slides; for a squeal's volume or smoke without reading the wheels (3D; 0 in 2D)." {
    min 0
    max 1
    readonly true
  }
  field model u32 "VEHICLE_ARCADE 0 (engineForce, faded towards maxSpeed) or VEHICLE_DRIVETRAIN 1 (the Drivetrain fields; engineForce and maxSpeed unused). The driven wheels' torque is torqueCurve's at the engine's speed × throttle × the gear's ratio × finalDrive; engine speed follows the wheels while a gear is in." {
    min 0
    max 1
  }
  field handbrake f32 "The handbrake, 0 to 1: it brakes the handbrake wheels (3D: the rear pair, or the og.VehicleWheel children flagged `handbrake`; 2D: the rear wheel) with that share of brakeForce, takes their drive and loosens their grip, sideways and in all, towards handbrakeGrip." {
    guide "physics §7"
    min 0
    max 1
  }
  field handbrakeGrip f32 "The handbrake wheels' grip at full handbrake, sideways and in all, as a share of their own (3D); 1 keeps it." {
    default 0.2
    min 0
    max 1
    tunable true
  }
  field driftGrip f32 "Drift tuning: the handbrake wheels' sideways grip while the car drifts (the handbrake is on, or one of them slid last step), as a share of their own (3D). 1 is off." {
    default 1
    min 0
    max 1
    tunable true
  }
  field gears List.f32.8 "Drivetrain: the forward gears' ratios, first gear first; empty uses 3.4, 2.2, 1.6, 1.25, 1.0, 0.82."
  field reverseRatio f32 "Drivetrain: the reverse gear's ratio." {
    default 3.2
    min 0
    tunable true
  }
  field finalDrive f32 "Drivetrain: the final drive's ratio, after every gear." {
    default 3.6
    min 0
    tunable true
  }
  field idleRpm f32 "Drivetrain: the engine's speed at rest." {
    default 900
    min 0
    unit rpm
    tunable true
  }
  field maxRpm f32 "Drivetrain: the rev limit: the engine gives no torque at or above it." {
    default 6800
    min 1
    unit rpm
    tunable true
  }
  field torqueCurve List.f32.8 "Drivetrain: the engine's torque (N·m) at full throttle, at evenly spaced speeds from 0 to maxRpm, interpolated between them; empty uses 140, 170, 195, 210, 215, 205, 185, 150 (a 1,000 kg road car's)."
  field automatic bool "Drivetrain: the gearbox shifts by itself, up at shiftUpRpm and down at shiftDownRpm, into reverse when throttle is negative at a standstill and back into first when it is positive. False leaves `gear` to the game (a manual gearbox)." {
    default true
  }
  field shiftUpRpm f32 "Drivetrain, automatic: the engine speed it shifts up at." {
    default 6000
    min 0
    unit rpm
    tunable true
  }
  field shiftDownRpm f32 "Drivetrain, automatic: the engine speed it shifts down at." {
    default 2800
    min 0
    unit rpm
    tunable true
  }
  field clutchTime f32 "Drivetrain: how long a gear change takes; the engine drives nothing meanwhile." {
    default 0.25
    min 0
    unit s
    tunable true
  }
  field gear i32 "Drivetrain: the gear, -1 reverse, 0 neutral, 1 and up forward. An automatic gearbox writes it; for a manual one the game sets it, and each change takes clutchTime." {
    min -1
    max 8
  }
  field rpm f32 "Written, drivetrain: the engine's speed, for the engine's sound and the tachometer." {
    unit rpm
    readonly true
  }
}
```

<sub>Module `physics` line 705</sub>
