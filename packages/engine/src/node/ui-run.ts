// `og ui check` and `og ui preview` (docs/ui.md §16.4, §16.5): a game's
// screens laid out over a matrix of cells (styles × breakpoints × locales ×
// text scale), each cell an engine of its own on the native host driven by
// a command script as `og run --commands` drives one, its UI frame read
// (`og_node_ui_frame`) and its frame captured. `check` runs the checker
// (ui-check.ts) on every cell and merges the findings; `preview` writes a
// contact sheet and a JSON index of every cell; `--film` a filmstrip of a
// motion at chosen times.
//
//   bun src/node/ui-run.ts check|preview --entry <bundle> --root <project> --out <dir>
//       [--screen <name>[=<commands>]]... [--show <template, default show:{screen}>]
//       [--styles a,b] [--style-cmd <template, default style:{style}>]
//       [--breakpoints desktop,phonePortrait,phoneLandscape,tablet] [--locales game,pseudo,ja,ar]
//       [--scales 100,150] [--high-contrast] [--settle S] [--film <commands> --at t,t,...] [--json]
//
// A screen's commands are `t:action[:args];...` from the cell's start (a
// bare action is at 0); without a screen the game's first screen is the
// one cell row. The viewport of each breakpoint is set through
// og.ui.previewViewport (the size the kit's `viewport:` command sets), the
// pseudo-locale by lengthening text in process (og.i18n.t and the text of
// og.ui's widgets), the other locales by the engine's locale.

import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { attachRenderer, loadNativeHost, native } from "./index.ts";
import { nativePlatform } from "./platform.ts";
import { bundleGame } from "./build.ts";
import { decodePng, encodePng, type Image } from "./png.ts";
import type { PlaytestDesc } from "../playtest.ts";
import { boundHost, createEngine, type Engine, type GameToken } from "../runtime.ts";
import { og } from "../index.ts";
import { cellName, checkFrame, checkLocales, checkScale, formatText, mergeFindings, pseudo, RULES, screensFromHelp, screensFromPlaytest, type UiCell, type UiFinding, type UiFrameSnapshot } from "./ui-check.ts";
import { sheet, type SheetCell } from "./ui-sheet.ts";

/** The surfaces of the matrix: CSS size, DPR 1 (the sheet's), safe area, pointer. Phone and tablet as the kit's `viewport:` command lays them out. */
export const BREAKPOINTS: Record<string, { width: number; height: number; safeArea: { left: number; top: number; right: number; bottom: number }; coarsePointer: boolean }> = {
  desktop: { width: 1280, height: 720, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false },
  phonePortrait: { width: 390, height: 844, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true },
  phoneLandscape: { width: 844, height: 390, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true },
  tablet: { width: 820, height: 1180, safeArea: { left: 0, top: 24, right: 0, bottom: 20 }, coarsePointer: true },
};

const argv = process.argv.slice(2);
const mode = argv[0];
const opt = (name: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1] : undefined;
};
const opts = (name: string) => argv.flatMap((a, i) => (a === name && argv[i + 1] !== undefined ? [argv[i + 1]!] : []));
const list = (name: string, d: string) => (opt(name) ?? d).split(",").map((s) => s.trim()).filter(Boolean);

function fail(reason: string): never {
  process.stderr.write(`og ui ${mode ?? ""}: ${reason}\n`);
  process.exit(2);
}

/** `t:action;action` → each step timed (a bare action at `at`). */
function timed(commands: string, at = 0): string[] {
  return commands.split(";").map((s) => s.trim()).filter(Boolean).map((s) => (/^\d+(\.\d+)?:/.test(s) ? s.replace(/^(\d+(?:\.\d+)?)/, (t) => String(Number(t) + at)) : `${at}:${s}`));
}

interface Cell extends UiCell {
  index: number;
  commands: string;
  frame: string | null;
  uiFrame: string;
  findings: number;
}

