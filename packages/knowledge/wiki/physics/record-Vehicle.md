<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Vehicle"></a>

### `og.Vehicle` (component)

Independent wheels on a dynamic chassis: forward +z in 3D, +x in 2D. Four implicit wheels in 3D, two physical wheels in 2D, or up to sixteen VehicleWheel children in either profile. Game code copies vehicleWheels poses to its own Transforms; the engine draws no wheels.

Suspension stiffness and damping are per chassis kilogram. Drive and brake are wheel torques in N·m. Grip is the real friction coefficient multiplied by the ground friction. New numeric tuning fields are off at zero.

3D wheels cast the full suspension droop, skip sensors and honour ContactHook.

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
| `compressionDamping` | `number` | `0` | ≥ 0 | — | tunable | Compression damping per chassis kilogram; zero disables it. |
| `reboundDamping` | `number` | `0` | ≥ 0 | — | tunable | Rebound damping per chassis kilogram; zero disables it. |
| `bumpStopStiffness` | `number` | `0` | ≥ 0 | — | tunable | Progressive bump stop coefficient, N/m²; zero disables it. |
| `antiRollStiffness` | `number` | `0` | ≥ 0 | — | tunable | Opposite wheels on the same axle exchange this many N per metre of compression difference; zero disables it (3D). |
| `grip` | `number` | `1` | ≥ 0 | — | tunable | The real tyre friction coefficient, multiplied by the ground friction; no hidden scale. See `physics §7`. |
| `engineForce` | `number` | `2500` | ≥ 0 | — | tunable | Arcade drive torque per driven wheel in N·m, faded towards maxSpeed. See `physics §7`. |
| `maxSpeed` | `number` | `40` | ≥ 0 | `mps` | tunable | The top speed: the engine's force fades out towards it (throttle against the motion brakes first). |
| `brakeForce` | `number` | `2000` | ≥ 0 | — | tunable | Brake torque per wheel in N·m before brakeBias. Solved with drive and tyre reaction on wheel angular velocity. See `physics §7`. |
| `maxSteer` | `number` | `0.55` | 0 … 1.2 | `rad` | tunable | The front wheels' largest steering angle (3D). |
| `drive` | `VehicleDrive` | `"rear"` | — | — | — | Which wheels drive: rear, front or all. |
| `wheelMass` | `number` | `20` | ≥ 0.01 | `kg` | tunable | Wheel mass in both profiles; spin inertia is half the mass times radius squared. |
| `rollInfluence` | `number` | `0` | 0 … 1 | — | tunable | How much the tyres' sideways grip rolls the chassis in a bend (3D): 0 pushes at the centre of mass's height, so the car corners flat and never tips over from steering; towards 1 at the road, the roll that tips a tall car over. See `physics §7`. |
| `wheelCast` | `VehicleWheelCast` | `"ray"` | — | — | — | 3D wheel contact cast shape; cylinder axle follows steering, width is half its radius. |
| `brakeBias` | `number` | `0` | 0 … 1 | — | tunable | Zero disables bias (equal torque per wheel); otherwise the front axle share of service braking. |
| `aeroDrag` | `number` | `0` | ≥ 0 | — | tunable | Quadratic drag coefficient in kg/m; zero disables drag. |
| `aeroDownforce` | `number` | `0` | ≥ 0 | — | tunable | Downforce coefficient in kg/m along chassis down, times forward speed squared; zero disables it. |
| `rollingResistance` | `number` | `0` | ≥ 0 | — | tunable | Rolling resistance force per wheel, N; zero disables it. |
| `ackermann` | `number` | `0` | 0 … 1 | — | tunable | 3D steering blend from parallel (zero) to Ackermann (one). |
| `steeringFade` | `number` | `0` | ≥ 0 | — | tunable | 3D steering angle divides by one plus this times absolute speed; zero disables it. |
| `abs` | `number` | `0` | 0 … 1 | — | tunable | Service brake release at wheel slip, zero off to one full assistance; handbrake remains unassisted. |
| `tcs` | `number` | `0` | 0 … 1 | — | tunable | Drive torque reduction at wheel slip, zero off to one full assistance. |
| `speed` | `number` | `0` | — | `mps` | read only | Written: the forward speed (m/s; negative reversing). |
| `contacts` | `number` | `0` | — | — | read only | Written: how many wheels touch the ground. |
| `slip` | `number` | `0` | 0 … 1 | — | read only | Written after the step: largest regularised wheel slip, zero gripping towards one sliding, in either profile. |
| `model` | `VehicleModel` | `"arcade"` | — | — | — | Arcade torque or an engine flywheel coupled through a clutch and gearbox to measured wheel angular velocities. Drivetrain requires nonzero engineInertia and clutchStrength. |
| `handbrake` | `number` | `0` | 0 … 1 | — | — | Extra brake torque on flagged wheels (implicit rear), independent of drive torque; blends their friction towards handbrakeGrip. See `physics §7`. |
| `handbrakeGrip` | `number` | `0.2` | 0 … 1 | — | tunable | Handbrake wheels' friction multiplier at full handbrake, in either profile. |
| `gears` | `readonly number[]` | `[]` | — | — | a list of at most 8 `f32` (`List.f32.8`): reads a frozen copy, writes a whole array | Drivetrain: the forward gears' ratios, first gear first; empty uses 3.4, 2.2, 1.6, 1.25, 1.0, 0.82. |
| `reverseRatio` | `number` | `3.2` | ≥ 0 | — | tunable | Drivetrain: the reverse gear's ratio. |
| `finalDrive` | `number` | `3.6` | ≥ 0 | — | tunable | Drivetrain: the final drive's ratio, after every gear. |
| `idleRpm` | `number` | `900` | ≥ 0 | `rpm` | tunable | Drivetrain: the engine's speed at rest. |
| `maxRpm` | `number` | `6800` | ≥ 1 | `rpm` | tunable | Drivetrain: the rev limit: the engine gives no torque at or above it. |
| `torqueCurve` | `readonly number[]` | `[]` | — | — | a list of at most 8 `f32` (`List.f32.8`): reads a frozen copy, writes a whole array | Drivetrain: the engine's torque (N·m) at full throttle, at evenly spaced speeds from 0 to maxRpm, interpolated between them; empty uses 140, 170, 195, 210, 215, 205, 185, 150 (a 1,000 kg road car's). |
| `automatic` | `boolean` | `true` | — | — | — | Drivetrain: the gearbox shifts by itself, up at shiftUpRpm and down at shiftDownRpm, into reverse when throttle is negative at a standstill and back into first when it is positive. False leaves `gear` to the game (a manual gearbox). |
| `shiftUpRpm` | `number` | `6000` | ≥ 0 | `rpm` | tunable | Drivetrain, automatic: the engine speed it shifts up at. |
| `shiftDownRpm` | `number` | `2800` | ≥ 0 | `rpm` | tunable | Drivetrain, automatic: the engine speed it shifts down at. |
| `clutchTime` | `number` | `0.25` | ≥ 0 | `s` | tunable | Drivetrain: clutch re-engagement ramp after a gear change; first engagement has no delay. |
| `engineInertia` | `number` | `0` | ≥ 0 | — | tunable | Engine flywheel inertia in kg·m²; zero disables the engine. |
| `clutchStrength` | `number` | `0` | ≥ 0 | — | tunable | Maximum clutch torque in N·m; zero disconnects the engine. |
| `engineBraking` | `number` | `0` | ≥ 0 | — | tunable | Engine drag torque per rad/s above idle with closed throttle; zero disables drag. |
| `differential` | `VehicleDifferential` | `"open"` | — | — | — | Open equal-torque, limited slip, or locked wheel angular velocities. |
| `differentialStrength` | `number` | `0` | ≥ 0 | — | tunable | LSD angular-speed convergence rate per second; zero disables LSD coupling. |
| `awdBias` | `number` | `0` | 0 … 1 | — | tunable | Zero disables axle bias (equal driven-wheel torque); otherwise front axle drive share when both axles drive. |
| `gear` | `number` | `0` | -1 … 8 | — | — | Drivetrain: the gear, -1 reverse, 0 neutral, 1 and up forward. An automatic gearbox writes it; for a manual one the game sets it, and each change takes clutchTime. |
| `rpm` | `number` | `0` | — | `rpm` | read only | Written, drivetrain: the engine's speed, for the engine's sound and the tachometer. |

