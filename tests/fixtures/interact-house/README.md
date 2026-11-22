# Interact house

og.systems.interact on the real stack: two rooms on one navmesh, split by
a wall at x 9–11 and joined only by a door's off-mesh link (shut at the
start). The player (og.Interactor: 3 m, a 0.6 rad cone, sight, the centred
rule, E with a 4-tick buffer) turns between four headings with D:

- east, the door (a 30-tick hold): held 12 ticks nothing happens; held 30
  it opens and og.nav.setLinkOpen opens its link, and a guard in the east
  room (og.NavAgent) walks through to the west room (og.NavLinkReached);
- north, a lever (og.Toggle), flipped on a press;
- west, a lever behind a pillar: in range and in the cone, never focused;
- south, a crate (og.Grab on a dynamic body): picked up, carried, and
  thrown onto a pressure plate (an og.Toggle on a sensor), which opens a
  gate (an og.Door following it).

Every number is in `src/main.ts`.

Readers: its `tests/house.og.ts` (the whole walk, two runs on one hash)
and `packages/engine/tests/interact.test.ts` (the hold to the tick, the
three rules and sight from one place, a restore while carrying).

Inputs: og-runtime, og-play, og-input, og-physics, og-nav, packages/engine.