/** The text arguments of og.ui's widgets and og.i18n.t, lengthened while `on`. */
function pseudoAbi(on: boolean): () => void {
  if (!on) return () => {};
  const abi = boundHost()!.abi as unknown as Record<string, (...a: unknown[]) => unknown>;
  const saved: Record<string, (...a: unknown[]) => unknown> = {};
  const wrap = (name: string, f: (orig: (...a: unknown[]) => unknown) => (...a: unknown[]) => unknown) => {
    if (typeof abi[name] !== "function") return;
    saved[name] = abi[name]!;
    abi[name] = f(saved[name]!);
  };
  wrap("og_i18n_t", (o) => (...a) => pseudo(String(o(...a))));
  // (id, text, desc): label, button; (id, on, label): toggle, checkbox.
  for (const name of ["og_ui_label", "og_ui_button"]) wrap(name, (o) => (id, text, ...rest) => o(id, pseudo(String(text)), ...rest));
  for (const name of ["og_ui_toggle", "og_ui_checkbox"]) wrap(name, (o) => (id, on, text, ...rest) => o(id, on, pseudo(String(text)), ...rest));
  return () => Object.assign(abi, saved);
}

async function main(): Promise<void> {
  if (mode !== "check" && mode !== "preview") fail("usage: bun src/node/ui-run.ts check|preview --entry <bundle> --root <project> --out <dir> [...]");
  const entry = opt("--entry"), root = opt("--root"), out = opt("--out");
  if (!entry || !root || !out) fail("--entry, --root and --out are required");
  const outDir = resolve(out!);
  mkdirSync(join(outDir, "cells"), { recursive: true });
  const projectFile = join(root!, "project.ogt");
  const project = existsSync(projectFile) ? readFileSync(projectFile, "utf8") : undefined;
  const profile = project ? (/^\s*profile\s+"(2d|3d)"/m.exec(project)?.[1] as "2d" | "3d" | undefined) : undefined;
  const show = opt("--show") ?? "show:{screen}";
  /** A screen of the matrix: its name, the commands that reach it, and when it is captured (default the settle time). */
  const screens: Array<{ name: string; commands: string; at?: number }> = opts("--screen").map((s) => {
    const eq = s.indexOf("=");
    return eq < 0 ? { name: s, commands: show.replaceAll("{screen}", s) } : { name: s.slice(0, eq), commands: s.slice(eq + 1) };
  });
  const styles = list("--styles", "");
  const styleCmd = opt("--style-cmd") ?? "style:{style}";
  const bps = list("--breakpoints", mode === "check" ? "desktop,phonePortrait,phoneLandscape,tablet" : "desktop,phonePortrait,phoneLandscape,tablet");
  for (const b of bps) if (!BREAKPOINTS[b]) fail(`--breakpoints: '${b}' is not one of ${Object.keys(BREAKPOINTS).join(", ")}`);
  // The locales: the game's own, pseudo, then a CJK and a right-to-left one the game has strings for.
  const have = existsSync(join(root!, "assets", "strings")) ? readdirSync(join(root!, "assets", "strings")).map((f) => f.replace(/\.ogt$/, "")) : [];
  const cjk = have.find((l) => /^(ja|zh|ko)/.test(l)) ?? "ja";
  const rtl = have.find((l) => /^(ar|he|fa|ur)/.test(l)) ?? "ar";
  const locales = list("--locales", `game,pseudo,${cjk},${rtl}`).map((l) => (l === "game" ? "" : l));
  // The check holds every screen to 200 % text (§15.2) in the game's
  // locale and the pseudo-locale (the longest text: CJK and right to left
  // add no length) unless --scales asks for every cell; a preview draws
  // 100 % unless asked.
  const scalesGiven = opt("--scales") !== undefined;
  const scales = list("--scales", mode === "check" ? "100,150" : "100").map(Number);
  const settle = Number(opt("--settle") ?? 0.8);
  const highContrast = argv.includes("--high-contrast");

  loadNativeHost();
  const extras = native() as unknown as { og_node_ui_frame(id: number): string };
  if (typeof extras.og_node_ui_frame !== "function") fail("this native addon has no og_node_ui_frame: rebuild it (packages/engine/scripts/build.sh)");
  // Each cell runs a fresh copy of the bundle: bun keys file:// modules by path and ignores a query, so one import
  // shared by every cell carried the game's module state over (a kit's dialogue handle from the last cell's engine
  // left every later cell of the screen an empty conversation, measured as passing).
  let copies = 0;
  const leftovers: string[] = [];
  process.on("exit", () => {
    for (const f of leftovers) rmSync(f, { force: true });
  });
  const freshGame = async (): Promise<GameToken> => {
    const from = resolve(entry!);
    const copy = from.replace(/(\.[cm]?[jt]s)?$/, (ext) => `.cell-${copies++}${ext || ".js"}`);
    copyFileSync(from, copy);
    leftovers.push(copy);
    // bun reads `<module>.map` when a stack is formed: kept until exit, so a cell's errors still name the .ts lines.
    if (existsSync(`${from}.map`)) {
      copyFileSync(`${from}.map`, `${copy}.map`);
      leftovers.push(`${copy}.map`);
    }
    const g = ((await import(pathToFileURL(copy).href)) as { default?: GameToken }).default;
    if (!g) fail(`${entry} has no default export`);
    return g!;
  };

  /** One run: an engine of its own, the script, then its UI frame and the frames it captured. */
  const runCell = async (bp: string, locale: string, steps: string[], shots: Array<{ at: number; name: string }>, scale = 100): Promise<{ frame: UiFrameSnapshot | null; images: Map<string, Image>; warnings: string[] }> => {
    const game = await freshGame();
    const vp = BREAKPOINTS[bp]!;
    const restore = pseudoAbi(locale === "pseudo");
    const created = createEngine(game, { root: resolve(root!), deferStart: true, ...(project === undefined ? {} : { project }), ...(profile ? { profile } : {}), ...(locale && locale !== "pseudo" ? { locale } : {}) });
    if (!created.ok) {
      restore();
      fail(created.reason);
    }
    const engine: Engine = created.value;
    const warnings: string[] = [];
    const images = new Map<string, Image>();
    try {
      engine.attachPlatform(nativePlatform());
      const attached = attachRenderer(engine.id, vp.width, vp.height);
      if (!attached.ok) fail(attached.reason);
      const started = engine.advance(0);
      if (!started.ok) fail(started.reason);
      // The player's text size: configure keeps it through the game's own
      // configure calls (a field they leave out keeps its value).
      if (scale !== 100) og.ui.configure(engine.world, { textScale: scale / 100 });
      og.ui.previewViewport(engine.world, { width: vp.width, height: vp.height, dpr: 1, safeArea: vp.safeArea, coarsePointer: vp.coarsePointer, highContrast, breakpoint: bp } as never);
      const script = [...steps, ...shots.map((s) => `${s.at}:screenshot:${s.name}.png`)];
      const report = engine.probe.commands(script.join(";")) as { untilTick: number } | null;
      if (!report) fail(`the script is malformed: ${script.join(";")}`);
      for (let n = 0; n < 6000; n++) {
        const a = engine.frame(1000 / engine.rate);
        if (!a.ok) {
          warnings.push(a.reason ?? "the frame failed");
          break;
        }
        for (const o of engine.takeOutputs()) if (o.kind === "png") images.set(basename(o.file).replace(/\.png$/, ""), decodePng(Uint8Array.from(atob(o.data), (c) => c.charCodeAt(0))));
        if (a.quit || (a.pending ?? ((a.tick ?? 0) > report!.untilTick ? 0 : 1)) === 0) break;
      }
      for (const l of engine.takeLog()) if (l.warning) warnings.push(`tick ${l.tick}: ${l.text}`);
      const f = JSON.parse(extras.og_node_ui_frame(engine.id)) as UiFrameSnapshot & { error?: string };
      return { frame: f.error ? null : f, images, warnings };
    } finally {
      engine.destroy();
      restore();
    }
  };

  /** The probe's `help` text (every action with its usage), from a fresh engine. */
  const runHelp = (game: GameToken): string => {
    const created = createEngine(game, { root: resolve(root!), deferStart: true, ...(project === undefined ? {} : { project }), ...(profile ? { profile } : {}) });
    if (!created.ok) fail(created.reason);
    const engine = created.value;
    try {
      engine.attachPlatform(nativePlatform());
      engine.advance(0);
      engine.probe.commands("0:help:help.txt");
      for (let n = 0; n < 4; n++) {
        engine.frame(1000 / engine.rate);
        const help = engine.takeOutputs().find((o) => o.action === "help");
        if (help) return help.data;
      }
      return "";
    } finally {
      engine.destroy();
    }
  };

  // No screen given or declared (og-cli reads project.ogt's): the game's
  // show command's enumerated values, else the playtests' screenshots, else
  // the start screen. A bare check sees every screen the project has.
  if (screens.length === 0) {
    const helpGame = await freshGame();
    const found = await defaultScreens(() => runHelp(helpGame), show, opts("--playtest"));
    screens.push(...found.screens);
    process.stderr.write(`og ui ${mode}: ${screens.length} screen(s) from ${found.from}\n`);
  }

  // --film: one run, a frame at each time, as a strip.
  const film = opt("--film");
  if (film !== undefined) {
    const at = list("--at", "0,0.1,0.2,0.3,0.4,0.5").map(Number);
    const bp = bps[0]!;
    const screen = screens[0]!;
    const base = [...(styles[0] ? timed(styleCmd.replaceAll("{style}", styles[0])) : []), ...timed(screen.commands)];
    const start = Math.max(settle, 0);
    const steps = [...base, ...timed(film, start)];
    const r = await runCell(bp, locales[0] ?? "", steps, at.map((t, i) => ({ at: Number((start + t).toFixed(4)), name: `film-${i}` })));
    const rate = 60;
    const cells: SheetCell[] = at.map((t, i) => ({ image: r.images.get(`film-${i}`) ?? null, lines: [`+${t}S`, `TICK ${Math.round((start + t) * rate)}`] }));
    const img = sheet(cells, at.length, 260, `FILM ${screen.name} ${bp}: ${film}`.slice(0, 120));
    const file = join(outDir, "film.png");
    writeFileSync(file, encodePng(img));
    const index = { film, screen: screen.name, breakpoint: bp, start, frames: at.map((t, i) => ({ t, tick: Math.round((start + t) * rate), shown: r.images.has(`film-${i}`) })), warnings: r.warnings };
    writeFileSync(join(outDir, "film.json"), JSON.stringify(index, null, 1) + "\n");
    process.stdout.write(argv.includes("--json") ? JSON.stringify({ film: file, index: join(outDir, "film.json") }) + "\n" : `og ui preview: filmstrip ${file}\n`);
    return;
  }

  // The matrix.
  const cells: Cell[] = [];
  const all: UiFinding[] = [];
  const images: Image[] = [];
  const t0 = performance.now();
  for (const screen of screens) for (const style of styles.length ? styles : [""]) for (const bp of bps) {
    let base: UiFrameSnapshot | null = null;
    for (const locale of locales) {
      let unscaled: UiFrameSnapshot | null = null;
      for (const scale of scales) {
        if (scale !== 100 && !scalesGiven && locale !== "" && locale !== "pseudo") continue;
        const cell: UiCell = { screen: screen.name, breakpoint: bp, locale, style, scale, highContrast };
        const index = cells.length;
        const uiFrame = join("cells", `${index}.uiframe.json`);
        const steps = [...(style ? timed(styleCmd.replaceAll("{style}", style)) : []), ...timed(screen.commands)];
        const r = await runCell(bp, locale, steps, [{ at: screen.at ?? settle, name: "cell" }], scale);
        const img = r.images.get("cell") ?? null;
        let frame: string | null = null;
        if (img) {
          frame = join("cells", `${index}.png`);
          writeFileSync(join(outDir, frame), encodePng(img));
        }
        let found: UiFinding[] = [];
        if (r.frame) {
          writeFileSync(join(outDir, uiFrame), JSON.stringify(r.frame));
          found = checkFrame(r.frame, cell, { image: img });
          if (scale === 100) {
            unscaled = r.frame;
            if (locale === "") base = r.frame;
            else if (base) found.push(...checkLocales(base, r.frame, cell));
          } else if (unscaled) found.push(...checkScale(unscaled, r.frame, cell));
        }
        for (const w of r.warnings) process.stderr.write(`og ui ${mode}: ${cellName(cell)}: ${w}\n`);
        all.push(...found);
        cells.push({ ...cell, index, commands: steps.join(";"), frame, uiFrame: r.frame ? uiFrame : "", findings: found.length });
        images.push(img as Image);
      }
    }
  }
  const findings = mergeFindings(all);
  const errors = findings.filter((f) => f.severity === "error").length;
  const project_ = basename(resolve(root!));
  const rules = RULES.map((r) => ({ ...r, findings: findings.filter((f) => f.rule === r.id).length }));
  writeFileSync(join(outDir, "check.json"), JSON.stringify({ project: project_, cells: cells.length, errors, warnings: findings.length - errors, findings, rules }, null, 1) + "\n");
  writeFileSync(join(outDir, "check.txt"), formatText(findings, `og ui check: ${project_}, ${cells.length} cell(s) (${screens.length} screen(s) × ${Math.max(1, styles.length)} style(s) × ${bps.length} breakpoint(s) × ${locales.length} locale(s) × ${scales.length} scale(s))`));
  if (mode === "preview") {
    // One sheet per screen and style: a row per breakpoint, a column per locale and scale.
    const sheets: string[] = [];
    for (const screen of screens) for (const style of styles.length ? styles : [""]) {
      const mine = cells.filter((c) => c.screen === screen.name && c.style === style);
      const sc: SheetCell[] = mine.map((c) => ({ image: images[c.index] ?? null, flagged: c.findings > 0, lines: [`#${c.index} ${c.breakpoint} ${c.locale || "GAME"}${c.scale !== 100 ? ` ${c.scale}%` : ""}`, `${c.findings} FINDING${c.findings === 1 ? "" : "S"}`] }));
      const img = sheet(sc, locales.length * scales.length, 300, `${screen.name}${style ? ` ${style}` : ""}`);
      const file = join(outDir, `sheet-${screen.name}${style ? `-${style}` : ""}.png`);
      writeFileSync(file, encodePng(img));
      sheets.push(file);
    }
    writeFileSync(join(outDir, "index.json"), JSON.stringify({ project: project_, sheets: sheets.map((s) => basename(s)), axes: { screens: screens.map((s) => s.name), styles, breakpoints: bps, locales: locales.map((l) => l || "game"), scales }, cells: cells.map((c) => ({ ...c, locale: c.locale || "game" })) }, null, 1) + "\n");
    if (argv.includes("--json")) process.stdout.write(JSON.stringify({ sheets, index: join(outDir, "index.json"), cells: cells.length, findings: findings.length }) + "\n");
    else process.stdout.write(`og ui preview: ${cells.length} cell(s) in ${((performance.now() - t0) / 1000).toFixed(1)} s; ${sheets.map((s) => basename(s)).join(", ")}; ${join(outDir, "index.json")}\n`);
    return;
  }
  if (argv.includes("--json")) process.stdout.write(JSON.stringify({ cells: cells.length, errors, warnings: findings.length - errors, findings }) + "\n");
  else process.stdout.write(formatText(findings, `og ui check: ${project_}, ${cells.length} cell(s) in ${((performance.now() - t0) / 1000).toFixed(1)} s (${join(outDir, "check.json")})`));
  process.exit(errors > 0 ? 3 : 0);
}

