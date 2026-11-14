# Physics and character control

How to build a game's physical world on og.physics and og.destruction: which
body and collider to pick, the units and the 2D or 3D profile, what queries
and collision events see, how to tune the character controllers and
vehicles, what the soft simulations (ropes, cloth, garments, soft bodies,
fluids, ragdolls, buoyancy) cost, and how destruction is set up and budgeted.
The determinism rules physics keeps are `determinism §2` and `determinism §4`;
this guide does not repeat them.

The API: `og api show physics`, `og api show destruction`; one entry with
`og api show RigidBody`, `og api show CharacterController`, and so on.

## 1. Bodies and colliders

An entity collides when it has an og.Transform and an og.Collider. An
og.RigidBody makes it a body; without one the collider is static at its
transform.

| Want | Use |
|---|---|
| Walls, floors, level geometry | a Collider alone, or a `fixed` body |
| Crates, balls, debris, anything pushed | a `dynamic` body |
| Doors, lifts, moving platforms, scripted movers | a `kinematic` body moved by og.Velocity |
| The player or an NPC that walks | a controller (§6), not a dynamic body |
| A trigger area, pickup, zone | a Collider with `sensor` |

A kinematic body moves by its og.Velocity each tick; writing its og.Transform
teleports it. Unlike Rapier there are no separate position-based and
velocity-based kinematic modes: to move a platform from A to B, set the
velocity that gets it there, never a series of transforms (it would not
carry what stands on it, and would not push).

Shapes, cheapest first: sphere, capsule, box, cylinder, convex hull, triangle
mesh, height field. Give dynamic bodies primitives or convex hulls; a
triangle mesh is for static and kinematic bodies (a level, a terrain mesh),
and a height field is the terrain's (`og.terrain.load`'s geometry). The
diamond and hexagon shapes are isometric and hex tiles' footprints; turn a
hexagon a quarter about z for a flat-top one. A glTF's geometry is
registered for convex and mesh colliders when it loads; `og.physics.setMesh`
registers anything else, and must run again before a world that names it is
restored.

Mass: leave RigidBody.mass 0 to derive it from the collider's density
(default 1 kg/m³, so a 1 m box weighs 1 kg) or set it outright; a car, a
person or a crate wants its real mass (§10). `centerOfMass` moves the centre
of mass without changing the inertia: lower a car's below its wheel mounts to
keep it on its wheels at full lock, or put a weight at a toy's base so it
rights itself.

Air: `airDrag` makes a body move with the air, og.Wind and every
og.WindMotor reaching it (`render §8`). Its velocity closes on the air's
at `airDrag` per second: 0.2 to 1 drifts a crate or a boat's sail, 3 to 10
blows a leaf, a balloon or a paper lantern along. Leave it 0 for anything
the wind should not move; `linearDamping` slows a body against the world,
`airDrag` against the air.

Compound bodies. A Collider without a RigidBody on a child entity whose
og.Parent chain reaches a body is part of the nearest such body: a table of a
top and four legs, a ship of a hull and a mast. It sits at its transform
relative to the body (the transforms between them composed, the body's
scale applied). Its mass adds to the body's from its density; when the body
sets RigidBody.mass, that mass is shared among its colliders by their
density. Children join in entity order, whatever the query order. Changing a
child's collider, or its transform relative to the body, rebuilds the body,
which loses its contacts' warm starts. A controller's body takes no
children, and a collider whose chain reaches no body stays static at its
world transform.

Layers: a collider is on `layers` and meets `mask`; two colliders interact
when each one's layers meet the other's mask. Give each kind of thing a bit
(player 1, enemies 2, debris 4, ...) and clear the bits a kind ignores.
og.ContactHook goes further per collider: one-way platforms, conveyor
belts, its own friction and restitution, and a list of entities it passes
through (a projectile and its shooter).

Joints: a hinge for doors and wheels, a slider for pistons, a spring for
suspension, a rope for chains, a ball joint for shoulders, a fixed joint to
weld. Put the og.Joint on an entity of its own when a body needs several.
`breakForce` counts the load the solver carries through the step (weight,
what rests or presses on it), so a shelf breaks under a pile; a lone impulse
is absorbed before it counts.

## 2. Units, scales and the profile

