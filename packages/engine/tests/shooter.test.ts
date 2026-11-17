// The shooter components as a game uses them (docs/ui.md §6.1): built in,
// used by name from the game's own pages with nothing copied or loaded but
// the pages, using the default style, each member mounted from named stores
// and driven through the probe's touch, key and pad commands; the HUD is
// laid out again in the fantasy and scifi packs.
//
// Ways it could fail: an intent sent without its row key or for a disabled
// (locked) slot; selection following a row's index instead of its key
// through a reorder, a removal or a restore; the empty loadout without its
// outcome or its actions; a hit mark that does not replay on a new id; the
// killfeed out of the collection's order or crowding a phone; HUD readouts
// under the phone's safe area or on top of each other, in the default style
// or under a pack's larger type and rims; a reticle a pack leaves undrawn
// because one of the pack's :root tokens has the name of its own variable;
// a pack's ink on the magazine count hiding its low and empty inks; a pack's
// button plate drawn again over a slot's content when the keyboard focuses it.
import { afterAll, beforeAll, expect, setDefaultTimeout, test } from "bun:test";

// The tests run on ticks, not the clock; a time limit only guards a hang
// (the cook of the pack's fonts takes most of it).
setDefaultTimeout(60_000);
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { cookProject } from "./support/og-bin.ts";

type El = { id: string; path: string; text: string; x: number; y: number; width: number; height: number; states?: string[]; disabled?: boolean; name?: string; role?: string };
type Event = { mount: string; path: string; rowKey: string; kind: string; name: string };

const ENGINE = dirname(dirname(fileURLToPath(import.meta.url)));
const PACKS = ["fantasy", "scifi"] as const;
let root = "";

const SLOTS = [
  { id: "strider", category: "primary", name: "Strider AR", icon: "sword", detail: "25 / 75", equipped: true, locked: false, cost: "2,900" },
  { id: "halcyon", category: "primary", name: "Halcyon", icon: "sword", detail: "30 / 90", equipped: false, locked: false, cost: "2,900" },
  { id: "pike", category: "primary", name: "Pike .50", icon: "sword", detail: "5 / 10", equipped: false, locked: true, cost: "4,700" },
  { id: "wren", category: "secondary", name: "Wren Pistol", icon: "bolt", detail: "15 / 45", equipped: true, locked: false, cost: "500" },
];
// A game's own label wins; an empty one (and the empty title) takes the component's string.
const CATEGORIES = [{ id: "primary", label: "Rifles" }, { id: "secondary", label: "" }];
const FEED = ["k4", "k3", "k2", "k1"].map((id, i) => ({ id, killer: `Killer${i}`, victim: `Victim${i}`, weapon: "Halcyon", headshot: i === 0, self: i === 1, team: i % 2 ? "enemy" : "ally" }));

const PAGES: Record<string, string> = {
  loadoutTest: `<meta name="screen" content="loadoutTest" data-layer="menu" data-input="menu">
<div style="width: stretch; height: stretch; padding: 16px"><Loadout title={$title} categories={$categories} slots={$slots} selected={$selected} /></div>`,
  hudTest: `<meta name="component" content="hudTest">
<div style="width: stretch; height: stretch"><CombatHud health={$health} healthMax={100} shield={40} shieldMax={100} spread={$spread} hit={$hit} hitAt={$hitAt} aimed={false} shape="cross" weapon="Halcyon Carbine" mode="auto" magazine={$magazine} capacity={28} reserve={112} reloading={$reloading} hits={$hits} feed={$feed} /></div>`,
};

// The HUD members the combat page lays out, each mounted from every fixture
// its document declares, as a game's tooling reads them.
const HUD_MEMBERS = ["shooter/vitals", "shooter/squad", "shooter/objectives", "shooter/radar", "shooter/round", "shooter/pickups", "shooter/damage", "shooter/prompt", "shooter/effects", "shooter/abilities", "hud/ammo-counter", "hud/killfeed", "hud/crosshair", "hud/damage-direction", "world/compass"].map((path) => {
  const text = readFileSync(join(ENGINE, `src/ui/components/${path}.svelte`), "utf8");
  const component = /<meta name="component" content="([^"]+)">/.exec(text)![1]!;
  const fixtures = Object.fromEntries([...text.matchAll(/<meta name="fixture:([^"]+)" content='([^']*)'>/g)].map((m) => [m[1]!, JSON.parse(m[2]!) as Record<string, unknown>]));
  return { path, component, fixtures };
});

beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-shooter-kit-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  mkdirSync(join(root, "assets/fonts"), { recursive: true });
  cpSync(join(ENGINE, "../../templates/styles/shared/InstrumentSans-Regular.ttf"), join(root, "assets/fonts/body.ttf"));
  writeFileSync(join(root, "assets/fonts/body.ttf.ogt"), 'type import\nversion 1\n---\nfont {\n  charset latin\n}\n');
  for (const [name, text] of Object.entries(PAGES)) writeFileSync(join(root, "assets/ui", `${name}.svelte`), text);
  // A game mounts its own page, which uses the member by name: one host page per member, its props from stores.
  for (const m of HUD_MEMBERS) {
    const props = Object.keys(m.fixtures.default!).map((k) => `${k}={$${k}}`).join(" ");
    writeFileSync(join(root, `assets/ui/host-${m.component}.svelte`), `<meta name="screen" content="Host_${m.component}" data-layer="menu" data-input="menu">\n<div style="width: stretch; height: stretch; padding: 16px"><${m.component} ${props} /></div>\n`);
  }
  // A look naming a pack is what makes the cook bundle it (@styles/<pack>).
  for (const pack of PACKS) writeFileSync(join(root, `assets/looks/${pack}.ogt`), `type look\nversion 1\n---\nname "shooter kit ${pack}"\nui "${pack}"\n`);
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Shooter kit"\nid "shooter-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  cookProject(root, root);
});

// rmSync("") would remove the working directory (the engine package).
afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });

const DOCUMENTS = ["ui/loadoutTest.svelte", "ui/hudTest.svelte"];
/** The documents an engine loads: the two pages, and a member's host page when it mounts one. */
const documents = (page: string): string[] => page.startsWith("Host_") ? [...DOCUMENTS, `ui/host-${page.slice(5)}.svelte`] : DOCUMENTS;

