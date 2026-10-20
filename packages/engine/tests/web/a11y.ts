// The accessibility mirror in headless Chrome (src/web/a11y.ts): the dev
// web export of templates/ui-kit on a 1280×720 canvas, driven one tick and
// one render per `frame` call as reference.ts drives it, on its settings
// screen. Checked:
//
// - The DOM mirror is og.ui.a11y()'s tree: one element per node, in tree
//   order, nested under its parent, each at its node's bounds over the
//   canvas, with its role, name, value and states as ARIA (the settings
//   dialog, the section tabs with Audio selected, the Master slider at
//   100 %, the Mute switch unchecked, the Done button), and Chrome's own
//   accessibility tree reads them as a screen reader would.
// - A screen reader's adjust gesture on the Master slider (mirrored as a
//   native range: its value moved and an `input` event, what VoiceOver's
//   swipe dispatches) lowers the volume, and the range returns to its
//   middle step.
// - A screen-reader-style click (`element.click()`: a click event with no
//   pointer, what VoiceOver's and NVDA's default action dispatches) on the
//   mirrored Mute switch reaches the UI as input: the setting toggles
//   (dump_ui's switch is `checked`, the mirror's aria-checked follows), the
//   UI's focus moved to it and the DOM focus followed. A click on the
//   Graphics tab changes the section; Done saves and the "Settings saved"
//   toast is written into the polite live region.
// - An unchanged screen sends no tree: frames after the screen settled
//   carry none (the Worker posts `a11y` only when the digest moved).
// - The page's preferences: `prefers-contrast: more` (CDP's emulated media)
//   reaches the theme with no reload (the frame changes, and goes back when
//   the emulation ends); under `prefers-reduced-motion: reduce` the pause
//   dialog arrives at its full size at once (it fades) where it otherwise
//   scales in from 92 %.
//
//   bun tests/web/a11y.ts [--webgl] [--out <dir>] [--chrome <path>]
//
// Writes <out>/a11y-<tier>.json (default dist/test-a11y/out) and the
// screenshots of the contrast check. Exits 1 when a check fails.

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";
import { ogBin } from "../support/og-bin.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "ui-kit");
const webgl = args.includes("--webgl");
const tier = webgl ? "webgl2" : "webgpu";
const work = join(pkg, "dist", "test-a11y");
const out = resolve(opt("--out") ?? join(work, "out"));
mkdirSync(out, { recursive: true });
const chrome = chromePath(args);
const [WIDTH, HEIGHT] = [1280, 720];