Metres, kilograms, seconds, and radians for every rotation, joint limit
and steering angle; a slope (`maxSlope`) is in degrees (`conventions §2`
has the whole rule). Keep moving things
between about 0.05 m and 100 m; scale the game, not gravity
(`determinism §2`).

The profile is project.ogt's `profile "2d"` or `"3d"`, and the same
components serve both. In 2D (rapier2d) bodies move in the xy plane and turn
about z; a transform keeps its z, vectors use x and y, rotations z, and the
only lock bits that mean anything are 1, 2 and 32. The cylinder and
height-field shapes, vehicle steering and og.VehicleWheel, Garment, Fluid3D,
Buoyancy and destructible props are 3D only; PlatformerController,
WaterLine and PixelTerrain are made for 2D, and Fluid2D runs in either
profile in the xy plane.

Setting Physics.profile to the other value at run time builds the other
profile's simulation from the components on the next step, so a 3D game can
play a 2D scene (a minigame, a side-on section) and switch back. Contacts,
joints' warm starts, ropes, cloth, fluids and ragdoll bodies start again.

## 3. The step and what it costs

og.physics runs after the game's systems each tick: it reads the
components, steps the world (`substeps` times), moves the controllers and
the soft simulations, then writes back the dynamic bodies and sends the
events.

- `substeps` 2 to 4 for tall stacks, fast bodies and stiff joints; each
  costs a full step.
- Sleep: set Physics.sleepTicks to about 30 (new projects do; the default 0
  never sleeps). A settled pile then sleeps whole and costs nothing until
  something wakes it: a body landing on it, a push, a joint, a moving
  kinematic body, a write to its Transform, Velocity, RigidBody or Collider,
  or its support going away. og.BodySlept is the cue to stop a rolling
  sound. `canSleep false` keeps a body (and what touches it) awake: a
  player's ball, a body game code moves every tick.
- `restSpeed` stops the slow creep of resting bodies; 0 turns it off.

`og.physics.stats(world)` lists the last step's phases: `gather` (reading
the components), `sync` (bodies and static colliders into rapier), `joints`
(joints and vehicles), `ragdolls`, `water` (buoyancy before the step),
`rapier` (its step, every substep), `readout` (touching pairs and bodies
read back, the rest rule), `controllers` (characters and platformers),
`soft` (ropes and cloth), `fluids` (2D), `fluids3d` and `writeBack`
(components and events). Its times are wall clock: compare them, never
feed them to the game.

## 4. Queries

```ts
const hit = must(og.physics.castRay(world, {
  origin: eye, direction: forward, maxDistance: 50,
  filter: { mask: ENEMY | WORLD, exclude: player },
}));
if (hit) damage(hit.body);
```

| Question | Query |
|---|---|
| What is in front of me, the first thing | castRay |
| Everything a piercing shot passes, everything under a cursor | castRayAll |
| Would this capsule fit, what does a thick sweep hit | castShape |
| What is inside this area right now | overlap |
| Nearest surface, closest cover, am I inside a solid | projectPoint |

Inside a system every query sees the colliders as they were at the start of
the tick, before any system ran: a body spawned or moved this tick is seen
on the next. Between ticks they see the world as it is. A hit's `entity` is
the collider's entity; `body` is the body it belongs to, which differs for a
compound child: damage the body, play the hit effect on the collider.
Sensors are skipped unless the filter asks for them. A ray per tick is the
cure for fast bullets tunnelling (`determinism §2`).

## 5. Collision events

Events of tick N are read in tick N + 1, after the solver has answered: by
then og.Velocity holds the bodies parting. How hard two things met is in the
event, taken in the step itself:

- `CollisionStarted.impulse` (N·s): the solver's momentum answer; use it for
  damage and for the volume of an impact sound.
- `max(0, relativeVelocity · normal)` (m/s): the closing speed, independent
  of mass; use it for "landed hard" or a speed threshold.

`CollisionEnded` and `SensorExited` also fire when one side despawns. A
sensor reports og.SensorEntered and og.SensorExited and pushes nothing.
og.Contact lists every touching pair every tick: read it for "what is
touching what right now" rather than tracking start and end yourself.

## 6. Character control

