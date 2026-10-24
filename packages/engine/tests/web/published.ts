// A playtest's script on the bytes that ship (examples/WEB-FINDINGS.md #6):
// the project published as its author publishes it (`og build --publish`,
// the og binary, the project's profile), served as the platform opens a
// release (play-origin.ts: the top-level page with `window.__OG_GAME`, a
// stand-in play origin answering Player protocol 2 over HTTP, the build's
// own `_headers`), and played in headless Chrome by a script of the
// playtests' own syntax, `<seconds>:<action>[:<args>]` steps joined by `;`,
// through nothing the publish build does not keep: real input events on
// the page, and the shipped probe's `tick`, `state` and `capture`. The
// publish build has no command channel (og-build §6), so a step that is
// not input or a capture is refused, never skipped.
//
//   bun tests/web/published.ts --project examples/hollowbell [--webgl] [--no-build] [--no-isolation]
//       [--script "<steps>" [--then "<steps>"] | --playtest <file>] [--title <node>]
//       [--references <dir>] [--prefix <name prefix>]
//       [--out <dir>] [--chrome <path>]
//
// The steps (a step at <s> seconds runs at tick round(s × 60), as `og run
// --commands` runs it):
//
//   key_down:[Mod+]<code>, key_up:[Mod+]<code>   a real key event (KeyboardEvent.code; Shift, Ctrl, Alt, Meta)
//   type:<text>                                  each character a real key press with its text, two ticks apart
//   touch_move_to:<id or path>, move_to_text:[=]<text>[@N]
//                                                the mouse moves to the control's centre, found in the
//                                                accessibility mirror by its og-ui path (or its last ids)
//                                                or its name (`=` exact; else a part of it)
//   touch_move:x,y, scroll:dx,dy                 the mouse at, or a wheel at, CSS pixels of the canvas
//   touch_start, touch_end                       the left button pressed, released where the mouse is
//   screenshot:<file>                            the probe's capture at that tick, compared with
//                                                <references>/<tier>/<prefix><file>, else metal's, under
//                                                og-tests' rule (tests/support/png.ts compareFrames)
//   state:<file>                                 the probe's state() (tick, facts, timings) into <file>
//   mirror:<file>                                the accessibility mirror's nodes and the focused element
//   quit                                         the end of the script
//
// The clock is the harness's. Before the engine Worker's first line runs
// (cdp.ts prepareWorkers), its requestAnimationFrame and performance.now
// are replaced by a clock that gives it one display frame of one tick when
// the harness asks, so every step lands on its tick and every capture is of
// the tick it names, as `og run` does natively; nothing of the build is
// changed, and everything else on the page (the shell, the platform, the
// lanes pool, the GPU) runs on real time. Input steps go in at the command
// channel's boundary: at tick T, before the next world step. A capture
// waits for its readback with the clock held.
//
// What a run proves, each a check in published.json: the game boots on the
// published page, its title node is in the accessibility mirror behind the
// play gate, the gate opens to a real click, every screenshot matches its
// reference, the project's own checks over its `state` and `mirror` files
// hold (a real input reached the game), and the page, its Workers and
// Chrome logged no warning or error. --playtest runs a playtest file's own
// commands (its `then` steps as relaunches: reloads under the service
// worker), which must be input and screenshot steps only; its `check`
// reads files a published page cannot write, so only its frames are
// judged. Without --script or --playtest the project's plan
// below is the script: examples/hollowbell (the title against metal's
// title.png, New game pressed, a name typed into the focused field, Begin,
// ArrowRight held), examples/brasswick (title.png and start.png, frames.og.ts's
// presses through the level select into the first level, ArrowRight held,
// Escape, Español in the settings, a reload in Spanish),
// examples/meridian-loop (tests/published.og.ts's four pub-*.png frames,
// Time trial, the countdown, W up the straight, Escape; then Deutsch in the
// settings and a reload in German) and templates/ui-kit (the gallery against
// kit-desktop-gallery.png, Settings pressed, Escape back). A plan may name a
// playtest of its project whose steps run with its own (`playtest`).
//
// Writes <out>/<id>/<tier>[-single]/published.json, the captures (and a
// <file>.diff.rgba on a mismatch) and console.txt (default dist/published).
// Exits 1 when a check fails. One Chrome at a time; not in a gate lane.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { Browser, chromePath, type Page } from "./cdp.ts";
import { findInMirror, keyEvent, MIRROR_TREE, OBSERVE_MIRROR, playOrigin, publish } from "./play-origin.ts";
import { ogBin } from "../support/og-bin.ts";
import { compareFrames, decodePng, TOLERANCE } from "../support/png.ts";
import { bundleGame } from "../../src/node/build.ts";
import type { PlaytestDesc } from "../../src/playtest.ts";

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const projectRel = opt("--project") ?? "examples/hollowbell";
const project = join(repo, projectRel);
const id = projectRel.split("/").pop()!;
const webgl = args.includes("--webgl");
const tier = webgl ? "webgl2" : "webgpu";
const isolated = !args.includes("--no-isolation");
const work = resolve(opt("--out") ?? join(pkg, "dist", "published"));
const out = join(work, id, `${tier}${isolated ? "" : "-single"}`);
mkdirSync(out, { recursive: true });
const TICK_RATE = 60;
const [WIDTH, HEIGHT] = [1280, 720] as const;
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

// ── the plans: each project's script and what it must show ─────────────

