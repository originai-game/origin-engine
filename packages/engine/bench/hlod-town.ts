// What HLOD saves on a large scene, on the native host (docs/design.md
// §10.6): a town of `--side`² lots (40² by default, 1,156 of them built
// on, the rest streets; 20 m apart over 800 m; each building of walls, a
// roof, windows placed floor by floor, a door, and a tank or a mast with a
// ball on top) cooked
// twice, as `town.ogt` with an `hlod { size 80 }` block and as
// `town-plain.ogt` without one, and seen from three places:
//
//   street    eye height in the middle of the town: HLOD should cost nothing
//   overlook  60 m up at its edge, looking across it: the usual game view
//   far       500 m off its edge and 220 m up: the whole town in view
//   top       900 m straight above its middle (with --views only): every roof
//             faces the camera, where `far` sees them at a grazing angle
//
// Per scene and view: the frame's main-pass triangles and draw calls across
// every pass (the probe's dump_render), the proxies drawn and the models
// they replaced, and the time of a frame (engine.frame plus the GPU wait,
// og_node_wait_gpu) and of its CPU part alone, after the warm-up frames.
// One screenshot each, and one with `debugView lod` for the HLOD scene.
//
//   bun bench/hlod-town.ts [--side 40] [--kinds 24] [--views street,overlook,far] [--size 1920x1080]
//                          [--frames 120] [--warm 20] [--out DIR]
//
// `--kinds` is how many models the buildings are drawn from (24 by
// default; 0: every building its own). It decides what HLOD saves in draw
// calls: copies of one model are one instanced draw a pass already, while
// every proxy is a draw of its own in each pass that sees it, so a town of few kinds gains draws
// only where a proxy stands in for more kinds than it adds passes.
//
// The project is generated under target/evidence/hlod-town/ (not
// committed). Draws and triangles are the same on every adapter; frame
// times on a software adapter (lavapipe) measure the CPU, not a GPU.

import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => (argv.includes(name) ? argv[argv.indexOf(name) + 1]! : fallback);
const SIDE = Number(opt("--side", "40"));
const FRAMES = Number(opt("--frames", "120"));
const WARM = Number(opt("--warm", "20"));
/** How many kinds of building the town draws from; 0: every building its own. */
const KINDS = Number(opt("--kinds", "24"));
const work = join(repo, "target", "evidence", "hlod-town");
const project = join(work, `project-${SIDE}-${KINDS}`);
const out = opt("--out", join(work, "results"));
const [W, H] = opt("--size", "1920x1080").split("x").map(Number) as [number, number];

/** Metres between buildings. */
const PITCH = 20;
const SPAN = SIDE * PITCH;

// ── The town ────────────────────────────────────────────────────────────

/** A number in 0..1 from two integers and a seed. */
function hash(x: number, y: number, seed: number): number {
  let h = (Math.imul(x | 0, 0x8da6b343) ^ Math.imul(y | 0, 0xd8163841) ^ Math.imul(seed, 0x9e3779b1)) >>> 0;
  h = Math.imul(h ^ (h >>> 13), 0x85ebca6b) >>> 0;
  h = Math.imul(h ^ (h >>> 16), 0xc2b2ae35) >>> 0;
  return ((h ^ (h >>> 15)) >>> 0) / 4294967296;
}
const n3 = (v: number) => Math.round(v * 1000) / 1000;

