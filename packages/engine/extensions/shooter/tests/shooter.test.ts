// The shooter kit's members as a game mounts them (docs/ui.md §6.1): the kit
// installed under assets/ui-kits/shooter with the library StatBar its
// manifest names, dressed by the clean pack, each member mounted from named
// stores and driven through the probe's touch, key and pad commands.
//
// Ways it could fail: an intent sent without its row key or for a disabled
// (locked) slot; selection following a row's index instead of its key
// through a reorder, a removal or a restore; the empty loadout without its
// outcome or its actions; a hit mark that does not replay on a new id; the
// killfeed out of the collection's order or crowding a phone; HUD readouts
// under the phone's safe area or on top of each other.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { cpSync, mkdirSync, mkdtempSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";
import { loadNativeHost } from "../../../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../../../src/runtime.ts";
import { cookProject } from "../../../tests/support/og-bin.ts";

type El = { id: string; path: string; text: string; x: number; y: number; width: number; height: number; states?: string[]; disabled?: boolean; name?: string; role?: string };
type Event = { mount: string; path: string; rowKey: string; kind: string; name: string };

const KIT = dirname(dirname(fileURLToPath(import.meta.url)));
const LIBRARY = join(KIT, "../../src/ui/components");
let root = "";

const SLOTS = [
  { id: "strider", category: "primary", name: "Strider AR", icon: "sword", detail: "25 / 75", equipped: true, locked: false, cost: "2,900" },
  { id: "halcyon", category: "primary", name: "Halcyon", icon: "sword", detail: "30 / 90", equipped: false, locked: false, cost: "2,900" },
  { id: "pike", category: "primary", name: "Pike .50", icon: "sword", detail: "5 / 10", equipped: false, locked: true, cost: "4,700" },
  { id: "wren", category: "secondary", name: "Wren Pistol", icon: "bolt", detail: "15 / 45", equipped: true, locked: false, cost: "500" },
];
const CATEGORIES = [{ id: "primary", label: "Primary" }, { id: "secondary", label: "Secondary" }];
const FEED = ["k4", "k3", "k2", "k1"].map((id, i) => ({ id, killer: `Killer${i}`, victim: `Victim${i}`, weapon: "Halcyon", headshot: i === 0, self: i === 1, team: i % 2 ? "enemy" : "ally" }));

const PAGES: Record<string, string> = {
  loadoutTest: `<meta name="screen" content="loadoutTest" data-layer="menu" data-input="menu">
<div style="width: stretch; height: stretch; padding: 16px"><Loadout title={$title} categories={$categories} slots={$slots} selected={$selected} /></div>`,
  hudTest: `<meta name="component" content="hudTest">
<div style="width: stretch; height: stretch"><CombatHud health={$health} healthMax={100} shield={40} shieldMax={100} spread={$spread} hit={$hit} hitAt={$hitAt} aimed={false} shape="cross" weapon="Halcyon Carbine" mode="auto" magazine={$magazine} capacity={28} reserve={112} reloading={$reloading} hits={$hits} feed={$feed} /></div>`,
};

beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-shooter-kit-"));
  const kit = join(root, "assets/ui-kits/shooter");
  mkdirSync(kit, { recursive: true });
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  for (const f of readdirSync(KIT)) if (/\.(svelte|ogt|json)$/.test(f)) cpSync(join(KIT, f), join(kit, f));
  for (const f of ["stat-bar.svelte", "stat-bar.en.ogt"]) cpSync(join(LIBRARY, f), join(root, "assets/ui", f));
  for (const [name, text] of Object.entries(PAGES)) writeFileSync(join(root, "assets/ui", `${name}.svelte`), text);
  writeFileSync(join(root, "assets/looks/clean.ogt"), 'type look\nversion 1\n---\nname "clean"\nui "clean"\n');
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Shooter kit"\nid "shooter-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  cookProject(root, root);
});

afterAll(() => rmSync(root, { recursive: true, force: true }));

const DOCUMENTS = ["ui/stat-bar.svelte", ...["crosshair", "ammo-counter", "damage-direction", "killfeed", "loadout", "combat-hud"].map((d) => `ui-kits/shooter/${d}.svelte`), "ui/loadoutTest.svelte", "ui/hudTest.svelte"];

