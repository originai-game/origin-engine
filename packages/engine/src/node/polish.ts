// `og polish`: a project's playtests played headless, each frame sampled,
// and the mechanical part of og-taste and og-feel scored from what was
// seen and heard, into a markdown report with the frames, the numbers and
// the fix for each finding (the skill section that says how).
//
//   bun src/node/polish.ts --entry <game bundle> --root <project> --out <dir>
//       [--warnings <cook warnings.json>] [--frames N (in all, default 8)] [--json] <tests/x.og.ts>...
//   bun src/node/polish.ts --one <tests/x.og.ts> --entry <bundle> --root <project> --out <dir>
//
// Each playtest runs in its own process (`--one`), on the native host as
// `og run --headless --commands` runs it, with three additions to its
// script: `dump_ui` beside every `screenshot` (the UI tree of the frame),
// a `dump_pulse` marker on every input step (the tick it landed) and a
// `dump_pulse` at the start of every tick (crates/og-runtime/src/polish.rs: time
// scale, hit stop, trauma, floating texts, tweens, particles, voices by
// bus, named entities, UI bounds, step and render times). The run's audio
// is captured to a WAV. The playtest's checks do not run: `og test` does
// that. What is measured, and the thresholds, are in og-taste §11 and
// og-feel §14.

import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { bundleGame } from "./build.ts";
import { loadNativeHost, native } from "./index.ts";
import { openAudio } from "./audio.ts";
import { nativePlatform } from "./platform.ts";
import { decodePng, encodePng, half, type Image } from "./png.ts";
import { createEngine, type Engine, type GameToken } from "../runtime.ts";
import type { PlaytestDesc } from "../playtest.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const argv = process.argv.slice(2);
const opt = (name: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1] : undefined;
};
const here = dirname(fileURLToPath(import.meta.url));
const VALUED = new Set(["--one", "--entry", "--root", "--out", "--warnings", "--frames"]);
const positional = argv.filter((a, i) => !a.startsWith("--") && !VALUED.has(argv[i - 1] ?? ""));

function fail(reason: string): never {
  process.stderr.write(`og polish: ${reason}\n`);
  process.exit(2);
}

// ── the run of one playtest ────────────────────────────────────────────────

/** One sample, as crates/og-runtime/src/polish.rs writes it. */
export interface Pulse {
  tick: number;
  scale: number;
  hitStop: number;
  trauma: number;
  punch: number;
  kick: number;
  floating: number;
  spawned: number;
  texts: string[];
  ghosts: number;
  tweens: number;
  particles: number[];
  voices: Array<[number, number, number]>;
  /** Name, position, scale x and y, flipbook clip × 1000 + frame (-1: none). */
  named: Array<[string, number, number, number, number, number, number]>;
  ui: Array<[string, number, number, number, number, number]>;
  stepMs: number;
  renderMs: number;
}

interface UiDump {
  tick: number;
  /** The surface the CSS-pixel bounds are on (dpr: device pixels per CSS pixel). */
  surface?: { width: number; height: number; dpr: number } | null;
  elements: Array<{ id: string; parent: string; kind?: string; disabled?: boolean; clipped?: boolean; text: string; x: number; y: number; width: number; height: number }>;
}

/** What `--one` writes as run.json. */
interface RunRecord {
  playtest: string;
  title: string;
  size: { width: number; height: number };
  tier: string;
  rate: number;
  ticks: number;
  quit: boolean;
  samples: Pulse[];
  /** Facts by tick, where they changed. */
  facts: Array<{ tick: number; facts: unknown }>;
  inputs: Array<{ step: string; action: string; tick: number }>;
  shots: Array<{ name: string; file: string; tick: number; ui: string | null }>;
  wav: string;
  warnings: string[];
}

const INPUT = /^(key_down|touch_start|touch|gamepad|type|scroll)$/;

function projectProfile(dir: string): "2d" | "3d" | undefined {
  if (!existsSync(join(dir, "project.ogt"))) return undefined;
  const m = /^\s*profile\s+"(2d|3d)"/m.exec(readFileSync(join(dir, "project.ogt"), "utf8"));
  return m ? (m[1] as "2d" | "3d") : undefined;
}