/** Building kind `k`: its model text. */
function building(k: number): string {
  const r = (i: number) => hash(k, i, 7);
  const w = 8 + Math.round(r(0) * 6), d = 8 + Math.round(r(1) * 6);
  const floors = 2 + Math.floor(r(2) * r(2) * 9);
  const h = floors * 3;
  const wall = [0.35 + 0.5 * r(3), 0.3 + 0.45 * r(4), 0.25 + 0.4 * r(5)].map(n3);
  const roof = [0.15 + 0.4 * r(6), 0.1 + 0.2 * r(7), 0.08 + 0.2 * r(8)].map(n3);
  const lit = r(9) > 0.6;
  let t = `type model\nversion 1\n---\n# bench/hlod-town.ts: kind ${k}, ${w} x ${h} x ${d} m.\n`;
  t += `material "wall" {\n  color ${wall.join(" ")}\n  roughness 0.85\n}\n`;
  t += `material "roof" {\n  color ${roof.join(" ")}\n  roughness 0.6\n}\n`;
  t += `material "glass" {\n  color 0.05 0.09 0.14\n  roughness 0.15\n${lit ? "  emissive 0.9 0.7 0.35\n  emissiveIntensity 1.5\n" : ""}}\n`;
  t += `material "metal" {\n  color 0.5 0.52 0.55\n  roughness 0.35\n  metalness 1\n}\n`;
  t += `part "walls" {\n  material "wall"\n  position 0 ${h / 2} 0\n  boxGeometry {\n    width ${w}\n    height ${h}\n    depth ${d}\n  }\n}\n`;
  if (r(10) > 0.5) {
    // A hipped roof.
    t += `part "roof" {\n  material "roof"\n  position 0 ${h + 1.5} 0\n  quaternion 0 0.3826834 0 0.9238795\n  coneGeometry {\n    radius ${n3(Math.max(w, d) * 0.75)}\n    height 3\n    radialSegments 4\n  }\n}\n`;
  } else {
    // A flat roof with a parapet's slab and a tank.
    t += `part "roof" {\n  material "roof"\n  position 0 ${h + 0.2} 0\n  boxGeometry {\n    width ${w + 0.6}\n    height 0.4\n    depth ${d + 0.6}\n  }\n}\n`;
    t += `part "tank" {\n  material "metal"\n  position ${n3(w * 0.25)} ${h + 1.4} ${n3(-d * 0.2)}\n  cylinderGeometry {\n    radiusTop 0.9\n    radiusBottom 0.9\n    height 2\n    radialSegments 16\n  }\n}\n`;
  }
  // A mast with a ball: detail no proxy needs.
  t += `part "mast" {\n  material "metal"\n  position ${n3(-w * 0.3)} ${h + 2.5} ${n3(d * 0.3)}\n  cylinderGeometry {\n    radiusTop 0.05\n    radiusBottom 0.05\n    height 4\n    radialSegments 6\n  }\n}\n`;
  t += `part "ball" {\n  material "metal"\n  position ${n3(-w * 0.3)} ${h + 4.6} ${n3(d * 0.3)}\n  sphereGeometry {\n    radius 0.25\n  }\n}\n`;
  t += `part "door" {\n  material "metal"\n  position 0 1.1 ${n3(d / 2 + 0.06)}\n  boxGeometry {\n    width 1.4\n    height 2.2\n    depth 0.12\n  }\n}\n`;
  // Windows on the front and back, then the sides: one mesh each, placed.
  const along = (span: number) => Math.max(2, Math.floor(span / 2.6));
  const rows = (name: string, span: number, far: number, turned: boolean) => {
    let p = `part "${name}" {\n  material "glass"\n  boxGeometry {\n    width ${turned ? 0.12 : 1.2}\n    height 1.5\n    depth ${turned ? 1.2 : 0.12}\n  }\n`;
    const n = along(span);
    for (let f = 0; f < floors; f++) {
      for (let i = 0; i < n; i++) {
        const a = n3(-span / 2 + (span * (i + 0.5)) / n), y = f * 3 + 1.7;
        if (f === 0 && !turned && Math.abs(a) < 1.2) continue;
        for (const side of [-1, 1]) {
          const at = turned ? `${n3(side * (far / 2 + 0.06))} ${y} ${a}` : `${a} ${y} ${n3(side * (far / 2 + 0.06))}`;
          p += `  at {\n    position ${at}\n  }\n`;
        }
      }
    }
    return p + "}\n";
  };
  t += rows("windows", w, d, false) + rows("side-windows", d, w, true);
  return t;
}

