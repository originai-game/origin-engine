// Shared Walk: the og.shared demo. Each visitor is a capsule on a grey
// field; every other visitor near it is drawn as another capsule, moved by
// the engine. Without keys held the visitor walks a circle of its own, so two
// headless clients (`og run --shared`) see each other move; WASD steers.
// A gate in the middle is a shared object: E (or every few seconds, without
// keys) opens or shuts it for everyone, red shut and green open.
// The host supplies the connection: this file names no server and no token.
import { Entity, defineComponent, defineResource, defineSystem, math, og, u32, f32, f64 } from "@origingame/engine";
import type { Geometry, Material, World } from "@origingame/engine";

/** What every visitor shares besides its transform: its colour, one of eight. */
export const Look = defineComponent("Look", { hue: u32(0, { min: 0, max: 7 }) });

/** The shared gate's state: every visitor near it sees the same. */
export const Gate = defineComponent("Gate", { open: u32(0, { min: 0, max: 1 }) });
const GATE = { x: 0, y: 1, z: 0 };

/** This visitor: its entity, where its circle is centred and how far round it is. */
const Me = defineResource("Me", { entity: Entity(), geometry: f64(0), m0: f64(0), m1: f64(0), m2: f64(0), m3: f64(0), m4: f64(0), m5: f64(0), m6: f64(0), m7: f64(0), cx: f32(0), cz: f32(0), angle: f32(0), placed: u32(0), seen: u32(0), corrected: u32(0), box: f64(0), gate: Entity(), gateVersion: u32(0), gateOpen: u32(0), useT: f32(0), held: u32(0), taken: u32(0), refused: u32(0) });

const HUES = [
  { r: 0.9, g: 0.3, b: 0.2 }, { r: 0.2, g: 0.6, b: 0.9 }, { r: 0.3, g: 0.8, b: 0.3 }, { r: 0.9, g: 0.8, b: 0.2 },
  { r: 0.7, g: 0.3, b: 0.9 }, { r: 0.2, g: 0.8, b: 0.8 }, { r: 0.9, g: 0.5, b: 0.1 }, { r: 0.9, g: 0.9, b: 0.9 },
];
const SPEED = 3.5; // m/s, under the server's 4 m/s walk
const RADIUS = 6;

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** A stable small number from a player id: where its circle is and its colour. */
function hash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619);
  return h >>> 0;
}

const SLOTS = ["m0", "m1", "m2", "m3", "m4", "m5", "m6", "m7"] as const;

/** The capsule in a colour: the geometry and the eight materials made at start, kept in Me. */
function capsule(world: World, hue: number): { geometry: Geometry; material: Material } {
  const me = must(world.resource(Me));
  return { geometry: me.geometry as unknown as Geometry, material: me[SLOTS[hue % SLOTS.length]!] as unknown as Material };
}

function start(world: World): void {
  must(og.shared.shape(world, { avatar: Look, object: Gate }));
  const res = must(world.resource(Me));
  res.geometry = must(og.render.capsuleGeometry(world, { radius: 0.4, height: 1 })) as unknown as number;
  HUES.forEach((color, i) => { res[SLOTS[i]!] = must(og.render.standardMaterial(world, { color, roughness: 0.6 })) as unknown as number; });
  res.box = must(og.render.boxGeometry(world, { width: 4, height: 2, depth: 0.4 })) as unknown as number;
  for (const [name, key] of [["left", "KeyA"], ["right", "KeyD"], ["up", "KeyW"], ["down", "KeyS"], ["use", "KeyE"]] as const) {
    must(og.input.defineAction(world, { name, kind: "button", bindings: [{ control: key }] }));
  }
  const ground = must(world.spawn({ name: "field" }));
  world.insert(ground, og.Mesh, {
    geometry: must(og.render.planeGeometry(world, { width: 200, height: 200 })),
    material: must(og.render.standardMaterial(world, { color: { r: 0.45, g: 0.5, b: 0.45 }, roughness: 1 })),
  });
  world.insert(ground, og.Transform, { position: { x: 0, y: 0, z: 0 }, quaternion: og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2) });

  const me = must(world.spawn({ name: "me" }));
  world.insert(me, og.Transform, { position: { x: 0, y: 0.9, z: 0 } });
  world.insert(me, og.SharedAvatar, { kind: 0 });
  world.insert(me, Look, { hue: 7 });
  world.insert(me, og.Mesh, capsule(world, 7));
  res.entity = me;

  const cam = must(world.resource(og.Camera));
  cam.position = { x: 0, y: 18, z: 22 };
  cam.quaternion = og.transform.lookAt(cam.position, { x: 0, y: 0, z: 0 });
  cam.fov = 50;

  must(og.shared.join(world, { world: "main" }));
}

