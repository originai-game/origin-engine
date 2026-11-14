// The rhythm components as a game uses them (docs/ui.md §6.1): built in,
// used by name from the game's own pages with nothing copied or loaded but
// the pages, dressed by the fantasy pack, each member mounted from every
// fixture its document declares and driven through the probe's touch, key
// and pad commands.
//
// Ways it could fail: an intent sent without its row key, or sent for a
// locked song or chart (by tap, by X over its row, or by its Preview) or a
// Retry the game refused; Preview reporting the wrong song; a failed load
// with no way to retry; a frame that mounts with no width; Escape or pad B
// not reaching Next; a fixture state that no longer mounts or spills past a
// phone's width; the judgement not replaying
// for a new hit id; a state (locked, cleared, danger, failed, full combo)
// shown by its dress alone.
import { afterAll, beforeAll, expect, setDefaultTimeout, test } from "bun:test";

// The tests run on ticks, not the clock; a time limit only guards a hang.
setDefaultTimeout(180_000);
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
const GROUP = join(ENGINE, "src/ui/components/rhythm");
const STYLES = join(ENGINE, "../../templates/styles");
let root = "";
let styles = "";

/** Each member's component name, parts and fixtures, read from its document as a game's tooling reads them. */
const MEMBERS: Record<string, { component: string; parts: string[]; fixtures: Record<string, Record<string, unknown>> }> = {};