async function runOne(file: string, entry: string, root: string, out: string): Promise<void> {
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
  const bundled = join(out, ".playtest.js");
  const b = await bundleGame(file, bundled, "native");
  if (!b.ok) fail(`${file} does not bundle: ${b.reason}`);
  const desc = ((await import(pathToFileURL(bundled).href)) as { default?: PlaytestDesc }).default;
  if (!desc || typeof desc.commands !== "string") fail(`${file} has no default export: export default playtest({ name, commands, check })`);
  const m = /^(\d+)x(\d+)$/.exec(desc!.size ?? "1280x720");
  const size = { width: Number(m?.[1] ?? 1280), height: Number(m?.[2] ?? 720) };

  // The script, with a dump_ui beside each screenshot and a marker on each input.
  const steps = desc!.commands.split(";").map((s) => s.trim()).filter(Boolean);
  const script: string[] = [];
  const inputSteps: Array<{ step: string; action: string; marker: string }> = [];
  const shotSteps: Array<{ name: string; ui: string }> = [];
  for (const step of steps) {
    script.push(step);
    const [t, action = "", ...args] = step.split(":");
    if (action === "screenshot") {
      const name = (args[0] && !/^(full|\d+)$/.test(args[0]) ? args[0] : `shot-${shotSteps.length}.png`).replace(/\.png$/, "");
      const ui = `polish-ui-${shotSteps.length}.json`;
      script.push(`${t}:dump_ui:${ui}`);
      shotSteps.push({ name, ui });
    } else if (action === "touch_end" || (action === "touch" && args[1] === "up")) {
      // A release: a UI button answers it (activation on release), so a press is also measured from it.
      const marker = `polish-input-${inputSteps.length}.json`;
      script.push(`${t}:dump_pulse:${marker}`);
      inputSteps.push({ step, action: "release", marker });
    } else if (INPUT.test(action) && !(action === "gamepad" && args[0] === "off") && !(action === "touch" && args[1] !== "down")) {
      const marker = `polish-input-${inputSteps.length}.json`;
      script.push(`${t}:dump_pulse:${marker}`);
      inputSteps.push({ step, action, marker });
    }
  }

  loadNativeHost();
  const extras = native();
  const game = ((await import(pathToFileURL(resolve(entry)).href)) as { default?: GameToken }).default;
  if (!game) fail(`${entry} has no default export`);
  const profile = projectProfile(root);
  // GameDesc.start runs once the renderer is attached (as og run and the web Worker do).
  const created = createEngine(game!, { root, deferStart: true, ...(profile ? { profile } : {}), ...(desc!.seed === undefined ? {} : { seed: desc!.seed }) });
  if (!created.ok) fail(`${entry}: ${created.reason}`);
  const engine: Engine = created.value;
  engine.attachPlatform(nativePlatform());
  const opened = openAudio(engine.id, "capture");
  if (!opened.ok) fail(`audio: ${opened.reason}`);
  const sink = opened.value;
  const attached = JSON.parse(extras.og_node_attach_offscreen(engine.id, size.width, size.height)) as { ok: boolean; value?: string; reason?: string };
  const tier = attached.ok ? attached.value! : "none";
  const started = engine.advance(0);
  if (!started.ok) fail(started.reason);

  const warnings: string[] = [];
  const markers = new Map<string, number>();
  const uiDumps = new Map<string, string>();
  const shotsByName = new Map<string, { file: string; tick: number }>();
  const samples: Pulse[] = [];
  const drain = () => {
    for (const o of engine.takeOutputs()) {
      if (o.action === "dump_pulse") {
        if (o.file.startsWith("polish-input-")) markers.set(o.file, (JSON.parse(o.data) as Pulse).tick);
        else samples.push(JSON.parse(o.data) as Pulse);
      } else if (o.action === "dump_ui" && o.file.startsWith("polish-ui-")) {
        writeFileSync(join(out, o.file), o.data);
        uiDumps.set(o.file, o.file);
      } else if (o.kind === "png") {
        const path = join(out, "frames", basename(o.file));
        mkdirSync(dirname(path), { recursive: true });
        writeFileSync(path, Uint8Array.from(atob(o.data), (c) => c.charCodeAt(0)));
        shotsByName.set(basename(o.file).replace(/\.png$/, ""), { file: path, tick: o.tick });
      }
    }
    for (const l of engine.takeLog()) if (l.warning) warnings.push(`tick ${l.tick}: ${l.text}`);
  };
  drain();
  // A dump_pulse at the start of every tick through the script's last step
  // (the state the previous tick left), each its own output.
  const rate = engine.game.tick;
  const last = Math.max(0, ...steps.map((st) => Number(st.split(":")[0]) || 0));
  // Steps ascend in time: the pulses go in order, before a step of the same tick.
  const timed = script.map((st, i) => ({ at: Number(st.split(":")[0]) || 0, order: 1, i, st }));
  for (let k = 1; k <= Math.ceil(last * rate) + 1; k++) timed.push({ at: Number((k / rate).toFixed(6)), order: 0, i: k, st: `${(k / rate).toFixed(6)}:dump_pulse` });
  timed.sort((a, b) => a.at - b.at || a.order - b.order || a.i - b.i);
  const report = engine.probe.commands(timed.map((x) => x.st).join(";")) as { untilTick: number } | null;
  drain();
  if (!report) fail(`${file}: the script is malformed`);
  const facts: Array<{ tick: number; facts: unknown }> = [];
  let lastFacts = "";
  let quit = false;
  for (;;) {
    const answer = engine.frame(1000 / rate);
    if (!answer.ok) fail(`${file}: ${answer.reason ?? "the frame failed"}`);
    sink.pump();
    drain();
    const state = engine.probe.state() as { tick: number; facts: unknown };
    const f = JSON.stringify(state.facts ?? null);
    if (f !== lastFacts) {
      facts.push({ tick: state.tick, facts: state.facts ?? null });
      lastFacts = f;
    }
    if (answer.quit) {
      quit = true;
      break;
    }
    if ((answer.tick ?? 0) > report!.untilTick) break;
  }
  drain();
  const wav = join(out, "audio.wav");
  writeFileSync(wav, sink.wav());
  sink.close();
  const record: RunRecord = {
    playtest: basename(file).replace(/\.og\.ts$/, ""),
    title: desc!.name,
    size,
    tier,
    rate,
    ticks: samples.at(-1)?.tick ?? 0,
    quit,
    samples,
    facts,
    inputs: inputSteps.map((s) => ({ step: s.step, action: s.action, tick: markers.get(s.marker) ?? -1 })),
    shots: shotSteps.map((s) => {
      const shot = shotsByName.get(s.name);
      return { name: s.name, file: shot?.file ?? "", tick: shot?.tick ?? -1, ui: uiDumps.has(s.ui) ? join(out, s.ui) : null };
    }),
    wav,
    warnings,
  };
  writeFileSync(join(out, "run.json"), JSON.stringify(record));
  process.exit(0);
}

// ── measures ───────────────────────────────────────────────────────────────

const BUS = { master: 0, music: 1, sfx: 2, ui: 3 };

/** Relative luminance (WCAG) of an sRGB byte triple. */
function luminance(r: number, g: number, b: number): number {
  const c = (v: number) => {
    const s = v / 255;
    return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * c(r) + 0.7152 * c(g) + 0.0722 * c(b);
}

const median = (xs: number[]) => {
  if (xs.length === 0) return 0;
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.floor(s.length / 2)]!;
};
const pct = (xs: number[], p: number) => {
  if (xs.length === 0) return 0;
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.min(s.length - 1, Math.floor(p * s.length))]!;
};
const r2 = (x: number) => Math.round(x * 100) / 100;
/** 100 at or under `good`, 0 at or over `bad`, linear between. */
const band = (x: number, good: number, bad: number) => Math.round(100 * Math.max(0, Math.min(1, (bad - x) / (bad - good))));

interface TextBox {
  id: string;
  text: string;
  contrast: number;
  /** The tallest run of ink rows, px: a line's ascenders to its descenders. */
  ink: number;
}

/** Kinds that show a picture, whose `text` is the picture's name (`heart`, `coin`), not words. */
const PICTURES = new Set(["icon", "image"]);

/**
 * Whether an element's text is read as text: not a picture (og-ui reports
 * an icon's name as its text), not cut by a scroll container (a sliver of a
 * line is no measure of it) and not a disabled control or inside one (WCAG
 * exempts inactive controls from contrast). The dump names each element's
 * `kind`, `clipped` and `disabled`.
 */
function readsAsText(e: UiDump["elements"][number], byId: Map<string, UiDump["elements"][number]>): boolean {
  if (PICTURES.has(e.kind ?? "") || e.clipped) return false;
  for (let at: UiDump["elements"][number] | undefined = e, n = 0; at && n < 64; at = at.parent ? byId.get(at.parent) : undefined, n++) {
    if (at.disabled) return false;
  }
  return true;
}

