# Lantern Vale's physics playtest: the pile no longer moves

`examples/lantern-vale/tests/physics.og.ts` failed "the middle rock was
pushed off its spot": `rock-4` moved 0.085 m between the 0.5 s and 2.6 s
dumps, where the check wanted more than 0.2 m.

## Bisect

The first bad commit is 8f226ceb, "physics: the platformer pushes with
pushBodies and a pushStrength both controllers share (brasswick #6)". I
confirmed it with one run on each side, using the playtest's own script
through `og run --headless --no-render`:

| commit | rock-4 before | rock-4 after | moved |
| --- | --- | --- | --- |
| 9fe3a417 (8f226ceb's parent) | [4.5010905, -0.26530266, 9.997277] | [4.85486, 0.4185434, 7.479753] | 2.5423 m |
| 8f226ceb | [4.5010905, -0.26530266, 9.997277] | [4.4879384, -0.18549962, 9.913633] | 0.0847 m |
| b80aa70c (main) | the same | the same | 0.0847 m |

The previous attempt's numbers (64c78455 2.7917 m, 54e6ef79 2.5423 m,
0.0847 m from 8f226ceb on) agree. 54e6ef79 is not an ancestor of
8f226ceb, so the tight pair is 8f226ceb against its parent.

## Cause

Before 8f226ceb, `ext::push` gave each body the hero walked into an
impulse of 0.8 × its mass × the speed into it, every tick, without any
cap. In effect the push force had no limit: any mass was shoved at
walking speed. brasswick #6 shows the result, a 1,000 kg crate moving at
6.4 m/s. 8f226ceb caps the bodies of one tick at `pushStrength × dt`
(default 1000 N, 16.7 N·s per tick at 60 Hz).

Lantern Vale's rocks are density-900 spheres of radius 0.42, so 279 kg
each. The hero aims at the 0.21 m gap between the front pair, rock-6 and
rock-7. Under the cap it wedges there: in two seconds it parts them by
0.39 m and stops at z 10.56. It reaches rock-4 only as the key is
released, and holds it a little (0.085 m, against 0.10 m when nobody
walks in). Under the old push, the hero bulldozed through to z 7.63 and
threw rock-4 2.54 m and 0.68 m up, onto the pile.

The previous attempt explained the stop with sliding friction
(μmg ≈ 2466 N against 1000 N). That does not apply here: a sphere pushed
at its centre rolls, and nothing static holds it. What holds the rocks is
the pile's combined mass and the wedge. I confirmed the result by tracing
the positions every 0.2 s.

## Decision: (b), the example was too heavy, and the engine is right

- **The old expectation was wrong.** The old 2.5 to 2.8 m was the
  uncapped push treating any mass as weightless. The same rule is
  brasswick #6's defect, which 8f226ceb fixed on purpose and which
  `crates/og-physics/tests/controllers.rs` covers (2 kg shoved, 100 kg
  creeps, 1,000 kg holds). Its end state, a rock tossed 0.68 m into the
  air by a walk, is not a shove a player believes.
- **The formula is a force cap, the convention the main engines use.**
  - Unreal's CharacterMovementComponent pushes with a force
    (`PushForceFactor`, `MaxTouchForce`), not scaled to mass unless
    `bPushForceScaledToMass` is set.
  - Godot's documented CharacterBody recipe applies a fixed
    `push_force` impulse.
  - Unity's CharacterController does not push. Its manual's
    `OnControllerColliderHit` example sets the body's velocity and
    ignores mass, which is the old rule.
  - Rapier's `solve_character_collision_impulses` takes a
    `character_mass` and solves an inelastic contact each tick. Because a
    kinematic character keeps its velocity, that impulse comes back every
    tick, so the sustained force has no bound and depends on the frame
    rate. It is the old defect with a smaller constant.
  - A walking character's push is limited by its traction (μ·m·g, about
    800 to 1000 N for 80 to 100 kg on μ ≈ 1), not by its speed or by the
    mass in front of it.

  So the per-tick budget is the right shape. Scaling it with a controller
  mass would need a field the controllers do not have, and `pushStrength`
  already is that knob, in newtons.
