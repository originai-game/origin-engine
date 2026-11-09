// The rpg kit's members as a game mounts them (docs/ui.md §6.1): the kit
// installed under assets/ui-kits/rpg, dressed by the fantasy pack, each
// member mounted from named stores and driven through the probe's touch,
// key and pad commands.
//
// Ways it could fail: an intent sent without its row key, or for a disabled
// (locked) row; selection following a row's index instead of its key
// through a reorder, a removal or a restore; an empty member without its
// outcome; a phone layout that pushes a primary action off screen or puts
// a touch target under 44 px; a marker not where the game projects it.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { cpSync, mkdirSync, mkdtempSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";
import { loadNativeHost } from "../../../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../../../src/runtime.ts";
import { cookProject } from "../../../tests/support/og-bin.ts";

type El = { id: string; path: string; text: string; x: number; y: number; width: number; height: number; states?: string[]; disabled?: boolean; name?: string; role?: string; selected?: boolean };
type Event = { mount: string; path: string; rowKey: string; kind: string; name: string };

const KIT = dirname(dirname(fileURLToPath(import.meta.url)));
const MEMBERS = ["inventory", "equipment", "journal", "tracker", "dialogue", "party", "turn-order", "world-map", "compass"];
let root = "";

const ITEMS = [
  { id: "blade", name: "Emberwake Blade", icon: "sword", quantity: 1, rarity: "epic", kind: "Sword", equipped: true, fresh: false, locked: false, broken: false },
  { id: "shield", name: "Oakheart Shield", icon: "shield", quantity: 1, rarity: "rare", kind: "Shield", equipped: false, fresh: true, locked: false, broken: false },
  { id: "potion", name: "Healing Draught", icon: "potion", quantity: 12, rarity: "common", kind: "Potion", equipped: false, fresh: false, locked: false, broken: false },
  { id: "key", name: "Ferryman’s Key", icon: "key", quantity: 1, rarity: "uncommon", kind: "Quest item", equipped: false, fresh: false, locked: true, broken: false },
];
const FILTERS = [{ id: "all", label: "All", count: 4 }, { id: "gear", label: "Gear", count: 2 }];
const QUESTS = [
  { id: "ferry", name: "The Drowned Ferryman", region: "Saltmarsh", level: "Lv 8", kind: "main", tracked: true, fresh: false, state: "active" },
  { id: "bells", name: "Bells Beneath the Abbey", region: "Greyfen", level: "Lv 6", kind: "side", tracked: false, fresh: true, state: "active" },
  { id: "pelts", name: "Wolf Pelts for Marta", region: "Hollin", level: "", kind: "task", tracked: false, fresh: false, state: "active" },
];
const CHOICES = [
  { id: "c1", key: "1", text: "What did he owe?", check: "", seen: false, locked: false, ends: false },
  { id: "c2", key: "2", text: "You know more than you say.", check: "Insight · DC 12", seen: false, locked: false, ends: false },
  { id: "c3", key: "3", text: "Pay me first.", check: "Requires 50 gold", seen: false, locked: true, ends: false },
  { id: "c4", key: "4", text: "I will see to it.", check: "", seen: true, locked: false, ends: true },
];
const MEMBERS_ = [
  { id: "brenna", name: "Brenna", portrait: 0, level: 7, health: 48, healthMax: 64, resource: 0, resourceMax: 0, status: "", active: true, down: false },
  { id: "idris", name: "Idris", portrait: 0, level: 7, health: 21, healthMax: 38, resource: 14, resourceMax: 30, status: "Concentrating", active: false, down: false },
  { id: "sela", name: "Sela", portrait: 0, level: 6, health: 0, healthMax: 42, resource: 0, resourceMax: 0, status: "Unconscious", active: false, down: true },
];
const MARKERS = [
  { id: "you", x: 0.5, y: 0.5, icon: "person", label: "You", kind: "player", tracked: false, discovered: true },
  { id: "abbey", x: 0.2, y: 0.3, icon: "home", label: "Greyfen Abbey", kind: "waypoint", tracked: false, discovered: true },
  { id: "lair", x: 0.8, y: 0.75, icon: "skull", label: "Drowned Hollow", kind: "danger", tracked: false, discovered: false },
];

