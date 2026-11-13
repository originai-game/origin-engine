// The rpg components as a game uses them (docs/ui.md §6.1): built in, used
// by name from the game's own pages with nothing copied or loaded but the
// pages, dressed by the fantasy pack, each member mounted from its own
// fixture:default and driven through the probe's touch, key and pad commands.
//
// Ways it could fail: an intent sent without its row key, or sent for a
// disabled command or a skill still cooling down; the selection following a
// row's index instead of its key; a member that only fits the desktop; a
// member not loaded with the page that uses it; the pack losing to a
// component rule of equal specificity (og-components < og-pack).
import { afterAll, beforeAll, expect, setDefaultTimeout, test } from "bun:test";

// The tests run on ticks, not the clock; a time limit only guards a hang.
// macrunner shares its cores with other lanes' gates (load averages of 40),
// where one engine boot takes several seconds, so the limits leave room.
setDefaultTimeout(60_000);
import { cpSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "ignore"; stderr: "pipe" }): { exitCode: number; stderr: Uint8Array } };
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

type El = { id: string; path: string; text: string; x: number; y: number; width: number; height: number; states?: string[]; disabled?: boolean; name?: string; role?: string };
type Event = { mount: string; path: string; rowKey: string; kind: string; name: string };

const ENGINE = dirname(dirname(fileURLToPath(import.meta.url)));
const GROUP = join(ENGINE, "src/ui/components/rpg");
const STYLES = join(ENGINE, "../../templates/styles");
let root = "";
let styles = "";

/** Each member's component name and its fixture:default, read from its document as a game's tooling reads them. */
const MEMBERS: Record<string, { component: string; fixture: Record<string, unknown> }> = {};

// The backpack's grid padding (`.grid { padding: var(--space-xs) }`) and a
// pack rule of equal specificity on it: the pack's must win.
const PROBE_PACK = ".grid { padding: 37px; }\n";

beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-rpg-kit-"));
  styles = mkdtempSync(join(tmpdir(), "og-rpg-styles-"));
  for (const d of ["fantasy", "shared"]) cpSync(join(STYLES, d), join(styles, d), { recursive: true });
  writeFileSync(join(styles, "fantasy/zz-cascade-probe.css"), PROBE_PACK);
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Rpg kit"\nid "rpg-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  // A look naming the pack is what makes the cook bundle it (@styles/fantasy).
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "rpg kit fantasy"\nui "fantasy"\n');
  for (const f of readdirSync(GROUP).filter((f) => f.endsWith(".svelte"))) {
    const text = readFileSync(join(GROUP, f), "utf8");
    const component = /<meta name="component" content="([^"]+)">/.exec(text)![1]!;
    const fixture = /<meta name="fixture:default" content='([^']*)'>/.exec(text);
    const name = f.replace(".svelte", "");
    MEMBERS[name] = { component, fixture: fixture ? JSON.parse(fixture[1]!) : {} };
    // A game mounts its own page, which uses the member by name: one host page per member, its props from stores.
    const props = Object.keys(MEMBERS[name]!.fixture).map((k) => `${k}={$${k}}`).join(" ");
    writeFileSync(join(root, `assets/ui/host-${name}.svelte`), `<meta name="screen" content="Host_${component}" data-layer="menu" data-input="menu">\n<div style="width: stretch; height: stretch; padding: 16px"><${component} ${props} /></div>\n`);
  }
  // og cook bundles the pack from OG_STYLES_DIR: the copy holding the probe sheet.
  const cook = Bun.spawnSync([ogBin(), "cook", root], { cwd: root, env: { ...process.env, OG_STYLES_DIR: styles }, stdout: "ignore", stderr: "pipe" });
  if (cook.exitCode !== 0) throw new Error(`cooking ${root}: ${new TextDecoder().decode(cook.stderr)}`);
});

// A setup that throws before mkdtempSync leaves these empty, and
// rmSync("") removes the working directory: the engine package.
afterAll(() => {
  if (root) rmSync(root, { recursive: true, force: true });
  if (styles) rmSync(styles, { recursive: true, force: true });
});

const DOCUMENTS = () => Object.keys(MEMBERS).map((m) => `ui/host-${m}.svelte`);

type View = { width: number; height: number; dpr: number; safeArea: { left: number; top: number; right: number; bottom: number }; coarsePointer: boolean; breakpoint: string };
const DESKTOP: View = { width: 1280, height: 720, dpr: 1.5, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktopHd" };
const PHONE: View = { width: 390, height: 844, dpr: 2, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };

/** An engine mounting `component` dressed by the fantasy pack (or by the default style), its document events collected in `events`. */
function engine(component: string, stores: Record<string, unknown>, o: { events?: Event[]; view?: View; pack?: boolean } = {}): Engine {
  const events = o.events ?? [];
  const Collect = defineSystem("RpgKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      if (o.pack !== false) {
        const style = og.load.style(world, "@styles/fantasy");
        if (!style.ok) throw new Error(style.reason);
        og.ui.configure(world, { style: style.value });
      }
      for (const path of DOCUMENTS()) {
        const loaded = og.load.document(world, path);
        if (!loaded.ok) throw new Error(`${path}: ${loaded.reason}`);
      }
    },
    ui(world) {
      const scope = og.ui.scope(world, "t");
      const inputs = Object.fromEntries(Object.entries(stores).map(([k, v]) => [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
      const mounted = scope.mount("page", `Host_${component}`, inputs);
      if (!mounted.ok) throw new Error(mounted.reason);
    },
  }) as unknown as GameToken, { root });
  if (!r.ok) throw new Error(r.reason);
  og.ui.previewViewport(r.value.world, (o.view ?? DESKTOP) as never);
  return r.value;
}