| Game | Controller |
|---|---|
| 3D third or first person, top-down 2D or 3D | CharacterController |
| 2D side-on platformer | PlatformerController (2D profile) |
| A ball, a vehicle, a ragdoll, a physics toy | a dynamic body |

Both are kinematic: they slide, step and snap by their own settings, push
dynamic bodies they walk into and ride the platform they stand on. Give the
entity a capsule (CharacterController) or a box or capsule
(PlatformerController), set the input fields every tick from one system,
and read the written fields after the step.

```ts
c.move = { x: stick.x * 6, y: 0, z: stick.y * 6 };
c.jump = jumpHeld;
```

CharacterController tuning: `maxSlope` 45° walks hills and refuses roofs;
`stepHeight` 0.3 m climbs stairs (a step is climbed when the ground beyond it
is at most that over the ground it stands on); `snapDistance` 0.2 m keeps
it on stairs and slopes going down (0 lets it leave every edge); `skin` is
the gap it keeps off surfaces and stands that far over the ground. For
top-down movement set `gravityScale` 0, and `move.y` then counts.

PlatformerController is tuned in designer units: `jumpHeight` and
`timeToApex` set gravity and the jump speed; `fallGravity` 1.5 to 2.5 gives
a snappy landing, `jumpCut` lets a released jump stop short, `apexGravity`
under 1 gives hang time. `coyoteTime` 0.08 to 0.12 s and `jumpBuffer` 0.1
to 0.15 s forgive late and early presses. Colliders on `oneWayMask` are
solid from above only; `down` drops through. `cornerCorrection` slides a
jump that clips a ceiling corner past it.

A knock-back, a spring or a recoil is `addVelocity`, taken on the next step:
`{ x: away * 6, y: 7 }` with `run` 0 for its few ticks. An upward y leaves
the ground.

Pushing: each tick the bodies a character walks into share an impulse of at
most `pushStrength` times the tick's length. With the default 1000 N a box
of a few kilograms is shoved at the character's speed, and a tonne on the
ground does not move; set crates' real mass (og.RigidBody.mass) so their
weight tells.

og.CharacterEvent is read the tick after: a jump pressed in tick T moves the
character in T and its CHARACTER_JUMPED arrives in T + 1. Drive dust,
squash, sounds and camera kicks from it.

## 7. Vehicles

