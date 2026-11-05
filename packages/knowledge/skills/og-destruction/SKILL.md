---
name: og-destruction
description: Props that break on Origin Engine with og.destruction — walls, crates, pillars, glass panes and small structures prefractured by the cook (`fracture { }`: a fracture profile per material — brick along its mortar, chunky concrete, wood splintering along its grain, glass cracking radially from a blow, plain Voronoi — pieces each chunk shatters into, an interior material with its own maps for the cut faces), made breakable with og.Destructible (toughness, strength, friction, density, impact threshold), broken by hits, og.destruction.damage, explode and fracture, brought down by NVIDIA Blast's stress solver when their support is shot away, with og.Fractured for sound and camera shake, each material's break and hit fx (dust, splinters, glass shards that glint) and GPU debris through particle emitters, and budgets for loose chunks. Use when anything in a game should shatter, splinter, crumble or collapse.
---

A breakable prop on Origin Engine is a model the cook has already cut
into chunks and a component that says what it is made of. The game never
cuts meshes, makes bodies or tracks bonds: it gives the model a
`fracture` block, puts `og.Destructible` on the entity that draws it,
and hits it. The API is `og api show destruction`; this skill is how to
use it. Bodies, colliders and collision events are og-physics; emitters
are og-render's particles.

- §1 Cutting the model: the `fracture` block.
- §2 Making it breakable: og.Destructible's numbers by material.
- §3 Breaking it: hits, damage, explosions, the stress solver.
- §4 What the game sees: og.Fractured, og.Chunk, info.
- §5 The look and the budgets: break and hit fx per material, GPU debris, despawning.

## 1. Cutting the model: the `fracture` block

Add a `fracture` block at the top level of a `type model` (og-model §9),
or in a glTF's import settings (`models/barrel.glb.ogt`). `og cook` then
cuts the model into chunks bonded where they touch, by the `style` of
its material, writes the model with one node per chunk and writes
`<model>#fracture`, which the engine loads with the model.

```
type model
version 1
---
material "brick" {
  color 1 1 1
  roughness 0.9
  map "../textures/brick.png"       # courses and mortar, a tile a metre
  normalMap "../textures/brick_n.png"
}
part "wall" {
  material "brick"
  position 0 1.2 0
  boxGeometry {
    width 4
    height 2.4
    depth 0.3
    uvRepeat 4 2.4       # the texture once a metre, so its mortar meets the cracks
  }
}
fracture {
  style brick          # voronoi | concrete | brick | wood | glass (below)
  brick 0.25 0.1       # brick: a brick's length and height, as the texture draws them
  chunks 200           # support chunks, 2 to 256: what breaks loose and bears load
  seed 11              # the same seed always makes the same chunks
  children 32          # pieces each chunk shatters into when hit hard (0 to 32)
  anchor bottom ends   # bottom (rests on the world) and/or ends (tied in at both
                       # ends of its long axis, as a wall between two others) | none
  interiorScale 2      # the cut faces' uv repeats a metre
  interior {           # the cut faces' material: og-model §9's keys, maps included
    color 1 1 1
    roughness 1
    map "../textures/brick_cut.png"
  }
}
```

Pick the style by what it is made of; each breaks as that material does:

| style | breaks into | keys | use for |
|---|---|---|---|
| `brick` | runs of one to three whole bricks along a course, laid in the bond so every joint is staggered, each falling apart into its bricks | `brick w h` (0.25 0.1) | brick and block walls, stone courses, tiles |
| `concrete` | chunky, rounded lumps (cells relaxed twice), crumbling into smaller lumps | | concrete, rock, plaster, a statue |
| `wood` | long splinters along the grain | `grain x\|y\|z` (the longest axis) | crates, planks, beams, furniture |
| `glass` | wedge shards radiating from the blow, crossed by `rings` concentric cracks | `rings` 1 to 8 (2), `impact` the blow | panes, windows, mirrors, ice sheets |
| `voronoi` | even Voronoi cells | `pattern`, `impact` | anything else |

- Chunk counts by prop: a crate 8 to 16, a pane 20 to 30. For `concrete`
  and `voronoi`, give the prop about as many cells along its length as
  fit its thickness: a pillar 0.5 m across and 3 m tall wants about 2 ×
  12 × 2, 40 to 50, where 16 cut it into thin discs that fall as one
  drum. A
  `brick` wall's count is its bricks over two (a 4 × 2.4 m wall of
  0.25 × 0.1 bricks, about 400 bricks: 200); its blocks never run past
  three bricks or one course. More chunks cost more bodies when it falls.
