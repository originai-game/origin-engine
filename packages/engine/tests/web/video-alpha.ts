// Keyed footage on the web (`alpha true`, docs/design.md §2.9): the video
// fixture's `keyed` command plays clips/keyed.ogt, an AV1 stream whose
// alpha is packed below its colour, on a UI image over black, in headless
// Chrome on WebGPU from the dev web export. The browser decodes the packed
// picture (WebCodecs, else rav1d-wasm), copyExternalImageToTexture puts it
// in its staging texture, and og-render's fold (video_fold.wgsl) writes the
// colour and the matte as alpha into the stream texture. The source is white
// with its alpha ramping 0 to 255 left to right over black. How a host
// blends a translucent UI pixel depends on its output (in display values on
// an 8-bit sRGB one, in linear light on the web's wide-gamut canvas, whose
// paint pass composites a layer of its own), so the fixture draws white
// swatches at 25, 50 and 75 % opacity beside the image: the host's own blend
// of those alphas.
//
// Checks, on the web frame and on the native one
// (references/<recording tier>/keyed.png): the ramp's ends (alpha 0 and
// 255) black and white, the columns of alpha 64, 128 and 191 within ±10 of
// their swatch, and the rest of the half without the image black.
//
// Ways this could fail: the fold skipped (the image is the packed picture
// squashed, its lower half the grey matte, or opaque white); the matte read
// from the colour's half or a row off; the colour sRGB-decoded twice (the
// ramp darkened) or the matte decoded as colour (sRGB-encoded alpha); the
// staging copy sized to the frame, not twice it (the copy is cut, the matte
// lost).
//
//   bun tests/web/video-alpha.ts [--out <dir>] [--chrome <path>]

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../../src/node/web-build.ts";
import { cookProject } from "../support/og-bin.ts";
import { decodePng, RECORDING, type Image } from "../support/png.ts";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(pkg, "tests", "fixtures", "video");
const out = resolve(opt("--out") ?? join(pkg, "dist", "test-reference", "video-alpha"));
const [WIDTH, HEIGHT] = [1280, 720];
const TICK_RATE = 30;
mkdirSync(out, { recursive: true });