Give a dynamic chassis (a box collider) an og.Vehicle and its real mass:
`RigidBody.mass 1000` for the default road car. A car-sized box at the
default density weighs a few kilograms, and the engine's and brakes'
newtons would throw it. The chassis's forward is its local +Z, as every
object's (`facing §1`). The suspension and grip scale with the chassis's
mass (Bullet's units); the engine and the brakes do not, so a heavier car
wants proportionally more `engineForce` and `brakeForce`.

| Field | Default means | Change it |
|---|---|---|
| `engineForce` 2500 (per driven wheel) | two rear wheels push 1,000 kg at 5 m/s² | scale with mass |
| `brakeForce` 2000 (per wheel) | four wheels stop 1,000 kg at 8 m/s² | scale with mass |
| `grip` 2 | a road car holds a bend | below 1 slides like a drift car |
| `rollInfluence` 0 | corners flat, never tips from steering | above 0 for body roll |
| `handbrakeGrip` 0.2 | the tail steps out in a bend at speed | 1 holds the line |
| `driftGrip` 1 | off | about 0.6 holds a slide that throttle and steering balance |

`rollInfluence` puts the tyres' side force between the centre of mass's
height (0) and the road (1). Rapier's own car uses 0.1, which rolls a
1,000 kg car at full lock above about 20 m/s; raise it a little only with
`centerOfMass` lowered.

A handbrake turn is a pull and a release: held, the car slows and the slide
dies with its speed. `driftGrip` applies while the handbrake is on or a
handbrake wheel slid last step.

The drivetrain model (`model: "drivetrain"`) replaces `engineForce` and
`maxSpeed` with a torque curve, gears and an automatic or manual gearbox;
`rpm` drives the engine sound and the tachometer.

Draw the wheels from `og.physics.vehicleWheels`; the implicit wheels are not
entities. Start smoke and skid marks at a wheel's `slip` above about 0.2 and
fade a squeal in with it; `sideSpeed` sets its pitch. Vehicle.slip is the
largest of them, for one sound without reading the wheels. og.VehicleWheel
children give six-wheeled trucks, trikes and rear-wheel steering.

The wheel rays use the chassis's layers and mask, so two cars on layers
that do not meet (a ghost car beside the player) pass through each other.

## 8. Soft bodies, cloth, fluids, ragdolls and buoyancy

Every one of these is one-way: it is pushed by colliders and pushes nothing
back (tie bodies together with a rope joint, not og.Rope).

| Component | Use for | Cost |
|---|---|---|
| Rope | cables, vines, chains drawn as a strip | segments × iterations; cheap |
| Cloth | flags, banners, curtains, sails | columns × rows × iterations × substeps; self-collision (`thickness`) costs most |
| Garment | skirts, capes, coat tails on a skinned character | the mesh's points × iterations |
| SoftBody | jelly, water balloons, slimes | points × iterations × substeps |
| Fluid2D | 2D water, lava, mixing liquids | grows with `count` |
| Fluid3D | pools, pours, dam breaks | about 0.9 ms CPU a tick for 2,000 particles at 1 substep, 1.8 ms at 2, native and wasm alike; the surface about 1.1 ms GPU (Apple M4 Pro, a third of a 1280×720 frame), growing with the pixels it covers |
| Ragdoll | death falls, hit reactions | a capsule body per bone |
| Buoyancy | boats, crates and barrels floating | `samples`³ points per body |

Fluid3D draws as a screen-space surface: the particles' depth smoothed into
one surface that refracts what is behind by its thickness, reflects the sky
and is lit by the scene's lights. Keep a scene's 3D fluids to a few thousand
simulated particles; the interaction
`radius` does not change the cost, the particle count and `substeps` do.
A SoftBody at detail 3 (642 points) with stiffness below 1 needs more than
the default 8 iterations to hold its shape.

**Simulated or visual.** Cloth, SoftBody, Garment and Fluid3D take
`simulation`. `gameplay` (the default) runs in the tick, on the CPU:
its points are world state that the game can read (`clothPoints`,
`softBodyPoints`, `garmentPoints`, `fluid3dParticles`). `visual` runs
on the GPU in the renderer, for looks only, and a headless server pays
nothing. Choose it for anything the game never asks about: a flag, a cape,
a decorative pool. What it means for the rest of the world:

- The tick leaves it unsolved: its points are the rest shape carried
  rigidly by the entity (a garment's, the skinned pose); SoftBody.volume
  stays 1 and SoftBody.center is that shape's mean. A visual Fluid3D has no
  particles in the tick at all: buoyancy and fluid groups ignore it and
  the snapshot carries nothing of it.
- The drawn motion runs on tick time: frames that skip ticks step those too,
  up to eight; a world restored, rolled back or more than eight ticks ahead
  starts again from the rest shape (a fluid, from the spawn box).
- It meets the colliders drawn this frame: boxes, spheres, capsules and
  height fields. Convex, cylinder and triangle-mesh colliders are passed
  through. With `thickness` a visual cloth collides with itself.
- Visual cloth and garments take the wind (og.Wind and their own `wind`
  added) as air moving along it at its magnitude in m/s, meeting each
  triangle with drag and lift against the triangle's own motion (scaled so
  a still face square to it feels the wind);
  `gust` sends bands of stronger, veering air down it, so it ripples and
  flaps rather than leaning.
- Where RenderCapabilities.visualSimulation is false (no compute or no
  vertex storage) it draws the rest shape, still.

A visual Fluid3D runs MLS-MPM (Hu et al. 2018) inside its container
(`bounds`; three times `size` on an open axis), on a grid of cells about
`radius` across (kept between two and eight spawned particles a cell, and
larger for a container past two million cells), with the emitter and the
sink (a particle entering the sink comes back at the emitter, or goes
without one). `stiffness` and `viscosity` scale the solver's own (water at
their defaults); `restDensity`, `nearStiffness`, `group`, `density` and
`mixing` do not apply, and `substeps` is a least. Changing count, seed,
size, bounds, radius or maxCount restarts it. Measured on an Apple M4 Pro: a
dam break of 60,000 growing to 110,000 particles in 5 cm cells, three steps
a tick, took 4.9 ms of GPU a frame with its surface.