/** An engine mounting `page` with these stores, in the default style or in `pack`; its document events are collected in `events`. */
function engine(page: string, stores: Record<string, unknown>, events: Event[] = [], pack = ""): Engine {
  const Collect = defineSystem("ShooterKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      // A pack brings its fonts; the default style is given one.
      if (pack) {
        const style = og.load.style(world, `@styles/${pack}`);
        if (!style.ok) throw new Error(style.reason);
        og.ui.configure(world, { style: style.value });
      } else {
        const font = og.load.font(world, "fonts/body.ttf");
        if (!font.ok) throw new Error(font.reason);
        og.ui.configure(world, { font: font.value });
      }
      for (const path of documents(page)) {
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
  engine("loadoutTest", { title: "", categories: CATEGORIES, slots, selected }, events);
const intents = (events: Event[]) => events.filter((e) => ["select", "equip", "back"].includes(e.name)).map((e) => `${e.name}:${e.rowKey}`);

test("Loadout: a tap selects by row key, a locked slot sends nothing, and Equip and Back send once", () => {
  const events: Event[] = [];
  const e = loadout(events);
  try {
    e.advance(3);
    expect(find(e, /\/Title$/)!.text).toBe("Loadout");
    // A style's label role chooses casing; the group names and order stay.
    expect(elements(e).filter((x) => x.role === "heading" && x.path.includes("/Groups/")).map((x) => x.text.toLowerCase())).toEqual(["rifles", "secondary"]);
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

test("Loadout: keyboard and pad select the focused slot, and the Equip action equips it", () => {
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

test("Loadout: the Equip action on a focused slot equips that slot, not the selected one", () => {
  const events: Event[] = [];
  const e = loadout(events, "halcyon");
  try {
    e.advance(3);
    // Focus Wren, a row other than the selected one, then fire Equip once:
    // only the focused row's handler takes it, so one intent, Wren's.
    const focused = () => elements(e).find((x) => x.states?.includes("focused"))?.path ?? "";
    for (let i = 0; i < 8 && !/#wren$/.test(focused()); i++) {
      e.probe.commands("0:key_down:ArrowDown;0.05:key_up:ArrowDown");
      e.advance(8);
    }
    expect(focused()).toMatch(/#wren$/);
    expect(intents(events)).toEqual([]);
    e.probe.commands("0:key_down:KeyX;0.05:key_up:KeyX");
    e.advance(8);
    expect(intents(events)).toEqual(["equip:wren"]);
  } finally {
    e.destroy();
  }
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

// What the native host drew, as og ui check reads it: each line of text with
// its ink, and every filled rectangle text can sit on, in CSS px.
type Frame = { texts: { text: string; color: number[] }[]; grounds: [number, number, number, number][] };
const frame = (e: Engine): Frame => JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id)) as Frame;

for (const pack of ["", ...PACKS]) {
  test(`AmmoCounter in ${pack || "the default style"}: the count's ink tells a low magazine and an empty one from a full one`, () => {
    const e = engine("hudTest", HUD, [], pack);
    try {
      // 24 of 28 rounds, then 6 (under the quarter that is low), then none.
      const ink = (magazine: number): string => {
        store(e).writable("magazine", 0).set(magazine);
        e.advance(30);
        const count = frame(e).texts.find((t) => t.text === String(magazine));
        return count ? count.color.map((c) => c.toFixed(2)).join(" ") : `no count ${magazine}`;
      };
      const [full, low, empty] = [ink(24), ink(6), ink(0)];
      expect(`${full} | ${low} | ${empty}`).not.toContain("no count");
      expect(new Set([full, low, empty]).size).toBe(3);
    } finally {
      e.destroy();
    }
  });
}

for (const pack of PACKS) {
  test(`Loadout in ${pack}: a slot the keyboard focuses keeps its content, with no plate drawn over it`, () => {
    const e = engine("loadoutTest", { title: "", categories: CATEGORIES, slots: SLOTS, selected: "halcyon" }, [], pack);
    try {
      e.advance(3);
      // The filled rectangles covering a slot: its own plate's layers.
      const plates = (slot: El): number => frame(e).grounds.filter(([x, y, w, h]) => Math.abs(x - slot.x) <= 2 && Math.abs(y - slot.y) <= 2 && Math.abs(w - slot.width) <= 4 && Math.abs(h - slot.height) <= 4).length;
      const slots = () => elements(e).filter((x) => /#(strider|halcyon|pike|wren)$/.test(x.path));
      const before = Object.fromEntries(slots().map((x) => [x.path, plates(x)]));
      e.probe.commands("0:key_down:ArrowDown;0.05:key_up:ArrowDown");
      e.advance(8);
      const focused = slots().find((x) => x.states?.includes("focused"));
      expect(focused).toBeDefined();
      expect(`${focused!.path.split("#").pop()} ${plates(focused!)}`).toBe(`${focused!.path.split("#").pop()} ${before[focused!.path]}`);
    } finally {
      e.destroy();
    }
  });
}

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
const LANDSCAPE = { width: 844, height: 390, dpr: 2, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true, breakpoint: "phoneLandscape" };
const DESKTOP = { width: 1920, height: 1080, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };
const meet = (a: El, b: El): boolean => a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;

for (const pack of ["", ...PACKS]) for (const [name, viewport] of [["desktop", DESKTOP], ["phone", PHONE], ["landscape phone", LANDSCAPE]] as const) {
  test(`CombatHud on a ${name} in ${pack || "the default style"}: readouts inside the safe area and clear of each other; a phone shows the newest feed row upright and three on its side`, () => {
    const e = engine("hudTest", HUD, [], pack);
    try {
      og.ui.previewViewport(e.world, viewport as never);
      e.advance(4);
      // The reticle's arms and dot have their size under a pack's tokens too.
      for (const mark of ["Up", "Right", "Dot"]) {
        const drawn = find(e, new RegExp(`Aim/[^/]+/${mark}$`));
        expect(`${mark} ${Math.min(drawn?.width ?? 0, drawn?.height ?? 0) > 0}`).toBe(`${mark} true`);
      }
      const box = (name: string) => find(e, new RegExp(`page/[^/]+/${name}$`))!;
      const readouts = ["Vitals", "Ammo", "Feed"].map(box);
      for (const r of readouts) {
        expect(r.y).toBeGreaterThanOrEqual(viewport.safeArea.top);
        expect(r.y + r.height).toBeLessThanOrEqual(viewport.height - viewport.safeArea.bottom);
        expect(r.x).toBeGreaterThanOrEqual(viewport.safeArea.left);
        expect(r.x + r.width).toBeLessThanOrEqual(viewport.width - viewport.safeArea.right);
      }
      for (let i = 0; i < readouts.length; i++) for (let j = i + 1; j < readouts.length; j++) expect(`${i}-${j} ${meet(readouts[i]!, readouts[j]!)} ${JSON.stringify([readouts[i], readouts[j]].map((r) => [r!.x, r!.y, r!.width, r!.height]))}`).toContain(" false ");
      const shown = elements(e).filter((x) => /Feed\/[^/]+\/[^/]+#k\d$/.test(x.path) && x.height > 0).length;
      expect(shown).toBe(name === "desktop" ? 4 : name === "phone" ? 1 : 3);
    } finally {
      e.destroy();
    }
  });
}

test("CombatHud and Loadout text follows the player's text scale", () => {
  // The magazine count, a killfeed name and a slot's name, by their text.
  for (const [page, stores, text] of [["hudTest", HUD, "24"], ["hudTest", HUD, "Killer0"], ["loadoutTest", { title: "", categories: CATEGORIES, slots: SLOTS, selected: "halcyon" }, "Halcyon"]] as const) {
    const height = (scale: number): number => {
      const e = engine(page, stores);
      try {
        og.ui.previewViewport(e.world, DESKTOP as never);
        og.ui.configure(e.world, { textScale: scale });
        e.advance(4);
        return elements(e).find((x) => x.text === text && x.height > 0)!.height;
      } finally {
        e.destroy();
      }
    };
    // A line box rounds to whole pixels, so the ratio is 1.5 within a few hundredths.
    expect(`${text} ${(height(1.5) / height(1)).toFixed(1)}`).toBe(`${text} 1.5`);
  }
});

const hudMember = (component: string, fixture = "default", view: typeof DESKTOP | typeof PHONE = DESKTOP): Engine => {
  const m = HUD_MEMBERS.find((x) => x.component === component)!;
  const e = engine(`Host_${component}`, { ...m.fixtures.default, ...m.fixtures[fixture] });
  og.ui.previewViewport(e.world, view as never);
  e.advance(4);
  return e;
};
/** The words shown, lower-cased: the HUD sets its labels in capitals. */
const texts = (e: Engine): string[] => elements(e).filter((x) => x.width > 0).map((x) => x.text.toLowerCase());

for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  for (const m of HUD_MEMBERS) {
    test(`${m.component} mounts from every fixture on a ${view} without errors, inside the screen's width`, () => {
      for (const fixture of Object.keys(m.fixtures)) {
        const e = hudMember(m.component, fixture, viewport);
        try {
          expect(`${fixture}: ${store(e).takeErrors().join("; ")}`).toBe(`${fixture}: `);
          const spill = elements(e).filter((x) => x.width > 0 && (x.x < -0.5 || x.x + x.width > viewport.width + 0.5)).map((x) => `${x.path} [${x.x}, ${x.x + x.width}]`);
          expect(`${fixture}: ${spill.join(", ")}`).toBe(`${fixture}: `);
        } finally {
          e.destroy();
        }
      }
    });
  }
}

test("The HUD members show what the game told them, in words as well as colour", () => {
  const check = (component: string, fixture: string, want: string[], not: string[] = []) => {
    const e = hudMember(component, fixture);
    try {
      for (const w of want) expect(`${component}.${fixture} ${texts(e).includes(w)} ${w}`).toBe(`${component}.${fixture} true ${w}`);
      for (const w of not) expect(`${component}.${fixture} ${texts(e).includes(w)} ${w}`).toBe(`${component}.${fixture} false ${w}`);
    } finally {
      e.destroy();
    }
  };
  check("ShooterVitals", "default", ["ysolde", "lv 24", "64", "/ 100"]);
  check("ShooterVitals", "noshield", ["150", "/ 200"], ["lv 0"]);
  check("ShooterSquad", "default", ["moraine", "reviving", "quill"]);
  check("ShooterObjectives", "default", ["objective", "secure the relay tower", "2 / 3", "0:45"]);
  check("ShooterObjectives", "empty", [], ["objective"]);
  check("ShooterRound", "default", ["round 2", "1:42", "7", "5"]);
  check("ShooterRound", "clock", ["14:58"], ["0"]);
  check("ShooterPickups", "default", ["frag grenade", "+1", "light rounds", "+30"]);
  check("ShooterDamage", "default", ["headshot", "86", "31"]);
  check("ShooterDamage", "shield", ["24"], ["headshot"]);
  check("ShooterPrompt", "default", ["e", "open supply bin"]);
  check("ShooterPrompt", "disabled", ["needs a keycard"]);
  check("ShooterEffects", "default", ["0:12", "×3 4.2", "6"]);
  check("ShooterAbilities", "default", ["q", "6", "×2", "74%", "38"]);
  check("ShooterAbilities", "ready", ["z"], ["100%"]);
  check("AmmoCounter", "default", ["hollow smg", "32", "2", "24", "120", "vk-7 carbine", "auto"]);
  check("AmmoCounter", "low", ["2", "brace-12"], ["hollow smg"]);
  check("Killfeed", "default", ["you", "vanta", "hs"], ["vk-7 carbine"]);
  check("Killfeed", "words", ["brace-12", "halden"], ["pike .50"]);
  check("Compass", "default", ["042°", "n", "ne", "e", "85m"]);
});
