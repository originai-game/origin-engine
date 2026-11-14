// The puzzle components as a game uses them (docs/ui.md §6.1): built in,
// used by name from the game's own pages with nothing copied or loaded but
// the pages, dressed by the fantasy pack, each member mounted from every
// fixture its document declares and driven through the probe's touch, key
// and pad commands.
//
// Ways it could fail: an intent sent without its row key, or sent for a
// locked level (by a tap, or by X or the pad's Y) or an unavailable booster;
// Next offered when the game said there is none; selection or the armed booster following a row index
// instead of its key; a fixture state that no longer mounts or spills past
// a phone's width; a state (locked, done, low, armed, unavailable, a new
// best) shown by its dress alone.
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
const GROUP = join(ENGINE, "src/ui/components/puzzle");
const STYLES = join(ENGINE, "../../templates/styles");
let root = "";
let styles = "";

/** Each member's component name, parts and fixtures, read from its document as a game's tooling reads them. */
const MEMBERS: Record<string, { component: string; parts: string[]; fixtures: Record<string, Record<string, unknown>> }> = {};

beforeAll(() => {
  // The temporary directories first, so cleanup only ever sees what was made here.
  root = mkdtempSync(join(tmpdir(), "og-puzzle-kit-"));
  styles = mkdtempSync(join(tmpdir(), "og-puzzle-styles-"));
  loadNativeHost();
  for (const d of ["fantasy", "shared"]) cpSync(join(STYLES, d), join(styles, d), { recursive: true });
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Puzzle kit"\nid "puzzle-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  // A look naming the pack is what makes the cook bundle it (@styles/fantasy).
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "puzzle kit fantasy"\nui "fantasy"\n');
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

function engine(component: string, stores: Record<string, unknown>, o: { events?: Event[]; view?: View } = {}): Engine {
  const events = o.events ?? [];
  const Collect = defineSystem("PuzzleKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      const style = og.load.style(world, "@styles/fantasy");
      if (!style.ok) throw new Error(style.reason);
      og.ui.configure(world, { style: style.value });
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
const key = (e: Engine, k: string): void => {
  e.probe.commands(`0:key_down:${k};0.05:key_up:${k}`);
  e.advance(8);
};
const pad = (e: Engine, spec: string): void => {
  e.probe.commands(`0:gamepad:${spec};0.05:gamepad:-`);
  e.advance(8);
};
const intents = (events: Event[], names: string[]) => events.filter((e) => names.includes(e.name)).map((e) => `${e.name}:${e.rowKey}`);
const member = (name: string, fixture = "default", over: Record<string, unknown> = {}, o: Parameters<typeof engine>[2] = {}) =>
  engine(MEMBERS[name]!.component, { ...MEMBERS[name]!.fixtures.default, ...MEMBERS[name]!.fixtures[fixture], ...over }, o);
const run = (e: Engine, f: (e: Engine) => void) => {
  try {
    e.advance(3);
    f(e);
  } finally {
    e.destroy();
  }
};
const errors = (e: Engine): readonly string[] => og.ui.scope(e.world, "t").takeErrors();
/** A row's button by its key and role (the key is on the row, or on a row the button sits in). */
const row = (e: Engine, role: string, k: string): El => elements(e).find((x) => x.role === role && new RegExp(`#${k}(/|$)`).test(x.path))!;
/** The words shown, lower-cased: a pack may set them in capitals. */
const texts = (e: Engine): string[] => elements(e).filter((x) => x.width > 0).map((x) => x.text.toLowerCase());
const focused = (e: Engine): El | undefined => elements(e).find((x) => x.states?.includes("focused"));

// One test per member and view: each fixture boots its own engine.
for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  for (const name of ["level-path", "goals", "summary", "boosters"]) {
    test(`${name} mounts from every fixture on a ${view} without errors, inside the screen's width`, () => {
      for (const fixture of Object.keys(MEMBERS[name]!.fixtures)) {
        run(member(name, fixture, {}, { view: viewport }), (e) => {
          expect(`${name}/${fixture}: ${errors(e).join("; ")}`).toBe(`${name}/${fixture}: `);
          const box = find(e, /^t:page\/[^/]+$/)!;
          expect(`${name}/${fixture} ${box.x + box.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
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
    const text = readFileSync(join(GROUP, `${name}.svelte`), "utf8");
    for (const p of m.parts) expect(`${name} ${p} ${p.startsWith(`${m.component}.`) && text.includes(`part="${p}"`)}`).toBe(`${name} ${p} true`);
    for (const used of text.matchAll(/part="([^"]+)"/g)) expect(m.parts).toContain(used[1]!);
  }
});

test("PuzzleLevelPath: a node chooses by key, a locked node is disabled and silent, and the selection keeps its key through a reorder", () => {
  const events: Event[] = [];
  run(member("level-path", "selected", {}, { events }), (e) => {
    expect(row(e, "option", "l2").states).toContain("selected");
    tap(e, row(e, "option", "l3"));
    tap(e, row(e, "option", "l1"));
    expect(intents(events, ["choose", "details"])).toEqual(["choose:l3", "choose:l1"]);
    // The current level says Play; done levels name their stars in words.
    expect(texts(e)).toContain("play");
    expect(row(e, "option", "l1").name?.toLowerCase()).toContain("3 of 3 stars");
    const levels = og.ui.scope(e.world, "t").collection("levels", MEMBERS["level-path"]!.fixtures.selected!.levels as { id: string }[], { key: "id" });
    levels.move("l2", "l1");
    e.advance(3);
    expect(row(e, "option", "l2").states).toContain("selected");
    expect(row(e, "option", "l1").states ?? []).not.toContain("selected");
  });
  const locked: Event[] = [];
  run(member("level-path", "locked", {}, { events: locked }), (e) => {
    const node = row(e, "option", "l41");
    expect(node.disabled).toBeTrue();
    expect(node.name?.toLowerCase()).toContain("locked");
    tap(e, node);
    expect(intents(locked, ["choose", "details"])).toEqual([]);
  });
  run(member("level-path", "empty"), (e) => expect(texts(e)).toContain("no levels yet"));
});

test("PuzzleLevelPath: X and pad Y on the focused node report details with its key", () => {
  for (const [device, press] of [["keyboard", "key_down:KeyX;0.05:key_up:KeyX"], ["pad", "gamepad:y;0.05:gamepad:-"]] as const) {
    const events: Event[] = [];
    run(member("level-path", "default", {}, { events }), (e) => {
      for (let i = 0; i < 8 && !/#l\d+/.test(focused(e)?.path ?? ""); i++) device === "keyboard" ? key(e, "Tab") : pad(e, "-:0,1");
      const k = /#(l\d+)/.exec(focused(e)?.path ?? "")?.[1] ?? "";
      expect(`${device} ${k}`).toMatch(/ l\d+$/);
      e.probe.commands(`0:${press}`);
      e.advance(8);
      expect(`${device} ${intents(events, ["details"])}`).toBe(`${device} details:${k}`);
    });
  }
});

for (const [device, press] of [["keyboard", "key_down:KeyX;0.05:key_up:KeyX"], ["pad", "gamepad:y;0.05:gamepad:-"]] as const) {
  test(`PuzzleLevelPath: ${device === "keyboard" ? "X" : "pad Y"} never reports details for a locked level, wherever the focus goes`, () => {
    // All locked: nothing to report from, whatever holds the focus.
    const locked: Event[] = [];
    run(member("level-path", "locked", {}, { events: locked }), (e) => {
      for (let i = 0; i < 4; i++) {
        device === "keyboard" ? key(e, "Tab") : pad(e, "-:0,1");
        e.probe.commands(`0:${press}`);
        e.advance(8);
      }
      expect(`${device} ${intents(locked, ["choose", "details"])}`).toBe(`${device} `);
    });
    // Four open levels over a locked one: walking the focus round reports the open ones only.
    const events: Event[] = [];
    run(member("level-path", "default", {}, { events }), (e) => {
      for (let i = 0; i < 8; i++) {
        device === "keyboard" ? key(e, "Tab") : pad(e, "-:0,1");
        e.probe.commands(`0:${press}`);
        e.advance(8);
      }
      const sent = intents(events, ["details"]);
      expect(`${device} ${sent.length > 0}`).toBe(`${device} true`);
      expect(`${device} ${sent.filter((x) => x === "details:l5")}`).toBe(`${device} `);
    });
  }, 120_000);
}

test("PuzzleGoals: moves, goals left and score as the game counts them; a met goal and a low counter say so in words", () => {
  run(member("goals"), (e) => {
    for (const want of ["moves", "18", "12", "4", "12,450", "score"]) expect(texts(e)).toContain(want);
    // The score's spoken name carries the figure, not the caption alone.
    expect(elements(e).some((x) => x.role === "status" && x.name?.toLowerCase() === "score 12,450")).toBeTrue();
    expect(texts(e)).not.toContain("running low");
    expect(elements(e).some((x) => x.name?.toLowerCase() === "leaves: done")).toBeTrue();
    expect(elements(e).some((x) => x.name?.toLowerCase() === "red sweets: 12 left")).toBeTrue();
  });
  run(member("goals", "low"), (e) => expect(texts(e)).toContain("running low"));
  run(member("goals", "time"), (e) => {
    expect(texts(e)).toContain("time");
    expect(elements(e).some((x) => x.role === "timer" && x.width > 0)).toBeTrue();
  });
  run(member("goals", "empty"), (e) => expect(texts(e)).toContain("reach the target score"));
});

test("PuzzleSummary: Next, Retry and Map report once each; a lost level offers no Next; Escape goes to the map", () => {
  const events: Event[] = [];
  run(member("summary", "default", {}, { events }), (e) => {
    tap(e, find(e, /\/Next$/)!);
    tap(e, find(e, /\/Retry$/)!);
    tap(e, find(e, /\/Menu$/)!);
    expect(intents(events, ["next", "retry", "menu"])).toEqual(["next:", "retry:", "menu:"]);
    expect(texts(e)).toContain("2 of 3 stars");
    expect(texts(e)).toContain("total");
    expect(texts(e)).toContain("level complete");
    expect(texts(e)).not.toContain("new best");
  });
  run(member("summary", "full"), (e) => {
    expect(texts(e)).toContain("new best");
    expect(texts(e)).toContain("3 of 3 stars");
  });
  const lost: Event[] = [];
  run(member("summary", "lose", {}, { events: lost }), (e) => {
    expect(texts(e)).toContain("level failed");
    const next = find(e, /\/Next$/);
    expect(!next || next.width === 0 || next.disabled === true).toBeTrue();
    key(e, "Escape");
    expect(intents(lost, ["next", "retry", "menu"])).toEqual(["menu:"]);
  });
});

test("PuzzleBoosters: a booster reports its key; an unavailable one is disabled, worded and silent; the armed one is pressed and worded, by key through a reorder", () => {
  const events: Event[] = [];
  run(member("boosters", "default", {}, { events }), (e) => {
    tap(e, row(e, "button", "shuffle"));
    const burst = row(e, "button", "burst");
    expect(burst.disabled).toBeTrue();
    expect(burst.name?.toLowerCase()).toContain("unavailable");
    expect(texts(e)).toContain("unavailable");
    tap(e, burst);
    expect(intents(events, ["choose"])).toEqual(["choose:shuffle"]);
    expect(texts(e)).toContain("3");
    expect(texts(e)).not.toContain("armed");
  });
  run(member("boosters", "selected"), (e) => {
    expect(texts(e)).toContain("armed");
    expect(row(e, "button", "hammer").name?.toLowerCase()).toContain("armed");
    const boosters = og.ui.scope(e.world, "t").collection("boosters", MEMBERS.boosters!.fixtures.selected!.boosters as { id: string }[], { key: "id" });
    boosters.move("hammer", "moves");
    e.advance(3);
    expect(row(e, "button", "hammer").name?.toLowerCase()).toContain("armed");
    expect(row(e, "button", "shuffle").name?.toLowerCase()).not.toContain("armed");
  });
  // The game arms a booster it then refuses: it reads Unavailable, not Armed.
  run(member("boosters", "disabled", { armed: "hammer" }), (e) => {
    expect(texts(e)).toContain("unavailable");
    expect(texts(e)).not.toContain("armed");
    expect(row(e, "button", "hammer").name?.toLowerCase()).toContain("unavailable");
  });
  const off: Event[] = [];
  run(member("boosters", "disabled", {}, { events: off }), (e) => {
    for (const k of ["hammer", "shuffle", "moves"]) tap(e, row(e, "button", k));
    key(e, "Enter");
    expect(intents(off, ["choose"])).toEqual([]);
  });
  run(member("boosters", "empty"), (e) => expect(texts(e)).toContain("no boosters"));
});

test("PuzzleBoosters: keyboard and pad reach a booster and use it", () => {
  for (const [device, press] of [["keyboard", "key_down:Enter;0.05:key_up:Enter"], ["pad", "gamepad:a;0.05:gamepad:-"]] as const) {
    const events: Event[] = [];
    run(member("boosters", "default", {}, { events }), (e) => {
      for (let i = 0; i < 6 && !/#[a-z]+$/.test(focused(e)?.path ?? ""); i++) device === "keyboard" ? key(e, "Tab") : pad(e, "-:1,0");
      const k = (focused(e)?.path ?? "").split("#").pop()!;
      e.probe.commands(`0:${press}`);
      e.advance(8);
      expect(`${device} ${intents(events, ["choose"])}`).toBe(`${device} choose:${k}`);
    });
  }
});