/**
 * The text elements of a frame: the contrast between ink and ground, and the
 * height of the ink's lines. Inside the element's rectangle (inset from its
 * rounded ends), the ground is the median luminance and the ink the 1 % of
 * pixels farthest from it; a rectangle with no ink is not drawn (covered by
 * a transition, faded out) and is left out.
 */
function textBoxes(img: Image, ui: UiDump): TextBox[] {
  const out: TextBox[] = [];
  const byId = new Map(ui.elements.map((e) => [e.id, e]));
  for (const e of ui.elements) {
    if (!/[\p{L}\p{N}]/u.test(e.text) || !readsAsText(e, byId) || e.width < 6 || e.height < 6) continue;
    // A pill's rounded ends show what is behind it: inset a wide element's ends.
    const ix = e.width > 3 * e.height ? Math.min(e.height / 2, 12) : 0, iy = e.height * 0.12;
    const x0 = Math.max(0, Math.floor(e.x + ix)), y0 = Math.max(0, Math.floor(e.y + iy));
    const x1 = Math.min(img.width, Math.ceil(e.x + e.width - ix)), y1 = Math.min(img.height, Math.ceil(e.y + e.height - iy));
    if (x1 - x0 < 3 || y1 - y0 < 3) continue;
    const w = x1 - x0;
    const lum: number[] = [];
    for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) {
      const i = (y * img.width + x) * 4;
      lum.push(luminance(img.rgba[i]!, img.rgba[i + 1]!, img.rgba[i + 2]!));
    }
    const ground = median(lum);
    const dev = lum.map((v) => Math.abs(v - ground));
    const far = pct(dev, 0.99);
    if (far < 0.01) continue;
    // The ink: the 1 % farthest, on the side most of them are.
    let up = 0, down = 0, upSum = 0, downSum = 0;
    for (let k = 0; k < lum.length; k++) {
      if (dev[k]! < far) continue;
      if (lum[k]! > ground) {
        up++;
        upSum += lum[k]!;
      } else {
        down++;
        downSum += lum[k]!;
      }
    }
    const ink = up >= down ? upSum / Math.max(1, up) : downSum / Math.max(1, down);
    const contrast = (Math.max(ink, ground) + 0.05) / (Math.min(ink, ground) + 0.05);
    const rows: boolean[] = [];
    for (let y = 0; y < y1 - y0; y++) {
      let any = false;
      for (let x = 0; x < w && !any; x++) any = dev[y * w + x]! >= far / 4 && (lum[y * w + x]! > ground) === (up >= down);
      rows.push(any);
    }
    const runs: number[] = [];
    let run = 0;
    // A run of ink rows, bridging one-row gaps (thin strokes between a glyph's bars).
    for (let y = 0; y <= rows.length; y++) {
      if (rows[y] || (rows[y - 1] && rows[y + 1])) run++;
      else if (run > 0) {
        runs.push(run);
        run = 0;
      }
    }
    out.push({ id: e.id, text: e.text, contrast: r2(contrast), ink: Math.max(0, ...runs.filter((h) => h >= 2)) });
  }
  return out;
}

interface Occupancy {
  /** Leaf elements' union over the surface. */
  share: number;
  /** Of the central third (the middle ninth), how much leaves cover. */
  centre: number;
  /** A menu or modal: three or more text leaves in the central third. */
  menu: boolean;
}

function occupancy(ui: UiDump, size: { width: number; height: number }): Occupancy {
  const parents = new Set(ui.elements.map((e) => e.parent));
  const leaves = ui.elements.filter((e) => !parents.has(e.id) && e.width * e.height < 0.9 * size.width * size.height && e.width > 0 && e.height > 0);
  const cell = 8;
  const cw = Math.ceil(size.width / cell), ch = Math.ceil(size.height / cell);
  const grid = new Uint8Array(cw * ch);
  for (const e of leaves) {
    for (let y = Math.max(0, Math.floor(e.y / cell)); y < Math.min(ch, Math.ceil((e.y + e.height) / cell)); y++)
      for (let x = Math.max(0, Math.floor(e.x / cell)); x < Math.min(cw, Math.ceil((e.x + e.width) / cell)); x++) grid[y * cw + x] = 1;
  }
  let all = 0, centre = 0, centreCells = 0;
  for (let y = 0; y < ch; y++) for (let x = 0; x < cw; x++) {
    const inCentre = x >= cw / 3 && x < (2 * cw) / 3 && y >= ch / 3 && y < (2 * ch) / 3;
    if (inCentre) centreCells++;
    if (grid[y * cw + x]) {
      all++;
      if (inCentre) centre++;
    }
  }
  const inCentre = (e: UiDump["elements"][number]) => {
    const cx = e.x + e.width / 2, cy = e.y + e.height / 2;
    return cx > size.width / 3 && cx < (2 * size.width) / 3 && cy > size.height / 3 && cy < (2 * size.height) / 3;
  };
  return { share: r2((100 * all) / (cw * ch)), centre: r2((100 * centre) / Math.max(1, centreCells)), menu: leaves.filter((e) => e.text.trim() && inCentre(e)).length >= 3 };
}

/** Numeric leaves of the facts by path. */
function leaves(v: unknown, path = "", out = new Map<string, number>()): Map<string, number> {
  if (typeof v === "number" && Number.isFinite(v)) out.set(path, v);
  else if (Array.isArray(v)) v.forEach((x, i) => leaves(x, `${path}[${i}]`, out));
  else if (v && typeof v === "object") for (const [k, x] of Object.entries(v)) leaves(x, path ? `${path}.${k}` : k, out);
  return out;
}

const HEALTH = /(^|[.\]])(hp|health|life|lives|shield)$/i;
const HITS = /(^|[.\]])(damage|damaged|hits|kills|crits?|dealt|taken)$/i;

interface Signals {
  /** A named entity accelerated or changed scale. */
  moved: boolean;
  /** A named entity's flipbook changed clip. */
  clip: boolean;
  /** Anything moved at all, or a flipbook advanced a frame (activity, not an answer). */
  moving: boolean;
  ui: boolean;
  uiMoved: boolean;
  uiScreen: boolean;
  sound: boolean;
  sfx: boolean;
  uiSound: boolean;
  tween: boolean;
  floating: boolean;
  ghosts: boolean;
  shake: boolean;
  hitStop: boolean;
  particles: boolean;
  /** Anything a player would see or hear change. */
  any: boolean;
}

