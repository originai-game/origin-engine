// The strategy components as a game uses them (docs/ui.md §6.1): built in,
// used by name from the game's own pages with nothing copied or loaded but
// the pages, dressed by the fantasy pack, each member mounted from every
// fixture its document declares and driven through the probe's touch and
// key commands.
//
// Ways it could fail: an intent sent without its row key or with a row
// index; a disabled command, a locked, unaffordable or maxed upgrade, or a
// refused sale that still reports; a disabled command whose reason is only a
// colour; a fixture state that no longer mounts or spills past a phone's
// width; Call early shown when the game refused it.
import { afterAll, beforeAll, expect, setDefaultTimeout, test } from "bun:test";

// The tests run on ticks, not the clock; a time limit only guards a hang.
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
const GROUP = join(ENGINE, "src/ui/components/strategy");
const STYLES = join(ENGINE, "../../templates/styles");
let root = "";
let styles = "";

/** Each member's component name, parts and fixtures, read from its document as a game's tooling reads them. */
const MEMBERS: Record<string, { component: string; parts: string[]; fixtures: Record<string, Record<string, unknown>> }> = {};

beforeAll(() => {
  // The temporary directories first: afterAll removes only what was made here.
  root = mkdtempSync(join(tmpdir(), "og-strategy-kit-"));
  styles = mkdtempSync(join(tmpdir(), "og-strategy-styles-"));
  loadNativeHost();
  for (const d of ["fantasy", "shared"]) cpSync(join(STYLES, d), join(styles, d), { recursive: true });
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Strategy kit"\nid "strategy-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  // A look naming the pack is what makes the cook bundle it (@styles/fantasy).
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "strategy kit fantasy"\nui "fantasy"\n');
  for (const f of readdirSync(GROUP).filter((f) => f.endsWith(".svelte"))) {
    const text = readFileSync(join(GROUP, f), "utf8");
    const component = /<meta name="component" content="([^"]+)">/.exec(text)![1]!;
    const parts = /<meta name="parts" content="([^"]+)">/.exec(text)![1]!.split(" ");
    const fixtures = Object.fromEntries([...text.matchAll(/<meta name="fixture:([^"]+)" content='([^']*)'>/g)].map((m) => [m[1]!, JSON.parse(m[2]!)]));
    const name = f.replace(".svelte", "");
    MEMBERS[name] = { component, parts, fixtures };
    // A game mounts its own page, which uses the member by name: one host page per member, its props from stores.
    const props = Object.keys(fixtures.default!).map((k) => `${k}={$${k}}`).join(" ");
    writeFileSync(join(root, `assets/ui/host-${name}.svelte`), `<meta name="screen" content="Host_${component}" data-layer="menu" data-input="menu">\n<div style="width: stretch; height: stretch; padding: 16px"><${component} ${props} /></div>\n`);
  }
  const cook = Bun.spawnSync([ogBin(), "cook", root], { cwd: root, env: { ...process.env, OG_STYLES_DIR: styles }, stdout: "ignore", stderr: "pipe" });
  if (cook.exitCode !== 0) throw new Error(`cooking ${root}: ${new TextDecoder().decode(cook.stderr)}`);
});

// Only what beforeAll made: an empty path (beforeAll failed first) would be the working directory.
afterAll(() => {
  if (root) rmSync(root, { recursive: true, force: true });
  if (styles) rmSync(styles, { recursive: true, force: true });
});

// Only the host page an engine mounts: each load reads the whole style pack again.
const DOCUMENTS = (component: string) => Object.keys(MEMBERS).filter((m) => MEMBERS[m]!.component === component).map((m) => `ui/host-${m}.svelte`);

