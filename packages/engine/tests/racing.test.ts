// The racing components as a game uses them (docs/ui.md §6.1): built in,
// used by name from the game's own pages with nothing copied or loaded but
// the pages, dressed by the fantasy pack, each member mounted from every
// fixture its document declares and driven through the probe's touch, key
// and pad commands.
//
// Ways it could fail: an intent sent without its row key, or sent for a
// locked car, a car the game says is not affordable, the car already
// driven or a Retry the game refused; the
// garage's action following the tile index instead of the shown car's key;
// a fixture state that no longer mounts or spills past a phone's width;
// the player's row not marked in the standings; the banner not replaying
// its entrance for a new call; a state shown by its dress alone.
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
const GROUP = join(ENGINE, "src/ui/components/racing");
const STYLES = join(ENGINE, "../../templates/styles");
let root = "";
let styles = "";

/** Each member's component name, parts and fixtures, read from its document as a game's tooling reads them. */
const MEMBERS: Record<string, { component: string; parts: string[]; fixtures: Record<string, Record<string, unknown>> }> = {};

beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-racing-kit-"));
  styles = mkdtempSync(join(tmpdir(), "og-racing-styles-"));
  for (const d of ["fantasy", "shared"]) cpSync(join(STYLES, d), join(styles, d), { recursive: true });
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Racing kit"\nid "racing-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  // A look naming the pack is what makes the cook bundle it (@styles/fantasy).
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "racing kit fantasy"\nui "fantasy"\n');
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

