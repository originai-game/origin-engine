// An authored LDR sky on the native host (render §4, Matching an authored
// sky's exposure). og.load.environmentSunRotation: a cooked sky with a
// small sun disc, turned so the sun stands at an asked azimuth around a view.
// The answer is checked against og.load.environmentSun at that rotation, the
// engine's own turn (og-runtime's Quat about +Y), not this module's maths.
//
// Ways it could fail, and the check that catches each:
// - the turn's sign or the view's right mirrored, so the sun lands on the
//   other side → the measured azimuth, for asymmetric azimuths and views
// - a pitched view's vertical part counted → the pitched view's azimuth
// - an elevation reported that is not the sky's → `elevation` against the
//   turned direction's
// - a map with no sun or a vertical view answered with a rotation → refused
// The exposure recipe's claims, on a GPU: auto exposure meters the lit
// subject alone (the sky is drawn as given and not metered), so albedo 0.4
// and 0.8 come out alike; a fixed exposure with toneMapping none at
// intensity 1 draws a diffuse surface at its albedo times the sky's light.
import { afterAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import type { EnvironmentMap, World } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { encodePng } from "../src/node/png.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";
import { gpuTest } from "./support/gpu.ts";
import { ogBin } from "./support/og-bin.ts";
import { decodeCapture, pixel } from "./support/png.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

/** A 256 × 128 blue sky, its sun a white disc 2 px across at column 200, row 34 (about 42° up). */
function sky(): Uint8Array {
  const width = 256, height = 128;
  const rgba = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const sun = Math.hypot(x - 200, y - 34) < 2;
      const i = 4 * (y * width + x);
      rgba.set(sun ? [255, 255, 255, 255] : [80 + (y >> 2), 120 + (y >> 2), 180, 255], i);
    }
  }
  return encodePng({ width, height, rgba });
}

// At module level: a hook's 5 s would not cover the cook on a loaded machine.
const root = (() => {
  const base = join(repo, ".og", "tmp");
  mkdirSync(base, { recursive: true });
  const dir = mkdtempSync(join(base, "environment-sun-"));
  mkdirSync(join(dir, "assets", "env"), { recursive: true });
  mkdirSync(join(dir, "src"));
  writeFileSync(join(dir, "src", "main.ts"), "export {};\n");
  writeFileSync(join(dir, "project.ogt"), 'type project\nversion 1\n---\nname "Sun"\nid "sun"\nengine "0.9.0"\nentry "src/main.ts"\ntick 60\nseed 1\n');
  writeFileSync(join(dir, "assets", "env", "sky.png"), sky());
  const env = (sun: string) => `type environment\nversion 1\n---\nsource "sky.png"\nsize 16\nsamples 16\nrotation 30\n${sun}`;
  writeFileSync(join(dir, "assets", "env", "sky.ogt"), env(""));
  writeFileSync(join(dir, "assets", "env", "kept.ogt"), env("sun keep\n"));
  // A uniform grey sky, sRGB 128 (linear 0.216), with nothing to extract and no expansion.
  const grey = new Uint8Array(256 * 128 * 4);
  for (let i = 0; i < grey.length; i += 4) grey.set([128, 128, 128, 255], i);
  writeFileSync(join(dir, "assets", "env", "grey.png"), encodePng({ width: 256, height: 128, rgba: grey }));
  writeFileSync(join(dir, "assets", "env", "grey.ogt"), 'type environment\nversion 1\n---\nsource "grey.png"\nsize 32\nsamples 64\nsun keep\nexpand 0\n');
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", dir]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  return dir;
})();

afterAll(() => rmSync(root, { recursive: true, force: true }));

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function world(): World {
  const made = createEngine(og.game({}) as GameToken, { root });
  if (!made.ok) throw new Error(made.reason);
  return made.value.world as unknown as World;
}

type V = { x: number; y: number; z: number };

/** Where `d` stands around `view`: radians from straight ahead, positive to the view's right. */
function azimuthOf(d: V, view: V): number {
  const f = Math.hypot(view.x, view.z);
  const fx = view.x / f, fz = view.z / f;
  return Math.atan2(d.x * -fz + d.z * fx, d.x * fx + d.z * fz);
}

const wrap = (a: number) => Math.atan2(Math.sin(a), Math.cos(a));