const PAGES: Record<string, string> = {
  invTest: `<meta name="screen" content="invTest" data-layer="menu" data-input="menu">
<div style="width: stretch; height: stretch; padding: 16px"><RpgInventory title="Backpack" filters={$filters} filter={$filter} items={$items} selected={$selected} weight="12 / 60" load={0.2} gold="40" detailName={$detailName} detailIcon="shield" detailRarity="rare" detailKind="Shield" detailText="Oak." detailStats={$stats} detailValue="" canUse={false} canEquip={$canEquip} equipLabel="" canDrop={$canDrop} /></div>`,
  equipTest: `<meta name="screen" content="equipTest" data-layer="menu" data-input="menu">
<div style="width: stretch; height: stretch; padding: 16px"><RpgEquipment name="Brenna" subtitle="Fighter" portrait={0} slots={$slots} selected={$selected} stats={$stats} /></div>`,
  journalTest: `<meta name="screen" content="journalTest" data-layer="menu" data-input="menu">
<div style="width: stretch; height: stretch; padding: 16px"><RpgJournal title="Journal" filters={$filters} filter="active" quests={$quests} selected={$selected} detailName={$detailName} detailMeta="Main quest" detailText="Black water." objectives={$objectives} rewards={$rewards} tracked={$tracked} canTrack={$canTrack} canLocate={true} /></div>`,
  dialogueTest: `<meta name="screen" content="dialogueTest" data-layer="menu" data-input="menu">
<div style="width: stretch; height: stretch; padding: 16px; justify-content: end"><RpgDialogue speaker={$speaker} epithet="Ferry warden" portrait={0} line={$line} choices={$choices} selected={$selected} canContinue={$canContinue} canLog={true} /></div>`,
  partyTest: `<meta name="screen" content="partyTest" data-layer="menu" data-input="menu">
<div style="width: stretch; height: stretch; padding: 16px; gap: 16px"><RpgTurnOrder round={$round} entries={$entries} selected={$inspected} /><RpgParty members={$members} selected={$selected} orientation="vertical" compact={false} /></div>`,
  mapTest: `<meta name="screen" content="mapTest" data-layer="menu" data-input="menu">
<div style="width: stretch; height: stretch; padding: 16px"><RpgWorldMap map={0} region="Saltmarsh" markers={$markers} filters={$filters} selected={$selected} detailName={$detailName} detailText="Waypoint" canTravel={$canTravel} canTrack={true} /></div>`,
  hudTest: `<meta name="screen" content="hudTest" data-layer="hud" data-input="game">
<div style="width: stretch; height: stretch; padding: 16px; gap: 16px; align-items: center"><RpgCompass facing="NE" points={$points} markers={$markers} /><RpgTracker quest={$quest} distance="240 m" objectives={$objectives} updated={$updated} /></div>`,
};

beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-rpg-kit-"));
  const kit = join(root, "assets/ui-kits/rpg");
  mkdirSync(kit, { recursive: true });
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  for (const f of readdirSync(KIT)) if (/\.(svelte|ogt|json)$/.test(f)) cpSync(join(KIT, f), join(kit, f));
  for (const [name, text] of Object.entries(PAGES)) writeFileSync(join(root, "assets/ui", `${name}.svelte`), text);
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "fantasy"\nui "fantasy"\n');
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "RPG kit"\nid "rpg-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  cookProject(root, root);
});

afterAll(() => rmSync(root, { recursive: true, force: true }));

const DOCUMENTS = [...MEMBERS.map((d) => `ui-kits/rpg/${d}.svelte`), ...Object.keys(PAGES).map((p) => `ui/${p}.svelte`)];