/** What a plan's checks read: the files the `state` and `mirror` steps wrote. */
interface Run {
  /** A `state:<file>` step's probe state. */
  state(file: string): { tick: number; facts: Record<string, unknown> | null };
  /** A `mirror:<file>` step's mirror. */
  mirror(file: string): Mirrored;
  /** The Player calls the stand-in play origin answered: before the first relaunch (0), after relaunch n (n). */
  calls(launch: number): Array<{ method: string; args: Record<string, unknown> }>;
}
interface Mirrored {
  tick: number;
  nodes: Array<{ path: string; role: string | null; name: string; states: string[] }>;
  /** The element with the DOM focus: its tag, role, accessible name, and whether it or an ancestor is aria-hidden. */
  focused: { tag: string; role: string | null; label: string | null; hidden: boolean } | null;
}
interface Plan {
  /** A node (a path's last ids, or a name) that says the title is up. */
  readonly title: string;
  /** Where the screenshots' references are (`<dir>/<tier>/<prefix><file>`), relative to the repository. */
  readonly references?: string;
  readonly prefix?: string;
  readonly script: readonly string[];
  /** A playtest of the project (relative to it) whose input and screenshot steps run with `script`, so the frames are the playtest's own; its `quit` is dropped. */
  readonly playtest?: string;
  /** Relaunches after the script, in order, each a reload of the page (under its service worker) with its own script from tick 0, as a playtest's `then`. */
  readonly then?: ReadonlyArray<readonly string[]>;
  /** Each check: whether it held, and what it says. */
  readonly check?: (run: Run) => Array<[boolean, string]>;
}