- **The 1000 N default is a strong walker,** above the 300 to 600 N a
  person sustains. og-game's SKILL.md promises the numbers it gives
  ("2 kg is shoved at the run speed, 100 kg creeps, a tonne on the ground
  holds") and tells a game to set masses. docs/design.md promises nothing
  about pushing. Nothing in the docs makes a 279 kg rock shove-light.
- **Mass or pushStrength.** The two fixes are the same simulation.
  Gravity, friction and contact impulses all scale with mass, and damping
  does not depend on it. The idle pile settles to identical positions at
  density 150 and 900, so the slump does not depend on the mass. The
  choice is therefore about meaning. The lamplighter is a person, and the
  engine's walking-character default is already generous. The rocks'
  density was never tuned, because under the old push it had no effect.
  So the value to fix is the rocks' density.

## Fix

- `examples/lantern-vale/src/world.ts`: the pile's rocks are density 150
  (46 kg each), with a comment saying why.
- `examples/lantern-vale/tests/physics.og.ts`: the checks describe what a
  player sees. I found that the pile slumps by itself: its top rocks
  roll off, and `Game.rockMoved` reaches 0.60 m by 2.6 s with nobody near
  (the same at both densities). The old "the pile was shoved"
  (`rockMoved > 0.5`) therefore passed without any shove, so I replaced
  it. The checks now measure from the moment the hero sets off:
  - the front two rocks parted: their gap widened by more than 0.5 m;
  - the middle rock rolled off its spot by more than 0.5 m (it was
    0.2 m);
  - it rolled along the ground and was not thrown onto the pile: it rose
    less than 0.3 m;
  - the hero walked on into the pile: its z is below 10.2 at 2.6 s;
  - the trunk check is unchanged.

No engine change, so no engine test. The other users of `pushBodies` and
`pushStrength` behave as they do on main: `templates/archetypes/topdown`
sets `pushBodies: false`, brasswick's platformer uses the default (its
FINDINGS #6 is the bug 8f226ceb fixed), and fx-showcase does not push.

## Numbers (the playtest's script, `og run --headless --no-render`)

| run | front gap widened | rock-4 moved | rock-4 rose | hero z at 2.6 s | trunk z |
| --- | --- | --- | --- | --- | --- |
| this branch (density 150) | 1.106 ✓ | 1.1739 ✓ | 0.029 ✓ | 9.663 ✓ | 4.508 ✓ |
| main (density 900) | 0.393 ✗ | 0.0847 ✗ | 0.080 ✓ | 10.555 ✗ | 4.508 ✓ |
| idle, no KeyW (density 150) | -0.099 ✗ | 0.1009 ✗ | 0.115 ✓ | 13.500 ✗ | 4.508 ✓ |
| 9fe3a417, the uncapped push (density 900) | 1.227 ✓ | 2.5423 ✓ | 0.684 ✗ | 7.629 ✓ | 4.508 ✓ |

On this branch, rock-4 goes from [4.501089, -0.2653028, 9.997275] to
[4.680179, -0.2364394, 8.837095].

**Determinism.** Two runs of the playtest's script give byte-identical
dumps (`diff -r` empty). Two runs of replay.og.ts's script end on the
same hash, `tick 576 hash 6cc33b60f0e7fd02`, with lit 4, hurts 2,
rockMoved 1.33, farLoads 1 and look 1, so its checks hold. The rocks'
rest positions at 0.5 s move by about 1e-6 m (float rounding of the
scaled masses), and no reference frame shows a shoved pile, so no frame
should move. I did not re-record any frames.

## Files touched

- `examples/lantern-vale/src/world.ts`
- `examples/lantern-vale/tests/physics.og.ts`
- `tests/evidence/debt/lantern-vale-physics/NOTES.md`

## Left out

- **The pile's own slump.** The top rocks sit over 0.21 m gaps between
  bottom rocks that do not touch, and they roll off in the first seconds
  of a session, before the player gets there. Fixing the layout
  (bottom rocks touching, or the top ones removed) changes the look and
  its metal reference frames, which this orb cannot record.
- **replay.og.ts's "shoved the pile" check (`rockMoved > 0.3`)** is just
  as vacuous because of the slump. I left it alone: that test is about
  determinism, and its hash covers the shove.
- **`examples/brasswick/src/hero.ts`** still has a comment from before
  8f226ceb ("has no pushBodies"), and its `pushSpeed` workaround is still
  in place.
- **The budget goes to the first hit.** `ext::push` gives each tick's
  budget to the first body in hit order, so when the hero is wedged
  between two rocks, one rock takes the whole push that tick. This is
  deterministic and was not the cause here. I noted it rather than
  changed it.
- **Environment.** In this orb, a rendered lantern-vale run crashes bun
  (SIGSEGV, also on `0.1:quit`, while templates/game renders). Every
  number above is from `--no-render` runs. Rendering never feeds into the
  simulation, so the physics numbers are the same. I did not run og test
  or the gate (stream rules).