test("the rotation answered puts the sun at the asked azimuth, as environmentSun turns it", () => {
  const w = world();
  const map = must(og.load.environment(w, "env/sky.ogt"));
  const unturned = must(og.load.environmentSun(w, map, 0, 1));
  expect(unturned.intensity).toBeGreaterThan(0);
  const views: V[] = [{ x: 0, y: 0, z: -1 }, { x: 1, y: 0, z: 0.5 }, { x: -0.3, y: -0.6, z: 0.8 }];
  const azimuths = [0, 1.2, -0.7, -2.4, Math.PI];
  for (const view of views) {
    for (const azimuth of azimuths) {
      const turn = must(og.load.environmentSunRotation(w, map, view, azimuth));
      expect(Math.abs(turn.rotation)).toBeLessThanOrEqual(Math.PI);
      const sun = must(og.load.environmentSun(w, map, turn.rotation, 1));
      expect(Math.abs(wrap(azimuthOf(sun.direction, view) - azimuth))).toBeLessThan(1e-4);
      for (const k of ["x", "y", "z"] as const) expect(Math.abs(turn.direction[k] - sun.direction[k])).toBeLessThan(1e-5);
      expect(Math.abs(turn.elevation - Math.asin(sun.direction.y))).toBeLessThan(1e-5);
    }
  }
  // The disc is about 42° up; no turn moves it.
  expect(Math.abs(must(og.load.environmentSunRotation(w, map, views[0]!, 0)).elevation - (Math.PI / 2) * (1 - 34.5 / 64))).toBeLessThan(0.05);
});

test("a map without a sun, or a vertical view, is refused with the reason", () => {
  const w = world();
  const kept = must(og.load.environment(w, "env/kept.ogt")) as EnvironmentMap;
  const none = og.load.environmentSunRotation(w, kept, { x: 0, y: 0, z: -1 }, 0);
  expect(!none.ok && none.reason).toContain("holds no sun");
  const map = must(og.load.environment(w, "env/sky.ogt"));
  const down = og.load.environmentSunRotation(w, map, { x: 0, y: -1, z: 0 }, 0);
  expect(!down.ok && down.reason).toContain("straight up or down");
});

/** A wall of `albedo` in the grey sky, drawn with `auto` exposure and `tone`; its centre pixel's sRGB grey and the sky's. */
function wallIn(auto: boolean, tone: string, albedo: number): { wall: number; sky: number } {
  const game = og.game({
    start(w: World) {
      const env = must(w.resource(og.Environment));
      env.map = must(og.load.environment(w, "env/grey.ogt"));
      env.background = true;
      env.intensity = 1;
      const rs = must(w.resource(og.RenderSettings));
      rs.autoExposure = auto;
      rs.toneMapping = tone as never;
      // A light of 0 in place of the renderer's default sun: the sky lights alone.
      w.insert(must(w.spawn({ name: "sun" })), og.DirectionalLight, { intensity: 0 });
      const wall = must(w.spawn({ name: "wall", position: { x: 0, y: 0, z: -6 } }));
      w.insert(wall, og.Mesh, {
        geometry: must(og.render.planeGeometry(w, { width: 4, height: 4 })),
        material: must(og.render.standardMaterial(w, { color: { r: albedo, g: albedo, b: albedo }, roughness: 1, metalness: 0 })),
      });
    },
  });
  const made = createEngine(game as GameToken, { root });
  if (!made.ok) throw new Error(made.reason);
  const attached = JSON.parse(native().og_node_attach_offscreen(made.value.id, 160, 90)) as { ok: boolean; reason?: string };
  if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
  for (let i = 0; i < 30; i++) must(made.value.advance(1));
  const img = decodeCapture(made.value.probe.capture(160, 90));
  return { wall: pixel(img, 80, 45)[1], sky: pixel(img, 3, 3)[1] };
}

const srgb = (c: number) => Math.round(255 * (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055));

gpuTest("auto exposure meters the subject alone; a fixed exposure without a curve keeps the sky's brightness", () => {
  // The sky is drawn as its source in every case.
  const auto = [0.4, 0.8].map((a) => wallIn(true, "aces", a));
  for (const f of auto) expect(Math.abs(f.sky - 128)).toBeLessThanOrEqual(2);
  // Auto exposure: albedo 0.4 and 0.8 meter to the same grey, whatever the sky.
  expect(Math.abs(auto[0]!.wall - auto[1]!.wall)).toBeLessThanOrEqual(4);
  // Fixed exposure 1, toneMapping none, intensity 1: albedo × the sky's linear 0.216.
  for (const a of [0.4, 0.8]) expect(Math.abs(wallIn(false, "none", a).wall - srgb(a * 0.216))).toBeLessThanOrEqual(6);
  // ACES lifts these mid-tones: the same wall is brighter than the sky's light makes it.
  expect(wallIn(false, "aces", 0.8).wall).toBeGreaterThan(srgb(0.8 * 0.216) + 12);
}, 120_000);