function signals(s: Pulse[], i: number): Signals {
  const a = s[i]!, p = s[i - 1], pp = s[i - 2];
  const none: Signals = { moved: false, clip: false, moving: false, ui: false, uiMoved: false, uiScreen: false, sound: false, sfx: false, uiSound: false, tween: false, floating: false, ghosts: false, shake: false, hitStop: false, particles: false, any: false };
  if (!p) return none;
  const pos = (x: Pulse | undefined) => new Map((x?.named ?? []).map((n) => [n[0], n]));
  const P = pos(p), PP = pos(pp);
  let moved = false, clip = false, moving = false;
  for (const n of a.named) {
    const q = P.get(n[0]);
    if (!q) continue;
    const r = PP.get(n[0]) ?? q;
    const accel = Math.hypot(n[1] - 2 * q[1] + r[1], n[2] - 2 * q[2] + r[2], n[3] - 2 * q[3] + r[3]);
    if (accel > 1e-3 || Math.abs(n[4] - q[4]) > 1e-3 || Math.abs(n[5] - q[5]) > 1e-3) moved = true;
    if (Math.floor(n[6] / 1000) !== Math.floor(q[6] / 1000)) clip = true;
    if (Math.hypot(n[1] - q[1], n[2] - q[2], n[3] - q[3]) > 1e-4 || n[6] !== q[6]) moving = true;
  }
  const ids = (x: Pulse) => new Set(x.ui.map((e) => e[0]));
  const A = ids(a), B = ids(p);
  let changedIds = 0;
  for (const id of A) if (!B.has(id)) changedIds++;
  for (const id of B) if (!A.has(id)) changedIds++;
  const prevUi = new Map(p.ui.map((e) => [e[0], e]));
  let uiMoved = false, uiText = false;
  for (const e of a.ui) {
    const q = prevUi.get(e[0]);
    if (!q) continue;
    if (Math.abs(e[1] - q[1]) + Math.abs(e[2] - q[2]) + Math.abs(e[3] - q[3]) + Math.abs(e[4] - q[4]) > 0.05) uiMoved = true;
    if (e[5] !== q[5]) uiText = true;
  }
  const prevVoices = new Set(p.voices.map((v) => v[0]));
  const started = a.voices.filter((v) => !prevVoices.has(v[0]));
  const prevParticles = new Set(p.particles);
  const out: Signals = {
    moved,
    clip,
    moving,
    ui: changedIds > 0 || uiMoved || uiText,
    uiMoved,
    uiScreen: changedIds >= 3,
    sound: started.length > 0,
    sfx: started.some((v) => v[1] === BUS.sfx || v[1] === BUS.master),
    uiSound: started.some((v) => v[1] === BUS.ui),
    tween: a.tweens > p.tweens,
    floating: a.spawned > p.spawned,
    ghosts: a.ghosts > p.ghosts,
    shake: a.trauma > p.trauma + 1e-4 || a.punch > p.punch + 1e-4 || a.kick > p.kick + 1e-4,
    hitStop: a.hitStop > p.hitStop || (a.scale < 1 && p.scale >= 1),
    particles: a.particles.some((id) => !prevParticles.has(id)),
    any: false,
  };
  out.any = out.moved || out.clip || out.ui || out.sound || out.tween || out.floating || out.ghosts || out.shake || out.hitStop || out.particles;
  return out;
}

/** The WAV's level, clipping and silence, per tick of `rate`. */
function audioStats(path: string, rate: number) {
  const bytes = readFileSync(path);
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (bytes.length < 44) return { seconds: 0, peak: 0, rms: 0, clipped: 0, silentShare: 100 };
  const sampleRate = view.getUint32(24, true);
  const channels = view.getUint16(22, true);
  const n = (bytes.length - 44) >> 1;
  const perTick = Math.max(1, Math.round((sampleRate / rate) * channels));
  let peak = 0, sum = 0, clipped = 0, silent = 0, ticks = 0, acc = 0, k = 0;
  for (let i = 0; i < n; i++) {
    const x = view.getInt16(44 + i * 2, true) / 32767;
    const a = Math.abs(x);
    peak = Math.max(peak, a);
    sum += x * x;
    acc += x * x;
    if (a >= 0.999) clipped++;
    if (++k === perTick) {
      if (Math.sqrt(acc / k) < 1e-4) silent++;
      ticks++;
      acc = 0;
      k = 0;
    }
  }
  return { seconds: r2(n / channels / sampleRate), peak: r2(peak), rms: Math.round(Math.sqrt(sum / Math.max(1, n)) * 1e4) / 1e4, clipped, silentShare: r2((100 * silent) / Math.max(1, ticks)) };
}

interface Finding {
  measure: string;
  where: string;
  detail: string;
  fix: string;
}

interface PlaytestMeasures {
  playtest: string;
  title: string;
  tier: string;
  ticks: number;
  shots: Array<{ name: string; tick: number; frame: string | null; menu: boolean; hud: number; centre: number; texts: number; lowContrast: number; small: number }>;
  contrast: { texts: number; passing: number; worst: Array<{ shot: string; id: string; text: string; ratio: number }> };
  textSize: { texts: number; passing: number; smallest: Array<{ shot: string; id: string; text: string; ink: number }> };
  hud: { gameplayShots: number; worstShare: number; worstCentre: number };
  ui: { screenChanges: number; withMotion: number; presses: number; pressesWithSound: number; motionTicks: number };
  latency: { inputs: number; answered: number; medianTicks: number; maxTicks: number; baselineShare: number; sound: number };
  damage: { moments: number; hitStop: number; shake: number; floating: number; sound: number; flash: number; particles: number; facts: string[] };
  idle: { share: number };
  particles: { peakSystems: number; bursts: number };
  frame: { tier: string; stepMedianMs: number; stepP95Ms: number; renderMedianMs: number; renderP95Ms: number; frameP95Ms: number; frameMaxMs: number };
  audio: ReturnType<typeof audioStats>;
  warnings: string[];
}