const elements = (e: Engine): El[] => (e.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements;
const find = (e: Engine, path: RegExp): El | undefined => elements(e).find((x) => path.test(x.path));
const centre = (el: El): string => `${el.x + el.width / 2},${el.y + el.height / 2}`;
const tap = (e: Engine, el: El): void => {
  e.probe.commands(`0:touch_move:${centre(el)};0.02:touch_start;0.06:touch_end`);
  e.advance(8);
};
const intents = (events: Event[], names: string[]) => events.filter((e) => names.includes(e.name)).map((e) => `${e.name}:${e.rowKey}`);
const member = (name: string, over: Record<string, unknown> = {}, o: Parameters<typeof engine>[2] = {}) =>
  engine(MEMBERS[name]!.component, { ...MEMBERS[name]!.fixture, ...over }, o);
const run = (e: Engine, f: (e: Engine) => void) => {
  try {
    e.advance(3);
    f(e);
  } finally {
    e.destroy();
  }
};

const key = (e: Engine, k: string): void => {
  e.probe.commands(`0:key_down:${k};0.05:key_up:${k}`);
  e.advance(8);
};
/** A pad press: buttons (a, b, leftBumper…) or "-" with the left stick's x,y. */
const pad = (e: Engine, spec: string): void => {
  e.probe.commands(`0:gamepad:${spec};0.05:gamepad:-`);
  e.advance(8);
};
const errors = (e: Engine): readonly string[] => og.ui.scope(e.world, "t").takeErrors();
const row = (e: Engine, k: string): El => find(e, new RegExp(`#${k}$`))!;

for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  test(`every member mounts from its fixture on a ${view}, dressed by the pack, inside the screen's width`, () => {
    for (const name of Object.keys(MEMBERS)) {
      run(member(name, {}, { view: viewport }), (e) => {
        e.advance(3);
        expect(`${name}: ${errors(e).join("; ")}`).toBe(`${name}: `);
        const box = find(e, /^t:page\/[^/]+$/)!;
        expect(`${name} ${box.x + box.width <= viewport.width + 0.5}`).toBe(`${name} true`);
      });
    }
  }, 180_000);
}

test("the pack beats a component rule of equal specificity (og-components < og-pack), and the component rule holds without the pack", () => {
  const inset = (pack: boolean) => {
    const e = member("backpack", {}, { pack });
    try {
      e.advance(3);
      const all = elements(e);
      const grid = all.find((x) => x.role === "grid")!;
      const cell = all.find((x) => x.role === "gridcell")!;
      return Math.round(cell.x - grid.x);
    } finally {
      e.destroy();
    }
  };
  expect(inset(true)).toBe(37);
  expect(inset(false)).not.toBe(37);
  expect(inset(false)).toBeGreaterThan(0);
});

test("RpgCommands: a tap chooses by key, the disabled command sends nothing, and each plate is named by its label", () => {
  const events: Event[] = [];
  run(member("commands", {}, { events }), (e) => {
    expect(row(e, "attack").name).toBe("Attack");
    expect(row(e, "flee").name).toBe("Flee");
    expect(row(e, "attack").states).toContain("selected");
    tap(e, row(e, "attack"));
    tap(e, row(e, "flee"));
    expect(intents(events, ["choose"])).toEqual(["choose:attack"]);
  });
});

test("RpgActionBar: a ready skill and a quick item report use by key; a skill cooling down is disabled and silent", () => {
  const events: Event[] = [];
  const skills = [
    { id: "cut", icon: "sword", key: "1", name: "Cut", cooldown: 0, ready: true },
    { id: "burst", icon: "star", key: "2", name: "Burst", cooldown: 0.5, ready: false },
  ];
  run(member("action-bar", { skills, items: [{ id: "draught", icon: "potion", key: "Q", name: "Draught", count: 9 }] }, { events }), (e) => {
    expect(row(e, "burst").disabled).toBeTrue();
    tap(e, row(e, "burst"));
    tap(e, row(e, "cut"));
    tap(e, row(e, "draught"));
    expect(intents(events, ["use"])).toEqual(["use:cut", "use:draught"]);
    expect(elements(e).some((x) => x.text === "9")).toBeTrue();
  });
});