beforeAll(() => {
  // The temporary directories first, so the cleanup below only ever removes them.
  root = mkdtempSync(join(tmpdir(), "og-rhythm-kit-"));
  styles = mkdtempSync(join(tmpdir(), "og-rhythm-styles-"));
  loadNativeHost();
  for (const d of ["fantasy", "shared"]) cpSync(join(STYLES, d), join(styles, d), { recursive: true });
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Rhythm kit"\nid "rhythm-kit"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  // A look naming the pack is what makes the cook bundle it (@styles/fantasy).
  mkdirSync(join(root, "assets/looks"), { recursive: true });
  writeFileSync(join(root, "assets/looks/fantasy.ogt"), 'type look\nversion 1\n---\nname "rhythm kit fantasy"\nui "fantasy"\n');
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

const DOCUMENTS = () => Object.keys(MEMBERS).map((m) => `ui/host-${m}.svelte`);

type View = { width: number; height: number; dpr: number; safeArea: { left: number; top: number; right: number; bottom: number }; coarsePointer: boolean; breakpoint: string };
const DESKTOP: View = { width: 1280, height: 720, dpr: 1.5, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktopHd" };
const PHONE: View = { width: 390, height: 844, dpr: 2, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };

function engine(component: string, stores: Record<string, unknown>, o: { events?: Event[]; view?: View; reduced?: boolean } = {}): Engine {
  const events = o.events ?? [];
  const Collect = defineSystem("RhythmKitEvents", {}, (_q, _t, world) => {
    for (const e of og.ui.docEvents(world)) events.push(e as unknown as Event);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(world) {
      const style = og.load.style(world, "@styles/fantasy");
      if (!style.ok) throw new Error(style.reason);
      og.ui.configure(world, { style: style.value, reducedMotion: o.reduced ?? false });
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
/** A list option by its row key. */
const option = (e: Engine, k: string): El => elements(e).find((x) => x.role === "option" && x.path.endsWith(`#${k}`))!;
/** The words shown, lower-cased: a pack may set them in capitals. */
const texts = (e: Engine): string[] => elements(e).filter((x) => x.width > 0).map((x) => x.text.toLowerCase());
const focused = (e: Engine): string => elements(e).find((x) => x.states?.includes("focused"))?.path ?? "";

// One test per member and view: each fixture boots its own engine.
for (const [view, viewport] of [["desktop", DESKTOP], ["phone", PHONE]] as const) {
  for (const name of ["song-select", "difficulty", "judgement", "score-hud", "results"]) {
    test(`${name} mounts from every fixture on a ${view} without errors, inside the screen's width`, () => {
      for (const fixture of Object.keys(MEMBERS[name]!.fixtures)) {
        run(member(name, fixture, {}, { view: viewport }), (e) => {
          expect(`${name}/${fixture}: ${errors(e).join("; ")}`).toBe(`${name}/${fixture}: `);
          const box = find(e, /^t:page\/[^/]+$/)!;
          expect(`${name}/${fixture} ${box.x + box.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
          // The frame is drawn and sits on the screen.
          const frame = find(e, /^t:page\/Root\/node1$/)!;
          expect(`${name}/${fixture} ${frame.width > 0 && frame.x >= -0.5 && frame.x + frame.width <= viewport.width + 0.5}`).toBe(`${name}/${fixture} true`);
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
    // Every part the document draws is declared.
    for (const used of text.matchAll(/part="([^"]+)"/g)) expect(m.parts).toContain(used[1]!);
  }
});

test("RhythmSongSelect: a row chooses by key, Preview reports the selected song's key, a locked song sends nothing", () => {
  const events: Event[] = [];
  run(member("song-select", "default", {}, { events }), (e) => {
    expect(option(e, "skyline").states).toContain("selected");
    tap(e, option(e, "ember"));
    tap(e, option(e, "vault"));
    tap(e, find(e, /\/Preview(#[^/]+)?$/)!);
    expect(intents(events, ["choose", "preview"])).toEqual(["choose:ember", "preview:skyline"]);
    expect(option(e, "vault").disabled).toBeTrue();
    // Every state is a word.
    for (const want of ["selected", "new", "cleared", "locked", "128 bpm"]) expect(texts(e)).toContain(want);
  });
  const previewing: Event[] = [];
  run(member("song-select", "selected", {}, { events: previewing }), (e) => {
    expect(texts(e)).toContain("previewing");
    expect(find(e, /\/Preview(#[^/]+)?$/)!.states ?? []).toContain("pressed");
  });
  // A locked song's Preview is disabled and a tap on it sends nothing.
  const locked: Event[] = [];
  run(member("song-select", "disabled", {}, { events: locked }), (e) => {
    const preview = find(e, /\/Preview(#[^/]+)?$/)!;
    expect(preview.disabled).toBeTrue();
    tap(e, preview);
    expect(intents(locked, ["choose", "preview"])).toEqual([]);
  });
  run(member("song-select", "empty"), (e) => expect(texts(e)).toContain("no songs yet"));
  run(member("song-select", "loading"), (e) => {
    expect(texts(e)).toContain("loading songs");
    expect(find(e, /\/Retry$/)).toBeUndefined();
  });
  // A failed load says so and offers Retry, which reports once.
  const failed: Event[] = [];
  run(member("song-select", "error", {}, { events: failed }), (e) => {
    expect(texts(e)).toContain("the songs could not be loaded");
    tap(e, find(e, /\/Retry$/)!);
    expect(intents(failed, ["retry", "choose", "preview"])).toEqual(["retry:"]);
  });
});

test("RhythmSongSelect: X never previews a locked row, by focus or under the pointer", () => {
  const events: Event[] = [];
  run(member("song-select", "default", {}, { events }), (e) => {
    // The pointer rests on the locked row: X is the focused row's action
    // (a pointer does not move the focus), so it previews that row, never
    // the locked one under the pointer.
    e.probe.commands(`0:touch_move:${centre(option(e, "vault"))}`);
    e.advance(4);
    const row = focused(e).split("#").pop();
    expect(row).not.toBe("vault");
    key(e, "KeyX");
    expect(intents(events, ["choose", "preview"])).toEqual([`preview:${row}`]);
    // X at every focus stop: the playable rows preview, the locked one is never reached.
    const stops: string[] = [];
    for (let i = 0; i < 12; i++) {
      key(e, "Tab");
      stops.push(focused(e));
      key(e, "KeyX");
    }
    expect(stops.some((p) => /#vault$/.test(p))).toBeFalse();
    expect(intents(events, ["preview"]).length).toBeGreaterThan(0);
    expect(intents(events, ["preview"])).not.toContain("preview:vault");
  });
});

test("RhythmSongSelect: the selection keeps its key through a reorder, and X on a focused row previews that row", () => {
  const events: Event[] = [];
  run(member("song-select", "default", {}, { events }), (e) => {
    const songs = og.ui.scope(e.world, "t").collection("songs", MEMBERS["song-select"]!.fixtures.default!.songs as { id: string }[], { key: "id" });
    songs.move("skyline", "static");
    e.advance(3);
    expect(option(e, "skyline").states).toContain("selected");
    expect(option(e, "ember").states ?? []).not.toContain("selected");
    for (let i = 0; i < 8 && !/#static$/.test(focused(e)); i++) key(e, "Tab");
    expect(focused(e)).toMatch(/#static$/);
    key(e, "KeyX");
    expect(intents(events, ["preview"])).toEqual(["preview:static"]);
  });
});

test("RhythmDifficulty: a chart chooses by key; a locked chart is disabled and silent; states are words", () => {
  const events: Event[] = [];
  run(member("difficulty", "default", {}, { events }), (e) => {
    expect(option(e, "hard").states).toContain("selected");
    tap(e, option(e, "normal"));
    tap(e, option(e, "expert"));
    expect(intents(events, ["choose"])).toEqual(["choose:normal"]);
    expect(option(e, "expert").disabled).toBeTrue();
    for (const want of ["lv 9", "selected", "locked", "not played", "s", "995,120"]) expect(texts(e)).toContain(want);
  });
  run(member("difficulty", "empty"), (e) => expect(texts(e)).toContain("no charts for this song"));
});

test("RhythmDifficulty: keyboard and pad reach a chart and confirm it with its key", () => {
  for (const [device, press] of [["keyboard", "key_down:Enter;0.05:key_up:Enter"], ["pad", "gamepad:a;0.05:gamepad:-"]] as const) {
    const events: Event[] = [];
    run(member("difficulty", "default", {}, { events }), (e) => {
      for (let i = 0; i < 6 && !/#normal$/.test(focused(e)); i++) device === "keyboard" ? key(e, "Tab") : pad(e, "-:1,0");
      expect(`${device} ${focused(e)}`).toMatch(/#normal$/);
      e.probe.commands(`0:${press}`);
      e.advance(8);
      expect(`${device} ${intents(events, ["choose"])}`).toBe(`${device} choose:normal`);
    });
  }
});

test("RhythmJudgement: the word and timing, a new hit id remakes the plate, the combo and its break in words", () => {
  run(member("judgement"), (e) => {
    expect(texts(e)).toContain("perfect");
    expect(texts(e)).toContain("128");
    expect(texts(e)).toContain("combo");
    const plate = () => elements(e).find((x) => /^t:page\/Root\/node1\/[^/]+$/.test(x.path) && x.width > 0)!.path;
    const before = plate();
    const scope = og.ui.scope(e.world, "t");
    scope.writable("combo", 128).set(129);
    e.advance(2);
    expect(plate()).toBe(before);
    scope.writable("hit", 1).set(2);
    e.advance(2);
    expect(plate()).not.toBe(before);
  });
  run(member("judgement", "great"), (e) => {
    expect(texts(e)).toContain("great");
    expect(texts(e)).toContain("early");
  });
  run(member("judgement", "good"), (e) => expect(texts(e)).toContain("late"));
  run(member("judgement", "miss"), (e) => {
    expect(texts(e)).toContain("miss");
    expect(texts(e)).toContain("combo broken");
  });
  run(member("judgement", "none"), (e) => expect(texts(e).some((x) => /perfect|great|good|miss|combo/.test(x))).toBeFalse());
  run(member("judgement", "overflow"), (e) => expect(texts(e)).toContain("transcendent critical perfect"));
  // Under reduced motion the word is there at once (it fades, not pops; the frames show it).
  run(member("judgement", "default", {}, { reduced: true }), (e) => expect(texts(e)).toContain("perfect"));
});

test("RhythmScoreHud: score with grouping, the life gauge hidden when negative, Danger and Failed in words", () => {
  const meters = (e: Engine) => elements(e).filter((x) => x.role === "progressbar" && x.width > 0).length;
  run(member("score-hud"), (e) => {
    for (const want of ["482,310", "97.42%"]) expect(texts(e)).toContain(want);
    expect(texts(e)).not.toContain("danger");
    expect(meters(e)).toBe(2);
  });
  run(member("score-hud", "danger"), (e) => expect(texts(e)).toContain("danger"));
  run(member("score-hud", "failed"), (e) => {
    expect(texts(e)).toContain("failed");
    expect(texts(e)).not.toContain("danger");
  });
  run(member("score-hud", "nolife"), (e) => {
    expect(meters(e)).toBe(1);
    expect(texts(e)).not.toContain("accuracy");
  });
});

test("RhythmResults: Retry and Next report once each; Escape and pad B are Next; a refused Retry is silent", () => {
  const events: Event[] = [];
  run(member("results", "default", {}, { events }), (e) => {
    for (const want of ["s", "cleared", "962,410", "new best", "best 948,220", "412", "97.42%", "perfect", "388"]) expect(texts(e)).toContain(want);
    tap(e, find(e, /\/Retry$/)!);
    tap(e, find(e, /\/Next$/)!);
    key(e, "Escape");
    expect(intents(events, ["retry", "next"])).toEqual(["retry:", "next:", "next:"]);
  });
  const padded: Event[] = [];
  run(member("results", "default", {}, { events: padded }), (e) => {
    pad(e, "b");
    expect(intents(padded, ["retry", "next"])).toEqual(["next:"]);
  });
  const refused: Event[] = [];
  run(member("results", "disabled", {}, { events: refused }), (e) => {
    expect(find(e, /\/Retry$/)!.disabled).toBeTrue();
    tap(e, find(e, /\/Retry$/)!);
    expect(intents(refused, ["retry"])).toEqual([]);
  });
  run(member("results", "failed"), (e) => {
    expect(texts(e)).toContain("failed");
    expect(texts(e)).not.toContain("new best");
  });
  run(member("results", "fullCombo"), (e) => expect(texts(e)).toContain("full combo"));
  run(member("results", "allPerfect"), (e) => expect(texts(e)).toContain("all perfect"));
  run(member("results", "empty"), (e) => {
    expect(texts(e)).toContain("results");
    expect(texts(e)).toContain("no notes judged");
  });
});