const PLANS: Record<string, Plan> = {
  hollowbell: {
    title: "new",
    script: [
      // tests/frames.og.ts's title.png is this tick's frame (its 0.2:name:Wren is no part of the title).
      "1.0:screenshot:title.png",
      "1.0:state:title.json",
      // New game, pressed with the mouse where the mirror says it is: the name dialog, its field focused.
      "1.1:touch_move_to:new",
      "1.15:touch_start",
      "1.2:touch_end",
      "1.6:mirror:naming.json",
      // Typed: real key presses into the focused field (the shell's text entry, text-input.ts).
      "1.7:type:Wren",
      "2.0:state:named.json",
      "2.1:touch_move_to:Begin",
      "2.15:touch_start",
      "2.2:touch_end",
      "2.8:state:begun.json",
      "2.9:key_down:ArrowRight",
      "3.6:key_up:ArrowRight",
      "3.7:state:ran.json",
      "3.8:quit",
    ],
    check(run) {
      const naming = run.mirror("naming.json");
      const named = run.state("named.json").facts ?? {};
      const begun = run.state("begun.json").facts ?? {};
      const ran = run.state("ran.json").facts ?? {};
      return [
        [run.state("title.json").facts?.["phase"] === "title", "tick 60 is the title (facts.phase)"],
        [naming.nodes.some((n) => n.role === "textbox"), "the press on New game opened the name dialog: a text box in the mirror"],
        [naming.focused?.tag === "TEXTAREA" && !naming.focused.hidden && !!naming.focused.label, `the focus is in a named text box no aria-hidden hides (${JSON.stringify(naming.focused)})`],
        [named["name"] === "Wren", `the typed keys reached the game: facts.name is ${JSON.stringify(named["name"])}`],
        [begun["phase"] === "play", `Begin started the run (facts.phase ${JSON.stringify(begun["phase"])})`],
        [typeof ran["x"] === "number" && typeof begun["x"] === "number" && (ran["x"] as number) > (begun["x"] as number) + 1, `ArrowRight held moved Wren right (x ${begun["x"]} to ${ran["x"]})`],
      ];
    },
  },
  brasswick: {
    title: "new",
    script: [
      // tests/frames.og.ts's title.png and start.png are these ticks' frames: its presses on the way, the
      // level select opened and closed, New game. Its 0.2:unlock:2 is a command (the select shows one level
      // open here, not two), and neither frame shows the select.
      "1.0:screenshot:title.png",
      "1.0:state:title.json",
      "1.1:touch_move_to:levels",
      "1.117:touch_start",
      "1.15:touch_end",
      "1.6:mirror:levels.json",
      "2.1:key_down:Escape",
      "2.134:key_up:Escape",
      "2.6:touch_move_to:new",
      "2.617:touch_start",
      "2.65:touch_end",
      "3.8:screenshot:start.png",
      "3.8:state:start.json",
      "4.0:key_down:ArrowRight",
      "5.0:key_up:ArrowRight",
      "5.1:state:ran.json",
      "5.3:key_down:Escape",
      "5.334:key_up:Escape",
      "5.8:state:paused.json",
      // The settings from pause: the Language tab, Español, Done (og.settings through the play origin's store).
      "6.0:touch_move_to:settings",
      "6.017:touch_start",
      "6.05:touch_end",
      "6.6:touch_move_to:Tab#3",
      "6.617:touch_start",
      "6.65:touch_end",
      "7.0:touch_move_to:Option#1",
      "7.017:touch_start",
      "7.05:touch_end",
      "7.4:touch_move_to:Done",
      "7.417:touch_start",
      "7.45:touch_end",
      "8.0:state:saved.json",
      "8.1:quit",
    ],
    // A returning player's reload: the title in Spanish (FINDINGS #16: the answers are read before the title is built).
    then: [["1.0:state:reloaded.json", "1.0:mirror:title-es.json", "1.1:quit"]],
    check(run) {
      const facts = (f: string) => run.state(f).facts ?? {};
      const start = facts("start.json");
      const ran = facts("ran.json");
      const levels = run.mirror("levels.json");
      const title = run.mirror("title-es.json");
      const set = run.calls(0).filter((c) => c.method === "store.set" && c.args["key"] === "og.settings");
      const read = run.calls(1).filter((c) => c.method === "store.get" && c.args["key"] === "og.settings");
      const newGame = title.nodes.find((n) => n.path === "new" || n.path.endsWith("/new"));
      return [
        [facts("title.json")["phase"] === "title", "tick 60 is the title (facts.phase)"],
        [levels.nodes.some((n) => n.path.split("/").includes("Levels")), "the press on Level select opened it: its cards in the mirror"],
        [start["phase"] === "play" && start["level"] === 1, `Escape closed the select and New game started the first level (facts ${JSON.stringify({ phase: start["phase"], level: start["level"] })})`],
        [typeof ran["x"] === "number" && typeof start["x"] === "number" && (ran["x"] as number) > (start["x"] as number) + 1, `ArrowRight held moved Wick right (x ${start["x"]} to ${ran["x"]})`],
        [facts("paused.json")["paused"] === true, "Escape in play paused (facts.paused)"],
        [facts("saved.json")["language"] === "es", `the Language tab's Español took (facts.language ${JSON.stringify(facts("saved.json")["language"])})`],
        [set.length >= 1, `og.settings went to the play origin's store (${set.length} store.set)`],
        [read.length === 1, `the reloaded page asked the play origin for og.settings (${read.length} store.get)`],
        [facts("reloaded.json")["language"] === "es" && newGame?.name === "Partida nueva", `and came back in Spanish (facts.language ${JSON.stringify(facts("reloaded.json")["language"])}, New game reads ${JSON.stringify(newGame?.name)})`],
      ];
    },
  },
  "meridian-loop": {
    title: "start",
    // tests/published.og.ts's four frames (pub-title, pub-countdown, pub-straight, pub-pause) are its own
    // steps, run with these: the race's state beside each, then the settings from pause and a reload.
    playtest: "tests/published.og.ts",
    script: [
      "1.2:state:title.json",
      "3.0:state:countdown.json",
      "8.5:state:straight.json",
      "10.0:state:paused.json",
      // The settings from pause: the Language tab, Deutsch, Done.
      "10.2:touch_move_to:settings",
      "10.217:touch_start",
      "10.25:touch_end",
      "10.8:touch_move_to:Tab#3",
      "10.817:touch_start",
      "10.85:touch_end",
      "11.2:touch_move_to:Option#1",
      "11.217:touch_start",
      "11.25:touch_end",
      "11.6:touch_move_to:Done",
      "11.617:touch_start",
      "11.65:touch_end",
      "12.2:state:saved.json",
      "12.3:quit",
    ],
    then: [["1.0:state:reloaded.json", "1.0:mirror:title-de.json", "1.1:quit"]],
    check(run) {
      const facts = (f: string) => run.state(f).facts ?? {};
      const straight = facts("straight.json");
      const title = run.mirror("title-de.json");
      const set = run.calls(0).filter((c) => c.method === "store.set" && c.args["key"] === "og.settings");
      const read = run.calls(1).filter((c) => c.method === "store.get" && c.args["key"] === "og.settings");
      const start = title.nodes.find((n) => n.path === "start" || n.path.endsWith("/start"));
      // Phase of src/components.ts: Title 0, Countdown 1, Racing 2.
      return [
        [facts("title.json")["phase"] === 0, "tick 72 is the title (facts.phase)"],
        [facts("countdown.json")["phase"] === 1, `the press on Time trial started the countdown (facts.phase ${JSON.stringify(facts("countdown.json")["phase"])})`],
        [straight["phase"] === 2 && typeof straight["kmh"] === "number" && (straight["kmh"] as number) > 20, `W held drove the car up the straight (facts.phase ${JSON.stringify(straight["phase"])}, ${straight["kmh"]} km/h)`],
        [facts("paused.json")["paused"] === true, "Escape over the race paused (facts.paused)"],
        [facts("saved.json")["language"] === "de", `the Language tab's Deutsch took (facts.language ${JSON.stringify(facts("saved.json")["language"])})`],
        [set.length >= 1, `og.settings went to the play origin's store (${set.length} store.set)`],
        [read.length === 1, `the reloaded page asked the play origin for og.settings (${read.length} store.get)`],
        [facts("reloaded.json")["language"] === "de" && start?.name === "Zeitfahren", `and came back in German (facts.language ${JSON.stringify(facts("reloaded.json")["language"])}, Time trial reads ${JSON.stringify(start?.name)})`],
      ];
    },
  },
  "ui-kit": {
    title: "UI kit",
    references: "crates/og-ui/references",
    prefix: "kit-desktop-",
    script: [
      // tests/frames-desktop.og.ts's gallery.png is this tick's frame (the gallery is the first screen).
      "1.1:screenshot:gallery.png",
      "1.1:state:gallery.json",
      "1.2:touch_move_to:settings",
      "1.25:touch_start",
      "1.3:touch_end",
      "1.9:state:settings.json",
      "2.0:key_down:Escape",
      "2.05:key_up:Escape",
      "2.5:state:back.json",
      // Settings again: Mute, the Language tab, 简体中文, Done (og.settings through the play origin's store).
      "2.6:touch_move_to:settings",
      "2.65:touch_start",
      "2.7:touch_end",
      "3.2:touch_move_to:Mute",
      "3.25:touch_start",
      "3.3:touch_end",
      "3.5:touch_move_to:Section/Tab#3",
      "3.55:touch_start",
      "3.6:touch_end",
      "3.9:touch_move_to:Languages/Option#1",
      "3.95:touch_start",
      "4.0:touch_end",
      "4.3:touch_move_to:Done",
      "4.35:touch_start",
      "4.4:touch_end",
      "5.0:state:saved.json",
      "5.1:quit",
    ],
    // A reload, as a returning player's: the settings come back from the store (WEB-FINDINGS #10).
    then: [["1.0:state:reloaded.json", "1.1:quit"]],
    check(run) {
      const screen = (f: string) => run.state(f).facts?.["screen"];
      const saved = run.state("saved.json").facts ?? {};
      const reloaded = run.state("reloaded.json").facts ?? {};
      const set = run.calls(0).filter((c) => c.method === "store.set" && c.args["key"] === "og.settings");
      const read = run.calls(1).filter((c) => c.method === "store.get" && c.args["key"] === "og.settings");
      return [
        [screen("gallery.json") === "gallery", "the kit opens on the gallery"],
        [screen("settings.json") === "settings", `the press on Settings opened it (facts.screen ${JSON.stringify(screen("settings.json"))})`],
        [screen("back.json") === "gallery", `Escape closed it (facts.screen ${JSON.stringify(screen("back.json"))})`],
        [saved["screen"] === "gallery" && saved["language"] === "zh-CN" && saved["saves"] === 1, `Mute, 简体中文 and Done saved once (facts ${JSON.stringify({ screen: saved["screen"], language: saved["language"], saves: saved["saves"] })})`],
        [set.length === 1, `og.settings went to the play origin's store (${set.length} store.set)`],
        [read.length === 1, `the reloaded page asked the play origin for og.settings (${read.length} store.get)`],
        [reloaded["language"] === "zh-CN" && reloaded["loads"] === 1, `and came back in 简体中文 (facts ${JSON.stringify({ language: reloaded["language"], loads: reloaded["loads"] })})`],
      ];
    },
  },
};

