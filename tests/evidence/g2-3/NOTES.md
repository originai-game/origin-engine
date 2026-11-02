# G2-3: physics sample (batch 1: compounds, sleep, breaking joints)

`sample/` is one 3D world. One camera looks at three stations on a grey floor. `Physics.sleepTicks` is 30.

- Left: a compound dumbbell. One `og.RigidBody` has three child colliders, two spheres and a bar. It is thrown spinning at (-7, 4, 0), lands, tumbles and rolls on its spheres.
- Centre: a pile of nine orange crates in three crossed layers. While `og.RigidBody.sleeping` is true, a crate is drawn blue, as the `Watch` system reads it.
- Right: a platform hangs from a hook at (7, 7, 0) by three steel links joined with ball joints. Only the hook's joint breaks, at `breakForce` 300 N, about 31 kg held. A 40 kg crate falls 1.5 m onto the platform and breaks that joint. On `og.JointBroken`, the top link turns red and the chain falls.

The entity `bench` carries `Bench`, which counts the crates asleep and the `og.BodySlept`, `og.BodyWoke` and `og.JointBroken` events seen.

## Run on the Mac satellite (`metal`, macos-27, dev build of 389d1e1f)

The three frames below were drawn there and read:

- At tick 24, `link-0`'s joint is not broken. The 40 kg crate has landed on the platform, and the chain holds it.
- At tick 72, `Bench` reads `broken 1`. The hook joint broke under the held load, not at the start.
- At tick 240, all nine crates are blue. The dumbbell is still awake: it is rolling slowly on its spheres.

A logic run in the orb (`OG_RENDERER=none`, the same scene) read `asleep 8` at tick 150 and `asleep 9`, `slept 14`, `woke 0` at tick 240. The other five sleeps are the chain, the platform and the load at rest. In both runs one crate, resting against the fallen platform, stays awake while that platform still moves. That is the island rule: a body sleeps only with every dynamic body it touches.

## Commands (seconds, not ticks)

```sh
og run tests/evidence/g2-3/sample --headless --size 1280x720 --out <dir> --commands \
  "0.4:screenshot:tumbling.png;0.4:get:link-0:Joint;1.2:screenshot:joint-broken.png;1.2:get:bench:Bench;4:screenshot:pile-asleep.png"
```

## What each frame shows

- `tumbling.png` (tick 24):
  - The dumbbell is in the air, turned, its two steel spheres joined by a bar.
  - The crates, all orange, have fallen from their crossed layers into a heap.
  - The chain hangs from the hook with the platform level and the 40 kg crate resting on its right half.
- `joint-broken.png` (tick 72):
  - The dumbbell lies on the floor on both spheres.
  - The top link is red. The chain, the tilted platform and the crate fall together under the hook.
  - The crates are still orange.
- `pile-asleep.png` (tick 240):
  - All nine crates are blue (asleep).
  - The platform lies on the floor with its crate, and the red top link sits on it.
  - The dumbbell lies on its spheres, a little right of where it landed.

# The vehicle sample (batch 2: the drivetrain and og.VehicleWheel)

`vehicle/` is a 1,500 kg truck with `model` VEHICLE_DRIVETRAIN on six `og.VehicleWheel` children. The front pair steers, the rear four drive, and the last pair is on the handbrake. A short-shifting automatic changes up at 3,000 rpm. A chase camera follows it over a 400 m road with a stripe every 10 m.

The `Driver` system runs a script:

- waits for 0.5 s;
- full throttle until 4 s;
- from 4 s to 4.5 s, full lock with the handbrake pulled and throttle 0.3;
- then throttle 1 with steer 0.4.

Wherever a rear tyre slides sideways faster than 0.8 m/s (`WheelPose.sideSpeed`), a dark mark is laid every other tick from a pool spawned hidden at the start.

## Run on the Mac satellite (`metal`, macos-27, dev build)

- At 3.9 s: `gear 2`, `rpm 1998`, `speed 10.6` m/s.
- At 6 s, `Drive` reads `marks 30`, `slide 3.01` (m/s, the largest rear slide), `topGear 2`.
- The final hash is `436fc148cf9b38c6`. A logic run in the orb (`OG_RENDERER=none`) of the scene before the mark pool ended at the same hash on both machines (`bb75869bc4d8c2d9`).

## Commands

```sh
og run tests/evidence/g2-3/vehicle --headless --size 1280x720 --out <dir> --commands \
  "2:screenshot:pulling-away.png;3.9:get:truck:Vehicle;4.3:screenshot:handbrake-turn.png;6:screenshot:after-the-turn.png;6:get:drive:Drive"
```

## What each frame shows

- `pulling-away.png` (tick 120): the orange truck from behind and to its right, driving away from the camera along the road, cab in front, six black tyres on the ground. The 10 m stripes cross the road ahead.
- `handbrake-turn.png` (tick 258): the truck turned right across the stripes. Two dark streaks run from its rear tyres back and out to the side, off its heading: the tail stepping out under the handbrake.
- `after-the-turn.png` (tick 360): the camera has swung round behind the truck, which now heads about 90° from where it started. The stripes run towards the horizon, and the marks are behind the camera.
