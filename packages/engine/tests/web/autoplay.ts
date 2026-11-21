// A page's own scan of its controls finds a TSX UI's controls and presses
// them through the engine (src/web/a11y.ts), in headless Chrome, on the
// dev web export of tests/fixtures/a11y (its settings dialog, 1280×720).
// The platform's autoplay reads a game page as any page: the elements that
// are controls by tag or role, not disabled or inert, at least 8×8 px,
// shown (visibility, opacity at least 0.1) and the element
// `elementFromPoint` hits at their centre, named by aria-label or text, and
// it taps their centres. Checked:
//
// - The scan (`SCAN`) finds the dialog's tabs, switches and Done by their
//   names, and nothing that is not a control; a point on the dialog's
//   title hits the canvas.
// - A mouse click on the Mute switch's centre toggles it once (the press
//   reaches the engine's hit test through the canvas, and the click after
//   it is not a second activation); a touch tap on Subtitles toggles it
//   once; a mouse drag from the Master slider off it reaches the canvas
//   whole (the canvas holds the pointer from the press).
// - A click on Done closes the dialog, and its controls leave the scan.
//
//   bun tests/web/autoplay.ts [--out <dir>] [--chrome <path>]
//
// Writes <out>/autoplay.json (default dist/test-autoplay/out). Exits 1 when
// a check fails.

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";
import { cookProject } from "../support/og-bin.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(pkg, "tests", "fixtures", "a11y");
const work = join(pkg, "dist", "test-autoplay");
const out = resolve(opt("--out") ?? join(work, "out"));
mkdirSync(out, { recursive: true });
const chrome = chromePath(args);
const [WIDTH, HEIGHT] = [1280, 720];

