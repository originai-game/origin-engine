// The arena components as a game uses them (docs/ui.md §6.1): built in,
// used by name from the game's own pages with nothing copied or loaded but
// the pages, dressed by the fantasy pack (and by the default style), each
// member mounted from every fixture its document declares and driven
// through the probe's touch, key and pad commands.
//
// Ways it could fail: an intent sent without its row key, or sent for a
// locked or taken character, a Lock in or Start the game refused; the
// selection following a cell's index instead of its key; a fixture state
// that no longer mounts or spills past a phone's width; the local player's
// row not marked; a team's rows drawn under the other team; a state shown
// by its dress alone.
import { afterAll, beforeAll, expect, setDefaultTimeout, test } from "bun:test";

// The tests run on ticks, not the clock; a time limit only guards a hang.
// A test boots up to five engines, each several seconds on a machine
// shared with other lanes' renders, so the limit leaves room.
setDefaultTimeout(180_000);
import { cpSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "ignore"; stderr: "pipe" }): { exitCode: number; stderr: Uint8Array } };
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

type El = { id: string; path: string; text: string; x: number; y: number; width: number; height: number; states?: string[]; disabled?: boolean; name?: string; role?: string };
type Event = { mount: string; path: string; rowKey: string; kind: string; name: string };

const ENGINE = dirname(dirname(fileURLToPath(import.meta.url)));
const GROUP = join(ENGINE, "src/ui/components/arena");
const STYLES = join(ENGINE, "../../templates/styles");
let root = "";
let styles = "";

/** Each member's component name, parts and fixtures, read from its document as a game's tooling reads them. */
const MEMBERS: Record<string, { component: string; parts: string[]; fixtures: Record<string, Record<string, unknown>> }> = {};