/** An engine mounting `page` with these stores; its document events are collected in `events`. */
function engine(page: string, stores: Record<string, unknown>, events: Event[] = []): Engine {
  const Collect = defineSystem("ShooterKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      const style = og.load.style(world, "@styles/clean/style.ogt");
      if (!style.ok) throw new Error(style.reason);
      og.ui.configure(world, { style: style.value });
      for (const path of DOCUMENTS) {
        const loaded = og.load.document(world, path);
        if (!loaded.ok) throw new Error(`${path}: ${loaded.reason}`);
      }
    },
    ui(world) {
      const scope = og.ui.scope(world, "t");
      const inputs = Object.fromEntries(Object.entries(stores).map(([k, v]) => [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
      const mounted = scope.mount("page", page, inputs);
      if (!mounted.ok) throw new Error(mounted.reason);
    },
  }) as unknown as GameToken, { root });
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const elements = (e: Engine): El[] => (e.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements;
/** The first element whose path matches: a component instance's root is an anonymous node, so paths are matched by their tail. */
const find = (e: Engine, path: RegExp): El | undefined => elements(e).find((x) => path.test(x.path));
const centre = (el: El): string => `${el.x + el.width / 2},${el.y + el.height / 2}`;
const tap = (e: Engine, el: El): void => {
  e.probe.commands(`0:touch_move:${centre(el)};0.02:touch_start;0.06:touch_end`);
  e.advance(8);
};
const store = (e: Engine) => og.ui.scope(e.world, "t");
const loadout = (events: Event[] = [], selected = "halcyon", slots: unknown[] = SLOTS) =>
  engine("loadoutTest", { title: "Loadout", categories: CATEGORIES, slots, selected }, events);
const intents = (events: Event[]) => events.filter((e) => ["select", "equip", "back"].includes(e.name)).map((e) => `${e.name}:${e.rowKey}`);

test("Loadout: a tap selects by row key, a locked slot sends nothing, and Equip and Back send once", () => {
  const events: Event[] = [];
  const e = loadout(events);
  try {
    e.advance(3);
    tap(e, find(e, /#strider$/)!);
    expect(intents(events)).toEqual(["select:strider"]);
    events.length = 0;
    const locked = find(e, /#pike$/)!;
    expect(locked.disabled).toBeTrue();
    tap(e, locked);
    expect(intents(events)).toEqual([]);
    tap(e, find(e, /\/Equip(#[^/]+)?$/)!);
    tap(e, find(e, /Actions\/Back$/)!);
    expect(intents(events)).toEqual(["equip:halcyon", "back:"]);
  } finally {
    e.destroy();
  }
});

test("Loadout: Equip is disabled for the equipped slot, and the empty loadout says so and keeps Back", () => {
  const e = loadout([], "strider");
  try {
    e.advance(3);
    expect(find(e, /\/Equip(#[^/]+)?$/)!.disabled).toBeTrue();
  } finally {
    e.destroy();
  }
  const events: Event[] = [];
  const empty = loadout(events, "", []);
  try {
    empty.advance(3);
    expect(find(empty, /\/Empty$/)!.text).toBe("No items");
    expect(find(empty, /\/Equip(#[^/]+)?$/)).toBeUndefined();
    tap(empty, find(empty, /Actions\/Back$/)!);
    expect(intents(events)).toEqual(["back:"]);
  } finally {
    empty.destroy();
  }
});

test("Loadout: keyboard and pad select the focused slot, and the Equip action equips the selected one", () => {
  const devices = [
    { name: "keyboard", move: "key_down:ArrowDown;0.05:key_up:ArrowDown", select: "key_down:Enter;0.05:key_up:Enter", equip: "key_down:KeyX;0.05:key_up:KeyX", back: "key_down:Escape;0.05:key_up:Escape" },
    { name: "pad", move: "gamepad:-:0,1;0.05:gamepad:-", select: "gamepad:a;0.05:gamepad:-", equip: "gamepad:y;0.05:gamepad:-", back: "gamepad:b;0.05:gamepad:-" },
  ];
  const prompts: string[] = [];
  for (const d of devices) {
    const events: Event[] = [];
    const e = loadout(events, "");
    try {
      e.advance(3);
      e.probe.commands(`0:${d.move}`);
      e.advance(8);
      e.probe.commands(`0:${d.select}`);
      e.advance(8);
      expect(`${d.name} ${intents(events)}`).toBe(`${d.name} select:halcyon`);
      // The game applies the intent; the kit shows what the game decides.
      store(e).writable("selected", "").set("halcyon");
      e.advance(2);
      events.length = 0;
      e.probe.commands(`0:${d.equip}`);
      e.advance(8);
      prompts.push(find(e, /\/EquipKey$/)!.text);
      e.probe.commands(`0:${d.back}`);
      e.advance(8);
      expect(`${d.name} ${intents(events)}`).toBe(`${d.name} equip:halcyon,back:`);
    } finally {
      e.destroy();
    }
  }
  // The Equip keycap follows the device used last (core prompts, docs/ui.md §3.2).
  expect(prompts).toEqual(["X", "Y"]);
});

test("Loadout: selection keeps its slot through a reorder, a removal and a restore", () => {
  const e = loadout([], "pike");
  try {
    e.advance(3);
    const selected = () => elements(e).filter((x) => x.states?.includes("selected")).map((x) => x.path.split("#").pop());
    expect(selected()).toEqual(["pike"]);
    const slots = store(e).collection("slots", SLOTS, { key: "id" });
    slots.move("pike", "strider");
    e.advance(2);
    expect(selected()).toEqual(["pike"]);
    const snap = e.probe.snapshot() as { id: string };
    slots.remove("strider");
    e.advance(2);
    expect(selected()).toEqual(["pike"]);
    expect(find(e, /#strider$/)).toBeUndefined();
    expect(e.probe.restore(snap.id)).toBeTrue();
    e.advance(2);
    expect(selected()).toEqual(["pike"]);
    expect(find(e, /#strider$/)).toBeDefined();
  } finally {
    e.destroy();
  }
});

const HUD = { health: 64, spread: 0, hit: "none", hitAt: 0, magazine: 24, reloading: -1, hits: [{ id: "h1", angle: 0.9, strength: 0.8, kind: "normal" }], feed: FEED };

test("Crosshair: spread moves the arms out, and a new hit id replays the mark", () => {
  const e = engine("hudTest", HUD);
  try {
    e.advance(3);
    const up = () => find(e, /Aim\/[^/]+\/Up$/)!;
    const right = () => find(e, /Aim\/[^/]+\/Right$/)!;
    const [settled, settledRight] = [up(), right()];
    store(e).writable("spread", 0).set(1);
    e.advance(30);
    expect(up().y).toBeLessThan(settled.y - 8);
    expect(right().x).toBeGreaterThan(settledRight.x + 8);
    const hit = () => elements(e).find((x) => /Aim\/[^/]+\/Hit[^/]*$/.test(x.path))!.path;
    const before = hit();
    store(e).writable("hit", "none").set("headshot");
    e.advance(2);
    expect(hit()).toBe(before);
    store(e).writable("hitAt", 0).set(1);
    e.advance(2);
    expect(hit()).not.toBe(before);
  } finally {
    e.destroy();
  }
});

test("AmmoCounter: the count, the reload in progress, and the empty magazine's prompt", () => {
  const e = engine("hudTest", { ...HUD, magazine: 1024 });
  try {
    e.advance(3);
    expect(find(e, /Rounds\/Count$/)!.text).toBe("1,024");
    expect(find(e, /Ammo\/[^/]+\/Reload$/)?.width ?? 0).toBe(0);
    store(e).writable("magazine", 0).set(0);
    e.advance(2);
    expect(find(e, /Ammo\/[^/]+\/Prompt$/)!.text.toLowerCase()).toBe("reload");
    store(e).writable("reloading", -1).set(0.5);
    e.advance(2);
    expect(find(e, /Ammo\/[^/]+\/Reload$/)!.width).toBeGreaterThan(0);
    expect(find(e, /Ammo\/[^/]+\/Prompt$/)?.width ?? 0).toBe(0);
  } finally {
    e.destroy();
  }
});

test("DamageDirection and Killfeed follow their collections by key and order", () => {
  const e = engine("hudTest", HUD);
  try {
    e.advance(3);
    const arcs = () => elements(e).filter((x) => /Damage\/[^/]+\/[^/]+#h\d$/.test(x.path)).length;
    expect(arcs()).toBe(1);
    const hits = store(e).collection("hits", HUD.hits, { key: "id" });
    hits.insert({ id: "h2", angle: -2, strength: 0.4, kind: "shield" });
    e.advance(2);
    expect(arcs()).toBe(2);
    hits.remove("h1");
    hits.remove("h2");
    e.advance(30);
    expect(arcs()).toBe(0);
    const rows = () => elements(e).filter((x) => /Feed\/[^/]+\/[^/]+#k\d$/.test(x.path) && x.height > 0).map((x) => x.path.split("#").pop());
    expect(rows()).toEqual(["k4", "k3", "k2", "k1"]);
    store(e).collection("feed", FEED, { key: "id" }).insert({ id: "k5", killer: "New", victim: "One", weapon: "Pike .50", headshot: false, self: false, team: "ally" }, "k4");
    e.advance(20);
    expect(rows()).toEqual(["k5", "k4", "k3", "k2", "k1"]);
  } finally {
    e.destroy();
  }
});

const PHONE = { width: 390, height: 844, dpr: 2, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };
const DESKTOP = { width: 1920, height: 1080, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };
const meet = (a: El, b: El): boolean => a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;

for (const [name, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  test(`CombatHud on a ${name}: readouts inside the safe area and clear of each other; a phone shows three feed rows`, () => {
    const e = engine("hudTest", HUD);
    try {
      og.ui.previewViewport(e.world, viewport as never);
      e.advance(4);
      const box = (name: string) => find(e, new RegExp(`page/[^/]+/${name}$`))!;
      const readouts = ["Vitals", "Ammo", "Feed"].map(box);
      for (const r of readouts) {
        expect(r.y).toBeGreaterThanOrEqual(viewport.safeArea.top);
        expect(r.y + r.height).toBeLessThanOrEqual(viewport.height - viewport.safeArea.bottom);
        expect(r.x).toBeGreaterThanOrEqual(0);
        expect(r.x + r.width).toBeLessThanOrEqual(viewport.width);
      }
      for (let i = 0; i < readouts.length; i++) for (let j = i + 1; j < readouts.length; j++) expect(`${i}-${j} ${meet(readouts[i]!, readouts[j]!)} ${JSON.stringify([readouts[i], readouts[j]].map((r) => [r!.x, r!.y, r!.width, r!.height]))}`).toContain(" false ");
      const shown = elements(e).filter((x) => /Feed\/[^/]+\/[^/]+#k\d$/.test(x.path) && x.height > 0).length;
      expect(shown).toBe(name === "phone" ? 3 : 4);
    } finally {
      e.destroy();
    }
  });
}