cookProject(project, repo);
const dist = join(pkg, "dist", "test-reference", "video");
const built = await webBuild(project, join(project, "src", "main.ts"), dist, false, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
const page = await Bun.build({ entrypoints: [join(here, "reference.page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(page.logs.map(String).join("\n"));
writeFileSync(join(dist, "reference-page.js"), await page.outputs[0]!.text());
const html = `<!doctype html><html><head><meta charset="utf-8"><title>video-alpha</title>
<style>html,body{margin:0;background:#000}canvas{display:block;width:${WIDTH}px;height:${HEIGHT}px}</style></head>
<body><canvas></canvas><script type="module" src="/game/reference-page.js"></script></body></html>`;
const server = serve({ dist, isolate: true, extra: { "/reference.html": html } });
const browser = await Browser.from(args, { chrome: chromePath(args), width: WIDTH, height: HEIGHT, reverse: [server.port] });

/** The mean red of source column `x` (0..191) down the image's middle rows. */
const column = (img: Image, x: number) => {
  const sx = Math.floor(640 + (x + 0.5) * (640 / 192));
  let s = 0;
  for (let y = 200; y < 520; y++) s += img.rgba[(y * img.width + sx) * 4]!;
  return s / 320;
};
/** The mean red of a swatch's middle (Frame's boxes: 40–120, 200–280, 360–440 across, 180–260 down). */
const swatch = (img: Image, left: number) => {
  let s = 0;
  for (let y = 200; y < 240; y++) for (let x = left + 20; x < left + 60; x++) s += img.rgba[(y * img.width + x) * 4]!;
  return s / 1600;
};
/** Source columns of alpha 64, 128 and 191, each with its swatch's left edge. */
const MIDDLE: ReadonlyArray<[number, number, number]> = [[48, 64, 40], [96, 128, 200], [143, 191, 360]];
const inSwatch = (x: number, y: number) => y >= 176 && y < 264 && MIDDLE.some(([, , l]) => x >= l - 4 && x < l + 84);

/** The checks on one host's frame; `host` names it in each line. */
const judge = (img: Image, host: string) => {
  const ramp: Record<string, number> = {};
  for (const [x, want] of [[0, 0], [191, 255]] as const) {
    const got = +column(img, x).toFixed(1);
    ramp[`alpha ${want}`] = got;
    check(Math.abs(got - want) <= 10, `${host}: alpha ${want} reads ${got} at source column ${x}, want ${want} ± 10`);
  }
  for (const [x, alpha, left] of MIDDLE) {
    const got = +column(img, x).toFixed(1);
    const want = +swatch(img, left).toFixed(1);
    ramp[`alpha ${alpha}`] = got;
    ramp[`swatch ${alpha}`] = want;
    check(Math.abs(got - want) <= 10, `${host}: alpha ${alpha} reads ${got} at source column ${x}, its swatch ${want} ± 10`);
  }
  let left = 0;
  for (let y = 0; y < img.height; y++) for (let x = 0; x < 600; x++) if (!inSwatch(x, y)) left = Math.max(left, img.rgba[(y * img.width + x) * 4]!);
  check(left <= 8, `${host}: the half without the image is black outside the swatches (brightest ${left})`);
  return ramp;
};

const log: string[] = [];
const result: Record<string, unknown> = {};
let failures = 0;
const check = (ok: boolean, what: string) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) failures++;
};
try {
  const tab = await browser.open("about:blank", log);
  await tab.send("Emulation.setDeviceMetricsOverride", { width: WIDTH, height: HEIGHT, deviceScaleFactor: 1, mobile: false });
  await tab.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/reference.html?project=${encodeURIComponent(readFileSync(join(project, "project.ogt"), "utf8"))}` });
  await tab.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 90_000);
  const failed = await tab.eval<string | undefined>("globalThis.__og_shell_error__");
  if (failed) throw new Error(`the engine did not start: ${failed}`);
  result["renderer"] = await tab.eval<string>("__og_shell__.tier");
  const scheduled = await tab.eval<{ scheduled?: number } | null>(`__og_shell__.call("probe", "commands", "0.2:keyed")`);
  if (!scheduled?.scheduled) throw new Error(`keyed was not scheduled: ${JSON.stringify(scheduled)}`);
  // Captures at ticks 30 and 40 (the native run's screenshot is at 30); the
  // browser's decoder answers asynchronously, so the later one is judged.
  const shots = await tab.eval<Array<{ png: string; tick: number } | null>>(`(async () => {
    const shell = globalThis.__og_shell__;
    const shots = [];
    for (let tick = 0; tick < 120 && shots.length < 2;) {
      const answer = await shell.call("frame", 1000 / ${TICK_RATE});
      if (!answer || answer.ok === false) throw new Error("frame refused at tick " + tick + ": " + JSON.stringify(answer));
      tick = answer.tick ?? tick + 1;
      if (tick < 30 || tick % 10 !== 0) continue;
      let shot = await shell.probe("capture", { width: ${WIDTH}, height: ${HEIGHT} });
      for (let again = 0; !shot && again < 240; again++) {
        await new Promise((r) => setTimeout(r, 16));
        shot = await shell.probe("capture", { width: ${WIDTH}, height: ${HEIGHT} });
      }
      shots.push(shot);
    }
    return shots;
  })()`);
  result["engine"] = (await tab.eval<string[]>("globalThis.__og_log__ ?? []")).slice(-30);
  const shot = shots.at(-1);
  if (!shot?.png) throw new Error(`no capture: ${JSON.stringify(shots).slice(0, 200)}`);
  const bytes = Uint8Array.from(atob(shot.png), (c) => c.charCodeAt(0));
  writeFileSync(join(out, "keyed.png"), bytes);
  const img = decodePng(bytes);
  result["web"] = judge(img, "web");
  const refs = join(project, "references");
  const against = join(refs, RECORDING, "keyed.png");
  if (!existsSync(against)) throw new Error(`no native frame at ${against.slice(repo.length + 1)}`);
  result["native"] = { against: against.slice(repo.length + 1), ...judge(decodePng(new Uint8Array(readFileSync(against))), "native") };
} catch (e) {
  failures++;
  result["error"] = String(e instanceof Error ? e.stack : e);
  console.log(`FAIL ${result["error"]}`);
} finally {
  await browser.close();
  server.stop();
}
result["log"] = log.slice(-40);
writeFileSync(join(out, "video-alpha.json"), JSON.stringify(result, null, 2) + "\n");
console.log(`${failures === 0 ? "PASS" : "FAIL"} video-alpha (web): ${join(out, "video-alpha.json")}`);
process.exit(failures === 0 ? 0 : 1);