let failures = 0;
const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
function check(ok: boolean, what: string, detail?: unknown) {
  checks.push({ check: what, ok, ...(ok && detail === undefined ? {} : { detail }) });
  if (!ok) failures++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${!ok && detail !== undefined ? `: ${JSON.stringify(detail).slice(0, 800)}` : ""}`);
}

if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
  const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`cooking ${project}: ${new TextDecoder().decode(r.stderr)}`);
}
const dist = join(work, "ui-kit");
const built = await webBuild(project, join(project, "src", "main.ts"), dist, false, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
const page = await Bun.build({ entrypoints: [join(here, "reference.page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(page.logs.map(String).join("\n"));
writeFileSync(join(dist, "reference-page.js"), await page.outputs[0]!.text());
const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>a11y</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${WIDTH}px;height:${HEIGHT}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/reference-page.js"></script></body></html>`;
const server = serve({ dist, isolate: true, extra: { "/a11y.html": html } });
const browser = await Browser.launch({ chrome, width: WIDTH, height: HEIGHT });
const log: string[] = [];
const result: Record<string, unknown> = { tier };

/** One mirrored element, as the DOM has it. */
type Mirrored = { path: string; parentPath: string | null; role: string | null; label: string | null; text: string; attrs: Record<string, string>; box: [number, number, number, number] };
type TreeNode = { path: string; parent: string; role: string; name: string; value: string; states: string[]; x: number; y: number; width: number; height: number };
type Element = { id: string; parent: string; role: string; name: string; states: string[]; width: number; height: number };

const MIRROR = `(() => [...document.querySelectorAll("[data-og-path]")].map((el) => {
  const r = el.getBoundingClientRect();
  const attrs = {};
  for (const a of el.attributes) if (a.name.startsWith("aria-") || a.name === "tabindex") attrs[a.name] = a.value;
  const parent = el.parentElement?.closest("[data-og-path]");
  return { path: el.dataset.ogPath, parentPath: parent ? parent.dataset.ogPath : null, role: el.getAttribute("role"), label: el.getAttribute("aria-label"),
    text: [...el.childNodes].filter((c) => c.nodeType === 3).map((c) => c.textContent).join(""), attrs, box: [r.left, r.top, r.width, r.height] };
}))()`;

let tab: Awaited<ReturnType<typeof browser.open>> | undefined;
try {
  tab = await browser.open("about:blank", log);
  const t = tab;
  await t.send("Emulation.setDeviceMetricsOverride", { width: WIDTH, height: HEIGHT, deviceScaleFactor: 1, mobile: false });
  await t.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/a11y.html${webgl ? "?webgl=1" : ""}` });
  await t.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 90_000);
  const failed = await t.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(`the engine did not start: ${failed}`);
  result["renderer"] = await t.eval<string>("__og_shell__.tier");

  /** `n` frames of one tick each; how many of their answers carried a tree. */
  const frames = (n: number) =>
    t.eval<number>(`(async () => {
      let trees = 0;
      for (let i = 0; i < ${n}; i++) {
        const a = await __og_shell__.call("frame", 1000 / 60);
        if (!a || a.ok === false) throw new Error("frame refused: " + JSON.stringify(a));
        if (a.a11y) trees++;
      }
      // Let the Worker's messages (the tree) reach the page.
      await new Promise((r) => setTimeout(r, 50));
      return trees;
    })()`);
  const commands = (script: string) => t.eval(`__og_shell__.call("probe", "commands", ${JSON.stringify(script)})`);
  const elements = async () => (await t.eval<{ elements: Element[] }>(`__og_shell__.probe("dump", { what: "ui" })`)).elements;
  const element = async (path: string) => {
    const els = await elements();
    const parts = path.split("/");
    return els.find((e) => {
      if (e.id !== parts[parts.length - 1]) return false;
      let p = e.parent;
      for (let k = parts.length - 2; k >= 0; k--) {
        if (p !== parts[k]) return false;
        p = els.find((x) => x.id === p)?.parent ?? "";
      }
      return true;
    });
  };

  await commands("0.05:show:settings");
  await frames(45);
  const tree = await t.eval<{ nodes: TreeNode[]; focusOrder: string[]; focused: string }>("__og_shell__.a11y.tree");
  const dom = await t.eval<Mirrored[]>(MIRROR);
  result["tree"] = tree.nodes.length;
  check(tree.nodes.length > 10, `the mirror has the settings screen's tree (${tree.nodes.length} nodes)`, tree.nodes.map((n) => n.path));
  check(JSON.stringify(dom.map((d) => d.path)) === JSON.stringify(tree.nodes.map((n) => n.path)), "one element per node, in tree order", { dom: dom.map((d) => d.path), tree: tree.nodes.map((n) => n.path) });
  const byPath = new Map(dom.map((d) => [d.path, d]));
  const nested = tree.nodes.filter((n) => (byPath.get(n.path)?.parentPath ?? "") !== n.parent);
  check(nested.length === 0, "each element is nested under its node's parent", nested.map((n) => n.path));
  const misplaced = tree.nodes.filter((n) => {
    const b = byPath.get(n.path)?.box;
    return !b || Math.abs(b[0] - n.x) > 0.5 || Math.abs(b[1] - n.y) > 0.5 || Math.abs(b[2] - Math.max(1, n.width)) > 0.5 || Math.abs(b[3] - Math.max(1, n.height)) > 0.5;
  });
  check(misplaced.length === 0, "each element lies at its node's bounds over the canvas", misplaced.map((n) => ({ path: n.path, node: [n.x, n.y, n.width, n.height], dom: byPath.get(n.path)?.box })));
  const wrongRole = tree.nodes.filter((n) => byPath.get(n.path)?.role !== n.role);
  check(wrongRole.length === 0, "each element has its node's role", wrongRole.map((n) => n.path));
  const find = (role: string, name: string) => dom.find((d) => d.role === role && (d.label === name || d.text === name));
  const dialog = find("dialog", "Settings");
  check(!!dialog && dialog.attrs["aria-modal"] === "true", "the settings dialog: role dialog, named Settings, modal", dialog);
  const audio = find("tab", "Audio");
  check(audio?.attrs["aria-selected"] === "true" && find("tab", "Graphics")?.attrs["aria-selected"] === "false", "the section tabs: Audio selected, Graphics not", { audio, graphics: find("tab", "Graphics") });
  const master = find("slider", "Master");
  check(!!master && master.attrs["aria-valuetext"] !== undefined && master.attrs["tabindex"] === "-1", "the Master slider: named by its row's label, its value, focusable", master);
  const mute = find("switch", "Mute everything");
  check(mute?.attrs["aria-checked"] === "false", "the Mute switch: unchecked", mute);
  check(!!find("button", "Done"), "the Done button", dom.filter((d) => d.role === "button"));
  // What a screen reader is given: Chrome's own accessibility tree.
  const ax = await t.send<{ nodes: Array<{ role?: { value?: string }; name?: { value?: string }; properties?: Array<{ name: string; value: { value?: unknown } }> }> }>("Accessibility.getFullAXTree");
  const axSwitch = ax.nodes.find((n) => n.role?.value === "switch" && n.name?.value === "Mute everything");
  const axChecked = axSwitch?.properties?.find((p) => p.name === "checked")?.value.value;
  check(!!axSwitch && axChecked === "false", "Chrome's accessibility tree reads a switch named Mute everything, not checked", axSwitch);
  check(ax.nodes.some((n) => n.role?.value === "slider" && n.name?.value === "Master"), "and a slider named Master", ax.nodes.filter((n) => n.role?.value === "slider").map((n) => n.name?.value));

  // An unchanged screen sends no tree.
  const quiet = await frames(20);
  check(quiet === 0, `a settled screen's frames carry no tree (${quiet} of 20 did)`);

  // A screen reader's own adjust gesture on the mirrored slider (VoiceOver's
  // swipe down): it moves the native range's value and fires `input`.
  const masterPath = master?.path ?? "";
  const range = `document.querySelector('[data-og-path="${masterPath}"]')`;
  const tag = await t.eval<string>(`${range}?.tagName ?? ""`);
  check(tag === "INPUT", "the Master slider is mirrored as a native range", tag);
  await t.eval(`(() => { const el = ${range}; el.value = "0"; el.dispatchEvent(new Event("input", { bubbles: true })); })()`);
  await frames(4);
  const lowered = (await t.eval<Mirrored[]>(MIRROR)).find((d) => d.path === masterPath);
  check(
    !!lowered && lowered.attrs["aria-valuetext"] !== master?.attrs["aria-valuetext"],
    "the range's input event reached the UI as a decrement: the Master volume went down",
    { before: master?.attrs["aria-valuetext"], after: lowered?.attrs["aria-valuetext"] },
  );
  const middle = await t.eval<string>(`${range}.value`);
  check(middle === "1", "and the range is back at its middle step", middle);

  // A screen reader's click on the mirrored switch.
  const mutePath = mute?.path ?? "";
  await t.eval(`document.querySelector('[data-og-path="${mutePath}"]').click()`);
  const changed = await frames(4);
  check(changed > 0, `the click changed the tree (${changed} frames carried one)`);
  const toggled = await element(mutePath);
  check(!!toggled?.states.includes("checked"), "the click reached the UI: the Mute switch is checked", toggled);
  const after = (await t.eval<Mirrored[]>(MIRROR)).find((d) => d.path === mutePath);
  check(after?.attrs["aria-checked"] === "true", "the mirror's aria-checked followed", after);
  const focusedPath = await t.eval<string | null>("document.activeElement?.dataset?.ogPath ?? null");
  check(focusedPath === mutePath && (await t.eval<{ focused: string }>("__og_shell__.a11y.tree")).focused === mutePath, "the UI's focus moved to the switch and the DOM focus followed", { focusedPath });

  // The Graphics tab, then Done: the toast is announced.
  const graphics = find("tab", "Graphics")!;
  await t.eval(`document.querySelector('[data-og-path="${graphics.path}"]').click()`);
  await frames(6);
  const graphicsDom = await t.eval<Mirrored[]>(MIRROR);
  check(graphicsDom.some((d) => d.role === "switch" && d.label === "High contrast") && !graphicsDom.some((d) => d.label === "Mute everything"), "clicking the Graphics tab changed the section (High contrast is there, Mute is gone)", graphicsDom.filter((d) => d.role === "switch").map((d) => d.label));
  const done = graphicsDom.find((d) => d.role === "button" && d.label === "Done")!;
  await t.eval(`document.querySelector('[data-og-path="${done.path}"]').click()`);
  await frames(10);
  const polite = await t.eval<string>(`document.querySelector('[data-og-a11y-live="polite"]').textContent`);
  check(/Settings saved/.test(polite), "Done saved, and the toast was written into the polite live region", polite);

  // The page's contrast preference reaches the theme, both ways, with no reload.
  await commands("0.02:show:settings");
  await frames(40);
  const shoot = async (name: string) => {
    const shot = await t.eval<{ png: string }>(`__og_shell__.probe("capture", { width: ${WIDTH}, height: ${HEIGHT} })`);
    writeFileSync(join(out, `${tier}-${name}.png`), Uint8Array.from(atob(shot.png), (c) => c.charCodeAt(0)));
    return shot.png;
  };
  const standard = await shoot("contrast-standard");
  await t.send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-contrast", value: "more" }] });
  await frames(6);
  const more = await shoot("contrast-more");
  await t.send("Emulation.setEmulatedMedia", { features: [] });
  await frames(6);
  const back = await shoot("contrast-back");
  check(more !== standard, "prefers-contrast: more changed the frame (the high-contrast theme)");
  check(back === standard, "and ending it brought the standard frame back");

  // Reduced motion: the pause dialog fades in at its full size.
  const pauseWidth = async (reduce: boolean) => {
    await t.send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: reduce ? "reduce" : "no-preference" }] });
    await commands("0.02:show:mainMenu");
    await frames(30);
    await commands("0.02:show:pause");
    await frames(3);
    const early = (await elements()).find((e) => e.role === "dialog");
    await frames(40);
    const settled = (await elements()).find((e) => e.role === "dialog");
    return { early: early?.width ?? 0, settled: settled?.width ?? 0 };
  };
  const moving = await pauseWidth(false);
  const reduced = await pauseWidth(true);
  await t.send("Emulation.setEmulatedMedia", { features: [] });
  result["pause"] = { moving, reduced };
  check(moving.settled > 0 && moving.early < moving.settled - 1, "without the preference the pause dialog scales in", moving);
  check(reduced.settled > 0 && Math.abs(reduced.early - reduced.settled) < 0.5, "under prefers-reduced-motion it arrives at its full size (a fade)", reduced);
  result["engine"] = (await t.eval<string[]>("globalThis.__og_log__ ?? []")).slice(-20);
} catch (e) {
  check(false, "the run completes", String(e instanceof Error ? e.stack : e));
  const engine = (await tab?.eval<string[]>("globalThis.__og_log__ ?? []").catch(() => [])) ?? [];
  result["engine"] = engine.slice(-20);
} finally {
  await browser.close();
  server.stop();
}
result["checks"] = checks;
result["log"] = log.slice(-20);
writeFileSync(join(out, `a11y-${tier}.json`), JSON.stringify(result, null, 2) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} a11y (${tier}): ${checks.length - failures} of ${checks.length} checks; ${join(out, `a11y-${tier}.json`)}`);
process.exit(failures === 0 ? 0 : 1);