**Garments.** Paint the mesh's vertex colours to say what moves: red or
alpha 0 stays on the skinned pose, red times alpha between 0 and 1 scales
`maxDistance`, white frees it. For a textured garment paint the alpha over a
white colour, since the red would tint it and an opaque material does not
show the alpha. `backstop` 0 keeps it out of the body; with `radius` above 0
the character's own collider counts, so keep that collider inside the body
or leave `radius` 0. The model draws the garment where the simulation leaves
it, with motion vectors for TAA.

**Ragdolls.** Build the rig once with `og.physics.ragdollRigFromSkeleton`
and give it with `setRagdollRig`; set `weight` 1 on a hit (with `impulse`)
and back to 0 to recover; the pose blends over `blendTime` both ways. Rigs
are assets: give them again after a reload.

**Buoyancy.** It floats a body on an og.WaterSurface, an og.Water (a flat
lake, or a sea with a wind) or both. Put `{}` beside the body's og.RigidBody
and og.Collider; it floats a body of the default density half under, as
wood floats (`waterDensity` 1 hangs it just under the surface). Each
cell of its hull under the surface pushes up by `waterDensity` × its share
of the colliders' volume × (gravity plus the water's own acceleration), so a
hull sinks with a crest that falls away rather than being flung clear. It
is computed in the tick, the same on every run and after a restore; the
sea's waves are the absolute world's, so a body moved by og.WorldOrigin rides
the same water. Raise `samples` for long hulls that should pitch on the
waves.

The same component floats what is not a rigid body. On an og.Ragdoll's
entity its bones float while it is limp, so a body knocked into a river
drifts off face down instead of sinking through the bed. On an
og.CharacterController's entity the character is lifted and slowed as far
as it is under and carried by the water's flow. Both count as density 1:
`waterDensity` 2 floats them half under, under 1 they sink. With no
og.Buoyancy a character walks the bed and a ragdoll sinks, as before.

**Wading and swimming.** A character with og.Buoyancy needs no mode
switch: keep setting `move`.

- Wading: on the bed it walks at `move` slowed towards `waterSpeed` (0.5)
  of itself by how much of it is under: knee deep a little, chest deep
  nearly half. It still jumps.
- Afloat: where the lift carries it off the bed it is not grounded, so it
  cannot jump; `move`'s x and z swim it along, at `waterSpeed` when fully
  under.
- `move`'s y is its stroke in the water. Down, it dives at that speed
  (times the same share) against any lift, as deep as the water goes. Up,
  it rises to where it floats and no higher; where it sinks
  (`waterDensity` under 1) a stroke up treads water with its head out, and
  it sinks again when the stroke stops. Out of the water y is ignored as
  ever.
- `waterDensity` chooses the swimmer: 2 bobs back up like cork the moment
  a dive stops; 1.05 to 1.2 hangs where it is left and drifts up slowly,
  as a person does; under 1 is armour.
- What it is doing is two written fields: wading is `grounded` with
  og.Buoyancy's `submerged` above 0; swimming is not `grounded` with
  `submerged` above 0; under water is `submerged` 1. Pick the clip, the
  splash (og.Splash) and the camera's underwater look from them.
- With `gravityScale` 0 `move` goes any way, slowed by the water the same.

**Water.** Every water answers the same three things, whichever component
draws or simulates it: an og.Water (a lake at its level, a sea with a
`wind`, a river along its course), an og.WaterSurface and an og.WaterLine.

- A body or character whose centre crosses the surface is an og.Splash,
  read the next tick: spawn the spray and the sound at its `point`, sized
  by its `speed` (negative going in).
- `og.physics.waterHeight` is the surface as it is drawn, waves and
  ripples included; outside the water it is the rest level.
- `og.physics.water` asks by place instead of by entity: the highest
  water over a point, the point's `depth` under it (negative over it), and
  the water's `velocity`, `acceleration` and `normal` there. It is null
  over no water. Use it for anything the engine does not float for you: a
  swimmer (`depth` at the chest), a float that tilts with `normal`, debris
  carried by `velocity`, a camera that goes under (`depth` at the eye).