```ogt
component Vehicle {
  doc "Independent wheels on a dynamic chassis: forward +z in 3D, +x in 2D. Four implicit wheels in 3D, two physical wheels in 2D, or up to sixteen VehicleWheel children in either profile. Game code copies vehicleWheels poses to its own Transforms; the engine draws no wheels."
  doc "Suspension stiffness and damping are per chassis kilogram. Drive and brake are wheel torques in N·m. Grip is the real friction coefficient multiplied by the ground friction. New numeric tuning fields are off at zero."
  doc "3D wheels cast the full suspension droop, skip sensors and honour ContactHook."
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
  field compressionDamping f32 "Compression damping per chassis kilogram; zero disables it." {
    min 0
    tunable true
  }
  field reboundDamping f32 "Rebound damping per chassis kilogram; zero disables it." {
    min 0
    tunable true
  }
  field bumpStopStiffness f32 "Progressive bump stop coefficient, N/m²; zero disables it." {
    min 0
    tunable true
  }
  field antiRollStiffness f32 "Opposite wheels on the same axle exchange this many N per metre of compression difference; zero disables it (3D)." {
    min 0
    tunable true
  }
  field grip f32 "The real tyre friction coefficient, multiplied by the ground friction; no hidden scale." {
    guide "physics §7"
    default 1
    min 0
    tunable true
  }
  field engineForce f32 "Arcade drive torque per driven wheel in N·m, faded towards maxSpeed." {
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
  field brakeForce f32 "Brake torque per wheel in N·m before brakeBias. Solved with drive and tyre reaction on wheel angular velocity." {
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
  field drive VehicleDrive "Which wheels drive: rear, front or all." {
  }
  field wheelMass f32 "Wheel mass in both profiles; spin inertia is half the mass times radius squared." {
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
  field wheelCast VehicleWheelCast "3D wheel contact cast shape; cylinder axle follows steering, width is half its radius."
  field brakeBias f32 "Zero disables bias (equal torque per wheel); otherwise the front axle share of service braking." {
    min 0
    max 1
    tunable true
  }
  field aeroDrag f32 "Quadratic drag coefficient in kg/m; zero disables drag." {
    min 0
    tunable true
  }
  field aeroDownforce f32 "Downforce coefficient in kg/m along chassis down, times forward speed squared; zero disables it." {
    min 0
    tunable true
  }
  field rollingResistance f32 "Rolling resistance force per wheel, N; zero disables it." {
    min 0
    tunable true
  }
  field ackermann f32 "3D steering blend from parallel (zero) to Ackermann (one)." {
    min 0
    max 1
    tunable true
  }
  field steeringFade f32 "3D steering angle divides by one plus this times absolute speed; zero disables it." {
    min 0
    tunable true
  }
  field abs f32 "Service brake release at wheel slip, zero off to one full assistance; handbrake remains unassisted." {
    min 0
    max 1
    tunable true
  }
  field tcs f32 "Drive torque reduction at wheel slip, zero off to one full assistance." {
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
  field slip f32 "Written after the step: largest regularised wheel slip, zero gripping towards one sliding, in either profile." {
    min 0
    max 1
    readonly true
  }
  field model VehicleModel "Arcade torque or an engine flywheel coupled through a clutch and gearbox to measured wheel angular velocities. Drivetrain requires nonzero engineInertia and clutchStrength." {
  }
  field handbrake f32 "Extra brake torque on flagged wheels (implicit rear), independent of drive torque; blends their friction towards handbrakeGrip." {
    guide "physics §7"
    min 0
    max 1
  }
  field handbrakeGrip f32 "Handbrake wheels' friction multiplier at full handbrake, in either profile." {
    default 0.2
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
  field clutchTime f32 "Drivetrain: clutch re-engagement ramp after a gear change; first engagement has no delay." {
    default 0.25
    min 0
    unit s
    tunable true
  }
  field engineInertia f32 "Engine flywheel inertia in kg·m²; zero disables the engine." {
    min 0
    tunable true
  }
  field clutchStrength f32 "Maximum clutch torque in N·m; zero disconnects the engine." {
    min 0
    tunable true
  }
  field engineBraking f32 "Engine drag torque per rad/s above idle with closed throttle; zero disables drag." {
    min 0
    tunable true
  }
  field differential VehicleDifferential "Open equal-torque, limited slip, or locked wheel angular velocities."
  field differentialStrength f32 "LSD angular-speed convergence rate per second; zero disables LSD coupling." {
    min 0
    tunable true
  }
  field awdBias f32 "Zero disables axle bias (equal driven-wheel torque); otherwise front axle drive share when both axles drive." {
    min 0
    max 1
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

<sub>Module `physics` line 947</sub>