let failures = 0;
const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
function check(ok: boolean, what: string, detail?: unknown) {
  checks.push({ check: what, ok, ...(detail === undefined ? {} : { detail }) });
  if (!ok) failures++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${detail !== undefined ? `: ${JSON.stringify(detail).slice(0, 800)}` : ""}`);
}

/** The page's controls as a page scanner reads them: label and centre. */
const SCAN = `(() => {
  const sel = "button, a[href], input, select, textarea, summary, [role=button], [role=link], [role=menuitem], [role=tab], [role=option], [role=checkbox], [role=radio], [role=switch], [role=slider]";
  const out = [];
  for (const el of document.querySelectorAll(sel)) {
    if (el.matches("[disabled], [aria-disabled=true]") || el.closest("[inert]")) continue;
    const r = el.getBoundingClientRect();
    if (r.width < 8 || r.height < 8) continue;
    const cs = getComputedStyle(el);
    if (cs.visibility !== "visible" || Number(cs.opacity) < 0.1) continue;
    if (el.checkVisibility && !el.checkVisibility({ opacityProperty: true, visibilityProperty: true })) continue;
    const cx = r.left + r.width / 2, cy = r.top + r.height / 2;
    const hit = document.elementFromPoint(cx, cy);
    if (!hit || !(hit === el || el.contains(hit))) continue;
    out.push({ label: (el.getAttribute("aria-label") || el.innerText || "").trim(), role: el.getAttribute("role") || el.tagName.toLowerCase(), x: cx, y: cy, checked: el.getAttribute("aria-checked") });
  }
  return out;
})()`;
type Control = { label: string; role: string; x: number; y: number; checked: string | null };

cookProject(project, repo);
const dist = join(work, "a11y");
const built = await webBuild(project, join(project, "src", "main.ts"), dist, false, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
const page = await Bun.build({ entrypoints: [join(here, "reference.page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(page.logs.map(String).join("\n"));
writeFileSync(join(dist, "reference-page.js"), await page.outputs[0]!.text());
const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>autoplay</title>
<style>html,body{margin:0;background:#14161b}canvas{display:block;width:${WIDTH}px;height:${HEIGHT}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/reference-page.js"></script></body></html>`;
const server = serve({ dist, isolate: true, extra: { "/autoplay.html": html } });
const browser = await Browser.launch({ chrome, width: WIDTH, height: HEIGHT });
const log: string[] = [];

type Element = { id: string; role: string; name: string; text: string; states: string[] };

let tab: Awaited<ReturnType<typeof browser.open>> | undefined;
try {
  tab = await browser.open("about:blank", log);
  const t = tab;
  await t.send("Emulation.setDeviceMetricsOverride", { width: WIDTH, height: HEIGHT, deviceScaleFactor: 1, mobile: false });
  await t.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
  await t.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/autoplay.html` });
  await t.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 90_000);
  const failed = await t.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(`the engine did not start: ${failed}`);

  /** `n` frames of one tick each, then the Worker's messages reach the page. */
  const frames = (n: number) =>
    t.eval(`(async () => {
      for (let i = 0; i < ${n}; i++) {
        const a = await __og_shell__.call("frame", 1000 / 60);
        if (!a || a.ok === false) throw new Error("frame refused: " + JSON.stringify(a));
      }
      await new Promise((r) => setTimeout(r, 50));
    })()`);
  const commands = (script: string) => t.eval(`__og_shell__.call("probe", "commands", ${JSON.stringify(script)})`);
  const elements = async () => (await t.eval<{ elements: Element[] }>(`__og_shell__.probe("dump", { what: "ui" })`)).elements;
  /** The UI's own word on a control: dump_ui's node of that role and name. */
  const ui = async (role: string, name: string) => (await elements()).find((e) => e.role === role && (e.name || e.text) === name);
  const scan = () => t.eval<Control[]>(SCAN);
  const find = (cs: Control[], label: string) => cs.find((c) => c.label === label);

  await commands("0.05:show:settings");
  await frames(45);
  const controls = await scan();
  // The controls draw nothing: the page looks the same with them as without them.
  const shown = await t.screenshot();
  await t.eval(`document.querySelector("[data-og-a11y]").style.visibility = "hidden"`);
  const bare = await t.screenshot();
  await t.eval(`document.querySelector("[data-og-a11y]").style.visibility = ""`);
  check(shown.length === bare.length && shown.every((b, i) => b === bare[i]), "the controls over the canvas draw nothing");
  const labels = controls.map((c) => c.label);
  for (const want of ["Audio", "Graphics", "Mute everything", "Subtitles", "Done"]) check(labels.includes(want), `the scan finds ${want}`, labels);
  check(controls.every((c) => c.label !== "" && c.role !== "div"), "every control it finds is one, with a name", controls);
  const title = await t.eval<string>(`(() => { const h = [...document.querySelectorAll("[data-og-path]")].find((e) => e.textContent === "Settings" && !e.getAttribute("role")?.match(/button|tab|radio|switch/)); if (!h) return "none"; const r = h.getBoundingClientRect(); return document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2)?.tagName ?? "none"; })()`);
  check(title === "CANVAS", "a point on what is not a control hits the canvas", title);

  // A mouse click: one press through the canvas, no second activation.
  const mute = find(controls, "Mute everything");
  const before = (await ui("switch", "Mute everything"))?.states.includes("checked") ?? false;
  if (mute) await t.click(mute.x, mute.y);
  await frames(20);
  const after = (await ui("switch", "Mute everything"))?.states.includes("checked") ?? false;
  check(!!mute && after === !before, "a click on Mute everything toggles it once", { before, after });
  check(find(await scan(), "Mute everything")?.checked === String(after), "the mirror's aria-checked follows it", find(await scan(), "Mute everything"));

  // A touch tap.
  const subs = find(await scan(), "Subtitles");
  const subsBefore = (await ui("switch", "Subtitles"))?.states.includes("checked") ?? false;
  if (subs) await t.tap(subs.x, subs.y);
  await frames(20);
  const subsAfter = (await ui("switch", "Subtitles"))?.states.includes("checked") ?? false;
  check(!!subs && subsAfter === !subsBefore, "a touch tap on Subtitles toggles it once", { subsBefore, subsAfter });

  // A drag: pressed on the slider, moved, released; the canvas has every event of it.
  const master = find(await scan(), "Master");
  await t.eval(`(() => { const c = document.querySelector("canvas"); globalThis.__drag__ = []; for (const ty of ["pointerdown", "pointermove", "pointerup"]) c.addEventListener(ty, (e) => e.buttons || ty === "pointerup" ? __drag__.push(ty) : 0); })()`);
  if (master) {
    await t.send("Input.dispatchMouseEvent", { type: "mousePressed", x: master.x, y: master.y, button: "left", buttons: 1, clickCount: 1 });
    for (let i = 1; i <= 8; i++) await t.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: master.x - i * 40, y: master.y + i * 30, button: "left", buttons: 1 });
    await t.send("Input.dispatchMouseEvent", { type: "mouseReleased", x: master.x - 320, y: master.y + 240, button: "left", buttons: 0, clickCount: 1 });
  }
  const drag = await t.eval<string[]>("__drag__");
  check(!!master && drag[0] === "pointerdown" && drag.filter((d) => d === "pointermove").length === 8 && drag.at(-1) === "pointerup", "a drag from a control reaches the canvas whole: its press, every move and its release", drag);

  // Done closes the dialog: its controls leave the scan.
  const done = find(await scan(), "Done");
  if (done) await t.click(done.x, done.y);
  await frames(30);
  const left = (await scan()).map((c) => c.label);
  check(!!done && !left.includes("Done") && !left.includes("Audio"), "a click on Done closes the dialog and its controls leave the scan", left);
} catch (e) {
  check(false, "the run", String(e));
  if (log.length) console.log(log.slice(-20).join("\n"));
} finally {
  await browser.close();
  server.stop();
}
writeFileSync(join(out, "autoplay.json"), JSON.stringify({ checks }, null, 2));
process.exit(failures ? 1 : 0);