beforeAll(() => {
  // The directories first: nothing below may throw before they exist.
  root = mkdtempSync(join(tmpdir(), "og-arena-kit-"));
  styles = mkdtempSync(join(tmpdir(), "og-arena-styles-"));
  loadNativeHost();
  for (const d of ["fantasy", "shared"]) cpSync(join(STYLES, d), join(styles, d), { recursive: true });
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Arena kit"\nid "arena-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  // A look naming the pack is what makes the cook bundle it (@styles/fantasy).
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "arena kit fantasy"\nui "fantasy"\n');
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

// Only what beforeAll made: an empty path would be the working directory.
afterAll(() => {
  if (root) rmSync(root, { recursive: true, force: true });
  if (styles) rmSync(styles, { recursive: true, force: true });
});

// Only the host page an engine mounts: each load reads the whole style pack again.
const DOCUMENTS = (component: string) => Object.keys(MEMBERS).filter((m) => MEMBERS[m]!.component === component).map((m) => `ui/host-${m}.svelte`);

type View = { width: number; height: number; dpr: number; safeArea: { left: number; top: number; right: number; bottom: number }; coarsePointer: boolean; breakpoint: string };
const DESKTOP: View = { width: 1280, height: 720, dpr: 1.5, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktopHd" };
const PHONE: View = { width: 390, height: 844, dpr: 2, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };

/** An engine mounting `component` dressed by the fantasy pack (or by the default style), its document events collected in `events`. */
function engine(component: string, stores: Record<string, unknown>, o: { events?: Event[]; view?: View; pack?: boolean; reduced?: boolean; textScale?: number } = {}): Engine {
  const events = o.events ?? [];
  const Collect = defineSystem("ArenaKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      if (o.pack !== false) {
        const style = og.load.style(world, "@styles/fantasy");
        if (!style.ok) throw new Error(style.reason);
        og.ui.configure(world, { style: style.value, reducedMotion: o.reduced ?? false, textScale: o.textScale ?? 1 });
      } else og.ui.configure(world, { reducedMotion: o.reduced ?? false, textScale: o.textScale ?? 1 });
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
/** A row or cell by its key. */
const row = (e: Engine, k: string): El => elements(e).find((x) => x.path.endsWith(`#${k}`) && x.width > 0)!;
/** The words shown, lower-cased: a pack may set them in capitals. */
const texts = (e: Engine): string[] => elements(e).filter((x) => x.width > 0).map((x) => x.text.toLowerCase());
const focused = (e: Engine) => elements(e).find((x) => x.states?.includes("focused"))?.path ?? "";

const NAMES = ["roster", "draft", "versus", "scoreboard", "round-hud", "lobby"];

// One test per member and view: each fixture boots its own engine.
for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  for (const name of NAMES) {
    test(`${name} mounts from every fixture on a ${view} without errors, inside the screen's width`, () => {
      for (const fixture of Object.keys(MEMBERS[name]!.fixtures)) {
        run(member(name, fixture, {}, { view: viewport }), (e) => {
          expect(`${name}/${fixture}: ${errors(e).join("; ")}`).toBe(`${name}/${fixture}: `);
          const box = find(e, /^t:page\/[^/]+$/)!;
          expect(`${name}/${fixture} ${box.width > 0 && box.x + box.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
          const frame = find(e, /^t:page\/Root\/node1$/)!;
          expect(`${name}/${fixture} ${frame.width > 0 && frame.height > 0 && frame.x >= -0.5 && frame.x + frame.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
        });
      }
    }, 180_000);
  }
}

// 150 % text on a phone: every word of each member's stress fixture is drawn
// inside the screen, and a scoreboard row's figures never run into each
// other. The drawn text's bounds (the dump's texts), not the layout boxes: a
// box keeps its width while the words spill past it. They are read from the
// native host's UI frame, as og ui sheet reads them.
type Drawn = { bounds: [number, number, number, number]; clip: [number, number, number, number]; text: string };
const drawn = (e: Engine): Drawn[] => (JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id)) as { texts?: Drawn[] }).texts ?? [];
test("every member's stress fixture is drawn inside a phone at 150 % text, and the scoreboard's figures keep apart", () => {
  for (const name of NAMES) {
    run(member(name, "overflow", {}, { view: PHONE, textScale: 1.5 }), (e) => {
      expect(`${name}: ${errors(e).join("; ")}`).toBe(`${name}: `);
      const words = drawn(e);
      expect(words.length).toBeGreaterThan(0);
      // What shows of each word: its bounds cut by its clip (an ellipsis or a scroll may hide the rest).
      for (const w of words) {
        const left = Math.max(w.bounds[0], w.clip[0]);
        const right = Math.min(w.bounds[0] + w.bounds[2], w.clip[0] + w.clip[2]);
        expect(`${name} "${w.text}" ${right <= left || (left >= -0.5 && right <= PHONE.width + 0.5)}`).toBe(`${name} "${w.text}" true`);
      }
      if (name !== "scoreboard") return;
      // The first team's first row: 128, 3, 214, 184,320 on one line.
      const figures = words.filter((w) => ["128", "3", "214", "184,320"].includes(w.text)).sort((p, q) => p.bounds[0] - q.bounds[0]);
      expect(figures.map((w) => w.text)).toEqual(["128", "3", "214", "184,320"]);
      // A figure cut short reads as another number: each is drawn whole.
      for (const w of figures) expect(`${w.text} ${w.bounds[0] >= w.clip[0] - 0.5 && w.bounds[0] + w.bounds[2] <= w.clip[0] + w.clip[2] + 0.5} ${JSON.stringify(w.bounds)} in ${JSON.stringify(w.clip)}`).toStartWith(`${w.text} true`);
      // The long name is drawn whole too, on its own line above the figures, and nothing on the figures' line runs into the first.
      const first = figures[0]!.bounds;
      const names = words.filter((w) => /maximilian|okonkwo/i.test(w.text));
      expect(names.length).toBeGreaterThan(0);
      for (const w of names) expect(`"${w.text}" ${w.bounds[0] >= w.clip[0] - 0.5 && w.bounds[0] + w.bounds[2] <= w.clip[0] + w.clip[2] + 0.5 && w.bounds[1] + w.bounds[3] <= first[1] + 0.5} ${JSON.stringify(w.bounds)} in ${JSON.stringify(w.clip)} above ${first[1]}`).toStartWith(`"${w.text}" true`);
      const beside = words.filter((w) => !figures.includes(w) && w.bounds[0] < first[0] && w.bounds[1] < first[1] + first[3] && w.bounds[1] + w.bounds[3] > first[1]);
      for (const w of beside) expect(`"${w.text}" ${w.bounds[0] + w.bounds[2] <= first[0] + 0.5} ${JSON.stringify(w.bounds)} before ${first[0]}`).toStartWith(`"${w.text}" true`);
      for (let i = 1; i < figures.length; i++) expect(`${figures[i - 1]!.text}|${figures[i]!.text} ${figures[i - 1]!.bounds[0] + figures[i - 1]!.bounds[2] <= figures[i]!.bounds[0] + 0.5}`).toBe(`${figures[i - 1]!.text}|${figures[i]!.text} true`);
    });
  }
});
test("every member mounts under the default style as well", () => {
  for (const name of NAMES) run(member(name, "default", {}, { pack: false }), (e) => expect(`${name}: ${errors(e).join("; ")}`).toBe(`${name}: `));
}, 120_000);

test("every member names its frame first and only its own parts", () => {
  expect(Object.keys(MEMBERS).sort()).toEqual([...NAMES].sort());
  for (const [name, m] of Object.entries(MEMBERS)) {
    expect(`${name} ${m.parts[0]}`).toBe(`${name} ${m.component}.frame`);
    // The markup only: a selector in <style> names a part without any element carrying it.
    const text = readFileSync(join(GROUP, `${name}.svelte`), "utf8").replace(/<style>[\s\S]*<\/style>/, "");
    expect(text).not.toContain("<style");
    for (const p of m.parts) expect(`${name} ${p} ${p.startsWith(`${m.component}.`) && text.includes(`part="${p}"`)}`).toBe(`${name} ${p} true`);
    for (const used of text.matchAll(/part="([^"]+)"/g)) expect(m.parts).toContain(used[1]!);
  }
});

test("ArenaRoster: a free cell chooses by key; locked and taken cells are disabled, silent and say why", () => {
  const events: Event[] = [];
  run(member("roster", "default", {}, { events }), (e) => {
    expect(row(e, "vex").states).toContain("selected");
    expect(row(e, "oarsman").disabled).toBeTrue();
    expect(row(e, "idris").disabled).toBeTrue();
    tap(e, row(e, "brenna"));
    tap(e, row(e, "oarsman"));
    tap(e, row(e, "idris"));
    tap(e, row(e, "sela"));
    expect(intents(events, ["choose"])).toEqual(["choose:brenna", "choose:sela"]);
    const all = texts(e);
    for (const want of ["locked", "taken by kai", "p1", "p2", "vanguard", "selected"]) expect(all).toContain(want);
  });
  run(member("roster", "empty"), (e) => expect(texts(e)).toContain("no characters available"));
});

test("ArenaRoster: the selection keeps its key through a reorder, and the pad walks the free cells and confirms", () => {
  const events: Event[] = [];
  run(member("roster", "default", {}, { events }), (e) => {
    const characters = og.ui.scope(e.world, "t").collection("characters", MEMBERS.roster!.fixtures.default!.characters as { id: string }[], { key: "id" });
    characters.move("vex", "brenna");
    e.advance(3);
    expect(row(e, "vex").states).toContain("selected");
    expect(row(e, "brenna").states ?? []).not.toContain("selected");
    expect(row(e, "vex").x).toBeLessThan(row(e, "brenna").x);
    for (let i = 0; i < 8 && !/#maren$/.test(focused(e)); i++) pad(e, "-:1,0");
    expect(focused(e)).toMatch(/#maren$/);
    pad(e, "a");
    expect(intents(events, ["choose"])).toEqual(["choose:maren"]);
  });
});

test("ArenaDraft: the phase, the turn and each slot's state are words; Lock in reports once per tap and X", () => {
  const events: Event[] = [];
  run(member("draft", "default", {}, { events }), (e) => {
    const all = texts(e);
    for (const want of ["pick phase", "blue team to choose", "banned", "picking", "open", "oarsman", "brenna", "kai"]) expect(all).toContain(want);
    expect(row(e, "ap2").states ?? []).toContain("current");
    expect(elements(e).some((x) => x.role === "timer" && x.width > 0)).toBeTrue();
    // Each team's slots sit in its own column.
    expect(row(e, "ap1").x).toBeLessThan(row(e, "bp1").x);
    tap(e, find(e, /\/Confirm$/)!);
    key(e, "KeyX");
    expect(intents(events, ["confirm"])).toEqual(["confirm:", "confirm:"]);
  });
});

test("ArenaDraft: a refused Lock in is disabled and silent to a tap and to X", () => {
  const refused: Event[] = [];
  run(member("draft", "ban", {}, { events: refused }), (e) => {
    expect(texts(e)).toContain("ban phase");
    expect(texts(e)).toContain("banning");
    expect(find(e, /\/Confirm$/)!.disabled).toBeTrue();
    tap(e, find(e, /\/Confirm$/)!);
    key(e, "KeyX");
    expect(intents(refused, ["confirm"])).toEqual([]);
  });
});

test("ArenaDraft: a finished draft names its teams and drops the timer; no slots says so", () => {
  run(member("draft", "done"), (e) => {
    expect(texts(e)).toContain("draft complete");
    expect(texts(e)).toContain("iron wolves");
    expect(elements(e).some((x) => x.role === "timer" && x.width > 0)).toBeFalse();
  });
  run(member("draft", "empty"), (e) => expect(texts(e)).toContain("no draft yet"));
});

test("ArenaVersus: both sides with names and ranks, the local player named You, an empty side waiting", () => {
  run(member("versus", "teams"), (e) => {
    const all = texts(e);
    for (const want of ["iron wolves", "ember court", "platinum iii", "you", "vs", "idris"]) expect(all).toContain(want);
    expect(row(e, "kai").x).toBeLessThan(row(e, "rook").x);
    expect(row(e, "you").states ?? []).toContain("current");
  });
  run(member("versus", "waiting"), (e) => expect(texts(e)).toContain("waiting for opponents"));
  run(member("versus", "teams", {}, { reduced: true }), (e) => expect(texts(e)).toContain("ember court"));
});

test("ArenaScoreboard: each team's rows under its own heading in the game's order, the local row current, disconnected in words", () => {
  run(member("scoreboard"), (e) => {
    const ys = ["kai", "you", "ana", "rook", "wren", "quill"].map((k) => row(e, k).y);
    expect([...ys].sort((a, b) => a - b)).toEqual(ys);
    expect(row(e, "you").states ?? []).toContain("current");
    const all = texts(e);
    for (const want of ["blue team", "red team", "7", "5", "k", "d", "a", "2,840", "disconnected", "you"]) expect(all).toContain(want);
  });
  run(member("scoreboard", "score"), (e) => {
    expect(texts(e)).not.toContain("k");
    expect(texts(e)).toContain("41");
  });
  run(member("scoreboard", "empty"), (e) => expect(texts(e)).toContain("no players"));
});

test("ArenaRoundHud: round of rounds, both scores, the timer, and the outcome banner in words", () => {
  run(member("round-hud"), (e) => {
    const all = texts(e);
    for (const want of ["round 4 of 7", "2", "1", "you"]) expect(all).toContain(want);
    expect(elements(e).some((x) => x.role === "timer" && x.width > 0)).toBeTrue();
    expect(all.some((x) => /round (won|lost|drawn)/.test(x))).toBeFalse();
  });
  run(member("round-hud", "won"), (e) => {
    expect(texts(e)).toContain("round won");
    expect(texts(e)).toContain("match point");
  });
  run(member("round-hud", "lost"), (e) => expect(texts(e)).toContain("round lost"));
  run(member("round-hud", "draw"), (e) => {
    expect(texts(e)).toContain("round drawn");
    expect(texts(e)).toContain("round 6");
  });
  run(member("round-hud", "won", {}, { reduced: true }), (e) => expect(texts(e)).toContain("round won"));
});

test("ArenaLobby: Invite reports the open seat's key and Ready reports; a refused Start is silent", () => {
  const events: Event[] = [];
  run(member("lobby", "default", {}, { events }), (e) => {
    const all = texts(e);
    for (const want of ["host", "you", "ready", "not ready", "open slot", "ready up", "start match"]) expect(all).toContain(want);
    const invites = elements(e).filter((x) => /\/Invite$/.test(x.path) && x.width > 0);
    expect(invites.length).toBe(2);
    tap(e, invites[1]!);
    expect(find(e, /\/Start$/)!.disabled).toBeTrue();
    tap(e, find(e, /\/Start$/)!);
    tap(e, find(e, /\/Ready$/)!);
    expect(intents(events, ["invite", "ready", "unready", "start"])).toEqual(["invite:s4", "ready:"]);
  });
});

// One engine boot per test: on a loaded machine one takes several seconds.
test("ArenaLobby: the host's Start and Cancel ready report once each", () => {
  const full: Event[] = [];
  run(member("lobby", "full", {}, { events: full }), (e) => {
    expect(texts(e)).toContain("cancel ready");
    tap(e, find(e, /\/Start$/)!);
    tap(e, find(e, /\/Unready$/)!);
    expect(intents(full, ["ready", "unready", "start"])).toEqual(["start:", "unready:"]);
  });
});

test("ArenaLobby: a guest sees no Start and no Invite, and X cancels ready", () => {
  const guest: Event[] = [];
  run(member("lobby", "guest", {}, { events: guest }), (e) => {
    expect(find(e, /\/Start$/)?.width ?? 0).toBe(0);
    expect(elements(e).filter((x) => /\/Invite$/.test(x.path) && x.width > 0).length).toBe(0);
    key(e, "KeyX");
    expect(intents(guest, ["unready"])).toEqual(["unready:"]);
  });
});

test("ArenaLobby: a refused Ready is silent to a tap and to X; no seats says so", () => {
  const down: Event[] = [];
  run(member("lobby", "disabled", {}, { events: down }), (e) => {
    tap(e, find(e, /\/Ready$/)!);
    key(e, "KeyX");
    expect(intents(down, ["ready"])).toEqual([]);
  });
  run(member("lobby", "empty"), (e) => expect(texts(e)).toContain("no slots"));
});

for (const [device, press] of [["keyboard", "key_down:Enter;0.05:key_up:Enter"], ["pad", "gamepad:a;0.05:gamepad:-"]] as const) {
  test(`ArenaLobby: the ${device} reaches Start and confirms it`, () => {
    const events: Event[] = [];
    run(member("lobby", "full", {}, { events }), (e) => {
      for (let i = 0; i < 10 && !/\/Start$/.test(focused(e)); i++) device === "keyboard" ? key(e, "Tab") : pad(e, "-:1,0");
      expect(`${device} ${focused(e)}`).toMatch(/\/Start$/);
      e.probe.commands(`0:${press}`);
      e.advance(8);
      expect(`${device} ${intents(events, ["start", "unready"])}`).toBe(`${device} start:`);
    });
  });
}
