// og.systems.interact (README.md): two rooms on one navmesh, split by a
// wall at x 9–11 and joined only by a door's off-mesh link, shut at the
// start. The player stands in the west room and turns between four
// headings (D): the door to the east (a 30-tick hold), a lever to the
// north, a lever to the west behind a pillar, and a crate to the south that
// it picks up and throws onto a pressure plate, which opens a gate. A guard
// in the east room walks to the west room once the door is open. Every
// number is here; the engine's systems carry them out.
//
//   og run tests/fixtures/interact-house --commands "0.5:key_down:KeyE;1.2:key_up:KeyE;4:quit"
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** What the player faces in turn (D steps on): the door, the lever, the hidden lever, the crate. */
export const HEADINGS = [Math.PI / 2, 0, -Math.PI / 2, Math.PI];
/** The door's hold, in ticks at 60 Hz. */
export const HOLD = 30;
/** The door's link on the navmesh. */
export const LINK = 1;
/** The names `focus` counts by, from 1 (0: no focus). */
export const NAMES = ["door", "lever", "hiddenLever", "crate"];

/** What the fixture counted. */
const House = defineResource("House", {
  /** Index into HEADINGS. */
  facing: u32(),
  interacted: u32(),
  linkReached: u32(),
});

export function build(world: World): void {
  must(og.input.defineAction(world, { name: "use", bufferTicks: 4, bindings: [{ control: "KeyE" }] }));
  must(og.input.defineAction(world, { name: "turn", bindings: [{ control: "KeyD" }] }));

  // The navmesh: both rooms, the wall between them blocked whole.
  const nav = must(
    og.nav.build(world, {
      min: { x: 0, y: 0, z: 0 },
      max: { x: 20, y: 0, z: 10 },
      blocked: [{ min: { x: 9, y: 0, z: 0 }, max: { x: 11, y: 2, z: 10 } }],
      cellSize: 0.25,
      agentRadius: 0.4,
      tileSize: 32,
    }),
  );
  must(og.nav.setLink(world, nav.mesh, LINK, { start: { x: 8.2, y: 0, z: 5 }, end: { x: 11.8, y: 0, z: 5 }, kind: "door", open: false }));

  const floor = must(world.spawn({ name: "floor", position: { x: 10, y: -0.5, z: 5 } }));
  must(world.insert(floor, og.Collider, { shape: "box", size: { x: 40, y: 1, z: 40 } }));

  const player = must(world.spawn({ name: "player", position: { x: 6, y: 0, z: 5 }, quaternion: og.transform.fromHeading(HEADINGS[0]!) }));
  must(world.insert(player, og.Interactor, { action: "use", range: 3, cone: 0.6, sight: true, rule: "centred", eye: { x: 0, y: 1, z: 0 } }));

  const door = must(world.spawn({ name: "door", position: { x: 8.6, y: 1, z: 5 } }));
  must(world.insert(door, og.Interactable, { holdTicks: HOLD }));
  must(world.insert(door, og.Door, { mesh: nav.mesh, link: LINK }));

  const lever = must(world.spawn({ name: "lever", position: { x: 6, y: 1, z: 7.5 } }));
  must(world.insert(lever, og.Interactable, {}));
  must(world.insert(lever, og.Toggle, {}));

  // A lever 2.5 m west, behind a pillar: in range and in the cone, out of sight.
  const hidden = must(world.spawn({ name: "hiddenLever", position: { x: 3.5, y: 1, z: 5 } }));
  must(world.insert(hidden, og.Interactable, {}));
  must(world.insert(hidden, og.Toggle, {}));
  const pillar = must(world.spawn({ name: "pillar", position: { x: 4.6, y: 1, z: 5 } }));
  must(world.insert(pillar, og.Collider, { shape: "box", size: { x: 0.4, y: 2, z: 1 } }));

  const crate = must(world.spawn({ name: "crate", position: { x: 6, y: 0.25, z: 3 } }));
  must(world.insert(crate, og.RigidBody, { kind: "dynamic", linearDamping: 2 }));
  must(world.insert(crate, og.Collider, { shape: "box", size: { x: 0.5, y: 0.5, z: 0.5 } }));
  must(world.insert(crate, og.Interactable, {}));
  must(world.insert(crate, og.Grab, { carry: { x: 0, y: 1, z: 0.8 }, throwSpeed: 8 }));

  // The pressure plate the crate is thrown onto, and the gate it opens.
  const plate = must(world.spawn({ name: "plate", position: { x: 6, y: 0.1, z: 1 } }));
  must(world.insert(plate, og.Collider, { shape: "box", size: { x: 3, y: 0.2, z: 2 }, sensor: true }));
  must(world.insert(plate, og.Toggle, { plate: true }));
  const gate = must(world.spawn({ name: "gate", position: { x: 1, y: 1, z: 1 } }));
  must(world.insert(gate, og.Door, { toggle: plate }));

  const guard = must(world.spawn({ name: "guard", position: { x: 16, y: 0, z: 5 } }));
  must(world.insert(guard, og.NavAgent, { nav: nav.mesh, target: { x: 4, y: 0, z: 8 }, seeking: true, speed: 4, acceleration: 40, stopDistance: 0.2 }));
}

/** The game's turn: D steps the player to the next heading. */
const Turn = defineSystem("Turn", { reads: [], writes: [og.Transform, House], before: ["og.interactors"] }, (_q, _t, world) => {
  if (!og.input.isActionJustPressed(world, "turn")) return;
  const house = must(world.resource(House));
  house.facing = (house.facing + 1) % HEADINGS.length;
  must(world.set(world.find("player")!, og.Transform, { quaternion: og.transform.fromHeading(HEADINGS[house.facing]!) }));
});

/** What og.systems.interact and og.nav reported. */
const Count = defineSystem("Count", { reads: [og.Interacted, og.NavLinkReached], writes: [House], after: ["og.interact"] }, (_q, _t, world) => {
  const house = must(world.resource(House));
  for (const _ of world.events(og.Interacted)) {
    house.interacted += 1;
    og.assert.reachable("something is used");
  }
  for (const r of world.events(og.NavLinkReached)) {
    if (r.link === LINK) house.linkReached += 1;
    og.assert.reachable("the guard goes through the door");
  }
});

/** What the fixture counted, for `og test`'s `state:` and interact.test.ts. */
export function facts(world: World) {
  const h = must(world.resource(House));
  const me = must(world.get(world.find("player")!, og.Interactor));
  const name = (e: Entity) => NAMES.findIndex((n) => world.find(n) === e) + 1;
  const crate = must(world.get(world.find("crate")!, og.Transform)).position;
  const guard = must(world.get(world.find("guard")!, og.Transform)).position;
  return {
    focus: name(me.focus),
    progress: me.progress,
    holding: me.holding ? 1 : 0,
    interacted: h.interacted,
    doorOpen: must(world.get(world.find("door")!, og.Door)).open ? 1 : 0,
    leverOn: must(world.get(world.find("lever")!, og.Toggle)).on ? 1 : 0,
    hiddenOn: must(world.get(world.find("hiddenLever")!, og.Toggle)).on ? 1 : 0,
    plateOn: must(world.get(world.find("plate")!, og.Toggle)).on ? 1 : 0,
    gateOpen: must(world.get(world.find("gate")!, og.Door)).open ? 1 : 0,
    linkReached: h.linkReached,
    guardWest: guard.x < 9 ? 1 : 0,
    crateX: crate.x,
    crateY: crate.y,
    crateZ: crate.z,
  };
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [House],
  systems: [Turn, ...og.systems.interact, Count],
  start: build,
  facts,
});
