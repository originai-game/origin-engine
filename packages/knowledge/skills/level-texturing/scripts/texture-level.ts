#!/usr/bin/env bun
// Textures a level from views an image model paints: `og bake projection`
// does the geometry (plan, clay, state, composite, projection, export); this
// script builds each view's prompt and calls the model. One view at a time,
// so each sees everything painted before it. The run directory is the whole
// state: --resume continues after the last projected view.
//
//   bun texture-level.ts --level level.glb --run runs/village --style "materials..." \
//     [--roof-style "..."] [--light overcast|indoor] [--views 36] [--aerial 6] [--roofs 12] [--holes 8] [--size 4096] \
//     [--model qwen-image-2.1-pro] [--provider handoff|http|dashscope] [--endpoint <url>] [--resume] [--dry]
//
// `handoff` (the default) calls no model itself: it stops at the first view
// that needs painting, prints one JSON line
// {"paint": {view, state, prompt, painted, model}} and exits 3. Whoever holds
// the image model (an agent's image-editing tool, a person) paints `state`
// with that model and prompt, saves the result to `painted`, and runs this
// again with --resume: the view is projected and the next one printed.
// `http` posts to an OpenAI-compatible image endpoint, `<endpoint>/v1/images/generations`
// (--endpoint, else OG_IMAGE_ENDPOINT), with OG_IMAGE_API_KEY as the bearer
// token and the state image as a data URL in `image`. `dashscope` calls
// Alibaba Model Studio directly with DASHSCOPE_API_KEY. --dry writes every
// prompt and stops before any model call. `agent` and `gateway` are the
// earlier names of handoff and http, still read from a run's settings.json.
import { parseArgs } from "node:util";
import { existsSync, mkdirSync, readFileSync, writeFileSync, appendFileSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";

const { values: a } = parseArgs({
  options: {
    level: { type: "string" }, run: { type: "string" }, style: { type: "string" },
    "roof-style": { type: "string", default: "old terracotta clay roof tiles in neat rows running down the slope, weathered" },
    views: { type: "string", default: "36" }, aerial: { type: "string", default: "0" }, roofs: { type: "string", default: "0" },
    holes: { type: "string", default: "0" }, size: { type: "string", default: "4096" }, spacing: { type: "string", default: "4" },
    model: { type: "string", default: "qwen-image-2.1-pro" }, provider: { type: "string" }, endpoint: { type: "string" },
    og: { type: "string", default: process.env.OG_BIN ?? "og" }, resume: { type: "boolean" }, dry: { type: "boolean" },
    limit: { type: "string" }, light: { type: "string", default: "overcast" },
  },
});
if (!a.run || (!a.resume && (!a.level || !a.style))) {
  console.error("usage: texture-level.ts --level <glb> --run <dir> --style <materials> [options] | --run <dir> --resume");
  process.exit(2);
}
const run = resolve(a.run);

function og(...args: string[]): any {
  const p = Bun.spawnSync([a.og!, "bake", "projection", ...args, "--run", run], { stderr: "pipe" });
  if (p.exitCode !== 0) throw new Error(`og bake projection ${args[0]}: ${p.stderr.toString().trim()}`);
  return JSON.parse(p.stdout.toString().trim().split("\n").pop()!);
}

// ---- prompts (trompe's providers/models.ts, instruction style) ---------------
const ADDITIONS = ["awnings", "market stalls", "shop fronts", "signs", "lamps", "plants", "crates", "barrels", "benches", "furniture", "people", "cars"];
function forbidden(parts: string) {
  const have = new Set(parts.toLowerCase().split(/[^a-z]+/).map((w) => w.replace(/s$/, "")));
  return ADDITIONS.filter((x) => !x.split(" ").some((w) => have.has(w.replace(/s$/, "")))).join(", ");
}
// Lighting the engine adds back later, so the paint carries none of its own.
const LIGHTS: Record<string, string> = {
  overcast: "soft, even overcast daylight with no cast shadows and no sunlit patches",
  indoor: "soft, even diffuse interior light with no cast shadows, no light shafts and no bright patches",
};
export function prompt(mode: string, parts: string, floor: boolean, look: string, roofLook: string, light: string, roofs: boolean): string {
  const LIGHT = LIGHTS[light] ?? LIGHTS.overcast!;
  const surfaces = `Walls may show flat details such as windows, doors, shutters and stonework, but nothing sticks out of them and nothing new is added: no ${forbidden(parts)}.` +
    (floor ? " The ground stays clear: bare paving." : "") +
    // Without it a hole view paints a roof in the walls' material.
    (roofs ? ` Roofs are ${roofLook}.` : "");
  if (mode === "roof") {
    return `Paint only the plain gray roof surfaces as ${roofLook}, following each roof's exact outline and slope, with ${LIGHT}. ` +
      "Everything else stays exactly as it is in the image. Add nothing on the roofs: no chimneys, antennas, windows, solar panels or plants.";
  }
  if (mode === "first") {
    return `Turn this untextured gray 3D render into a photorealistic photo of ${look}. Every gray shape keeps its exact outline and becomes the real surface it is` +
      `${parts ? `: ${parts}` : ""}. Keep exactly the same camera, perspective and edges, with ${LIGHT}, and give the surfaces realistic materials. ${surfaces}`;
  }
  return `This image is partly a photo of ${look} and partly plain gray untextured 3D render. Keep the camera, perspective, composition and every textured area exactly as they are. ` +
    `Paint the plain gray surfaces with photorealistic materials that continue the textured surfaces around them seamlessly, keeping their exact outlines` +
    `${parts ? ` (they show: ${parts})` : ""}, with ${LIGHT}. ${surfaces}`;
}

// ---- providers ---------------------------------------------------------------
async function paint(statePng: string, text: string, seed: number): Promise<Uint8Array> {
  const dataUrl = "data:image/png;base64," + Buffer.from(readFileSync(statePng)).toString("base64");
  if (provider === "dashscope") {
    const key = process.env.DASHSCOPE_API_KEY ?? process.env.DASHSCOPE_SINGAPORE_API_KEY;
    const base = process.env.DASHSCOPE_BASE ?? "https://dashscope-intl.aliyuncs.com/api/v1";
    const r = await fetch(`${base}/services/aigc/multimodal-generation/generation`, {
      method: "POST", headers: { Authorization: `Bearer ${key}`, "Content-Type": "application/json" },
      body: JSON.stringify({ model: a.model, input: { messages: [{ role: "user", content: [{ image: dataUrl }, { text }] }] },
        parameters: { n: 1, prompt_extend: false, watermark: false, seed } }),
    });
    const j: any = await r.json();
    const url = j?.output?.choices?.[0]?.message?.content?.find((c: any) => c.image)?.image;
    if (!r.ok || !url) throw new Error(`dashscope ${r.status}: ${JSON.stringify(j).slice(0, 300)}`);
    return new Uint8Array(await (await fetch(url)).arrayBuffer());
  }
  const base = (a.endpoint ?? settings.endpoint ?? process.env.OG_IMAGE_ENDPOINT ?? "").replace(/\/$/, "");
  if (!base) throw new Error("http: no endpoint (--endpoint or OG_IMAGE_ENDPOINT)");
  const r = await fetch(`${base}/v1/images/generations`, {
    method: "POST", headers: { Authorization: `Bearer ${process.env.OG_IMAGE_API_KEY}`, "Content-Type": "application/json" },
    body: JSON.stringify({ model: a.model, prompt: text, image: dataUrl, n: 1, seed, prompt_extend: false }),
  });
  const j: any = await r.json();
  const d = j?.data?.[0];
  if (!r.ok || !d) throw new Error(`http ${r.status}: ${JSON.stringify(j).slice(0, 300)}`);
  return d.b64_json ? Buffer.from(d.b64_json, "base64") : new Uint8Array(await (await fetch(d.url)).arrayBuffer());
}

// ---- the run -----------------------------------------------------------------
const logPath = join(run, "log.jsonl");
const settingsPath = join(run, "settings.json");
if (!a.resume) {
  mkdirSync(run, { recursive: true });
  // A fresh run never projects a picture painted for an earlier one.
  rmSync(join(run, "views"), { recursive: true, force: true });
  rmSync(logPath, { force: true });
  const init = og("init", a.level!, "--size", a.size!);
  const plan = og("plan", "--views", a.views!, "--aerial", a.aerial!, "--roofs", a.roofs!, "--spacing", a.spacing!);
  writeFileSync(settingsPath, JSON.stringify({ style: a.style, roofStyle: a["roof-style"], light: a.light, model: a.model, provider: a.provider ?? "handoff", endpoint: a.endpoint, holes: +a.holes! }, null, 1));
  console.log(`init: ${init.run.triangles} triangles, ${init.run.texels} texels; plan: ${plan.views} views (${plan.ground} ground, ${plan.roof} roof, ${plan.aerial} aerial)`);
}
const settings = JSON.parse(readFileSync(settingsPath, "utf8"));
// Runs started before the rename say agent or gateway.
const RENAMED: Record<string, string> = { agent: "handoff", gateway: "http" };
const provider: string = RENAMED[a.provider ?? settings.provider] ?? a.provider ?? settings.provider;
if (!["handoff", "http", "dashscope"].includes(provider)) {
  console.error(`--provider ${provider}: one of handoff, http, dashscope`);
  process.exit(2);
}
const done = new Set<number>(existsSync(logPath) ? readFileSync(logPath, "utf8").trim().split("\n").filter(Boolean).map((l) => JSON.parse(l).view) : []);
const t0 = Date.now();
let calls = 0;
const limit = a.limit ? +a.limit : Infinity;

async function doView(k: number) {
  const p = og("prepare", "--view", String(k));
  const text = prompt(p.mode, p.parts, p.floorShare > 0.5, settings.style, settings.roofStyle, settings.light ?? "overcast", p.roofShare > 0.05);
  writeFileSync(join(run, "views", String(k), "prompt.txt"), text);
  if (p.skip || a.dry) {
    appendFileSync(logPath, JSON.stringify({ view: k, kind: p.kind, mode: p.mode, skipped: p.skip, dry: !!a.dry }) + "\n");
    return;
  }
  const t = Date.now();
  const painted = join(run, "views", String(k), "painted.png");
  let img: Uint8Array | undefined;
  if (provider === "handoff") {
    if (!existsSync(painted)) {
      console.log(JSON.stringify({ paint: { view: k, kind: p.kind, mode: p.mode, state: p.state, prompt: text, painted, model: settings.model } }));
      process.exit(3);
    }
    img = readFileSync(painted);
  }
  for (let attempt = 0; attempt < 3 && !img; attempt++) {
    try { img = await paint(p.state, text, 7 + 1000 * attempt + k); } catch (e) { console.error(`view ${k}: ${(e as Error).message}`); }
  }
  if (!img) { appendFileSync(logPath, JSON.stringify({ view: k, failed: true }) + "\n"); return; }
  calls++;
  if (provider !== "handoff") writeFileSync(painted, img);
  const pr = og("project", "--view", String(k), "--image", painted);
  const rec = { view: k, kind: p.kind, mode: p.mode, model: settings.model, modelSeconds: (Date.now() - t) / 1000,
    unknown: p.unknownFraction, written: pr.written, left: pr.left, coverage: pr.coverage };
  appendFileSync(logPath, JSON.stringify(rec) + "\n");
  console.log(`view ${k} ${p.kind}/${p.mode}: ${rec.modelSeconds.toFixed(0)} s, ${pr.written} texels, coverage ${(pr.coverage * 100).toFixed(1)} %`);
}

const views: any[] = JSON.parse(readFileSync(join(run, "plan.json"), "utf8")).views;
for (let k = 0; k < views.length && calls < limit; k++) if (!done.has(k)) await doView(k);
const holeViews = views.filter((v) => v.kind === "fill").length;
for (let n = holeViews; n < settings.holes && calls < limit && !a.dry; n++) {
  const h = og("holes");
  if (h.view === null) break;
  await doView(h.view);
}
const ex = og("export");
console.log(`done: ${calls} model calls, ${((Date.now() - t0) / 60000).toFixed(1)} min, coverage ${(ex.coverage * 100).toFixed(1)} %, ${ex.glb}`);