function measure(rec: RunRecord, frameDir: string, frames: number, relTo: string): { m: PlaytestMeasures; findings: Finding[] } {
  const s = rec.samples;
  const sig = s.map((_, i) => signals(s, i));
  const tickIndex = new Map(s.map((p, i) => [p.tick, i]));
  const findings: Finding[] = [];

  // Frames: contrast, text size, HUD.
  const shots: PlaytestMeasures["shots"] = [];
  const allTexts: Array<TextBox & { shot: string }> = [];
  let gameplayShots = 0, worstShare = 0, worstCentre = 0;
  let written = 0;
  for (const shot of rec.shots) {
    if (!shot.file || !existsSync(shot.file)) continue;
    const img = decodePng(new Uint8Array(readFileSync(shot.file)));
    const ui: UiDump = shot.ui && existsSync(shot.ui) ? (JSON.parse(readFileSync(shot.ui, "utf8")) as UiDump) : { tick: shot.tick, elements: [] };
    // The bounds are CSS pixels of the UI's surface; the frame is device pixels, maybe halved.
    const scale = img.width / (ui.surface?.width ?? rec.size.width);
    const scaled: UiDump = { ...ui, elements: ui.elements.map((e) => ({ ...e, x: e.x * scale, y: e.y * scale, width: e.width * scale, height: e.height * scale })) };
    const boxes = textBoxes(img, scaled).map((b) => ({ ...b, ink: r2(b.ink / scale) }));
    const occ = occupancy(ui, ui.surface ? { width: ui.surface.width, height: ui.surface.height } : rec.size);
    let frame: string | null = null;
    if (written < frames) {
      mkdirSync(frameDir, { recursive: true });
      const path = join(frameDir, `${rec.playtest}-${shot.name}.png`);
      writeFileSync(path, encodePng(img.width > 800 ? half(img) : img));
      frame = relative(relTo, path);
      written++;
    }
    for (const b of boxes) allTexts.push({ ...b, shot: shot.name });
    if (!occ.menu && ui.elements.length > 0) {
      gameplayShots++;
      worstShare = Math.max(worstShare, occ.share);
      worstCentre = Math.max(worstCentre, occ.centre);
    }
    shots.push({ name: shot.name, tick: shot.tick, frame, menu: occ.menu, hud: occ.share, centre: occ.centre, texts: boxes.length, lowContrast: boxes.filter((b) => b.contrast < 4.5).length, small: boxes.filter((b) => b.ink > 0 && b.ink < 9).length });
  }
  const contrastPass = allTexts.filter((b) => b.contrast >= 4.5);
  const worstContrast = [...allTexts].sort((a, b) => a.contrast - b.contrast).slice(0, 3).map((b) => ({ shot: b.shot, id: b.id, text: b.text.slice(0, 40), ratio: b.contrast }));
  for (const w of worstContrast.filter((w) => w.ratio < 4.5)) findings.push({ measure: "contrast", where: `${rec.playtest} ${w.shot}: \`${w.id}\` "${w.text}"`, detail: `${w.ratio}:1 between ink and ground (body text wants 4.5:1)`, fix: "og-taste §3 (value before hue): darken the ground or lighten the ink in the skin's palette, never per call" });
  const sized = allTexts.filter((b) => b.ink > 0);
  // Ink lines of at least 9 px are a 13 px face (x-height and caps to descenders).
  const sizePass = sized.filter((b) => b.ink >= 9);
  const smallest = [...sized].sort((a, b) => a.ink - b.ink).slice(0, 3).map((b) => ({ shot: b.shot, id: b.id, text: b.text.slice(0, 40), ink: b.ink }));
  for (const w of smallest.filter((w) => w.ink < 9)) findings.push({ measure: "text size", where: `${rec.playtest} ${w.shot}: \`${w.id}\` "${w.text}"`, detail: `ink lines ${w.ink} CSS px tall (a 13 px face's capitals and digits are about 9)`, fix: "og-taste §4 and §10.6: use the ramp's body step or larger for anything read on a phone" });
  if (gameplayShots > 0 && (worstShare > 15 || worstCentre > 5))
    findings.push({ measure: "HUD occupancy", where: `${rec.playtest}`, detail: `the HUD covers up to ${worstShare} % of the surface and ${worstCentre} % of the central third`, fix: "og-taste §2 (the play surface is the hero; HUD weight): capsules in the corners, the centre third free" });

  // UI motion, transitions and sounds.
  let screenChanges = 0, withMotion = 0, motionTicks = 0;
  for (let i = 1; i < s.length; i++) {
    if (sig[i]!.uiMoved) motionTicks++;
    if (!sig[i]!.uiScreen || i < 3) continue;
    screenChanges++;
    let moving = 0;
    for (let k = Math.max(1, i - 2); k < Math.min(s.length, i + 20); k++) if (sig[k]!.uiMoved || sig[k]!.tween) moving++;
    if (moving >= 2) withMotion++;
  }
  let presses = 0, pressesWithSound = 0;
  // Input-to-response latency.
  const lat: number[] = [];
  let soundAnswered = 0;
  const quiet = new Array(s.length).fill(true) as boolean[];
  const releases = rec.inputs.filter((x) => x.action === "release" && x.tick >= 0).map((x) => x.tick);
  const inputs = rec.inputs.filter((x) => x.action !== "release");
  for (const r of releases) {
    const at = tickIndex.get(r + 1);
    if (at !== undefined) for (let k = Math.max(0, at - 1); k < Math.min(s.length, at + 30); k++) quiet[k] = false;
  }
  for (const inp of inputs) {
    const at = tickIndex.get(inp.tick + 1);
    if (inp.tick < 0 || at === undefined) continue;
    for (let k = Math.max(0, at - 1); k < Math.min(s.length, at + 30); k++) quiet[k] = false;
    // A press's answer may wait for its release (a button activates on release): measure from the release then.
    const touch = inp.action === "touch_start" || inp.action === "touch";
    const release = touch ? releases.find((r) => r >= inp.tick) : undefined;
    const releaseAt = release === undefined ? undefined : tickIndex.get(release + 1);
    let first = -1, uiAt = -1, soundAt = -1, from = at;
    for (let k = at; k < Math.min(s.length, at + 30); k++) {
      if (first < 0 && sig[k]!.any) {
        if (releaseAt !== undefined && k >= releaseAt && from === at) from = releaseAt;
        first = k - from;
      }
      if (uiAt < 0 && sig[k]!.ui) uiAt = k - at;
      if (soundAt < 0 && sig[k]!.sound) soundAt = k - from;
    }
    if (first >= 0) lat.push(first);
    if (soundAt >= 0 && soundAt <= 6) soundAnswered++;
    if (touch && uiAt >= 0 && uiAt <= 6 + (releaseAt === undefined ? 0 : releaseAt - at)) {
      presses++;
      let uiSound = false;
      for (let k = at; k < Math.min(s.length, (releaseAt ?? at) + 8); k++) uiSound ||= sig[k]!.uiSound;
      if (uiSound) pressesWithSound++;
    }
  }
  let baseQuiet = 0, baseActive = 0;
  for (let i = 10; i < s.length; i++) if (quiet[i]) {
    baseQuiet++;
    if (sig[i]!.any) baseActive++;
  }
  const baselineShare = r2((100 * baseActive) / Math.max(1, baseQuiet));
  const answered = lat.length;
  if (inputs.length > 0) {
    if (answered < inputs.length) findings.push({ measure: "input response", where: rec.playtest, detail: `${inputs.length - answered} of ${inputs.length} inputs changed nothing seen or heard within 30 ticks`, fix: "og-feel §1 (answer on the same tick) and og-taste §6: every press shows an answer, a refused one too" });
    else if (median(lat) > 1) findings.push({ measure: "input latency", where: rec.playtest, detail: `the median input is answered ${median(lat)} ticks after the tick it landed (0 is the same tick)`, fix: "og-feel §1: read input and act in the same tick; buffer, do not queue" });
  }
  if (screenChanges > 0 && withMotion < screenChanges) findings.push({ measure: "screen transitions", where: rec.playtest, detail: `${screenChanges - withMotion} of ${screenChanges} screen changes had no motion in the 20 ticks around them`, fix: "og-taste §5 (screens change through a transition): og.ui.screenTransition, one kind per game" });
  if (presses > 0 && pressesWithSound < presses) findings.push({ measure: "UI sounds", where: rec.playtest, detail: `${presses - pressesWithSound} of ${presses} UI presses played no sound on the ui bus`, fix: "og-taste §6 (feedback) and og-audio §3: give the skin or og.ui.configure its press and open sounds" });

  // Damage facts and their feedback.
  const moments: number[] = [];
  const factKeys = new Set<string>();
  for (let i = 1; i < rec.facts.length; i++) {
    const a = leaves(rec.facts[i - 1]!.facts), b = leaves(rec.facts[i]!.facts);
    let hit = false;
    for (const [k, v] of b) {
      const was = a.get(k);
      if (was === undefined) continue;
      if ((HEALTH.test(k) && v < was) || (HITS.test(k) && v > was)) {
        hit = true;
        factKeys.add(k.replace(/\[\d+\]/g, "[]"));
      }
    }
    const tick = rec.facts[i]!.tick;
    if (hit && (moments.length === 0 || tick - moments.at(-1)! > 3)) moments.push(tick);
  }
  const dmg = { moments: moments.length, hitStop: 0, shake: 0, floating: 0, sound: 0, flash: 0, particles: 0, facts: [...factKeys].slice(0, 6) };
  for (const t of moments) {
    const i = tickIndex.get(t) ?? -1;
    if (i < 0) continue;
    const w = sig.slice(Math.max(1, i - 3), Math.min(s.length, i + 9));
    const held = s.slice(Math.max(0, i - 3), Math.min(s.length, i + 9)).some((p) => p.hitStop > 0 || p.scale < 1);
    if (held || w.some((x) => x.hitStop)) dmg.hitStop++;
    if (w.some((x) => x.shake)) dmg.shake++;
    if (w.some((x) => x.floating)) dmg.floating++;
    if (w.some((x) => x.sfx)) dmg.sound++;
    if (w.some((x) => x.tween)) dmg.flash++;
    if (w.some((x) => x.particles)) dmg.particles++;
  }
  if (dmg.moments > 0) {
    const missing = [["hit stop", dmg.hitStop, "og-feel §3 (hit stop by impact weight)"], ["shake", dmg.shake, "og-feel §4 (trauma, not offsets)"], ["floating text", dmg.floating, "og-feel §7 (floating text)"], ["a sound", dmg.sound, "og-audio §1 (every event sounds; a hit's sound on its tick)"]] as const;
    for (const [what, n, fix] of missing) if (n < dmg.moments) findings.push({ measure: "hit feedback", where: rec.playtest, detail: `${dmg.moments - n} of ${dmg.moments} damage moments (${dmg.facts.join(", ")}) had no ${what} within 8 ticks`, fix });
  }

  // Idle frames, particles, frame time.
  let idle = 0;
  for (let i = 10; i < s.length; i++) {
    const p = s[i]!;
    if (!sig[i]!.any && !sig[i]!.moving && p.tweens === 0 && p.floating === 0 && p.particles.length === 0 && p.voices.length === 0) idle++;
  }
  const idleShare = r2((100 * idle) / Math.max(1, s.length - 10));
  if (idleShare > 25) findings.push({ measure: "idle frames", where: rec.playtest, detail: `${idleShare} % of ticks show and play nothing new (nothing moves or animates, no UI motion, no sound, no tween, text or particle)`, fix: "og-feel §9 (idle loops: breathing, a pickup's spin) and og-taste §5 (motion): something alive on every screen" });
  let peakSystems = 0, bursts = 0;
  for (let i = 0; i < s.length; i++) {
    peakSystems = Math.max(peakSystems, s[i]!.particles.length);
    if (sig[i]!.particles) bursts++;
  }
  const step = s.slice(5).map((p) => p.stepMs), render = s.slice(5).map((p) => p.renderMs), total = s.slice(5).map((p) => p.stepMs + p.renderMs);
  const frame = { tier: rec.tier, stepMedianMs: r2(median(step)), stepP95Ms: r2(pct(step, 0.95)), renderMedianMs: r2(median(render)), renderP95Ms: r2(pct(render, 0.95)), frameP95Ms: r2(pct(total, 0.95)), frameMaxMs: r2(Math.max(0, ...total)) };
  if (frame.frameP95Ms > 16.7) findings.push({ measure: "frame time", where: rec.playtest, detail: `p95 step + render ${frame.frameP95Ms} ms on ${rec.tier} (the frame is 16.7 ms)`, fix: "og-game §Performance: fewer draws and particles, cheaper systems; og run --json measures" });
  const audio = audioStats(rec.wav, rec.rate);
  if (audio.clipped > 0) findings.push({ measure: "audio clipping", where: rec.playtest, detail: `${audio.clipped} samples at full scale (peak ${audio.peak})`, fix: "og-audio §4 (the mix): lower the loud sources or put a limiter on master" });

  return {
    m: {
      playtest: rec.playtest,
      title: rec.title,
      tier: rec.tier,
      ticks: rec.ticks,
      shots,
      contrast: { texts: allTexts.length, passing: contrastPass.length, worst: worstContrast },
      textSize: { texts: sized.length, passing: sizePass.length, smallest },
      hud: { gameplayShots, worstShare, worstCentre },
      ui: { screenChanges, withMotion, presses, pressesWithSound, motionTicks },
      latency: { inputs: inputs.length, answered, medianTicks: median(lat), maxTicks: lat.length ? Math.max(...lat) : 0, baselineShare, sound: soundAnswered },
      damage: dmg,
      idle: { share: idleShare },
      particles: { peakSystems, bursts },
      frame,
      audio,
      warnings: rec.warnings,
    },
    findings,
  };
}