test("RpgBackpack: cells and tabs report by key, the bumpers page the tabs, and the selection follows its key through a reorder", () => {
  const events: Event[] = [];
  run(member("backpack", {}, { events }), (e) => {
    tap(e, row(e, "potion"));
    tap(e, row(e, "gear"));
    key(e, "KeyE");
    pad(e, "leftBumper");
    expect(intents(events, ["choose", "choosetab", "tabNext", "tabPrev"])).toEqual(["choose:potion", "choosetab:gear", "tabNext:", "tabPrev:"]);
    expect(row(e, "blade").states).toContain("selected");
    const items = og.ui.scope(e.world, "t").collection("items", MEMBERS.backpack!.fixture.items as { id: string }[], { key: "id" });
    items.move("potion", "blade");
    e.advance(3);
    expect(row(e, "blade").states).toContain("selected");
    expect(row(e, "potion").states).not.toContain("selected");
    expect(row(e, "blade").x).toBeGreaterThan(row(e, "potion").x);
  });
});

test("RpgMenuBar: a tab reports its key; the close button, Escape and the pad's B close", () => {
  const events: Event[] = [];
  run(member("menu-bar", {}, { events }), (e) => {
    tap(e, row(e, "inv"));
    tap(e, find(e, /\/Close$/)!);
    key(e, "Escape");
    expect(intents(events, ["choosetab", "close"])).toEqual(["choosetab:inv", "close:", "close:"]);
  });
  const padded: Event[] = [];
  run(member("menu-bar", {}, { events: padded }), (e) => {
    pad(e, "b");
    expect(intents(padded, ["close"])).toEqual(["close:"]);
  });
});

// One test per member: each boots its own engine (about a second, two under
// GPU load), so five in one test crossed bun's 5 s limit on a busy machine.
for (const [name, k, intent] of [["conversation", "r2", "reply"], ["quest-log", "heir", "choose"], ["world-map", "crown", "choose"], ["party", "l", "choose"], ["character", "neck", "choose"]] as const) {
  test(`${name} reports the row tapped by its key`, () => {
    const events: Event[] = [];
    run(member(name, {}, { events }), (e) => {
      tap(e, row(e, k));
      expect(intents(events, [intent])).toEqual([`${intent}:${k}`]);
    });
  });
}

test("RpgWorldMap: the player's own marker is not a target", () => {
  const events: Event[] = [];
  run(member("world-map", {}, { events }), (e) => {
    expect(row(e, "me").disabled).toBeTrue();
    tap(e, row(e, "me"));
    expect(intents(events, ["choose"])).toEqual([]);
  });
});

test("RpgItemCard: its buttons report once, and a refused action is disabled", () => {
  const events: Event[] = [];
  run(member("item-card", { canSecondary: false }, { events }), (e) => {
    tap(e, find(e, /\/Primary$/)!);
    expect(find(e, /\/Secondary$/)!.disabled).toBeTrue();
    tap(e, find(e, /\/Secondary$/)!);
    expect(intents(events, ["equip", "discard"])).toEqual(["equip:"]);
  });
});

test("RpgPlaceCard: Mark reports, and Fast travel refused is silent", () => {
  const placed: Event[] = [];
  run(member("place-card", { canTravel: false }, { events: placed }), (e) => {
    tap(e, find(e, /\/Mark$/)!);
    tap(e, find(e, /\/Travel$/)!);
    expect(intents(placed, ["mark", "travel"])).toEqual(["mark:"]);
  });
});

test("RpgQuestTracker shows its distance only when the game gives one", () => {
  run(member("quest-tracker"), (e) => expect(elements(e).some((x) => x.text === "312 m")).toBeTrue());
  run(member("quest-tracker", { distance: "" }), (e) => expect(elements(e).some((x) => /\bm$/.test(x.text))).toBeFalse());
});

test("RpgTurnOrder plates the round", () => {
  run(member("turn-order", { round: 7 }), (e) => expect(elements(e).some((x) => /\b7\b/.test(x.text))).toBeTrue());
});

test("RpgStats puts each group in its column", () => {
  run(member("stats"), (e) => {
    expect(elements(e).find((x) => x.text === "Health")!.x).toBeLessThan(elements(e).find((x) => /Fire/.test(x.text))!.x);
  });
});

test("RpgCombatLog keeps the game's order", () => {
  const lines = ["l3", "l1", "l2"].map((id) => ({ id, text: `line ${id}`, tone: "plain", detail: "" }));
  run(member("combat-log", { lines }), (e) => {
    const ys = lines.map((l) => elements(e).find((x) => x.text === l.text)!.y);
    expect([...ys].sort((a, b) => a - b)).toEqual(ys);
  });
});

test("keyboard and pad focus walk the commands in order and confirm the focused one", () => {
  const events: Event[] = [];
  const commands = ["attack", "skills", "spells"].map((id) => ({ id, label: id, icon: "sword", enabled: true }));
  run(member("commands", { commands }, { events }), (e) => {
    const focused = () => elements(e).find((x) => x.states?.includes("focused"))?.path ?? "";
    const first = focused();
    expect(first).toMatch(/#attack$/);
    key(e, "ArrowRight");
    expect(focused()).toMatch(/#skills$/);
    pad(e, "-:1,0");
    expect(focused()).toMatch(/#spells$/);
    pad(e, "a");
    key(e, "Enter");
    expect(intents(events, ["choose"])).toEqual(["choose:spells", "choose:spells"]);
  });
});