/** The town's scene, with the HLOD block or without, and the kinds it places. */
function town(hlod: boolean, kinds: Set<number>): string {
  let t = `type scene\nversion 1\n---\nname "Town"\n`;
  if (hlod) t += "hlod {\n  size 80\n}\n";
  t += `entity "ground" {\n  model "models/ground.ogt"\n  Transform {\n    position ${SPAN / 2} -0.1 ${SPAN / 2}\n  }\n}\n`;
  for (let j = 0; j < SIDE; j++) {
    for (let i = 0; i < SIDE; i++) {
      // Every sixth line is a street.
      if (i % 6 === 5 || j % 6 === 5) continue;
      const k = KINDS > 0 ? Math.floor(hash(i, j, 1) * KINDS) : j * SIDE + i;
      kinds.add(k);
      const x = n3(i * PITCH + 10 + (hash(i, j, 2) - 0.5) * 4), z = n3(j * PITCH + 10 + (hash(i, j, 3) - 0.5) * 4);
      const turn = hash(i, j, 4) > 0.5 ? "0 0.7071068 0 0.7071068" : "0 0 0 1";
      t += `entity "b-${i}-${j}" {\n  model "models/b${k}.ogt"\n  static true\n  Transform {\n    position ${x} 0 ${z}\n    quaternion ${turn}\n  }\n}\n`;
    }
  }
  return t;
}

/** Writes and cooks the project once; the cook's HLOD lines. */
function stage(): string {
  const mark = join(project, ".staged-1");
  const notes = join(project, "cook.txt");
  if (existsSync(mark)) return readFileSync(notes, "utf8");
  mkdirSync(join(project, "assets", "models"), { recursive: true });
  mkdirSync(join(project, "assets", "scenes"), { recursive: true });
  writeFileSync(join(project, "og-project.json"), '{"version":1,"name":"HLOD town bench","engine":"0.9.0"}\n');
  writeFileSync(join(project, "project.ts"), 'import {defineProject} from "@origingame/engine/author"; export default defineProject({id:"hlod-town",entry:"main.ts",tick:60,seed:1});\n');
  writeFileSync(join(project, "main.ts"), '// Cooked for its assets only: bench/hlod-town.ts drives the world.\nimport { og } from "@origingame/engine";\n\nexport default og.game();\n');
  writeFileSync(
    join(project, "assets", "models", "ground.ogt"),
    `type model\nversion 1\n---\nmaterial "ground" {\n  color 0.33 0.36 0.3\n  roughness 0.95\n}\npart "ground" {\n  material "ground"\n  boxGeometry {\n    width ${SPAN + 4000}\n    height 0.2\n    depth ${SPAN + 4000}\n  }\n}\n`,
  );
  const kinds = new Set<number>();
  writeFileSync(join(project, "assets", "scenes", "town.ogt"), town(true, kinds));
  writeFileSync(join(project, "assets", "scenes", "town-plain.ogt"), town(false, kinds));
  for (const k of kinds) writeFileSync(join(project, "assets", "models", `b${k}.ogt`), building(k));
  const ogBin = process.env.OG_BIN ?? join(pkg, "native", `og.${process.platform}-${process.arch}`);
  const t0 = performance.now();
  // Without the shared cook cache: the time is a cold cook's.
  const cooked = Bun.spawnSync([ogBin, "cook", project, "--report"], { env: { ...process.env, OG_ENGINE_PACKAGE: pkg, OG_COOK_CACHE: "off" } });
  const text = new TextDecoder().decode(cooked.stdout) + new TextDecoder().decode(cooked.stderr);
  if (cooked.exitCode !== 0) throw new Error(text);
  // The tree and the models left out, from the report; the cook's own total.
  const report = JSON.parse(readFileSync(join(project, "og-out", "cook-report.json"), "utf8")) as { assets: { hlod?: string[] }[] };
  const lines = [`${kinds.size} kinds of building`, ...report.assets.flatMap((a) => a.hlod ?? []).filter((n) => n.startsWith("hlod:"))];
  lines.push(...text.split("\n").filter((l) => /^og cook: \d+ assets/.test(l)));
  lines.push(`both scenes cooked in ${((performance.now() - t0) / 1000).toFixed(1)} s`);
  writeFileSync(notes, lines.join("\n") + "\n");
  writeFileSync(mark, "");
  return lines.join("\n") + "\n";
}

// ── One run ─────────────────────────────────────────────────────────────