const given = opt("--script");
/** A playtest's own commands (and its `then` steps' as relaunches), bundled and imported as `og test` does. */
async function readPlaytest(playtestFile: string): Promise<{ script: string[]; then?: string[][] }> {
  const file = resolve(project, playtestFile);
  const bundled = join(work, id, ".published.playtest.js");
  mkdirSync(dirname(bundled), { recursive: true });
  const b = await bundleGame(file, bundled, "native");
  if (!b.ok) throw new Error(`${file} does not bundle: ${b.reason}`);
  const desc = ((await import(pathToFileURL(bundled).href)) as { default: PlaytestDesc }).default;
  if (desc.size && desc.size !== `${WIDTH}x${HEIGHT}`) throw new Error(`${playtestFile} runs at ${desc.size}; the published page here is ${WIDTH}x${HEIGHT}`);
  return { script: [desc.commands], ...(desc.then?.length ? { then: desc.then.map((t) => [t.commands]) } : {}) };
}
/** --playtest <file>: a playtest's own commands, its screenshots against the project's references. */
const playtestFile = opt("--playtest");
const fromPlaytest = playtestFile ? await readPlaytest(playtestFile) : undefined;
const chosen: Plan | undefined = fromPlaytest
  ? {
      title: opt("--title") ?? "",
      ...(opt("--references") ? { references: opt("--references")! } : {}),
      ...(opt("--prefix") ? { prefix: opt("--prefix")! } : {}),
      ...fromPlaytest,
    }
  : given
  ? {
      title: opt("--title") ?? "",
      ...(opt("--references") ? { references: opt("--references")! } : {}),
      ...(opt("--prefix") ? { prefix: opt("--prefix")! } : {}),
      script: [given],
      ...(opt("--then") ? { then: [[opt("--then")!]] } : {}),
    }
  : PLANS[id];
if (!chosen) throw new Error(`no plan for ${id}: pass --script "<steps>" and --title <node> (published.ts lists the steps)`);
// A plan's playtest: its steps (to its `quit`, which the plan's own replaces) run beside the plan's.
const plan: Plan = chosen.playtest
  ? await (async () => {
      const own = await readPlaytest(chosen.playtest!);
      if (own.then) throw new Error(`${chosen.playtest} has relaunches: a plan names its own`);
      return { ...chosen, script: [...own.script.flatMap((s) => s.split(/(?<!\\);/)).filter((s) => s.split(":")[1] !== "quit"), ...chosen.script] };
    })()
  : chosen;

// ── the steps ──────────────────────────────────────────────────────────

interface Step {
  readonly at: number;
  readonly tick: number;
  readonly action: string;
  readonly arg: string;
  readonly text: string;
}
/** A script's steps in time order (`\\;` is a `;` inside a step, as in `og run --commands`). */
const parse = (script: readonly string[]): Step[] =>
  script
    .flatMap((s) => s.split(/(?<!\\);/))
    .filter((s) => s.trim() !== "")
    .map((s) => {
      const [at, action, ...rest] = s.split(":");
      const t = Number(at);
      if (!Number.isFinite(t) || !action) throw new Error(`'${s}': a step is <seconds>:<action>[:<args>]`);
      return { at: t, tick: Math.round(t * TICK_RATE), action, arg: rest.join(":").replace(/\\;/g, ";"), text: s };
    })
    .sort((a, b) => a.at - b.at);