// ── scores and the report ──────────────────────────────────────────────────

interface Scores {
  contrast: number | null;
  textSize: number | null;
  hud: number | null;
  transitions: number | null;
  uiSounds: number | null;
  latency: number | null;
  hitFeedback: number | null;
  idle: number | null;
  frameTime: number | null;
  inputSound: number | null;
  locale: number | null;
  taste: number | null;
  feel: number | null;
}

function scores(ms: PlaytestMeasures[], missingKeys: number, hasLocales: boolean): Scores {
  const sum = (f: (m: PlaytestMeasures) => number) => ms.reduce((n, m) => n + f(m), 0);
  const share = (num: number, den: number) => (den > 0 ? Math.round((100 * num) / den) : null);
  const lat = ms.filter((m) => m.latency.inputs > 0);
  const latency = lat.length === 0 ? null : Math.round(lat.reduce((n, m) => n + (m.latency.answered / m.latency.inputs) * (m.latency.medianTicks <= 1 ? 100 : m.latency.medianTicks <= 3 ? 70 : m.latency.medianTicks <= 6 ? 40 : 0), 0) / lat.length);
  const dm = sum((m) => m.damage.moments);
  const hitFeedback = dm === 0 ? null : Math.round((100 * sum((m) => Math.min(3, [m.damage.hitStop, m.damage.shake, m.damage.floating, m.damage.sound].filter((n) => n > 0).length) / 3 * m.damage.moments)) / dm);
  const gameplay = ms.filter((m) => m.hud.gameplayShots > 0);
  const hud = gameplay.length === 0 ? null : Math.min(...gameplay.map((m) => Math.min(band(m.hud.worstShare, 15, 45), band(m.hud.worstCentre, 5, 30))));
  const frameTime = ms.length === 0 ? null : Math.min(...ms.map((m) => band(m.frame.frameP95Ms, 8.3, 33.3)));
  const idle = ms.length === 0 ? null : Math.round(ms.reduce((n, m) => n + band(m.idle.share, 10, 60), 0) / ms.length);
  const s: Scores = {
    contrast: share(sum((m) => m.contrast.passing), sum((m) => m.contrast.texts)),
    textSize: share(sum((m) => m.textSize.passing), sum((m) => m.textSize.texts)),
    hud,
    transitions: share(sum((m) => m.ui.withMotion), sum((m) => m.ui.screenChanges)),
    uiSounds: share(sum((m) => m.ui.pressesWithSound), sum((m) => m.ui.presses)),
    latency,
    hitFeedback,
    idle,
    frameTime,
    inputSound: share(sum((m) => m.latency.sound), sum((m) => m.latency.inputs)),
    locale: hasLocales ? (missingKeys === 0 ? 100 : band(missingKeys, 0, 20)) : null,
    taste: null,
    feel: null,
  };
  const mean = (xs: Array<number | null>) => {
    const v = xs.filter((x): x is number => x !== null);
    return v.length ? Math.round(v.reduce((a, b) => a + b, 0) / v.length) : null;
  };
  s.taste = mean([s.contrast, s.textSize, s.hud, s.transitions, s.uiSounds, s.locale]);
  s.feel = mean([s.latency, s.hitFeedback, s.idle, s.frameTime, s.inputSound]);
  return s;
}

