// The idle components as a game uses them (docs/ui.md §6.1): built in,
// used by name from the game's own pages with nothing copied or loaded but
// the pages, dressed by the fantasy pack, each member mounted from every
// fixture its document declares and driven through the probe's touch, key
// and pad commands.
//
// Ways it could fail: an intent sent without its row key, or sent for a
// locked generator or an upgrade the game said cannot be bought; the amount
// choice reporting the wrong key or losing its checked state; a formatted
// figure the game passed losing to the component's own formatting; the
// prestige confirm shown before the game asks for it, or Escape not
// cancelling it; Double offered when the game refused it; a fixture state
// that no longer mounts or spills past a phone's width; a state shown by
// its dress alone.
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
const GROUP = join(ENGINE, "src/ui/components/idle");
const STYLES = join(ENGINE, "../../templates/styles");
let root = "";
let styles = "";

/** Each member's component name, parts and fixtures, read from its document as a game's tooling reads them. */
const MEMBERS: Record<string, { component: string; parts: string[]; fixtures: Record<string, Record<string, unknown>> }> = {};

beforeAll(() => {
  // The temporary directories first, so afterAll removes only what was made here.
  root = mkdtempSync(join(tmpdir(), "og-idle-kit-"));
  styles = mkdtempSync(join(tmpdir(), "og-idle-styles-"));
  loadNativeHost();
  for (const d of ["fantasy", "shared"]) cpSync(join(STYLES, d), join(styles, d), { recursive: true });
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Idle kit"\nid "idle-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  // A look naming the pack is what makes the cook bundle it (@styles/fantasy).
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "idle kit fantasy"\nui "fantasy"\n');
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
  const Collect = defineSystem("IdleKitEvents", {}, (_q, _t, world) => {
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
const all = (e: Engine, path: RegExp): El[] => elements(e).filter((x) => path.test(x.path) && x.width > 0);
const centre = (el: El): string => `${el.x + el.width / 2},${el.y + el.height / 2}`;
const tap = (e: Engine, el: El): void => {
  e.probe.commands(`0:touch_move:${centre(el)};0.02:touch_start;0.06:touch_end`);
  e.advance(8);
};
const key = (e: Engine, k: string): void => {
  e.probe.commands(`0:key_down:${k};0.05:key_up:${k}`);
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
/** The words shown, lower-cased: a pack may set them in capitals. */
const texts = (e: Engine): string[] => elements(e).filter((x) => x.width > 0).map((x) => x.text.toLowerCase());
/** A keyed row's element (a row's path ends with its key). */
const row = (e: Engine, role: string, k: string): El => elements(e).find((x) => x.role === role && x.path.endsWith(`#${k}`))!;
/** The Buy button in an upgrade's row. */
const buyIn = (e: Engine, k: string): El => elements(e).find((x) => /\/Buy(#[^/]+)?$/.test(x.path) && x.path.includes(`#${k}/`))!;

// One test per member and view: each fixture boots its own engine.
for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  for (const name of ["resources", "generators", "purchases", "prestige", "offline"]) {
    test(`${name} mounts from every fixture on a ${view} without errors, inside the screen's width`, () => {
      for (const fixture of Object.keys(MEMBERS[name]!.fixtures)) {
        run(member(name, fixture, {}, { view: viewport }), (e) => {
          expect(`${name}/${fixture}: ${errors(e).join("; ")}`).toBe(`${name}/${fixture}: `);
          const frame = find(e, /^t:page\/Root\/node1$/)!;
          expect(`${name}/${fixture} ${frame.width > 0 && frame.x >= -0.5 && frame.x + frame.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
          // Nothing drawn spills past the screen's right edge.
          const spill = elements(e).filter((x) => x.width > 0 && x.x + x.width > viewport.width + 0.5).map((x) => x.path);
          expect(`${name}/${fixture} ${spill.join(", ")}`).toBe(`${name}/${fixture} `);
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

test("IdleResources: the game's formatted figures win, bare numbers are compacted, Full is a word, loading shows placeholders", () => {
  run(member("resources"), (e) => {
    const shown = texts(e);
    for (const want of ["gold", "1.23m", "+4.82k/s", "320", "5,000 / 5,000", "+12/s", "full"]) expect(shown).toContain(want);
    // The gems' value is the game's string; no rate is drawn for a flat income.
    expect(shown.filter((x) => x === "full").length).toBe(1);
  });
  run(member("resources", "draining"), (e) => {
    expect(texts(e)).toContain("-36/s");
    expect(texts(e)).toContain("+0.25/s");
  });
  run(member("resources", "loading"), (e) => {
    expect(texts(e)).toContain("gold");
    expect(texts(e)).not.toContain("0");
    // One placeholder per row, no figure drawn, and nothing claiming to be a progress bar.
    expect(all(e, /\/Placeholder(#[^/]+)?$/).length).toBe(2);
    expect(all(e, /./).some((x) => x.role === "progressbar")).toBeFalse();
  });
  run(member("resources", "default"), (e) => expect(all(e, /\/Placeholder(#[^/]+)?$/).length).toBe(0));
  run(member("resources", "empty"), (e) => expect(texts(e)).toContain("no resources yet"));
});

test("IdleGenerators: a row chooses by key and is selected; a locked row is disabled, says Locked and sends nothing", () => {
  const events: Event[] = [];
  run(member("generators", "default", {}, { events }), (e) => {
    expect(row(e, "option", "mine").states).toContain("selected");
    tap(e, row(e, "option", "farm"));
    const locked = row(e, "option", "forge");
    expect(locked.disabled).toBeTrue();
    tap(e, locked);
    expect(intents(events, ["choose"])).toEqual(["choose:farm"]);
    const shown = texts(e);
    for (const want of ["×25", "lv 3", "auto", "×2", "locked", "0.4s", "+1.2k/s"]) expect(shown).toContain(want);
    // Each running cycle is a ProgressBar.
    expect(all(e, /./).filter((x) => x.role === "progressbar").length).toBe(2);
  });
  const none: Event[] = [];
  run(member("generators", "disabled", {}, { events: none }), (e) => {
    for (const k of ["forge", "tower"]) tap(e, row(e, "option", k));
    expect(intents(none, ["choose"])).toEqual([]);
    expect(texts(e)).toContain("reach 1m gold");
  });
  run(member("generators", "empty"), (e) => expect(texts(e)).toContain("nothing to build yet"));
});

test("IdlePurchases: the amount radios report their key and show the checked one; Buy reports the row's key only when it can be bought", () => {
  const events: Event[] = [];
  run(member("purchases", "default", {}, { events }), (e) => {
    const radios = all(e, /./).filter((x) => x.role === "radio");
    expect(radios.map((x) => x.text.toLowerCase())).toEqual(["×1", "×10", "×100", "max"]);
    expect(radios[0]!.states).toContain("checked");
    tap(e, radios[2]!);
    tap(e, radios[3]!);
    tap(e, buyIn(e, "pick"));
    // Not affordable, maxed (no button) and locked send nothing.
    for (const k of ["cart", "star"]) {
      expect(buyIn(e, k).disabled).toBeTrue();
      tap(e, buyIn(e, k));
    }
    expect(buyIn(e, "crown")?.width ?? 0).toBe(0);
    expect(intents(events, ["chooseamount", "buy"])).toEqual(["chooseamount:100", "chooseamount:max", "buy:pick"]);
    for (const want of ["not enough", "maxed", "locked", "1.2k", "lv 4"]) expect(texts(e)).toContain(want);
  });
  run(member("purchases", "max"), (e) => {
    expect(all(e, /./).filter((x) => x.role === "radio")[3]!.states).toContain("checked");
    expect(texts(e)).toContain("×37");
  });
  // The game's own wording of an amount wins over the key.
  run(member("purchases", "default", { amounts: [{ id: "half", label: "50%" }, { id: "25", label: "" }], amount: "half" }), (e) => {
    expect(all(e, /./).filter((x) => x.role === "radio").map((x) => x.text.toLowerCase())).toEqual(["50%", "×25"]);
  });
  run(member("purchases", "maxed"), (e) => expect(all(e, /./).some((x) => x.role === "radio")).toBeFalse());
  run(member("purchases", "empty"), (e) => expect(texts(e)).toContain("no upgrades available"));
});

test("IdlePrestige: request first; the game's confirming shows Reset and Cancel; Escape cancels; an unmet requirement disables it", () => {
  const events: Event[] = [];
  run(member("prestige", "default", {}, { events }), (e) => {
    for (const want of ["12", "+48", "×1.12", "×1.60", "ready"]) expect(texts(e)).toContain(want);
    expect(find(e, /\/Confirm$/)).toBeUndefined();
    tap(e, find(e, /\/Request$/)!);
    expect(intents(events, ["request", "confirm", "cancel"])).toEqual(["request:"]);
    // The game asks; the panel does not on its own.
    expect(find(e, /\/Confirm$/)).toBeUndefined();
    og.ui.scope(e.world, "t").writable("confirming", false).set(true);
    e.advance(3);
    expect(texts(e)).toContain("reset for +48 souls?");
    expect(find(e, /\/Request$/)?.width ?? 0).toBe(0);
    tap(e, find(e, /\/Confirm$/)!);
    tap(e, find(e, /\/Cancel$/)!);
    key(e, "Escape");
    expect(intents(events, ["request", "confirm", "cancel"])).toEqual(["request:", "confirm:", "cancel:", "cancel:"]);
  });
  const refused: Event[] = [];
  run(member("prestige", "disabled", {}, { events: refused }), (e) => {
    const button = find(e, /\/Request$/)!;
    expect(button.disabled).toBeTrue();
    tap(e, button);
    expect(intents(refused, ["request"])).toEqual([]);
    for (const want of ["not yet", "640k / 1m", "reach 1m gold"]) expect(texts(e)).toContain(want);
  });
  // Escape does nothing while the panel is not asking.
  const idle: Event[] = [];
  run(member("prestige", "default", {}, { events: idle }), (e) => {
    key(e, "Escape");
    expect(intents(idle, ["cancel"])).toEqual([]);
  });
});

test("IdleOffline: time away from the game's words or its seconds; Collect and Double report; Double only when the game offers it", () => {
  const events: Event[] = [];
  run(member("offline", "default", {}, { events }), (e) => {
    expect(texts(e)).toContain("you were away for 3h 45m");
    for (const want of ["+1.24m", "+5,000", "watch an ad"]) expect(texts(e)).toContain(want);
    tap(e, find(e, /\/Double$/)!);
    tap(e, find(e, /\/Collect$/)!);
    expect(intents(events, ["double", "collect"])).toEqual(["double:", "collect:"]);
  });
  const single: Event[] = [];
  run(member("offline", "disabled", {}, { events: single }), (e) => {
    expect(texts(e)).toContain("you were away for 8h (capped)");
    expect(find(e, /\/Double$/)?.width ?? 0).toBe(0);
    tap(e, find(e, /\/Collect$/)!);
    expect(intents(single, ["double", "collect"])).toEqual(["collect:"]);
  });
  run(member("offline", "empty"), (e) => expect(texts(e)).toContain("nothing was gathered while you were away"));
});

test("keyboard reaches Collect and Enter confirms it", () => {
  const events: Event[] = [];
  run(member("offline", "default", {}, { events }), (e) => {
    const focused = () => elements(e).find((x) => x.states?.includes("focused"))?.path ?? "";
    for (let i = 0; i < 6 && !/\/Collect$/.test(focused()); i++) key(e, "Tab");
    expect(focused()).toMatch(/\/Collect$/);
    key(e, "Enter");
    expect(intents(events, ["collect", "double"])).toEqual(["collect:"]);
  });
});