- og.Buoyancy floats a hull, a limp ragdoll or a character on any of them.

What differs is the lift. An og.Water alone lifts only bodies with
og.Buoyancy; everything else sinks through it. og.WaterSurface and
og.WaterLine also lift every body's box by their own `buoyancy`, so a pool
of crates floats without a component on each. An og.Water drawn over a
mesh (`geometry`) is a look only: it has no height to ask for and nothing
crosses it.

Force fields. An og.ForceField accelerates what is inside the og.Volume
beside it, a heavy body as a light one, so it is the tool for gravity that
differs by place; og.WindMotor moves things by their drag instead, light
things most.

| For | Field |
|---|---|
| A low-gravity room | directional, up, `strength` under 9.81 (6 leaves about 0.4 g) |
| A jump pad or an updraft | directional, up, `strength` over 9.81, a volume a few metres tall |
| A gravity well, a magnet | radial, negative `strength`, a sphere with a `fade` |
| A whirlpool, a tornado | vortex, with a weak radial pull beside it to hold things in |

- A body a field reaches does not fall asleep. Bound a field with a volume
  and turn `enabled` off when it is not needed; a field without a volume
  keeps every body in the world awake.
- A rope, a cloth, a soft body and a fluid take the field at the middle of
  their points: a field much smaller than one moves all of it or none.
- A character controller takes the vertical part only (it is lifted and
  falls slower); push it sideways through its `move`.

## 9. Destruction

Fracture is cooked, never computed at run time (`determinism §4`):

1. Add a `fracture { }` block at the top of a `type model`, or in a glTF's
   import settings, with a `style` (brick, concrete, wood, glass). The cook
   cuts the model into Voronoi chunks bonded where they touch, and anchors
   the chunks resting on its floor to the world.
2. Give the entity that draws it an og.Destructible tuned by material:

```ts
world.insert(wall, og.Destructible, {
  ...must(og.destruction.material(world, MATERIAL_BRICK)), breakFx: dust,
});
```

3. Break it with hits, `og.destruction.damage` or `explode`; read
   og.Fractured the tick after for the sound (by `volume`), the camera shake
   and the score. Each loose chunk is an og.Chunk entity on a dynamic body:
   query it, push it, despawn it.

Islands anchored to the world stay; every other island falls as one body.
The chunks of a dynamic prop (a crate with an og.RigidBody) fly with its
velocity; a prop without a body, or with a static one, stands on its
anchors. A damage or explosion called this tick splits the prop before this
tick's physics step.

| Material | toughness (N·s/m²) | Behaves |
|---|---|---|
| glass | about 50 | shatters at a touch, no stress solver |
| wood | 300 | light, weak across the grain, splinters |
| brick | 800 | no-tension masonry, steps back over its piers |
| concrete | 2000 | strong, chunky lumps, holds what is left |
| stone | tougher than brick | dry-laid, heavier, no tension |

Stress tuning: low `strength` lets an overhang fall and brings a wall shot
out at its foot down; high holds whatever is left. `friction` lets a joint
pressed by what stands on it hold where an unloaded one slips, so a wall
keeps its loaded piers and drops what hung over the hole, cracking in steps
along its courses. Masonry and dry stone hold almost no tension (mortar
cracks long before brick; Heyman's no-tension masonry, the premise of
masonry limit analysis): at `tension` about 0.1 no block is left hanging
from the course above over air and a wall shot out at its foot steps back
course by course. At 0.1 or below (with `strength` above 0) a block also needs its weight within the middle third
of what bears it from below (a cracked bed joint bears as well as a sound
one), so a block held only from its side, or by a tie-in into the wall
beside it, comes loose whole and the blocks it bore are judged again.

Hits: above `impactThreshold` a collision's impulse is damage, and a
standing prop's broken-loose pieces take the excess along the striker's
path, so a ball through a pane carries its shards on. `DamageDesc.impulse`
zero crushes without pushing, so what it breaks out of a standing prop can
stay standing. An explosion's `impulse` is per square metre of the face a
piece shows the blast, so a lone brick flies faster than a run of bricks
but not by their count, and a run by the charge breaks up rather than
flying off whole; a demolition charge on brick is about 2500. Push bodies
that are not chunks with `og.physics.applyImpulse`.

