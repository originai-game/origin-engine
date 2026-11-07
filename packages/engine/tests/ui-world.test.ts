// The world components (docs/ui.md §1, U6c) on the native host with the
// default og.Camera (at the origin, looking down -z), read through dump_ui.
//
// Ways they could fail, each covered below:
// - a nameplate keeps its title and name at every distance, or sheds them
//   for the target too;
// - a waypoint behind the camera leaves the screen instead of holding its
//   edge, or a ping outlives its time;
// - a prompt shows no glyph for an action without a binding, or a hold's
//   ring does not follow the hold;
// - a speech bubble shows its whole text before it types, or never leaves;
// - a loot label beams over a common item;
// - a ground shape is drawn off its centre's projection, or drawn when its
//   centre is behind the camera;
// - a ground shape on a slope floats at its given height instead of lying on
//   the terrain (sampled or cast down against its collider) under each vertex;
// - damage numbers spawn nothing, or a miss spawns a number;
// - a crowd of plates and loot overlaps, stacks off the top of a phone, or
//   a waypoint is pushed off its point by the tags around it.
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineSystem, SHAPE_HEIGHTFIELD } from "@origingame/engine";
import type { DamageHitDesc, GroundDesc, World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";
import { at, dump, engineFor, type El } from "./support/ui.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

loadNativeHost();

const point = (x: number, y: number, z: number) => ({ target: { entity: null, point: { x, y, z } } });
const states = (el: El | undefined): Record<string, string> => el?.visualStates ?? {};
const DESKTOP = { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };
const engine = (name: string, build: (w: World) => void) => engineFor(name, build, DESKTOP, [og.Camera]);

test("og.ui.nameplate: full near, title gone past the first distance, only the bar past the second, the target always full", () => {
  const e = engine("WorldPlates", () => {
    og.ui.nameplate("Near", { ...point(-3, 0, -10), name: "Grull", title: "<Ogres>", level: "12", health: 0.6 });
    og.ui.nameplate("Mid", { ...point(0, 0, -35), name: "Harn", title: "<Ogres>", health: 0.5 });
    og.ui.nameplate("Far", { ...point(3, 0, -60), name: "Ida", title: "<Ogres>", health: 0.4 });
    og.ui.nameplate("Target", { ...point(6, 0, -60), name: "Jory", title: "<Ogres>", health: 0.9, targeted: true });
  });
  e.advance(3);
  const els = dump(e);
  expect([at(els, "Near/Title")?.text, at(els, "Near/Head/Name")?.text, at(els, "Near")?.value]).toEqual(["<Ogres>", "Grull", "Level 12, 60% health"]);
  expect([at(els, "Mid/Title"), at(els, "Mid/Head/Name")?.text, states(at(els, "Mid")).lod]).toEqual([undefined, "Harn", "compact"]);
  expect([at(els, "Far/Head"), at(els, "Far/Bar") !== undefined, states(at(els, "Far")).lod]).toEqual([undefined, true, "bar"]);
  expect([at(els, "Target/Title")?.text, states(at(els, "Target")).selection]).toEqual(["<Ogres>", "targeted"]);
  // Above its point: the near plate's bottom sits over the projection of the point 2 m up.
  const near = at(els, "Near")!;
  expect(near.y + near.height).toBeLessThan(360);
  expect(near.x + near.width / 2).toBeLessThan(640);
});

test("og.ui.waypoint: on screen with its distance and label; behind the camera, held on the edge without its label; a ping leaves", () => {
  const e = engine("WorldWaypoints", () => {
    og.ui.waypoint("Ahead", { ...point(4, 0, -20), kind: "objective", label: "Defend" });
    og.ui.waypoint("Behind", { ...point(0, 0, 30), kind: "enemy", label: "Spotted" });
    og.ui.waypoint("Ping", { ...point(-2, 0, -8), kind: "ping", label: "Loot here", by: "Wraith", at: 0, durationMs: 1000 });
  });
  e.advance(3);
  let els = dump(e);
  expect(at(els, "Ahead/Label")?.text).toBe("Defend");
  expect(at(els, "Ahead/Distance")?.text).toMatch(/^2\dm$/);
  expect(states(at(els, "Ahead")).place).toBe("onScreen");
  const behind = at(els, "Behind")!;
  expect(states(behind).place).toBe("clamped");
  expect(at(els, "Behind/Label")).toBe(undefined);
  expect(behind.x >= 0 && behind.y >= 0 && behind.x + behind.width <= 1280 && behind.y + behind.height <= 720).toBe(true);
  expect(at(els, "Ping/Label")?.text).toBe("Loot here · Wraith");
  e.advance(70);
  els = dump(e);
  expect(at(els, "Ping")).toBe(undefined);
  expect(at(els, "Ahead")).toBeDefined();
});

test("og.ui.interactionPrompt and speechBubble: a key cap for an unbound action, an og.input action's binding, the hold's state; words type out, then leave", () => {
  let hold = 0.4;
  const e = engine("WorldPrompts", (w) => {
    og.ui.interactionPrompt(w, "Chest", { ...point(0, 0, -4), action: "no.such.action", verb: "Open", name: "Chest", hold });
    og.ui.interactionPrompt(w, "Door", { ...point(1.5, 0, -4), verb: "Open", disabled: true, reason: "Locked" });
    // A gameplay action's prompt shows its binding (og.input), not the fallback cap.
    og.input.defineAction(w, { name: "interact", bindings: [{ control: "KeyF" }, { control: "pad.x" }] });
    og.ui.interactionPrompt(w, "Lever", { ...point(-1.5, 0, -4), action: "interact", verb: "Pull" });
    og.ui.speechBubble("Say", { ...point(-1, 0, -6), speaker: "Tom Nook", text: "Hello there, friend!", at: 0, charsPerSecond: 20, durationMs: 500 });
  });
  e.advance(15);
  let els = dump(e);
  expect(at(els, "Chest/Frame/Words/Verb")?.text).toBe("Open");
  expect(at(els, "Chest/Frame/Glyph/Cap/Key")?.text).toBe("E");
  expect([at(els, "Chest")?.role, at(els, "Chest")?.name, states(at(els, "Chest")).use]).toEqual(["button", "Open Chest", "holding"]);
  expect([states(at(els, "Door")).use, at(els, "Door/Frame/Words/Reason")?.text]).toEqual(["disabled", "Locked"]);
  expect(at(els, "Lever")?.value).toBe("F");
  // Fifteen ticks at 20 a second: five characters typed; the screen reader has it all at once.
  const typed = at(els, "Say/Bubble/Text")?.text ?? "";
  expect(typed.length).toBeGreaterThan(2);
  expect(typed.length).toBeLessThan(10);
  expect(at(els, "Say")?.value).toBe("Hello there, friend!");
  expect(states(at(els, "Say")).speech).toBe("typing");
  hold = 1;
  e.advance(60);
  els = dump(e);
  expect(at(els, "Say/Bubble/Text")?.text).toBe("Hello there, friend!");
  expect(states(at(els, "Chest")).use).toBe("ready");
  e.advance(80);
  expect(at(dump(e), "Say")).toBe(undefined);
});

test("og.ui.lootLabel: a chip in its tier with pips, a beam only from rare up, the stack's count", () => {
  const e = engine("WorldLoot", () => {
    og.ui.lootLabel("Sword", { ...point(-1, 0, -6), name: "Doombringer", rarity: "legendary" });
    og.ui.lootLabel("Rags", { ...point(1, 0, -6), name: "Torn Rags", rarity: "common", count: 3 });
  });
  e.advance(3);
  const els = dump(e);
  expect(at(els, "Sword/Chip/Name")?.text).toBe("Doombringer");
  expect(at(els, "Sword/Beam")).toBeDefined();
  expect(els.filter((x) => x.path.startsWith("Sword/Chip/Pips/")).length).toBe(5);
  expect(at(els, "Sword/Chip")?.value).toBe("legendary");
  expect(at(els, "Rags/Beam")).toBe(undefined);
  expect(at(els, "Rags/Chip/Count")?.text).toBe("×3");
});

test("ground shapes sit on their centre's projection, and are not drawn behind the camera", () => {
  const e = engine("WorldGround", (w) => {
    og.ui.selectionRing(w, "Ring", { center: { x: 0, y: -1, z: -8 }, radius: 1 });
    og.ui.selectionRing(w, "Gone", { center: { x: 0, y: -1, z: 8 }, radius: 1 });
    og.ui.telegraph(w, "Cone", { center: { x: 2, y: -1, z: -10 }, shape: "cone", radius: 4, angle: 1.2, direction: Math.PI, progress: 0.5 });
    og.ui.telegraph(w, "Donut", { center: { x: -3, y: -1, z: -12 }, shape: "ring", inner: 2, radius: 4, progress: 1 });
    og.ui.pathLine(w, "Path", { points: [{ x: 0, y: -1, z: -4 }, { x: 2, y: -1, z: -8 }, { x: 2, y: -1, z: -14 }], reach: 6, label: "12 m · 2 AP" });
    og.ui.placementGhost(w, "Ghost", { center: { x: -2, y: -1, z: -9 }, width: 3, depth: 2, cell: 1, valid: false, label: "Blocked", range: 5 });
  });
  e.advance(3);
  const els = dump(e);
  const ring = at(els, "Ring")!;
  // The ring's centre (0, -1, -8) projects to (640, 360 + 1/8 of the focal length): its element is centred there.
  expect(Math.abs(ring.x + ring.width / 2 - 640)).toBeLessThan(1.5);
  expect(ring.y + ring.height / 2).toBeGreaterThan(360);
  expect(ring.width).toBeGreaterThan(ring.height * 2);
  expect(at(els, "Gone")).toBe(undefined);
  expect(at(els, "Cone/Box/Fill")).toBeDefined();
  expect(states(at(els, "Cone")).windup).toBe("charging");
  expect(states(at(els, "Donut")).windup).toBe("impact");
  // A ring is one area, its hole cut out even-odd.
  expect(at(els, "Donut/Box/Area")).toBeDefined();
  expect([at(els, "Donut/Box/Edge#0"), at(els, "Donut/Box/Edge#1")].every(Boolean)).toBe(true);
  expect(at(els, "Path/Box/Beyond")).toBeDefined();
  expect(at(els, "PathLabel/Chip/Text")?.text).toBe("12 m · 2 AP");
  expect(states(at(els, "Ghost")).place).toBe("invalid");
  expect(els.some((x) => x.path.startsWith("Ghost/Box/Hatch#"))).toBe(true);
  expect(at(els, "GhostLabel/Chip")?.name).toBe("Blocked");
});

test("og.ui.damageNumbers and overheadBars: hits spawn mass elements that live out their time; a miss is a word", () => {
  let hits: DamageHitDesc[] = [];
  let counts: number[] = [];
  const e = engine("WorldMass", () => {
    og.ui.damageNumbers("Dmg", { hits });
    og.ui.overheadBars("Bars", { bars: [{ key: "a", position: { x: 0, y: 1, z: -10 }, value: 0.4 }, { key: "b", position: { x: 1, y: 1, z: -10 }, value: 1 }], hideFull: true });
    const n = og.ui.memory("Dmg.number~mass", -1);
    const w = og.ui.memory("Dmg.word~mass", -1);
    counts = [n >= 0 ? og.ui.massCount(n) : -1, w >= 0 ? og.ui.massCount(w) : -1];
    hits = [];
  });
  hits = [
    { position: { x: 0, y: 1, z: -6 }, amount: 120 },
    { position: { x: 0, y: 1, z: -6 }, amount: 48210, kind: "critical", seed: 1 },
    { position: { x: 0, y: 1, z: -6 }, kind: "miss", word: "Dodge" },
  ];
  e.advance(1);
  expect(counts).toEqual([2, 1]);
  e.advance(120);
  expect(counts).toEqual([0, 0]);
});

test("ground shapes lie on a sloped terrain: sampled, or cast down against its collider, under each vertex", () => {
  // tests/fixtures/terrain: a 6 m hill (14 m wide) on a 0.05 slope, 2 m up, over 64 m.
  const fixture = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "terrain");
  const cooked = Bun.spawnSync([ogBin(), "cook", fixture]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  const eye = { x: 7, y: 14, z: 22 };
  const centre = { x: 7, y: 0, z: 3 };
  let data = 0;
  let hill = null as unknown;
  const Ui = defineSystem("GroundUi", { reads: [og.Camera] } as never, (_q, _t, w: World) => {
    const terrain: GroundDesc = { terrain: data as never };
    const colliders: GroundDesc = { colliders: true };
    og.ui.selectionRing(w, "Sampled", { center: centre, radius: 2, ground: terrain });
    og.ui.selectionRing(w, "Cast", { center: centre, radius: 2, ground: colliders });
    og.ui.selectionRing(w, "Flat", { center: centre, radius: 2 });
    og.ui.telegraph(w, "Slam", { center: { x: 0, y: 0, z: 0 }, shape: "circle", radius: 5, ground: terrain });
  });
  const game = og.game({
    systems: [Ui],
    start(world: World) {
      const info = og.terrain.load(world, "terrain/hill.ogt");
      if (!info.ok) throw new Error(info.reason);
      data = info.value.data as unknown as number;
      const e = world.spawn({ name: "hill" });
      if (!e.ok) throw new Error(e.reason);
      hill = e.value;
      world.insert(e.value, og.Terrain, { data: info.value.data });
      world.insert(e.value, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.value.geometry as never });
      const cam = world.resource(og.Camera);
      if (cam.ok) {
        cam.value.position = eye;
        cam.value.quaternion = og.transform.lookAt(eye, centre);
        cam.value.fov = 50;
      }
    },
  });
  const r = createEngine(game as unknown as GameToken, { root: fixture });
  if (!r.ok) throw new Error(r.reason);
  const e = r.value;
  og.ui.previewViewport(e.world, DESKTOP as never);
  e.advance(3);
  expect(hill).not.toBe(null);
  const cam = e.world.resource(og.Camera);
  if (!cam.ok) throw new Error(cam.reason);
  // The ring's vertices (56 round the centre, clockwise from +z), each on the surface 4 cm up, projected: what its line must span.
  const expected = (lift: (x: number, z: number) => number) => {
    let [x0, y0, x1, y1] = [Infinity, Infinity, -Infinity, -Infinity];
    for (let k = 0; k < 56; k++) {
      const a = (Math.PI * 2 * k) / 56;
      const x = centre.x + Math.sin(a) * 2;
      const z = centre.z + Math.cos(a) * 2;
      const s = cam.value.worldToScreen({ x, y: lift(x, z), z })!;
      [x0, y0, x1, y1] = [Math.min(x0, s.x), Math.min(y0, s.y), Math.max(x1, s.x), Math.max(y1, s.y)];
    }
    return { x: x0, y: y0, width: x1 - x0, height: y1 - y0 };
  };
  const surface = (x: number, z: number) => {
    const s = og.terrain.sample(e.world, { data: data as never, x, z });
    if (!s.ok) throw new Error(s.reason);
    return s.value.height + 0.04;
  };
  const onHill = expected(surface);
  const els = dump(e);
  // The line's element is its points' box grown by half its 2.5 px width and 2 px.
  const pad = 2.5 / 2 + 2;
  for (const name of ["Sampled", "Cast"]) {
    const line = at(els, `${name}/Box/Ring`)!;
    expect(line).toBeDefined();
    expect(Math.abs(line.x + pad - onHill.x)).toBeLessThan(1.5);
    expect(Math.abs(line.y + pad - onHill.y)).toBeLessThan(1.5);
    expect(Math.abs(line.width - 2 * pad - onHill.width)).toBeLessThan(2);
    expect(Math.abs(line.height - 2 * pad - onHill.height)).toBeLessThan(2);
  }
  // Without a ground the ring lies flat at y 0, metres under the flank: far below on the screen.
  const flat = at(els, "Flat/Box/Ring")!;
  const below = expected(() => 0);
  expect(Math.abs(flat.y + pad - below.y)).toBeLessThan(1.5);
  expect(flat.y - at(els, "Sampled/Box/Ring")!.y).toBeGreaterThan(40);
  // A telegraph over the hilltop is drawn, its edge on the summit's ground.
  expect(at(els, "Slam/Box/Area")).toBeDefined();
  expect(at(els, "Slam/Box/Edge#0")).toBeDefined();
});