type View = { width: number; height: number; dpr: number; safeArea: { left: number; top: number; right: number; bottom: number }; coarsePointer: boolean; breakpoint: string };
const DESKTOP: View = { width: 1280, height: 720, dpr: 1.5, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktopHd" };
const PHONE: View = { width: 390, height: 844, dpr: 2, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };

function engine(component: string, stores: Record<string, unknown>, o: { events?: Event[]; view?: View; textScale?: number } = {}): Engine {
  const events = o.events ?? [];
  const Collect = defineSystem("StrategyKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      const style = og.load.style(world, "@styles/fantasy");
      if (!style.ok) throw new Error(style.reason);
      og.ui.configure(world, { style: style.value, textScale: o.textScale ?? 1 });
      for (const path of DOCUMENTS(component)) {
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
const member = (name: string, fixture = "default", o: Parameters<typeof engine>[2] = {}) =>
  engine(MEMBERS[name]!.component, { ...MEMBERS[name]!.fixtures.default, ...MEMBERS[name]!.fixtures[fixture] }, o);
const run = (e: Engine, f: (e: Engine) => void) => {
  try {
    e.advance(3);
    f(e);
  } finally {
    e.destroy();
  }
};
const errors = (e: Engine): readonly string[] => og.ui.scope(e.world, "t").takeErrors();
/** The words shown, lower-cased: a pack may set them in capitals. */
const texts = (e: Engine): string[] => elements(e).filter((x) => x.width > 0).map((x) => x.text.toLowerCase());
/** The button of a keyed row: the row's own (its path ends in #key) or the one inside it (under #key/). */
const rowButton = (e: Engine, k: string): El => elements(e).find((x) => x.role === "button" && (x.path.endsWith(`#${k}`) || x.path.includes(`#${k}/`)) && x.width > 0)!;

// One test per member and view: each fixture boots its own engine.
for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  for (const name of ["selection", "queue", "wave", "tower"]) {
    test(`${name} mounts from every fixture on a ${view} without errors, inside the screen's width`, () => {
      for (const fixture of Object.keys(MEMBERS[name]!.fixtures)) {
        run(member(name, fixture, { view: viewport }), (e) => {
          expect(`${name}/${fixture}: ${errors(e).join("; ")}`).toBe(`${name}/${fixture}: `);
          const frame = find(e, /^t:page\/Root\/node1$/)!;
          expect(`${name}/${fixture} ${frame.x >= -0.5 && frame.x + frame.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
        });
      }
    }, 180_000);
  }
}

test("every member names its frame first and only its own parts", () => {
  for (const [name, m] of Object.entries(MEMBERS)) {
    expect(`${name} ${m.parts[0]}`).toBe(`${name} ${m.component}.frame`);
    // The markup only: a part named in the <style> alone is not drawn.
    const text = readFileSync(join(GROUP, `${name}.svelte`), "utf8").split("<style>")[0]!;
    for (const p of m.parts) expect(`${name} ${p} ${p.startsWith(`${m.component}.`) && text.includes(`part="${p}"`)}`).toBe(`${name} ${p} true`);
    for (const used of text.matchAll(/part="([^"]+)"/g)) expect(m.parts).toContain(used[1]!);
  }
});

test("StrategySelection: a cell chooses by key, a command reports its key, a disabled one is silent and says why", () => {
  const events: Event[] = [];
  run(member("selection", "default", { events }), (e) => {
    expect(elements(e).find((x) => x.role === "option" && x.path.endsWith("#spear"))!.states).toContain("selected");
    tap(e, elements(e).find((x) => x.role === "option" && x.path.endsWith("#archer"))!);
    tap(e, rowButton(e, "stop"));
    tap(e, rowButton(e, "rally"));
    tap(e, rowButton(e, "charge"));
    expect(intents(events, ["choose", "command"])).toEqual(["choose:archer", "command:stop"]);
    expect(rowButton(e, "rally").disabled).toBeTrue();
    // The reason and the cooldown are words, not colours; a toggle that is on says On.
    expect(texts(e)).toContain("needs the banner");
    expect(texts(e)).toContain("12s");
    expect(texts(e)).toContain("on");
    expect(texts(e)).toContain("×6");
    expect(texts(e)).toContain("spearmen");
  });
  run(member("selection", "disabled"), (e) => {
    for (const k of ["build", "repair", "burrow"]) expect(rowButton(e, k).disabled).toBeTrue();
    expect(texts(e)).toContain("no space here");
    expect(texts(e)).toContain("nothing damaged");
  });
  run(member("selection", "empty"), (e) => expect(texts(e)).toContain("nothing selected"));
});

test("StrategySelection: the chosen subgroup keeps its key through a reorder", () => {
  const events: Event[] = [];
  run(member("selection", "default", { events }), (e) => {
    const units = og.ui.scope(e.world, "t").collection("units", MEMBERS.selection!.fixtures.default!.units as { id: string }[], { key: "id" });
    units.move("captain", "spear");
    e.advance(3);
    expect(elements(e).find((x) => x.role === "option" && x.path.endsWith("#spear"))!.states).toContain("selected");
    tap(e, elements(e).find((x) => x.role === "option" && x.path.endsWith("#captain"))!);
    expect(intents(events, ["choose"])).toEqual(["choose:captain"]);
  });
});

test("StrategyQueue: cancel reports the order's key; count over limit, Queue full and each state are words", () => {
  const events: Event[] = [];
  run(member("queue", "default", { events }), (e) => {
    tap(e, rowButton(e, "o2"));
    tap(e, rowButton(e, "o3"));
    expect(intents(events, ["cancel"])).toEqual(["cancel:o2", "cancel:o3"]);
    for (const want of ["4/5", "waiting", "blocked", "need 3 supply", "×2", "0:07"]) expect(texts(e)).toContain(want);
    expect(texts(e)).not.toContain("queue full");
  });
  run(member("queue", "full"), (e) => {
    expect(texts(e)).toContain("queue full");
    expect(texts(e)).toContain("5/5");
  });
  run(member("queue", "paused"), (e) => expect(texts(e)).toContain("paused"));
  run(member("queue", "empty"), (e) => expect(texts(e)).toContain("nothing in production"));
});

test("StrategyWave: wave n of m, the countdown, the enemies by kind, and Call early once and only when allowed", () => {
  const events: Event[] = [];
  run(member("wave", "default", { events }), (e) => {
    for (const want of ["6", "/20", "goblin", "×18", "warg rider", "get ready"]) expect(texts(e)).toContain(want);
    expect(elements(e).some((x) => x.role === "timer" && x.width > 0)).toBeTrue();
    tap(e, find(e, /\/CallEarly$/)!);
    expect(intents(events, ["callEarly"])).toEqual(["callEarly:"]);
  });
  run(member("wave", "active"), (e) => {
    expect(texts(e)).toContain("under attack");
    expect(find(e, /\/CallEarly$/)?.width ?? 0).toBe(0);
    expect(elements(e).some((x) => x.role === "timer" && x.width > 0)).toBeFalse();
  });
  run(member("wave", "final"), (e) => {
    for (const want of ["final wave", "boss", "iron warlord", "not yet scouted"]) expect(texts(e)).toContain(want);
  });
  run(member("wave", "done"), (e) => expect(texts(e)).toContain("all waves cleared"));
});

for (const textScale of [1, 1.5]) {
  test(`on a phone at ${textScale * 100} % text, long names and reasons never push a row's or a command's button off the screen`, () => {
    for (const [name, list] of [["queue", "items"], ["tower", "paths"], ["selection", "commands"]] as const) {
      run(member(name, "overflow", { view: PHONE, textScale }), (e) => {
        for (const { id } of MEMBERS[name]!.fixtures.overflow![list] as { id: string }[]) {
          const b = rowButton(e, id);
          expect(`${textScale} ${name} ${id} ${!!b && b.x >= -0.5 && b.x + b.width <= PHONE.width + 0.5}`).toBe(`${textScale} ${name} ${id} true`);
        }
        // Command plates keep to their grid tracks: none overlaps the next.
        const plates = list === "commands" ? (MEMBERS.selection!.fixtures.overflow!.commands as { id: string }[]).map((c) => rowButton(e, c.id)).filter(Boolean) : [];
        for (const [i, a] of plates.entries()) for (const b of plates.slice(i + 1)) {
          const apart = a.x + a.width <= b.x + 0.5 || b.x + b.width <= a.x + 0.5 || a.y + a.height <= b.y + 0.5 || b.y + b.height <= a.y + 0.5;
          expect(`${textScale} ${a.path.split("#").pop()}/${b.path.split("#").pop()} ${apart}`).toBe(`${textScale} ${a.path.split("#").pop()}/${b.path.split("#").pop()} true`);
        }
      });
    }
  }, 120_000);
}

test("StrategyWave and StrategyTower: the keyboard reaches Call early and Sell and confirms them", () => {
  for (const [name, target, intent] of [["wave", /\/CallEarly$/, "callEarly:"], ["tower", /\/Sell$/, "sell:"]] as const) {
    const events: Event[] = [];
    run(member(name, "default", { events }), (e) => {
      const focused = () => elements(e).find((x) => x.states?.includes("focused"))?.path ?? "";
      for (let i = 0; i < 8 && !target.test(focused()); i++) {
        e.probe.commands("0:key_down:Tab;0.05:key_up:Tab");
        e.advance(8);
      }
      expect(`${name} ${focused()}`).toMatch(target);
      e.probe.commands("0:key_down:Enter;0.05:key_up:Enter");
      e.advance(8);
      expect(`${name} ${intents(events, ["callEarly", "sell", "upgrade"])}`).toBe(`${name} ${intent}`);
    });
  }
});

test("StrategyTower: only an affordable path upgrades, with its key; each state is a word; Sell reports and a refused sale is silent", () => {
  const events: Event[] = [];
  run(member("tower", "default", { events }), (e) => {
    tap(e, rowButton(e, "focus"));
    tap(e, rowButton(e, "chain"));
    tap(e, rowButton(e, "frost"));
    tap(e, find(e, /\/Sell$/)!);
    expect(intents(events, ["upgrade", "sell"])).toEqual(["upgrade:focus", "sell:"]);
    for (const want of ["250 gold", "too costly", "locked", "choose two paths at most", "tier 2/4", "42", "→ 51", "+180 gold"]) expect(texts(e)).toContain(want);
  });
  run(member("tower", "maxed"), (e) => {
    expect(texts(e).filter((x) => x === "maxed").length).toBe(2);
    expect(rowButton(e, "big").disabled).toBeTrue();
  });
  const refused: Event[] = [];
  run(member("tower", "disabled", { events: refused }), (e) => {
    expect(find(e, /\/Sell$/)!.disabled).toBeTrue();
    tap(e, find(e, /\/Sell$/)!);
    tap(e, rowButton(e, "a"));
    expect(intents(refused, ["sell", "upgrade"])).toEqual([]);
  });
  run(member("tower", "empty"), (e) => expect(texts(e)).toContain("no upgrades"));
});