/** An engine mounting `page` with these stores at `size`; its document events are collected in `events`. */
function engine(page: string, stores: Record<string, unknown>, events: Event[] = [], viewport?: unknown): Engine {
  const Collect = defineSystem("RpgKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      const style = og.load.style(world, "@styles/fantasy/style.ogt");
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
  if (viewport) og.ui.previewViewport(r.value.world, viewport as never);
  return r.value;
}

const elements = (e: Engine): El[] => (e.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements;
/** The first element whose path matches: a component instance's root is anonymous, so paths are matched by their tail. */
const find = (e: Engine, path: RegExp): El | undefined => elements(e).find((x) => path.test(x.path));
const all = (e: Engine, path: RegExp): El[] => elements(e).filter((x) => path.test(x.path));
const centre = (el: El): string => `${el.x + el.width / 2},${el.y + el.height / 2}`;
const tap = (e: Engine, el: El): void => {
  e.probe.commands(`0:touch_move:${centre(el)};0.02:touch_start;0.06:touch_end`);
  e.advance(8);
};
const keys = (e: Engine, ...k: string[]): void => {
  e.probe.commands(k.map((x, i) => `${i * 0.1}:key:${x}`).join(";"));
  e.advance(8 * k.length + 4);
};
const store = (e: Engine) => og.ui.scope(e.world, "t");
const intents = (events: Event[], names: string[]) => events.filter((e) => names.includes(e.name)).map((e) => `${e.name}:${e.rowKey}`);
const selected = (e: Engine, path: RegExp): string[] => all(e, path).filter((x) => x.states?.includes("selected")).map((x) => x.path.split("#").pop()!);

const press = (e: Engine, cmd: string): void => {
  e.probe.commands(`0:${cmd}`);
  e.advance(8);
};
const KEY = (code: string) => `key_down:${code};0.05:key_up:${code}`;
const PAD = (b: string) => `gamepad:${b};0.05:gamepad:-`;
const PHONE = { width: 390, height: 844, dpr: 2, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };

const INV = (over: Record<string, unknown> = {}) => ({ filters: FILTERS, filter: "all", items: ITEMS, selected: "shield", detailName: "Oakheart Shield", stats: [{ id: "a", label: "Armour", value: "38", delta: "+6", change: "better" }], canEquip: true, canDrop: true, ...over });
const JOURNAL = (over: Record<string, unknown> = {}) => ({ filters: [{ id: "active", label: "Active", count: 3 }, { id: "done", label: "Completed", count: 12 }], quests: QUESTS, selected: "ferry", detailName: "The Drowned Ferryman", objectives: [{ id: "o1", text: "Speak with Ysolde", progress: "", done: true, failed: false, optional: false }, { id: "o2", text: "Light the beacons", progress: "2 / 3", done: false, failed: false, optional: false }], rewards: [{ id: "xp", icon: "star", label: "850 XP" }], tracked: true, canTrack: true, ...over });
const SLOTS = [
  { id: "head", label: "Head", icon: "crown", name: "Warden’s Helm", rarity: "uncommon", empty: false, locked: false },
  { id: "hands", label: "Hands", icon: "", name: "", rarity: "common", empty: true, locked: false },
  { id: "ring", label: "Ring", icon: "gem", name: "Moonstone Band", rarity: "legendary", empty: false, locked: true },
];
const ENTRIES = [
  { id: "brenna", name: "Brenna", portrait: 0, side: "ally", current: true, done: false },
  { id: "drowned", name: "Drowned Oarsman", portrait: 0, side: "enemy", current: false, done: false },
  { id: "sela", name: "Sela", portrait: 0, side: "ally", current: false, done: true },
];
const MAP = (over: Record<string, unknown> = {}) => ({ markers: MARKERS, filters: [{ id: "quest", label: "Quests", icon: "flag", on: true }, { id: "danger", label: "Dangers", icon: "skull", on: false }], selected: "abbey", detailName: "Greyfen Abbey", canTravel: true, ...over });

test("RpgInventory: a tap chooses by row key; categories, Equip and Drop send once; a refused Drop sends nothing", () => {
  const events: Event[] = [];
  const e = engine("invTest", INV({ canDrop: false }), events);
  try {
    e.advance(3);
    tap(e, find(e, /Item#potion$/)!);
    tap(e, find(e, /Group#gear$|Category#gear$/)!);
    tap(e, find(e, /\/Equip$/)!);
    const drop = find(e, /\/Drop$/)!;
    expect(drop.disabled).toBeTrue();
    tap(e, drop);
    expect(intents(events, ["choose", "category", "equip", "drop"])).toEqual(["choose:potion", "category:gear", "equip:"]);
  } finally {
    e.destroy();
  }
});

test("RpgInventory: X and pad Y equip, Q/E and the bumpers step categories, and the keycap follows the device", () => {
  const prompts: string[] = [];
  for (const [name, details, next, prev] of [["keyboard", KEY("KeyX"), KEY("KeyE"), KEY("KeyQ")], ["pad", PAD("y"), PAD("rightBumper"), PAD("leftBumper")]]) {
    const events: Event[] = [];
    const e = engine("invTest", INV(), events);
    try {
      e.advance(3);
      press(e, details);
      press(e, next);
      press(e, prev);
      expect(`${name} ${intents(events, ["equip", "categoryNext", "categoryPrev"])}`).toBe(`${name} equip:,categoryNext:,categoryPrev:`);
      prompts.push(find(e, /\/EquipKey$/)!.text);
    } finally {
      e.destroy();
    }
  }
  expect(prompts).toEqual(["X", "Y"]);
});

test("RpgInventory: selection keeps its item through a reorder, a removal and a restore", () => {
  const e = engine("invTest", INV({ selected: "potion" }));
  try {
    e.advance(3);
    expect(selected(e, /Item#[a-z]+$/)).toEqual(["potion"]);
    const items = store(e).collection("items", ITEMS, { key: "id" });
    items.move("potion", "blade");
    e.advance(2);
    expect(selected(e, /Item#[a-z]+$/)).toEqual(["potion"]);
    expect(all(e, /Item#[a-z]+$/).map((x) => x.path.split("#").pop())[0]).toBe("potion");
    const snap = e.probe.snapshot() as { id: string };
    items.remove("blade");
    e.advance(2);
    expect(selected(e, /Item#[a-z]+$/)).toEqual(["potion"]);
    expect(find(e, /Item#blade$/)).toBeUndefined();
    expect(e.probe.restore(snap.id)).toBeTrue();
    e.advance(2);
    expect(selected(e, /Item#[a-z]+$/)).toEqual(["potion"]);
    expect(find(e, /Item#blade$/)).toBeDefined();
  } finally {
    e.destroy();
  }
});

test("RpgInventory: an empty bag says so and shows no card", () => {
  const e = engine("invTest", INV({ items: [], selected: "", detailName: "" }));
  try {
    e.advance(3);
    expect(elements(e).some((x) => x.text === "Your pack is empty")).toBeTrue();
    expect(find(e, /\/Equip$/)).toBeUndefined();
  } finally {
    e.destroy();
  }
});

test("RpgEquipment: a tap chooses a slot by key, a locked slot sends nothing, and Unequip sends once", () => {
  const events: Event[] = [];
  const e = engine("equipTest", { slots: SLOTS, selected: "head", stats: [{ id: "ac", label: "Armour class", value: "17", delta: "+2", change: "better" }] }, events);
  try {
    e.advance(3);
    tap(e, find(e, /Slot#hands$/)!);
    const ring = find(e, /Slot#ring$/)!;
    expect(ring.disabled).toBeTrue();
    tap(e, ring);
    tap(e, find(e, /\/Unequip$/)!);
    expect(intents(events, ["choose", "unequip"])).toEqual(["choose:hands", "unequip:"]);
  } finally {
    e.destroy();
  }
});

test("RpgJournal: a tap opens a quest by key, Track sends once from a tap, X or pad Y, and a refused Track sends nothing", () => {
  const events: Event[] = [];
  const e = engine("journalTest", JOURNAL(), events);
  try {
    e.advance(3);
    tap(e, find(e, /Quest#bells$/)!);
    tap(e, find(e, /\/Track$/)!);
    press(e, KEY("KeyX"));
    press(e, PAD("y"));
    expect(intents(events, ["choose", "track"])).toEqual(["choose:bells", "track:", "track:", "track:"]);
    store(e).writable("canTrack", true).set(false);
    e.advance(2);
    events.length = 0;
    tap(e, find(e, /\/Track$/)!);
    press(e, KEY("KeyX"));
    expect(intents(events, ["track"])).toEqual([]);
  } finally {
    e.destroy();
  }
  const empty = engine("journalTest", JOURNAL({ quests: [], selected: "", detailName: "", objectives: [], rewards: [] }));
  try {
    empty.advance(3);
    expect(elements(empty).some((x) => x.text === "No quests here yet")).toBeTrue();
    expect(find(empty, /\/Track$/)).toBeUndefined();
  } finally {
    empty.destroy();
  }
});

test("RpgDialogue: a reply sends its key once from a tap or the keyboard, a locked reply sends nothing, and narration continues on confirm", () => {
  const events: Event[] = [];
  const e = engine("dialogueTest", { speaker: "Old Ysolde", line: "Light the beacons.", choices: CHOICES, selected: "c1", canContinue: false }, events);
  try {
    e.advance(3);
    tap(e, find(e, /Reply#c2$/)!);
    const locked = find(e, /Reply#c3$/)!;
    expect(locked.disabled).toBeTrue();
    tap(e, locked);
    expect(find(e, /\/Continue$/)).toBeUndefined();
    expect(intents(events, ["choose", "advance"])).toEqual(["choose:c2"]);
    events.length = 0;
    press(e, KEY("ArrowDown"));
    press(e, KEY("Enter"));
    expect(intents(events, ["choose"]).length).toBe(1);
  } finally {
    e.destroy();
  }
  const narrationEvents: Event[] = [];
  const n = engine("dialogueTest", { speaker: "", line: "The bell tolls once.", choices: [], selected: "", canContinue: true }, narrationEvents);
  try {
    n.advance(3);
    expect(find(n, /\/Speaker$/)).toBeUndefined();
    press(n, PAD("a"));
    expect(intents(narrationEvents, ["advance"])).toEqual(["advance:"]);
  } finally {
    n.destroy();
  }
});

test("RpgParty and RpgTurnOrder: members and entries choose by key, in the game's order", () => {
  const events: Event[] = [];
  const e = engine("partyTest", { round: 3, entries: ENTRIES, inspected: "", members: MEMBERS_, selected: "brenna" }, events);
  try {
    e.advance(3);
    expect(all(e, /Entry#[a-z]+$/).map((x) => x.path.split("#").pop())).toEqual(["brenna", "drowned", "sela"]);
    tap(e, find(e, /Member#idris$/)!);
    tap(e, find(e, /Entry#drowned$/)!);
    expect(intents(events, ["choose"])).toEqual(["choose:idris", "choose:drowned"]);
    store(e).collection("entries", ENTRIES, { key: "id" }).move("brenna", "sela");
    e.advance(30);
    expect(all(e, /Entry#[a-z]+$/).map((x) => x.path.split("#").pop())).toEqual(["drowned", "brenna", "sela"]);
  } finally {
    e.destroy();
  }
  const empty = engine("partyTest", { round: 0, entries: [], inspected: "", members: [], selected: "" });
  try {
    empty.advance(3);
    expect(find(empty, /Entry#[a-z]+$/)).toBeUndefined();
    expect(elements(empty).some((x) => x.text === "No companions")).toBeTrue();
  } finally {
    empty.destroy();
  }
});

test("RpgWorldMap: each marker is centred where the game projects it, chooses by key, and a refused travel sends nothing", () => {
  const events: Event[] = [];
  const e = engine("mapTest", MAP({ canTravel: false }), events);
  try {
    e.advance(3);
    const field = find(e, /\/Field$/)!;
    for (const m of MARKERS) {
      const el = find(e, new RegExp(`Marker#${m.id}$`))!;
      expect(Math.abs(el.x + el.width / 2 - (field.x + m.x * field.width))).toBeLessThan(1.5);
      expect(Math.abs(el.y + el.height / 2 - (field.y + m.y * field.height))).toBeLessThan(1.5);
      expect(Math.min(el.width, el.height)).toBeGreaterThanOrEqual(44);
    }
    tap(e, find(e, /Marker#lair$/)!);
    tap(e, find(e, /Filter#danger$/)!);
    tap(e, find(e, /\/Travel$/)!);
    press(e, KEY("KeyX"));
    tap(e, find(e, /\/Track$/)!);
    expect(intents(events, ["choose", "filter", "travel", "track"])).toEqual(["choose:lair", "filter:danger", "track:"]);
  } finally {
    e.destroy();
  }
});

test("RpgCompass and RpgTracker: compass marks sit where the game projects them; the tracker hides with no quest", () => {
  const points = [{ id: "n", x: 0.2, label: "N" }, { id: "ne", x: 0.5, label: "NE" }];
  const markers = [{ id: "beacon", x: 0.6, icon: "flag", kind: "quest", distance: "240 m", above: false, below: true }];
  const e = engine("hudTest", { points, markers, quest: "The Drowned Ferryman", objectives: [{ id: "o", text: "Light the beacons", progress: "2 / 3", done: false, optional: false }], updated: "o" });
  try {
    e.advance(3);
    const strip = find(e, /\/Strip$/)!;
    const at = (el: El) => el.x + el.width / 2 - strip.x;
    expect(Math.abs(at(find(e, /#beacon$/)!) - 0.6 * strip.width)).toBeLessThan(1.5);
    expect(Math.abs(at(find(e, /#ne$/)!) - 0.5 * strip.width)).toBeLessThan(1.5);
    expect(elements(e).some((x) => x.text === "2 / 3")).toBeTrue();
    store(e).writable("quest", "").set("");
    e.advance(3);
    expect(elements(e).some((x) => x.text === "Light the beacons")).toBeFalse();
  } finally {
    e.destroy();
  }
});

for (const [page, stores] of [["invTest", INV()], ["journalTest", JOURNAL()], ["mapTest", MAP()], ["dialogueTest", { speaker: "Old Ysolde", line: "Light the beacons, and the water gives up what it keeps.", choices: CHOICES, selected: "c1", canContinue: false }], ["equipTest", { slots: SLOTS, selected: "head", stats: [] }]] as const) {
  test(`${page} on a 390 px phone: nothing runs off its width, and every button is a 44 px touch target`, () => {
    const e = engine(page, stores as Record<string, unknown>, [], PHONE);
    try {
      e.advance(3);
      const els = elements(e);
      const wide = els.filter((x) => x.width > 0 && x.x + x.width > 390.5 && !/Scroll|Strip/.test(x.path)).map((x) => `${x.path} ${x.x}+${x.width}`);
      expect(wide).toEqual([]);
      const small = els.filter((x) => x.role === "button" && x.width > 0 && Math.min(x.width, x.height) < 43.5).map((x) => `${x.path} ${x.width}x${x.height}`);
      expect(small).toEqual([]);
    } finally {
      e.destroy();
    }
  });
}