function engine(component: string, stores: Record<string, unknown>, o: { events?: Event[]; view?: View; reduced?: boolean } = {}): Engine {
  const events = o.events ?? [];
  const Collect = defineSystem("RacingKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      const style = og.load.style(world, "@styles/fantasy");
      if (!style.ok) throw new Error(style.reason);
      og.ui.configure(world, { style: style.value, reducedMotion: o.reduced ?? false });
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
/** A garage tile by its car's key (the carousel's pages carry the same keys). */
const tile = (e: Engine, k: string): El => elements(e).find((x) => x.role === "option" && x.path.endsWith(`#${k}`))!;
/** The words shown, lower-cased: a pack may set them in capitals. */
const texts = (e: Engine): string[] => elements(e).filter((x) => x.width > 0).map((x) => x.text.toLowerCase());

// One test per member and view: each fixture boots its own engine.
for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  for (const name of ["telemetry", "lap-hud", "standings", "banner", "garage", "results"]) {
    test(`${name} mounts from every fixture on a ${view} without errors, inside the screen's width`, () => {
      for (const fixture of Object.keys(MEMBERS[name]!.fixtures)) {
        run(member(name, fixture, {}, { view: viewport }), (e) => {
          expect(`${name}/${fixture}: ${errors(e).join("; ")}`).toBe(`${name}/${fixture}: `);
          const box = find(e, /^t:page\/[^/]+$/)!;
          expect(`${name}/${fixture} ${box.x + box.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
          // The frame is drawn and sits on the screen.
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
    // Every part the document draws is declared.
    for (const used of text.matchAll(/part="([^"]+)"/g)) expect(m.parts).toContain(used[1]!);
  }
});

test("RacingGarage: a tile chooses by key; Buy and Drive report the shown car's key; the car driven, a locked car and a car too costly send nothing", () => {
  const events: Event[] = [];
  run(member("garage", "default", {}, { events }), (e) => {
    expect(tile(e, "gt").states).toContain("selected");
    tap(e, tile(e, "rally"));
    tap(e, find(e, /\/Select(#[^/]+)?$/)!);
    tap(e, find(e, /\/Back$/)!);
    expect(intents(events, ["choose", "select", "buy", "back"])).toEqual(["choose:rally", "select:gt", "back:"]);
  });
  const bought: Event[] = [];
  run(member("garage", "buy", {}, { events: bought }), (e) => {
    expect(texts(e)).toContain("buy 18,500 cr");
    tap(e, find(e, /\/Buy(#[^/]+)?$/)!);
    expect(intents(bought, ["buy", "select"])).toEqual(["buy:rally"]);
  });
  const short: Event[] = [];
  run(member("garage", "short", {}, { events: short }), (e) => {
    const button = find(e, /\/Buy(#[^/]+)?$/)!;
    expect(button.disabled).toBeTrue();
    // Why Buy is refused is a word on the button, not a colour.
    expect(texts(e)).toContain("too costly");
    tap(e, button);
    expect(intents(short, ["buy", "select"])).toEqual([]);
  });
  const locked: Event[] = [];
  run(member("garage", "locked", {}, { events: locked }), (e) => {
    const button = find(e, /\/Locked(#[^/]+)?$/)!;
    expect(button.disabled).toBeTrue();
    // The reason is a word on the button, not a colour.
    expect(texts(e)).toContain("reach level 20");
    tap(e, button);
    expect(intents(locked, ["buy", "select"])).toEqual([]);
  });
  const driving: Event[] = [];
  run(member("garage", "selected", {}, { events: driving }), (e) => {
    const button = find(e, /\/Select(#[^/]+)?$/)!;
    expect(button.disabled).toBeTrue();
    expect(texts(e)).toContain("driving");
    tap(e, button);
    expect(intents(driving, ["select"])).toEqual([]);
  });
});

test("RacingGarage: the bumpers and Q/E step the cars, and the shown car keeps its key through a reorder", () => {
  const events: Event[] = [];
  run(member("garage", "default", {}, { events }), (e) => {
    key(e, "KeyE");
    pad(e, "leftBumper");
    expect(intents(events, ["prev", "next"])).toEqual(["next:", "prev:"]);
    const vehicles = og.ui.scope(e.world, "t").collection("vehicles", MEMBERS.garage!.fixtures.default!.vehicles as { id: string }[], { key: "id" });
    vehicles.move("gt", "hatch");
    e.advance(3);
    expect(tile(e, "gt").states).toContain("selected");
    expect(tile(e, "hatch").states ?? []).not.toContain("selected");
    events.length = 0;
    tap(e, find(e, /\/Select(#[^/]+)?$/)!);
    expect(intents(events, ["select"])).toEqual(["select:gt"]);
  });
});

test("RacingGarage: the empty garage says so and keeps Back", () => {
  const events: Event[] = [];
  run(member("garage", "empty", {}, { events }), (e) => {
    expect(texts(e)).toContain("no vehicles yet");
    tap(e, find(e, /\/EmptyBack$/)!);
    expect(intents(events, ["back"])).toEqual(["back:"]);
  });
});

test("RacingResults: Continue and Retry report once each; a refused Retry is disabled and silent to a tap and to X", () => {
  const events: Event[] = [];
  run(member("results", "default", {}, { events }), (e) => {
    tap(e, find(e, /\/Continue$/)!);
    tap(e, find(e, /\/Retry$/)!);
    expect(intents(events, ["continue", "retry"])).toEqual(["continue:", "retry:"]);
    // The fastest lap and the player's row are marked in words.
    expect(texts(e)).toContain("fastest lap");
    expect(texts(e)).toContain("you");
    expect(texts(e)).toContain("dnf");
  });
  const refused: Event[] = [];
  run(member("results", "disabled", {}, { events: refused }), (e) => {
    expect(find(e, /\/Retry$/)!.disabled).toBeTrue();
    tap(e, find(e, /\/Retry$/)!);
    key(e, "KeyX");
    expect(intents(refused, ["retry"])).toEqual([]);
    expect(texts(e)).toContain("dsq");
  });
  const empty: Event[] = [];
  run(member("results", "empty", {}, { events: empty }), (e) => {
    expect(texts(e)).toContain("no finishers");
    tap(e, find(e, /\/Continue$/)!);
    expect(intents(empty, ["continue"])).toEqual(["continue:"]);
  });
});

test("RacingResults: keyboard and pad reach Continue and confirm it", () => {
  for (const [device, press] of [["keyboard", () => "key_down:Enter;0.05:key_up:Enter"], ["pad", () => "gamepad:a;0.05:gamepad:-"]] as const) {
    const events: Event[] = [];
    run(member("results", "default", {}, { events }), (e) => {
      const focused = () => elements(e).find((x) => x.states?.includes("focused"))?.path ?? "";
      for (let i = 0; i < 6 && !/\/Continue$/.test(focused()); i++) device === "keyboard" ? key(e, "Tab") : pad(e, "-:1,0");
      expect(`${device} ${focused()}`).toMatch(/\/Continue$/);
      e.probe.commands(`0:${press()}`);
      e.advance(8);
      expect(`${device} ${intents(events, ["continue", "retry"])}`).toBe(`${device} continue:`);
    });
  }
});

test("RacingStandings: rows keep the game's order, the player's row is current and named You, and states carry words", () => {
  run(member("standings"), (e) => {
    const rows = elements(e).filter((x) => x.role === "listitem" && /#[a-z]+$/.test(x.path));
    expect(rows.map((x) => x.path.split("#").pop())).toEqual(["marlow", "okafor", "you", "lindqvist", "sato", "brandt"]);
    expect([...rows.map((x) => x.y)].sort((a, b) => a - b)).toEqual(rows.map((x) => x.y));
    expect(find(e, /#you$/)!.states).toContain("current");
    expect(texts(e)).toContain("you");
    expect(texts(e)).toContain("pit");
    expect(texts(e)).toContain("out");
  });
  run(member("standings", "empty"), (e) => expect(texts(e)).toContain("no racers yet"));
});

test("RacingLapHud: lap and place over their totals, blank times before the start, and the split's sense in words", () => {
  run(member("lap-hud"), (e) => {
    const all = texts(e);
    for (const want of ["2", "/3", "3", "/8", "0:48.207", "1:21.904", "1:23.116", "-0.412", "ahead"]) expect(all).toContain(want);
  });
  run(member("lap-hud", "grid"), (e) => {
    expect(texts(e).filter((x) => x === "–:––.–––").length).toBe(3);
    expect(texts(e)).not.toContain("ahead");
  });
  run(member("lap-hud", "finalLap"), (e) => {
    for (const want of ["behind", "final lap", "best lap"]) expect(texts(e)).toContain(want);
  });
  run(member("lap-hud", "checkpoint"), (e) => expect(elements(e).some((x) => x.role === "timer" && x.width > 0)).toBeTrue());
});

test("RacingTelemetry: the game's units and gear, the shift cue as a word, and the bar form", () => {
  run(member("telemetry"), (e) => {
    for (const want of ["187", "km/h", "4"]) expect(texts(e)).toContain(want);
    expect(texts(e)).not.toContain("shift");
  });
  run(member("telemetry", "redline"), (e) => expect(texts(e)).toContain("shift"));
  run(member("telemetry", "bar"), (e) => {
    expect(texts(e)).toContain("mph");
    expect(elements(e).filter((x) => x.role === "meter" && x.width > 0).length).toBeGreaterThanOrEqual(2);
  });
});

test("RacingBanner: each call shows its word, a new call id remakes the plate, and reduced motion shows it at once", () => {
  run(member("banner"), (e) => {
    expect(texts(e)).toContain("3");
    const plate = () => elements(e).find((x) => /^t:page\/Root\/node1\/[^/]+$/.test(x.path) && x.width > 0)!.path;
    const before = plate();
    const scope = og.ui.scope(e.world, "t");
    scope.writable("figure", 3).set(2);
    e.advance(2);
    expect(plate()).toBe(before);
    scope.writable("at", 1).set(2);
    e.advance(2);
    expect(plate()).not.toBe(before);
    expect(texts(e)).toContain("2");
    scope.writable("kind", "countdown").set("go");
    e.advance(2);
    expect(texts(e)).toContain("go!");
  });
  run(member("banner", "finish"), (e) => {
    expect(texts(e)).toContain("finish");
    expect(texts(e)).toContain("2nd");
  });
  run(member("banner", "none"), (e) => expect(texts(e).some((x) => /go|finish|final/.test(x))).toBeFalse());
  // Under reduced motion the call is shown at once. (The probe reports layout boxes, before translate and scale, so the entrance itself is reviewed in the sheet's frames.)
  run(member("banner", "go", {}, { reduced: true }), (e) => expect(texts(e)).toContain("go!"));
});