Budgets (og.Destruction): loose islands under `debrisVolume` despawn after
`debrisLifetime`; past `maxChunks` loose chunks the oldest islands go first.
`stressBudget` bounds the solver's work a tick in bond-iterations; a tick's
worst case is the budget plus one solve (six iterations a bond). The
200-chunk brick wall's largest solve is about 41,000, under the default
200,000. Cap `stressIterations` only on a slow device, knowing an
unconverged solve can leave standing what should fall.

Looks: `breakFx` bursts where each island breaks loose and `hitFx` at each
damaging hit, each the material's own (a lit dust puff for stone and brick,
splinters and a little dust for wood, shards and glints for glass, a few
sparks only for metal). `debris` turns pieces smaller than `debrisVolume`
into GPU-only visuals: each flies with its own shape and materials from
where and how fast it broke off (the blast's push included), throwing four
to six rubble chips of the cut-face material, and every broken joint sheds
three or four small chips of its faces and mortar (`fracture.mortar`; at
most 48 joints a tick). They land on box, sphere and terrain colliders and
pass through each other and through the chunks that are bodies; the emitter
(`render mesh`, best with `blend alpha`, `collision` and `lit true`) bursts
finer grains there. They never touch the simulation, and a device without
compute draws none (`capabilities().visualDebris`).

**2D: pixel terrain.** og.PixelTerrain is Worms and Noita ground:
`carve2d` digs, `fill2d` builds (or lays bedrock, which nothing carves),
`explode2d` blows a hole and throws the ring round it as pieces, and
`fracture2d` breaks a piece (or a terrain with no bedrock) into Voronoi
pieces. Ground cut loose from the bedrock and the bottom row falls as an
og.PixelPiece; og.PixelDetached reports each. Each chunk of cells collides
as its outline smoothed within `tolerance` cells, so a body rolls along a
carved slope rather than bumping down steps. The terrain draws as one
sprite of its cells, one texel a cell, remade only on a frame after they
changed. A smaller `chunk` (default 32) rebuilds less per carve but makes
more colliders. Despawning the terrain takes its colliders with it; its
pieces stay.

## 10. Thresholds

| Quantity | Value | Source |
|---|---|---|
| A character's height | about 1.8 m | determinism §2 |
| Gravity | 9.81 m/s²; change scale, not gravity | standard gravity (CGPM 1901) |
| Object size | 0.05 to 100 m | determinism §2 |
| Bullet speed that tunnels | over about 30 m/s: `ccd` or a ray per tick | determinism §2 |
| Physics.sleepTicks | 0: never sleeps | engine default |
| Physics.sleepTicks, recommended | about 30 for new projects | engine guidance (IDL at 9ba5b0957) |
| Physics.substeps | 1; raise for stacks, chains, fast bodies | engine default |
| Car mass | 1,000 kg | what the Vehicle defaults are sized for (§7) |
| CharacterController.stepHeight | 0.3 m; 0 for no stairs | engine default |
| PlatformerController coyote / buffer | 0.1 s / 0.12 s | engine defaults |
| Fluid3D particles | 2,000 cost about 0.9 ms CPU a tick at 1 substep | measured on an Apple M4 Pro (§8) |
| Destruction.maxChunks | 256 | engine default |

## 11. Pitfalls

- A dynamic body with no mass set and a big box collider weighs its
  density in kg/m³ (1 by default): cars fly, crates feel like foam. Set
  `mass`.
- A triangle-mesh collider on a dynamic body: use a convex hull or a
  compound of primitives instead.
- Moving a kinematic platform by writing its transform: it teleports and
  carries nothing. Set its og.Velocity.
- Reading a collision's hardness from og.Velocity in the next tick: the
  bodies are already parting. Use `impulse` or `relativeVelocity`.
- Expecting a query to see a body spawned this tick.
- Damaging `hit.entity` of a compound child instead of `hit.body`.
- A character that cannot climb a kerb: `stepHeight` must be at least the
  kerb's height over the ground it stands on.
- Game logic that reads a `visual` cloth or fluid: it reads the
  rest shape, or nothing.
- A garment with `radius` above 0 snagging on the character's own collider.
- Destructible props scaled non-uniformly: chunks take the scale's mean.