const launches: Step[][] = [parse(plan.script), ...(plan.then ?? []).map(parse)];
const written = launches.flat().filter((s) => ["screenshot", "state", "mirror"].includes(s.action)).map((s) => s.arg);
for (const f of new Set(written)) if (written.indexOf(f) !== written.lastIndexOf(f)) throw new Error(`two steps write ${f}: give each its own file`);
/** The steps the publish build can take: input, captures, the probe's state, the mirror. */
const INPUT = new Set(["key_down", "key_up", "type", "touch_move_to", "move_to_text", "touch_move", "touch_start", "touch_end", "scroll"]);
const TAKEN = new Set([...INPUT, "screenshot", "state", "mirror", "quit"]);

// ── the results ────────────────────────────────────────────────────────

const checks: Array<{ ok: boolean; what: string }> = [];
const check = (ok: boolean, what: string) => {
  checks.push({ ok, what });
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
};
const result: Record<string, unknown> = { project: projectRel, tier, isolated, script: launches.map((l) => l.map((s) => s.text)), checks, shots: {} as Record<string, unknown> };
const save = () => writeFileSync(join(out, "published.json"), JSON.stringify(result, null, 1) + "\n");

for (const s of launches.flat()) if (!TAKEN.has(s.action)) check(false, `'${s.text}' is not input or a capture: the publish build has no command channel (og-build §6), so it cannot run here`);
if (checks.some((c) => !c.ok)) {
  save();
  process.exit(1);
}

// ── 1. publish, as the author does ─────────────────────────────────────
const dist = join(work, id, "publish");
if (!args.includes("--no-build") || !existsSync(join(dist, "og-build.json"))) {
  const r = publish({ og: ogBin(), project, dist, repo, pkg });
  writeFileSync(join(work, id, "og-build.txt"), r.said);
  result["build"] = { exit: r.exit, ms: r.ms };
  if (r.exit !== 0) {
    check(false, `og build --publish failed (exit ${r.exit}): ${r.said.split("\n").slice(-5).join(" / ")}`);
    save();
    process.exit(1);
  }
}
const manifest = JSON.parse(readFileSync(join(dist, "og-build.json"), "utf8")) as { build: string; probe: { members: string[] }; variant: string; lanes: unknown };
result["package"] = { build: manifest.build, variant: manifest.variant, lanes: manifest.lanes !== null, probe: manifest.probe.members };
check(manifest.build === "publish" && !manifest.probe.members.includes("commands"), `the page is the publish build: its probe keeps ${manifest.probe.members.join(", ")}`);

// ── 2. the page, as the platform opens it ─────────────────────────────
const origin = playOrigin({ dist, id, isolated });

/**
 * The engine Worker's display clock (see the top): `__ogClock.advance(n)`
 * runs n display frames, each at one tick's time after the last, with a
 * task between them for the GPU, fetches and messages; it answers how many
 * frames the Worker had asked for. `advance(n, true)` runs them at the same
 * time as the last (no tick passes: the Worker steps by the time between
 * frames), which is how a capture's readback lands with the clock held.
 */
const CLOCK = `(() => {
  const step = 1000 / ${TICK_RATE} + 1e-6;
  let now = performance.now();
  let frames = 0;
  let next = 1;
  const queue = new Map();
  Object.defineProperty(performance, "now", { value: () => now, configurable: true, writable: true });
  self.requestAnimationFrame = (fn) => { const id = next++; queue.set(id, fn); return id; };
  self.cancelAnimationFrame = (id) => { queue.delete(id); };
  self.__ogClock = {
    get frames() { return frames; },
    get waiting() { return queue.size; },
    async advance(n, still) {
      let asked = 0;
      for (let i = 0; i < n; i++) {
        if (!still) now += step;
        frames++;
        const due = [...queue.values()];
        queue.clear();
        if (due.length) asked++;
        for (const fn of due) fn(now);
        await new Promise((r) => setTimeout(r, 0));
      }
      return asked;
    },
  };
  return true;
})()`;