const show = (x: number | null) => (x === null ? "n/a" : String(x));

function markdown(project: string, ms: PlaytestMeasures[], sc: Scores, findings: Finding[], warnings: string[], date: string): string {
  const L: string[] = [];
  L.push(`# og polish: ${project}`, "");
  L.push(`${date}, ${ms.length} playtest(s) on ${[...new Set(ms.map((m) => m.tier))].join(", ")}. Mechanical measures of og-taste §11 and og-feel §14; a score is 0 to 100, n/a when nothing in the playtests exercises it. The measures cannot judge style: read the frames.`, "");
  L.push("## Scores", "");
  L.push("| | Score | Measure |", "|---|---|---|");
  L.push(`| **Taste** | **${show(sc.taste)}** | mean of the taste rows |`);
  L.push(`| Contrast | ${show(sc.contrast)} | text boxes whose ink and ground reach 4.5:1 |`);
  L.push(`| Text size | ${show(sc.textSize)} | text boxes whose tallest ink line reaches 9 CSS px (a 13 px face's capitals) |`);
  L.push(`| HUD occupancy | ${show(sc.hud)} | gameplay frames: HUD at most 15 % of the surface and 5 % of the central third |`);
  L.push(`| Screen transitions | ${show(sc.transitions)} | screen changes with UI motion around them |`);
  L.push(`| UI sounds | ${show(sc.uiSounds)} | UI presses answered on the ui bus |`);
  L.push(`| Locale keys | ${show(sc.locale)} | string tables without missing keys (og cook) |`);
  L.push(`| **Feel** | **${show(sc.feel)}** | mean of the feel rows |`);
  L.push(`| Input latency | ${show(sc.latency)} | inputs answered, by the median ticks to the answer (0–1: 100) |`);
  L.push(`| Hit feedback | ${show(sc.hitFeedback)} | damage moments with hit stop, shake, floating text or a sound (three of four: 100) |`);
  L.push(`| Idle frames | ${show(sc.idle)} | ticks where nothing new is seen or heard (10 %: 100, 60 %: 0) |`);
  L.push(`| Frame time | ${show(sc.frameTime)} | p95 step + render (8.3 ms: 100, 33 ms: 0), worst playtest |`);
  L.push(`| Sound on input | ${show(sc.inputSound)} | inputs answered by a sound within 6 ticks |`, "");
  L.push("## Fixes", "");
  if (findings.length === 0) L.push("No finding crossed a threshold.", "");
  else {
    L.push("| Measure | Where | Found | Fix |", "|---|---|---|---|");
    for (const f of findings) L.push(`| ${f.measure} | ${f.where.replace(/\|/g, "\\|")} | ${f.detail.replace(/\|/g, "\\|")} | ${f.fix} |`);
    L.push("");
  }
  if (warnings.length) {
    L.push("Cook warnings:", "");
    for (const w of warnings) L.push(`- ${w}`);
    L.push("");
  }
  L.push("## Playtests", "");
  for (const m of ms) {
    L.push(`### ${m.playtest}`, "", `${m.title} (${m.ticks} ticks)`, "");
    L.push("| Measure | Value |", "|---|---|");
    L.push(`| Text boxes | ${m.contrast.texts}; ${m.contrast.passing} at 4.5:1; ${m.textSize.passing} of ${m.textSize.texts} at 9 px ink |`);
    if (m.contrast.worst.length) L.push(`| Lowest contrast | ${m.contrast.worst.map((w) => `${w.ratio}:1 \`${w.id}\` (${w.shot})`).join("; ")} |`);
    if (m.textSize.smallest.length) L.push(`| Smallest ink | ${m.textSize.smallest.map((w) => `${w.ink} px \`${w.id}\` (${w.shot})`).join("; ")} |`);
    L.push(`| HUD | ${m.hud.gameplayShots} gameplay frame(s), up to ${m.hud.worstShare} % of the surface, ${m.hud.worstCentre} % of the centre |`);
    L.push(`| UI | ${m.ui.screenChanges} screen change(s), ${m.ui.withMotion} with motion; ${m.ui.motionTicks} ticks of UI motion; ${m.ui.presses} press(es), ${m.ui.pressesWithSound} with a ui-bus sound |`);
    L.push(`| Inputs | ${m.latency.inputs}, ${m.latency.answered} answered, median ${m.latency.medianTicks} ticks, max ${m.latency.maxTicks}; ${m.latency.sound} with a sound; ${m.latency.baselineShare} % of input-free ticks also change something |`);
    L.push(`| Damage | ${m.damage.moments} moment(s)${m.damage.facts.length ? ` from ${m.damage.facts.map((f) => `\`${f}\``).join(", ")}` : ""}: hit stop ${m.damage.hitStop}, shake ${m.damage.shake}, floating text ${m.damage.floating}, sound ${m.damage.sound}, tween ${m.damage.flash}, particles ${m.damage.particles} |`);
    L.push(`| Idle | ${m.idle.share} % of ticks |`);
    L.push(`| Particles | peak ${m.particles.peakSystems} system(s), ${m.particles.bursts} burst(s) |`);
    L.push(`| Frame (${m.frame.tier}) | step ${m.frame.stepMedianMs} ms median, ${m.frame.stepP95Ms} p95; render ${m.frame.renderMedianMs} median, ${m.frame.renderP95Ms} p95; frame p95 ${m.frame.frameP95Ms}, max ${m.frame.frameMaxMs} |`);
    L.push(`| Audio | ${m.audio.seconds} s, peak ${m.audio.peak}, RMS ${m.audio.rms}, ${m.audio.clipped} clipped, ${m.audio.silentShare} % silent ticks |`, "");
    const framed = m.shots.filter((s) => s.frame);
    for (const s of framed) L.push(`![${m.playtest} ${s.name}](${s.frame}) `);
    if (framed.length) L.push("", framed.map((s) => `${s.name} (tick ${s.tick}${s.menu ? ", menu" : ""}: HUD ${s.hud} %, ${s.texts} texts, ${s.lowContrast} under 4.5:1, ${s.small} small)`).join("; "), "");
  }
  return L.join("\n") + "\n";
}