- `children` gives large chunks something to crumble into: a pillar 4 to
  6, a brick wall any count (its blocks' bricks, up to 32), a crate or a
  pane 0 (they are already small).
- `impact x y z r`: where it will be hit most (model space). The glass
  cracks radiate from it; other styles cut finer within `r` of it.
- `anchor none` for anything that moves (a crate, a barrel): nothing
  holds it to the world. `anchor bottom ends` for a wall between two
  others: shot out at its foot, its ends stay as piers.
- Make the outside read as its material (a brick, plank or concrete map
  and normal map) and give the cut faces their own: brick paler, dustier
  and rougher inside; concrete grey with its gravel showing (an
  aggregate map); wood lighter, with the grain across it; glass edges
  greener and catching the light (a faint `emissive`). Without
  `interior` it is the main material, darker and rough. Clear glass is
  `transmission 1` with `ior 1.5` and `roughness` near 0, not a low
  `opacity`: it refracts what lies behind it and reflects the sky. In a
  stylised look (og-taste) the interior takes the look's treatment like
  every other material: keep it within the look's palette.
- Scale the entity uniformly; a stretched prop's chunks collide as the
  mean of its axes.

## 2. Making it breakable: og.Destructible

```ts
const wall = must(world.instantiate(must(og.load.gltf(world, "props/wall.ogt")), { position: { x: -3, y: 0, z: 0 } }));
must(world.insert(wall, og.Destructible, { toughness: 800, strength: 3000, tension: 0.1, density: 1900, impactThreshold: 400, breakFx: brickDust, hitFx: brickChips, debris: chips }));
```

A standing prop (no og.RigidBody, or a static one) is its chunks from the
first tick: each a static convex collider on an entity with og.Chunk, and
the entity's own og.Collider and og.RigidBody are removed. A moving prop
(a dynamic crate) keeps its own body until its first break; its chunks
then fly on with its velocity. The model draws whole until the first
break, then as its chunks.

| material | toughness (N·s/m²) | strength (Pa) | tension | density (kg/m³) | impactThreshold (N·s) |
|---|---|---|---|---|---|
| glass | 25 to 60 | 0 (no stress) | 1 | 2500 | 2 |
| wood | 120 to 400 | 0 to 3000 | 1 | 600 | 30 to 50 |
| brick, stone | 600 to 1000 | 2000 to 5000 | 0.1 | 1900 | 300 to 500 |
| concrete | 1500 to 2500 | 15000 to 30000 | 1 | 2400 | 60 to 150 (150 where flying bricks reach it) |

- `toughness`: how much a square metre of bond takes. A blow's `amount`
  (N·s) over a bond's area times toughness is the share of its health it
  loses.
- `strength`: what the stress solver lets a bond carry in tension and
  shear (thirty times it in compression). Low, what hangs over a hole
  comes down; high, it hangs on whatever is left. 0 turns the solver off
  (glass, a free crate).
- `tension` (default 1): what a bond holds pulled apart, as a fraction
  of `strength`. Mortar cracks long before brick does, so masonry stands
  by its weight and friction (no-tension masonry, the premise of masonry
  limit analysis): at 0.1 a block hanging from the course above over
  air falls, and so does one resting on the corner of the block below
  with its weight past the edge (each joint is judged at both its edges,
  so a load far off a joint's middle opens it). A wall shot out at its
  foot then steps back course by course over its piers.
- `friction` (default 0.7): shear a bond gains from the compression
  across it. Joints pressed by what stands on them hold and unloaded
  ones slip, so a brick wall shot at its foot drops what hung over the
  hole in steps along its courses and keeps its loaded piers. A loose
  island at rest is solved on what it rests on too: a run of blocks lying
  across two piers bends and breaks over the gap, and one wedged at rest
  with nothing under it (blocks jammed between the piers into a flat
  arch) crumbles into its pieces 30 ticks after it broke free. A charge
  that takes the wall's middle to its top leaves two toothed piers and a
  pile; a smaller one leaves a sound arch of brickwork over the hole, as
  real walls do (the 4 × 2.4 m wall needs `radius` about 1.8 and
  `innerRadius` 1 at its foot). 0 makes
  every joint as weak as the next (a Voronoi prop of no grain).
- `impactThreshold`: the collision impulse (og.CollisionStarted.impulse)
  that does nothing; what is above it is the hit's damage, spread over
  `impactRadius`. A bullet body, a thrown rock and a falling crate all
  break things this way, with no game code. The pieces a hit breaks off
  a standing prop take the impulse above the threshold along the
  striker's path (a ball through a pane carries its shards on); a moving
  prop's pieces keep what its hit left it (a dropped crate's settle). Raise it with the weight of
  the pieces: a falling run of bricks lands at a hundred N·s or more, and
  below that every brick that drops knocks apart what it lands on.
- `anchored false`: every island falls (a structure knocked off its
  base).

## 3. Breaking it

```ts
og.destruction.damage(world, wall, { point, radius: 0.6, innerRadius: 0.2, amount: 3000, impulse: { x: 0, y: 0, z: -300 } });   // a shot, a hammer: impulse is its momentum, which what it breaks loose carries on
og.destruction.explode(world, { point, radius: 3, innerRadius: 0.5, damage: 6000, impulse: 400 }); // every prop in reach, and a push on loose chunks
og.destruction.fracture(world, wall);   // every bond at once: the anchored chunks stay, the rest fall
```

- Calls queue for this tick and apply after the game's systems, before
  physics, in the order made: a bond broken now splits the prop before
  this tick's physics step.
- `damage` takes the prop or any of its chunks (the entity a raycast
  hit), so a weapon never looks up the owner.
- `explode` pushes only loose chunks; push other bodies with
  og.physics.applyImpulse.
- A chunk with children that takes three times what breaks a bond of its
  size shatters: its pieces fly free, each its own body, and leave a hole.
- The stress solver (NVIDIA Blast's) runs on the anchored islands after
  damage: what can no longer carry its weight breaks over the next ticks,
  so a collapse plays out instead of popping.

Everything is fixed arithmetic in the tick, and the bonds, chunks and
queued calls are world state: snapshots, restores, replays and lockstep
(og-net) all cover a fracture. Never break props from presentation code
(a render callback, a frame-rate timer).

## 4. What the game sees

```ts
for (const f of world.events(og.Fractured)) {
  // f.entity the prop, f.body the island's body, f.chunks, f.point, f.volume, f.mass, f.cause (CAUSE_*)
  playCrash(f.point, Math.min(1, f.mass / 200));
  if (f.mass > 500) shake(0.3);
}
```

- Read og.Fractured the tick after, like every event. `cause` is
  CAUSE_IMPACT, CAUSE_DAMAGE, CAUSE_EXPLOSION, CAUSE_STRESS (a collapse),
  CAUSE_SHATTER or CAUSE_FRACTURE.
- og.Destructible's written fields: `fractured`, `broken` (bonds),
  `loose` (chunks broken free and alive). Score on them, open a path
  when `loose` passes a count.
- og.Chunk on each chunk: `destructible`, `chunk`, `level` (1 support, 2
  a shattered piece), `body` (the entity whose og.RigidBody moves it), and
  `looseTick`. Despawn a chunk to clear it away.
- `og.destruction.info(world, e)`: chunks, bonds, intact bonds, islands,
  anchored chunks and `health` (1 intact to 0 rubble), for a HUD or a
  test.

## 5. The look and the budgets

- `breakFx` bursts where each island breaks loose, `hitFx` at each
  damaging hit. Both are one-shot emitters (`looping false`, `rate 0`,
  one `burst`), and each material wants its own pair, never one set
  for every prop:

  | material | breakFx | hitFx |
  |---|---|---|
  | brick | a warm clay puff (alpha, lit) | clay chips (`render mesh tetrahedron`, `lit true`, `collision`) |
  | concrete | a grey puff | grey grit chips |
  | wood | a little pale sawdust | splinters (small brown tumbling meshes) |
  | glass | small shards nearly clear and smooth (lit meshes, `blend alpha` at an alpha about 0.2, `roughness 0.04`, `collision`): each glints only where it faces the sun or the sky, never as white salt | the same shards |
  | metal | none | a few sparks (additive, `align velocity`, `collision`), not a shower |

  Sparks (orange, additive, bright) read as metal, a grinder or a
  weld: on stone, wood or glass they look wrong. `examples/demolition-yard`
  has an emitter for each row but metal under `assets/fx/`.
- `debris`: fine pieces as visuals only. A mesh-particle emitter
  (`render mesh tetrahedron`, `lit true`, `collision`) burst where a
  chunk shatters, in place of its pieces smaller than
  og.Destruction.debrisVolume, which then never become bodies: hundreds
  of chips cost no physics. It runs on the GPU and never touches the
  simulation; `og.destruction.capabilities(world).visualDebris` says
  whether this device draws it, and the bodies are the same either way.
- og.Destruction budgets the world: `maxChunks` (256) loose chunks at
  once, the oldest islands despawning first; islands smaller than
  `debrisVolume` despawn `debrisLifetime` seconds after breaking loose;
  bursts despawn after `fxLifetime`. Lower maxChunks on a phone, raise it
  for a set piece.

`examples/demolition-yard` has a wall, a pillar, a crate and a pane, with
commands that break each one (`boom`, `strike`, `drop`, `throw`).
