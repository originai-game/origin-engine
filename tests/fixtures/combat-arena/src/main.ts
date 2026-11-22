// og.systems.combat (README.md): a fighter with a blade (an og.HitVolume
// in front of it) plays a light-light-heavy combo into a dummy with 40
// health and 10 ticks of invulnerability after a hit, then a light it
// dodges out of in recovery. Every number is here; the engine's systems
// carry them out.
//
//   og run tests/fixtures/combat-arena --commands "0.5:key_down:KeyJ;0.52:key_up:KeyJ;3:quit"
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The moves' frame data, in ticks at 60 Hz. */
export const LIGHT = { startup: 3, active: 2, recovery: 12, damage: 10, volumes: 1, cancelOpen: 7, cancelClose: 17 };
export const HEAVY = { startup: 6, active: 3, recovery: 18, damage: 30, volumes: 1, cancelOpen: 10, cancelClose: 27 };
/** A light's link into the next move: from its 4th tick to before its 14th. */
export const WINDOW = { open: 4, close: 14 };
/** How long a press of attack waits for a window. */
export const BUFFER = 4;
/** The dummy's numbers. */
export const DUMMY = { health: 40, invulnerableTicks: 10, knockback: 2 };

/** What the fixture counted. */
const Tally = defineResource("Tally", {
  /** The moves started, in order, two bits each from the low end: 1 light 1, 2 light 2, 3 heavy. */
  seq: u32(),
  starts: u32(),
  lastSwing: u32(),
  hits: u32(),
  damaged: u32(),
  dealt: u32(),
  died: u32(),
  dodges: u32(),
  /** Dodge presses while the attack could not be cancelled. */
  refused: u32(),
});

export function build(world: World): void {
  must(og.input.defineAction(world, { name: "attack", bufferTicks: BUFFER, bindings: [{ control: "KeyJ" }] }));
  must(og.input.defineAction(world, { name: "dodge", bindings: [{ control: "KeyK" }] }));

  const spawn = (name: string) => must(world.spawn({ name }));
  const light1 = spawn("light1");
  const light2 = spawn("light2");
  const heavy = spawn("heavy");
  const link = (name: string, to: Entity) => {
    const e = spawn(name);
    must(world.insert(e, og.AttackLink, { action: "attack", to, ...WINDOW }));
    return e;
  };
  const start = link("toLight1", light1);
  must(world.insert(light1, og.AttackMove, { ...LIGHT, links: [link("toLight2", light2)] }));
  must(world.insert(light2, og.AttackMove, { ...LIGHT, links: [link("toHeavy", heavy)] }));
  must(world.insert(heavy, og.AttackMove, { ...HEAVY }));

  const fighter = must(world.spawn({ name: "fighter", position: { x: 0, y: 1, z: 0 } }));
  const blade = must(world.spawn({ name: "blade", parent: fighter, position: { x: 0, y: 0, z: 0.8 } }));
  must(world.insert(blade, og.HitVolume, { shape: "box", size: { x: 0.8, y: 0.2, z: 0.6 }, team: 1 }));
  must(world.insert(fighter, og.Attack, { starters: [start], volumes: [blade] }));

  const dummy = must(world.spawn({ name: "dummy", position: { x: 0, y: 1, z: 1.2 } }));
  must(world.insert(dummy, og.HurtVolume, { shape: "box", size: { x: 0.6, y: 1.8, z: 0.4 }, team: 2 }));
  must(world.insert(dummy, og.Health, { value: DUMMY.health }));
  must(world.insert(dummy, og.Hurt, { invulnerableTicks: DUMMY.invulnerableTicks, knockback: DUMMY.knockback }));
}

/** The game's dodge: it cuts the attack only inside its cancel window. */
const Dodge = defineSystem("Dodge", { reads: [og.Attack], writes: [og.Attack, Tally], after: ["og.combat"] }, (_q, _t, world) => {
  const fighter = world.find("fighter")!;
  const tally = must(world.resource(Tally));
  const attack = must(world.get(fighter, og.Attack));
  if (!og.input.isActionJustPressed(world, "dodge")) return;
  if (!attack.cancelable) {
    tally.refused += 1;
    return;
  }
  og.input.consume(world, "dodge");
  must(world.set(fighter, og.Attack, { move: 0 as Entity }));
  tally.dodges += 1;
});

/** The moves started, the hits og.play sent, what og.systems.combat reported. */
const Count = defineSystem("Count", { reads: [og.Attack, og.HitEvent, og.Damaged, og.Died], writes: [Tally], after: ["og.combat"] }, (_q, _t, world) => {
  const tally = must(world.resource(Tally));
  const attack = must(world.get(world.find("fighter")!, og.Attack));
  if (attack.swing !== tally.lastSwing) {
    const id = ["light1", "light2", "heavy"].findIndex((n) => world.find(n) === attack.move) + 1;
    tally.seq += id << (2 * tally.starts);
    tally.starts += 1;
    tally.lastSwing = attack.swing;
  }
  for (const hit of world.events(og.HitEvent)) if (hit.target === world.find("dummy")) tally.hits += 1;
  for (const d of world.events(og.Damaged)) {
    tally.damaged += 1;
    tally.dealt += d.amount;
    og.assert.reachable("the dummy takes damage");
  }
  for (const _ of world.events(og.Died)) tally.died += 1;
});

/** What the fixture counted, for `og test`'s `state:` and combat.test.ts. */
export function facts(world: World) {
  const t = must(world.resource(Tally));
  const health = must(world.get(world.find("dummy")!, og.Health));
  const attack = must(world.get(world.find("fighter")!, og.Attack));
  return {
    seq: t.seq,
    starts: t.starts,
    hits: t.hits,
    damaged: t.damaged,
    dealt: t.dealt,
    died: t.died,
    dodges: t.dodges,
    refused: t.refused,
    health: health.value,
    dead: health.dead ? 1 : 0,
    swing: attack.swing,
    tick: attack.tick,
    idle: attack.phase === "idle" ? 1 : 0,
  };
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Tally],
  systems: [...og.systems.combat, Dodge, Count],
  start: build,
  facts,
});