const browser = await Browser.launch({ chrome: chromePath(args), width: WIDTH, height: HEIGHT });
const log: string[] = [];
const workerLog: string[] = [];
const browserLog: string[] = [];
let page: Page | undefined;
let engine: string | undefined;
browser.on((method, params, session) => {
  if (session && page?.workers.has(session) && (method === "Runtime.consoleAPICalled" || method === "Runtime.exceptionThrown")) {
    const w = page.workers.get(session)!;
    const where = w.url.replace(/^.*\/play\/[^/]+\/[^/]+\//, "./").replace(/^blob:.*/, "blob");
    if (method === "Runtime.exceptionThrown") workerLog.push(`${where} exception: ${(params["exceptionDetails"] as { exception?: { description?: string }; text?: string }).exception?.description ?? (params["exceptionDetails"] as { text?: string }).text}`);
    else workerLog.push(`${where} ${params["type"]}: ${((params["args"] as Array<{ value?: unknown; description?: string }>) ?? []).map((x) => (x.value !== undefined ? String(x.value) : (x.description ?? ""))).join(" ")}`);
  } else if (session === page?.session && method === "Log.entryAdded") {
    const e = params["entry"] as { level: string; source: string; text: string; url?: string };
    browserLog.push(`${e.level} ${e.source}: ${e.text}${e.url ? ` (${e.url})` : ""}`);
  }
});

/** The probe's tick: the last tick the engine stepped. */
const tickNow = () => page!.eval<number>("__runtime_probe__.tick()");
/** Runs display frames until the engine has stepped `tick`. */
async function advanceTo(tick: number): Promise<number> {
  for (;;) {
    const now = await tickNow();
    if (now >= tick) return now;
    const asked = await page!.evalWorker<number>(engine!, `__ogClock.advance(${Math.min(tick - now, 30)})`);
    if (asked === 0) throw new Error(`at tick ${now} the engine Worker asks for no frame: it is not playing (the gate did not open?)`);
  }
}
/**
 * The probe's capture at the tick the clock holds: frames run until the
 * readback lands, all at the same time (the frame that answers would
 * otherwise tick, worker.ts `frame`), so the clock is where it was after.
 */
async function capture(): Promise<{ png: string; tick: number; width: number; height: number }> {
  await page!.eval(`(window.__ogShot = null, __runtime_probe__.capture(${WIDTH}, ${HEIGHT}).then((v) => (window.__ogShot = { v }), (e) => (window.__ogShot = { e: String((e && e.message) || e) })), 0)`);
  for (let i = 0; i < 240; i++) {
    const got = await page!.eval<{ v?: { png: string; tick: number; width: number; height: number } | null; e?: string } | null>("window.__ogShot");
    if (got) {
      if (got.e || !got.v?.png) throw new Error(`capture answered ${got.e ?? JSON.stringify(got.v)}`);
      return got.v;
    }
    await page!.evalWorker(engine!, "__ogClock.advance(1, true)");
  }
  throw new Error("capture did not answer within 240 frames");
}
/** Where the canvas is on the page, CSS pixels. */
const canvasBox = () => page!.eval<{ x: number; y: number }>(`(() => { const r = document.querySelector("canvas").getBoundingClientRect(); return { x: r.left, y: r.top }; })()`);
/** A mirrored control's centre, asked again for a moment while the frame's tree reaches the page. */
async function locate(what: string, nth = 0): Promise<{ x: number; y: number; path: string } | null> {
  for (let i = 0; i < 20; i++) {
    const at = await page!.eval<{ x: number; y: number; path: string } | null>(findInMirror(what, nth));
    if (at) return at;
    await wait(25);
  }
  return null;
}
const MODIFIERS: Record<string, [string, number]> = { Shift: ["ShiftLeft", 8], Ctrl: ["ControlLeft", 2], Control: ["ControlLeft", 2], Alt: ["AltLeft", 1], Option: ["AltLeft", 1], Meta: ["MetaLeft", 4], Cmd: ["MetaLeft", 4] };
/** `[Mod+]<code>`: the key and its modifiers' codes and CDP bits. */
function chord(arg: string): { code: string; mods: Array<[string, number]>; bits: number } {
  const parts = arg.split("+");
  const code = parts.pop()!;
  const mods = parts.map((m) => {
    const k = MODIFIERS[m];
    if (!k) throw new Error(`'${m}' is no modifier (Shift, Ctrl, Alt, Meta)`);
    return k;
  });
  return { code, mods, bits: mods.reduce((a, [, b]) => a | b, 0) };
}
/** The code of a typed character's key, when one key types it. */
function codeOf(ch: string): { code: string; shift: boolean } | null {
  if (/^[a-z]$/.test(ch)) return { code: `Key${ch.toUpperCase()}`, shift: false };
  if (/^[A-Z]$/.test(ch)) return { code: `Key${ch}`, shift: true };
  if (/^[0-9]$/.test(ch)) return { code: `Digit${ch}`, shift: false };
  if (ch === " ") return { code: "Space", shift: false };
  return null;
}

const mirrorNow = (tick: number) =>
  page!.eval<Omit<Mirrored, "tick">>(`(() => {
    const nodes = ${MIRROR_TREE};
    const a = document.activeElement;
    let hidden = false;
    for (let e = a; e; e = e.parentElement) if (e.getAttribute && e.getAttribute("aria-hidden") === "true") hidden = true;
    const focused = a && a !== document.body ? { tag: a.tagName, role: a.getAttribute("role"), label: a.getAttribute("aria-label"), hidden } : null;
    return { nodes, focused };
  })()`).then((m) => ({ tick, ...m }));

const files = new Map<string, unknown>();
/** Where each launch's Player calls start in origin.calls. */
const callsFrom: number[] = [];
const run: Run = {
  state: (f) => (files.get(f) as ReturnType<Run["state"]>) ?? { tick: -1, facts: null },
  mirror: (f) => (files.get(f) as Mirrored) ?? { tick: -1, nodes: [], focused: null },
  calls: (n) => origin.calls.slice(callsFrom[n] ?? origin.calls.length, callsFrom[n + 1] ?? origin.calls.length),
};

/**
 * One launch: the page loaded (`load`: the navigation, or a reload), the
 * game booted with the engine Worker on the harness's clock, the title in
 * the mirror behind the gate, and the gate opened by a real click.
 */
async function launch(n: number, load: () => Promise<unknown>): Promise<void> {
  const label = n === 0 ? "" : ` (relaunch ${n})`;
  callsFrom[n] = origin.calls.length;
  engine = undefined;
  const t0 = performance.now();
  await load();
  // The new page's engine Worker (prepareWorkers gave it the clock): never the last page's probe.
  for (const t = performance.now(); !engine && performance.now() - t < 120_000; ) await wait(20);
  await page!.until("(typeof __runtime_probe__ === 'object' && document.readyState !== 'loading') || !document.getElementById('og-error')?.hidden", 120_000);
  const failed = await page!.eval<string | null>("document.getElementById('og-error') && !document.getElementById('og-error').hidden ? document.getElementById('og-error').innerText.slice(0, 2000) : null");
  check(!failed, failed ? `the published page did not start${label}: ${failed.split("\n").slice(0, 3).join(" / ")}` : `the game boots on the published page${label} (${Math.round(performance.now() - t0)} ms to the probe)`);
  if (failed) throw new Error("no start");
  if (n === 0) {
    result["hello"] = await page!.eval("__runtime_probe__.hello()");
    result["lanes"] = await page!.eval("__runtime_probe__.lanes");
  }
  check(!!engine && (await page!.evalWorker<boolean>(engine, "typeof __ogClock === 'object'")), `the engine Worker runs on the harness's clock${label}`);
  await page!.until("document.getElementById('og-gate') && !document.getElementById('og-gate').hidden", 60_000).catch(() => {});
  const title = plan.title;
  const titled = await page!
    .until(`(() => { const s = __dog.seen; return Object.keys(s).some((k) => k === ${JSON.stringify(title)} || k.endsWith("/" + ${JSON.stringify(title)}) || k === "name:" + ${JSON.stringify(title)}); })()`, 30_000)
    .then(() => true, () => false);
  check(titled, `the title's '${title}' is in the accessibility mirror behind the play gate${label} (tick ${await tickNow()})`);
  writeFileSync(join(out, n === 0 ? "gate.png" : `gate-${n}.png`), await page!.screenshot());
  // The gate: a real click in its corner (the game's controls are not there, so nothing is hovered when it opens).
  await page!.click(4, 4);
  const opened = await page!.until("document.getElementById('og-gate')?.hidden", 10_000).then(() => true, () => false);
  check(opened, `a real click opened the play gate${label}`);
  if (!opened) throw new Error("the gate did not open");
}

try {
  page = await browser.open("about:blank", log);
  await page.send("Log.enable");
  await page.send("Page.addScriptToEvaluateOnNewDocument", { source: OBSERVE_MIRROR });
  await page.send("Emulation.setDeviceMetricsOverride", { width: WIDTH, height: HEIGHT, deviceScaleFactor: 1, mobile: false });
  // A headless page keeps its focus as a shown one does (the text entry's focus is part of what is checked).
  await page.send("Emulation.setFocusEmulationEnabled", { enabled: true });
  // Chrome's accessibility tree on, as with a screen reader running (its aria-hidden warnings need it).
  await page.send("Accessibility.enable");
  // The engine Worker (game.js) gets the harness's clock before its first line; every Worker's console is heard.
  await page.prepareWorkers(async (w, session) => {
    await browser.send("Runtime.enable", {}, session).catch(() => {});
    if (/\/game\.js(\?|$)/.test(w.url)) {
      await page!.evalWorker(session, CLOCK);
      engine = session;
    }
  });
  await launch(0, () => page!.send("Page.navigate", { url: origin.url(webgl ? "?webgl=1" : "") }));

  // ── 3. the script, then each relaunch's ────────────────────────────
  const ran: Array<{ launch: number; step: string; tick: number }> = [];
  result["ran"] = ran;
  for (const [n, steps] of launches.entries()) {
    if (n > 0) await launch(n, () => page!.send("Page.reload", { ignoreCache: false }));
    await play(n, steps, ran);
  }
  for (const [ok, what] of plan.check?.(run) ?? []) check(ok, what);
  result["platformCalls"] = origin.calls.map((c) => c.method + (c.args["key"] ? ` ${c.args["key"]}` : ""));
  writeFileSync(join(out, "end.png"), await page.screenshot());
} catch (e) {
  check(false, `the run stopped: ${e instanceof Error ? (e.stack ?? e.message) : String(e)}`);
} finally {
  // Chrome asks the play origin's root for a favicon the build never names: not the build's.
  const noise = (l: string) => /favicon\.ico/.test(l);
  const bad = [
    ...log.filter((l) => /^(warn|warning|error|exception|assert)/.test(l)).map((l) => `page ${l}`),
    ...workerLog.filter((l) => / (warn|warning|error|assert)|exception/.test(l)),
    ...browserLog.filter((l) => /^(warning|error)/.test(l)).map((l) => `browser ${l}`),
  ].filter((l) => !noise(l));
  result["console"] = bad;
  check(bad.length === 0, bad.length === 0 ? "nothing on the console: no warning or error from the page, its Workers or Chrome" : `the console has ${bad.length} warning or error line(s): ${[...new Set(bad)].slice(0, 6).join(" | ")}`);
  writeFileSync(join(out, "console.txt"), [...log.map((l) => `page ${l}`), ...workerLog, ...browserLog.map((l) => `browser ${l}`)].join("\n") + "\n");
  await browser.close();
  origin.stop();
  save();
}
const failures = checks.filter((c) => !c.ok).length;
console.log(`${failures === 0 ? "PASS" : "FAIL"} ${projectRel} published (${tier}): ${checks.length - failures} of ${checks.length} checks; ${join(out, "published.json")}`);
process.exit(failures === 0 ? 0 : 1);

/** One launch's steps, each at its tick. */
async function play(n: number, steps: readonly Step[], ran: Array<{ launch: number; step: string; tick: number }>): Promise<void> {
  const tab = page!;
  let pointer = { x: WIDTH / 2, y: HEIGHT / 2 };
  for (const s of steps) {
    if (s.action === "quit") break;
    if (INPUT.has(s.action)) {
      // Probe::pump delivers a command at world.time().tick == T, before
      // the next world step, not before the step that reaches T.
      const at = await advanceTo(s.tick);
      ran.push({ launch: n, step: s.text, tick: at });
      switch (s.action) {
        case "key_down":
        case "key_up": {
          const k = chord(s.arg);
          if (s.action === "key_down") for (const [code] of k.mods) await keyEvent(tab, "keyDown", code);
          await keyEvent(tab, s.action === "key_down" ? "keyDown" : "keyUp", k.code, { modifiers: k.bits });
          if (s.action === "key_up") for (const [code] of [...k.mods].reverse()) await keyEvent(tab, "keyUp", code);
          break;
        }
        case "type":
          for (const ch of s.arg) {
            const key = codeOf(ch);
            if (key) {
              if (key.shift) await keyEvent(tab, "keyDown", "ShiftLeft", { modifiers: 8 });
              await keyEvent(tab, "keyDown", key.code, { text: ch, modifiers: key.shift ? 8 : 0 });
              await keyEvent(tab, "keyUp", key.code, { modifiers: key.shift ? 8 : 0 });
              if (key.shift) await keyEvent(tab, "keyUp", "ShiftLeft");
            } else await tab.send("Input.insertText", { text: ch });
            // Two ticks a character, as the native `type` takes them.
            await tab.evalWorker(engine!, "__ogClock.advance(2)");
          }
          break;
        case "touch_move_to":
        case "move_to_text": {
          const exact = s.action === "touch_move_to" || s.arg.startsWith("=");
          const [text, nth] = s.action === "move_to_text" ? (s.arg.replace(/^=/, "").match(/^(.*?)(?:@(-?\d+))?$/)!.slice(1) as [string, string | undefined]) : [s.arg, undefined];
          const found = await locate(exact ? text : `name~${text}`, Number(nth ?? 0));
          if (!found) throw new Error(`${s.text}: nothing in the accessibility mirror is '${text}' (it holds: ${(await tab.eval<Array<{ path: string; name: string }>>(MIRROR_TREE)).map((n) => `${n.path}=${n.name}`).join(", ").slice(0, 600)})`);
          pointer = { x: found.x, y: found.y };
          await tab.send("Input.dispatchMouseEvent", { type: "mouseMoved", ...pointer });
          break;
        }
        case "touch_move": {
          const [x, y] = s.arg.split(",").map(Number) as [number, number];
          const c = await canvasBox();
          pointer = { x: c.x + x, y: c.y + y };
          await tab.send("Input.dispatchMouseEvent", { type: "mouseMoved", ...pointer });
          break;
        }
        case "touch_start":
        case "touch_end":
          await tab.send("Input.dispatchMouseEvent", { type: s.action === "touch_start" ? "mousePressed" : "mouseReleased", ...pointer, button: "left", clickCount: 1 });
          break;
        case "scroll": {
          const [dx, dy] = s.arg.split(",").map(Number) as [number, number];
          await tab.send("Input.dispatchMouseEvent", { type: "mouseWheel", ...pointer, deltaX: dx, deltaY: dy });
          break;
        }
      }
      continue;
    }
    const at = await advanceTo(s.tick);
    if (at !== s.tick) check(false, `${s.text}: the engine was at tick ${at} when the step came (its tick is ${s.tick}): an earlier step ran late`);
    if (s.action === "state") {
      const st = await tab.eval<{ tick: number; facts: unknown; timings: unknown }>("__runtime_probe__.state()");
      files.set(s.arg, st);
      writeFileSync(join(out, s.arg), JSON.stringify(st, null, 1) + "\n");
    } else if (s.action === "mirror") {
      const m = await mirrorNow(at);
      files.set(s.arg, m);
      writeFileSync(join(out, s.arg), JSON.stringify(m, null, 1) + "\n");
    } else if (s.action === "screenshot") {
      const shot = await capture();
      const after = await tickNow();
      if (after !== s.tick) check(false, `${s.text}: the clock moved to tick ${after} while the capture waited`);
      const bytes = Uint8Array.from(atob(shot.png), (c) => c.charCodeAt(0));
      writeFileSync(join(out, s.arg), bytes);
      const refs = join(repo, plan.references ?? relative(repo, join(project, "references")));
      const name = `${plan.prefix ?? ""}${s.arg}`;
      const against = existsSync(join(refs, tier, name)) ? join(refs, tier, name) : join(refs, "metal", name);
      const row: Record<string, unknown> = { tick: shot.tick, size: `${shot.width}x${shot.height}`, against: relative(repo, against) };
      (result["shots"] as Record<string, unknown>)[s.arg] = row;
      if (!existsSync(against)) {
        check(false, `${s.arg}: no reference at ${relative(repo, against)}`);
        continue;
      }
      const d = compareFrames(decodePng(bytes), decodePng(new Uint8Array(readFileSync(against))));
      Object.assign(row, { outliers: +(d.outliers * 100).toFixed(3), rmse: +d.rmse.toFixed(5), ok: d.ok });
      if (!d.ok) writeFileSync(join(out, `${s.arg}.diff.rgba`), d.diff);
      check(d.ok && shot.tick === s.tick, `${s.arg} (tick ${shot.tick}, ${tier}): ${(d.outliers * 100).toFixed(3)} % outliers (≤ ${TOLERANCE.outliers * 100} %), RMSE ${d.rmse.toFixed(5)} (≤ ${TOLERANCE.rmse}) against ${relative(repo, against)}`);
    }
  }
}