/**
 * The screens a bare `og ui check` covers when none are given or declared:
 * the values the game's show action enumerates in its usage (`show:<a|b|c>`,
 * from the probe's help), else each screenshot the playtests take (reached
 * by the steps before it, captured at its time), else the start screen.
 */
async function defaultScreens(help: () => string, show: string, playtests: readonly string[]): Promise<{ screens: Array<{ name: string; commands: string; at?: number }>; from: string }> {
  const listed = screensFromHelp(help(), show);
  if (listed.length > 0) return { screens: listed, from: `the game's ${show.split(":")[0]} command's usage` };
  const out: Array<{ name: string; commands: string; at: number }> = [];
  const seen = new Set<string>();
  for (const file of playtests) {
    const bundled = join(dirname(resolve(file)), `.ui-check-${basename(file)}.js`);
    try {
      const b = await bundleGame(file, bundled, "native");
      if (!b.ok) continue;
      const desc = ((await import(pathToFileURL(bundled).href)) as { default?: PlaytestDesc }).default;
      for (const sc of screensFromPlaytest(basename(file).replace(/\.og\.ts$/, ""), desc?.commands ?? "")) {
        if (seen.has(sc.commands)) continue;
        seen.add(sc.commands);
        out.push(sc);
      }
    } catch {
      continue;
    } finally {
      rmSync(bundled, { force: true });
      rmSync(`${bundled}.map`, { force: true });
    }
  }
  if (out.length > 0) return { screens: out, from: `the screenshots of ${playtests.length} playtest(s)` };
  return { screens: [{ name: "start", commands: "" }], from: "the start screen (no screen declared: project.ogt's ui { screen \"<name>\" \"<commands>\" }, or a show command whose usage lists its screens)" };
}

declare function atob(data: string): string;

await main();