type Vec = { x: number; y: number; z: number };
const VIEWS: Record<string, { eye: Vec; at: Vec }> = {
  street: { eye: { x: SPAN / 2 + 10, y: 1.7, z: SPAN / 2 + 10 }, at: { x: SPAN, y: 6, z: SPAN / 2 + 10 } },
  overlook: { eye: { x: -40, y: 60, z: SPAN / 2 }, at: { x: SPAN * 0.6, y: 0, z: SPAN / 2 } },
  far: { eye: { x: -500, y: 220, z: SPAN / 2 - 150 }, at: { x: SPAN / 2, y: 0, z: SPAN / 2 } },
  top: { eye: { x: SPAN / 2, y: 900, z: SPAN / 2 + 1 }, at: { x: SPAN / 2, y: 0, z: SPAN / 2 } },
};
/** The views a run without --views takes. */
const DEFAULT_VIEWS = "street,overlook,far";

type Run = {
  scene: string; view: string; size: string; tier: string; frames: number;
  triangles: number; draws: number;
  hlod: { scenes: number; proxies: number; unloaded: number; replaced: number };
  ms: { p50: number; p95: number; mean: number };
  cpuMs: { p50: number; p95: number };
  renderMs: number;
};

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
const q = (s: number[], p: number) => s[Math.min(s.length - 1, Math.floor(p * s.length))]!;
const r3 = (v: number) => Math.round(v * 1000) / 1000;

function one(scene: string, view: string): Run {
  loadNativeHost();
  const game = og.game({
    start(world: World) {
      must(world.instantiate(must(og.load.scene(world, `scenes/${scene}.ogt`))));
      const sun = must(world.spawn({ name: "sun" }));
      world.insert(sun, og.DirectionalLight, { intensity: 3 });
      world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.5, y: -0.6, z: -0.4 }) });
    },
  });
  const engine = must(createEngine(game as GameToken, { root: project }));
  const attached = JSON.parse(native().og_node_attach_offscreen(engine.id, W, H)) as { ok: boolean; value?: string; reason?: string };
  if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
  must(engine.advance(1));
  const w = engine.world;
  must(w.resource(og.RenderSettings)).sky = true;
  const v = VIEWS[view]!;
  const cam = must(w.resource(og.Camera));
  cam.position = v.eye;
  cam.quaternion = og.transform.lookAt(v.eye, v.at);
  cam.fov = 60;
  cam.far = 6000;
  const wait = () => {
    const r = JSON.parse(native().og_node_wait_gpu(engine.id)) as { ok: boolean; reason?: string };
    if (!r.ok) throw new Error(r.reason);
  };
  const times: number[] = [], cpu: number[] = [];
  for (let i = -WARM; i < FRAMES; i++) {
    const t0 = performance.now();
    const f = engine.frame(1000 / 60);
    if (!f.ok) throw new Error(f.reason);
    const t1 = performance.now();
    wait();
    const t2 = performance.now();
    if (i >= 0) { times.push(t2 - t0); cpu.push(t1 - t0); }
  }
  // The dump runs as the next tick starts: the frame just timed.
  engine.probe.commands("0:dump_render:r.json");
  must(engine.advance(1));
  const dump = JSON.parse(engine.takeOutputs().find((x) => x.action === "dump_render")!.data) as { triangles: number; draws: number; renderMs: number; hlod: Run["hlod"] };
  const shot = (name: string) => {
    engine.probe.commands(`0:screenshot:${name}.png`);
    must(engine.advance(1));
    for (const o of engine.takeOutputs()) if (o.kind === "png") writeFileSync(join(out, o.file), Uint8Array.from(atob(o.data), (c) => c.charCodeAt(0)));
  };
  shot(`${scene}-${view}`);
  if (scene === "town") {
    engine.probe.commands("0:debug:lod");
    must(engine.advance(2));
    shot(`${scene}-${view}-lod`);
  }
  const sorted = [...times].sort((a, b) => a - b), cs = [...cpu].sort((a, b) => a - b);
  const run: Run = {
    scene, view, size: `${W}x${H}`, tier: attached.value!, frames: FRAMES,
    triangles: dump.triangles, draws: dump.draws, hlod: dump.hlod,
    ms: { p50: r3(q(sorted, 0.5)), p95: r3(q(sorted, 0.95)), mean: r3(times.reduce((a, b) => a + b, 0) / times.length) },
    cpuMs: { p50: r3(q(cs, 0.5)), p95: r3(q(cs, 0.95)) },
    renderMs: dump.renderMs,
  };
  engine.destroy();
  return run;
}