/** Once joined, take a circle and a colour from the player id (a teleport there); then walk. */
const Walk = defineSystem("Walk", { reads: [og.Input, og.SharedJoined, og.SharedCorrected], writes: [Me, og.Transform, og.Mesh, Look] }, (_q, time, world) => {
  const me = must(world.resource(Me));
  for (const j of world.events(og.SharedJoined)) {
    if (j.resumed && me.placed) continue;
    const h = hash(j.player);
    me.cx = ((h & 0xff) / 255 - 0.5) * 16;
    me.cz = (((h >> 8) & 0xff) / 255 - 0.5) * 16;
    me.angle = (((h >> 16) & 0xff) / 255) * Math.PI * 2;
    const hue = (h >> 24) % 7;
    world.set(me.entity, Look, { hue });
    world.insert(me.entity, og.Mesh, capsule(world, hue));
    must(og.shared.teleport(world, { x: me.cx + math.cos(me.angle) * RADIUS, y: 0.9, z: me.cz + math.sin(me.angle) * RADIUS }));
    me.placed = 1;
    me.useT = ((h >> 4) % 40) / 10;
  }
  for (const _ of world.events(og.SharedCorrected)) me.corrected++;
  if (!me.placed) return;
  const t = must(world.get(me.entity, og.Transform));
  const dt = time.dt;
  // The gate: E, or every 4 s without keys. Expect the version this visitor
  // saw, so two visitors pushing it at once can't both win.
  me.useT += dt;
  const use = og.input.actionValue(world, "use") > 0.5;
  const pressed = use && !me.held;
  me.held = use ? 1 : 0;
  if (pressed || me.useT > 4) {
    me.useT = 0;
    must(og.shared.put(world, "gate", Gate, { open: me.gateOpen ? 0 : 1 }, { position: GATE, expect: me.gateVersion }));
  }
  const kx = (og.input.actionValue(world, "right") > 0.5 ? 1 : 0) - (og.input.actionValue(world, "left") > 0.5 ? 1 : 0);
  const kz = (og.input.actionValue(world, "down") > 0.5 ? 1 : 0) - (og.input.actionValue(world, "up") > 0.5 ? 1 : 0);
  let { x, z } = t.position;
  let yaw: number;
  if (kx !== 0 || kz !== 0) {
    const n = kx !== 0 && kz !== 0 ? Math.SQRT2 : 1;
    x += (kx / n) * SPEED * dt;
    z += (kz / n) * SPEED * dt;
    yaw = math.atan2(kx, kz);
  } else {
    me.angle += (SPEED / RADIUS) * dt;
    x = me.cx + math.cos(me.angle) * RADIUS;
    z = me.cz + math.sin(me.angle) * RADIUS;
    yaw = math.atan2(-math.sin(me.angle), math.cos(me.angle));
  }
  world.set(me.entity, og.Transform, { position: { x, y: 0.9, z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, yaw) });
});

/** Every visitor that comes into sight gets a capsule in its colour; the gate, red shut or green open. */
const Paint = defineSystem("Paint", { reads: [og.PresenceEntered, Look, og.SharedObjectChanged, og.SharedObjectGone, og.SharedOpAnswered, Gate], writes: [Me, og.Mesh] }, (_q, _t, world) => {
  const me = must(world.resource(Me));
  for (const c of world.events(og.SharedObjectChanged)) {
    const g = world.get(c.entity, Gate);
    me.gate = c.entity;
    me.gateVersion = c.version;
    me.gateOpen = g.ok ? g.value.open : 0;
    world.insert(c.entity, og.Mesh, { geometry: me.box as unknown as Geometry, material: me[me.gateOpen ? "m2" : "m0"] as unknown as Material });
  }
  for (const _ of world.events(og.SharedObjectGone)) me.gateVersion = 0;
  for (const a of world.events(og.SharedOpAnswered)) {
    if (a.taken) me.taken++;
    else me.refused++;
  }
  for (const e of world.events(og.PresenceEntered)) {
    const look = world.get(e.entity, Look);
    world.insert(e.entity, og.Mesh, capsule(world, look.ok ? look.value.hue : 0));
    me.seen++;
  }
});

export default og.game({
  resources: [Me],
  systems: [Walk, Paint],
  start,
  facts: (world: World) => {
    const s = og.shared.status(world);
    const me = world.resource(Me);
    return {
      state: s.state, host: s.host, player: s.player, inSight: s.inSight,
      seen: me.ok ? me.value.seen : 0, corrected: me.ok ? me.value.corrected : 0, corrections: s.corrections,
      bytesIn: s.bytesIn, bytesOut: s.bytesOut, serverTick: s.serverTick,
      gateVersion: me.ok ? me.value.gateVersion : 0, gateOpen: me.ok ? me.value.gateOpen : 0,
      gateTaken: me.ok ? me.value.taken : 0, gateRefused: me.ok ? me.value.refused : 0,
    };
  },
});