test("a crowd of tags on a phone never overlaps, keeps inside the safe area, and leaves waypoints on their points", () => {
  const PHONE = { width: 390, height: 844, dpr: 1, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };
  const infos: Record<string, { alpha: number }> = {};
  const e = engineFor("WorldCrowd", (w) => {
    // At 150 % text the stack outgrows the room above it: it must not climb off the top.
    og.ui.configure(w, { textScale: 1.5 });
    for (let i = 0; i < 6; i++) infos[`P${i}`] = og.ui.nameplate(`P${i}`, { ...point(0.2 + i * 0.12, 0, -6 - i * 0.2), name: "Ember Imp", level: "12", health: 0.5, reaction: "hostile" });
    for (let i = 0; i < 4; i++) infos[`L${i}`] = og.ui.lootLabel(`L${i}`, { ...point(0.1 * i, 0, -5), name: "Linen Scraps", rarity: "uncommon", count: 3 }).info;
    infos.W = og.ui.waypoint("W", { ...point(2.4, 1.6, -6.2), kind: "objective", label: "Hold" });
  }, PHONE, [og.Camera]);
  e.advance(60);
  const els = dump(e);
  const shown = Object.keys(infos).filter((k) => infos[k]!.alpha > 0.05).map((k) => at(els, k)!).filter(Boolean);
  expect(shown.filter((a) => a.path.startsWith("P")).length).toBeGreaterThan(1);
  for (const a of shown) {
    if (a.path === "W") continue;
    expect(a.y).toBeGreaterThanOrEqual(47 - 0.5);
    expect(a.x + a.width).toBeLessThanOrEqual(390 + 0.5);
  }
  for (let i = 0; i < shown.length; i++) for (let j = i + 1; j < shown.length; j++) {
    const a = shown[i]!, b = shown[j]!;
    const w = Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x);
    const h = Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y);
    expect(w > 1 && h > 1 ? `${a.path} over ${b.path}` : "").toBe("");
  }
});