// ── Driver ──────────────────────────────────────────────────────────────

mkdirSync(out, { recursive: true });
const cookNotes = stage();
if (argv.includes("--scene")) {
  process.stdout.write(JSON.stringify(one(opt("--scene", "town"), opt("--view", "far"))) + "\n");
} else {
  // Each run in its own process, so one scene's models are not the other's.
  const views = opt("--views", DEFAULT_VIEWS).split(",").filter((v) => v in VIEWS);
  const runs: Run[] = [];
  for (const view of views) {
    for (const scene of ["town-plain", "town"]) {
      const r = Bun.spawnSync(["bun", fileURLToPath(import.meta.url), "--scene", scene, "--view", view, "--side", String(SIDE), "--kinds", String(KINDS), "--warm", String(WARM), "--frames", String(FRAMES), "--size", `${W}x${H}`, "--out", out], { cwd: pkg });
      if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
      const run = JSON.parse(new TextDecoder().decode(r.stdout).trim().split("\n").pop()!) as Run;
      runs.push(run);
      process.stderr.write(`${view} ${scene}: ${run.draws} draws, ${run.triangles} triangles, p50 ${run.ms.p50} ms\n`);
    }
  }
  writeFileSync(join(out, "hlod-town.json"), JSON.stringify(runs, null, 1) + "\n");
  // What the proxies weigh in the bundle: models and textures.
  const cooked = join(project, "cooked", "scenes");
  const bytes = (match: (name: string) => boolean) => readdirSync(cooked).filter((n) => n.includes(".hlod") && match(n)).reduce((sum, n) => sum + statSync(join(cooked, n)).size, 0);
  const mib = (n: number) => (n / 1048576).toFixed(1);
  const textures = bytes((n) => /-(base|surface|emissive)\./.test(n)), all = bytes(() => true);
  const pct = (a: number, b: number) => `${b <= a ? "−" : "+"}${Math.abs(Math.round((1 - b / a) * 100))} %`;
  const lines = [
    `HLOD on a town of ${SIDE}² lots (bun bench/hlod-town.ts, ${W}x${H}, ${runs[0]!.tier}; a frame is engine.frame plus the GPU wait, ${FRAMES} frames after ${WARM} warm-up).`,
    "",
    ...cookNotes.trim().split("\n"),
    `the proxies in the bundle: ${mib(all - textures)} MiB of models and ${mib(textures)} MiB of textures`,
    "",
    "| View | Scene | Draws | Triangles | Proxies | Models replaced | Frame p50 ms | Frame p95 ms | CPU p50 ms |",
    "|---|---|---|---|---|---|---|---|---|",
    ...runs.map((r) => `| ${r.view} | ${r.scene === "town" ? "HLOD" : "plain"} | ${r.draws} | ${r.triangles} | ${r.hlod.proxies} | ${r.hlod.replaced} | ${r.ms.p50} | ${r.ms.p95} | ${r.cpuMs.p50} |`),
    "",
    "| View | Draws | Triangles | Frame p50 | CPU p50 |",
    "|---|---|---|---|---|",
    ...views.map((view) => {
      const a = runs.find((r) => r.view === view && r.scene === "town-plain")!, b = runs.find((r) => r.view === view && r.scene === "town")!;
      return `| ${view} | ${pct(a.draws, b.draws)} | ${pct(a.triangles, b.triangles)} | ${pct(a.ms.p50, b.ms.p50)} | ${pct(a.cpuMs.p50, b.cpuMs.p50)} |`;
    }),
    "",
  ];
  writeFileSync(join(out, "hlod-town.md"), lines.join("\n"));
  process.stdout.write(lines.join("\n"));
}