async function main(): Promise<void> {
  const one = opt("--one");
  const entry = opt("--entry");
  const root = opt("--root");
  const out = opt("--out");
  if (!entry || !root || !out) fail("usage: bun src/node/polish.ts --entry <bundle> --root <project> --out <dir> [--warnings <file>] [--frames N] [--json] <tests/x.og.ts>...");
  if (one) return runOne(resolve(one), resolve(entry!), resolve(root!), resolve(out!));
  const files = positional.map((f) => resolve(f));
  if (files.length === 0) fail(`${root} has no playtests: og polish measures what playtests play; add tests/*.og.ts (og-build §Tests)`);
  const outDir = resolve(out!);
  mkdirSync(outDir, { recursive: true });
  const runs = join(outDir, ".runs");
  const warnings: string[] = opt("--warnings") && existsSync(opt("--warnings")!) ? (JSON.parse(readFileSync(opt("--warnings")!, "utf8")) as string[]) : [];
  // The report's frames: at most --frames in all, shared out over the playtests.
  const budget = Math.max(0, Math.floor(Number(opt("--frames") ?? 8)));
  let framesLeft = budget;
  const ms: PlaytestMeasures[] = [];
  const findings: Finding[] = [];
  for (const f of files) {
    const name = basename(f).replace(/\.og\.ts$/, "");
    const dir = join(runs, name);
    const t0 = performance.now();
    const r = Bun.spawnSync(["bun", join(here, "polish.ts"), "--one", f, "--entry", resolve(entry!), "--root", resolve(root!), "--out", dir], { stdout: "pipe", stderr: "pipe" });
    if (r.exitCode !== 0 || !existsSync(join(dir, "run.json"))) {
      process.stderr.write(`og polish: ${name} did not run (exit ${r.exitCode}): ${new TextDecoder().decode(r.stderr).trim().split("\n").slice(-3).join(" ")}\n`);
      findings.push({ measure: "run", where: name, detail: "the playtest did not run", fix: "og test shows why" });
      continue;
    }
    const rec = JSON.parse(readFileSync(join(dir, "run.json"), "utf8")) as RunRecord;
    const share = Math.min(framesLeft, Math.max(1, Math.floor(budget / files.length)));
    const got = measure(rec, join(outDir, "frames"), share, outDir);
    framesLeft -= got.m.shots.filter((x) => x.frame).length;
    ms.push(got.m);
    findings.push(...got.findings);
    process.stdout.write(`og polish: ${name} (${rec.samples.length} ticks, ${(performance.now() - t0).toFixed(0)} ms)\n`);
  }
  const missing = warnings.filter((w) => /key '/.test(w)).length;
  if (missing > 0) findings.push({ measure: "locale keys", where: "assets/strings", detail: `og cook warns about ${missing} key(s) a table lacks or formats differently`, fix: "og-i18n §1: every locale has every key, with the same arguments" });
  const hasLocales = existsSync(join(root!, "assets", "strings")) || warnings.length > 0;
  const sc = scores(ms, missing, hasLocales);
  const date = new Date().toISOString().slice(0, 10);
  const project = basename(resolve(root!));
  writeFileSync(join(outDir, "report.md"), markdown(project, ms, sc, findings, warnings, date));
  writeFileSync(join(outDir, "polish.json"), JSON.stringify({ project, date, scores: sc, findings, playtests: ms, warnings }, null, 1) + "\n");
  rmSync(runs, { recursive: true, force: true });
  if (argv.includes("--json")) process.stdout.write(JSON.stringify({ project, scores: sc, findings: findings.length, report: join(outDir, "report.md") }) + "\n");
  else process.stdout.write(`og polish: taste ${show(sc.taste)}, feel ${show(sc.feel)}; ${findings.length} finding(s); ${join(outDir, "report.md")}\n`);
}

declare function atob(data: string): string;

await main();
