// `og build`'s web half on the runner archetype: the exported directory is
// the same bytes for the same inputs wherever it is written, og-build.json
// describes exactly the files beside it, the service worker's cache is named
// by their content, and project.ogt's publish settings decide isolation, the
// badge and the icon. (The page itself is driven in Chrome by tests/web/.)
import { beforeAll, expect, test } from "bun:test";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { DEFAULT_PUBLISH, settingsOf, webBuild, type ProjectSettings } from "../src/node/web-build.ts";
import { ogBin } from "./support/og-bin.ts";

declare module "node:fs" {
  export function cpSync(from: string, to: string, options?: { recursive?: boolean }): void;
}
declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stderr: Uint8Array };
  CryptoHasher: new (a: string) => { update(d: Uint8Array): unknown; digest(e: "hex"): string };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "archetypes", "runner");
const work = join(repo, "target", "evidence", "web-export-test");
const entry = join(project, "src", "main.ts");
const settings = (publish: Partial<ProjectSettings["publish"]> = {}): ProjectSettings => ({ ...settingsOf(project), publish: { ...DEFAULT_PUBLISH, ...publish } });
const read = (dir: string, f: string) => readFileSync(join(dir, f), "utf8");

beforeAll(() => {
  rmSync(work, { recursive: true, force: true });
  mkdirSync(work, { recursive: true });
});

test("the same inputs give the same bytes, wherever they are written", async () => {
  // The project's cooked bundle, when no earlier run left one (a hook's
  // 5 s would not cover a cook).
  if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
    const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo });
    if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
  }
  const a = await webBuild(project, entry, join(work, "a"), true, settings());
  const b = await webBuild(project, entry, join(work, "deeper", "b"), true, settings());
  expect(a.ok && b.ok).toBe(true);
  expect(b.manifest!.version).toBe(a.manifest!.version);
  expect(b.manifest!.files).toEqual(a.manifest!.files);
  expect(read(join(work, "deeper", "b"), "og-build.json")).toBe(read(join(work, "a"), "og-build.json"));
}, 120_000);

test("og-build.json names every file beside it, with bytes and sha256, and the Player's facts", async () => {
  const dir = join(work, "a");
  const m = JSON.parse(read(dir, "og-build.json"));
  const hash = (f: string) => {
    const h = new Bun.CryptoHasher("sha256");
    h.update(new Uint8Array(readFileSync(join(dir, f))));
    return h.digest("hex");
  };
  for (const f of m.files) {
    expect(readFileSync(join(dir, f.path)).length).toBe(f.bytes);
    expect(hash(f.path)).toBe(f.sha256);
  }
  const paths = m.files.map((f: { path: string }) => f.path);
  for (const p of ["index.html", "play.js", "game.js", "audio-worklet.js", "origin-engine.js", "origin-engine_bg.wasm", "sw.js", "manifest.webmanifest", "_headers", "cooked/.cook.ogt", "icons/icon-512.png"]) expect(paths).toContain(p);
  expect(paths).not.toContain("og-build.json");
  expect(m.engine.name).toBe("origin-engine");
  expect(m.build).toBe("publish");
  expect(m.probe.members).toEqual(["hello", "tick", "state", "pick", "entity", "input", "capture"]);
  expect(m.crossOriginIsolation).toBe("preferred");
  expect(m.game).toEqual({ name: "Runner", id: "runner", profile: "2d" });
  const html = read(dir, "index.html");
  expect(html).toContain(`<meta name="og-engine" content="${m.engine.version}">`);
  expect(html).toContain('data-motion="genesis"');
  expect(html).toContain("Made with Origin Engine");
  // The service worker precaches what the page fetches, under a cache named by content.
  const sw = read(dir, "sw.js");
  const precache = JSON.parse(/^const OG_SW=(\{.*?\});/.exec(sw)![1]!) as { version: string; files: string[] };
  expect(precache.files).toContain("origin-engine_bg.wasm");
  expect(precache.files).toContain("cooked/.cook.ogt");
  expect(precache.files).not.toContain("_headers");
  expect(precache.files).not.toContain("og-build.json");
  const web = JSON.parse(read(dir, "manifest.webmanifest"));
  expect(web.display).toBe("standalone");
  expect(web.icons.map((i: { sizes: string }) => i.sizes)).toEqual(["192x192", "512x512"]);
}, 60_000);

test("publish settings: isolation, the badge and the service worker's version", async () => {
  const off = await webBuild(project, entry, join(work, "off"), true, settings({ isolation: "off", madeWith: false }));
  expect(off.ok).toBe(true);
  expect(existsSync(join(work, "off", "origingame.runtime.json"))).toBe(false);
  expect(read(join(work, "off"), "_headers")).not.toContain("Cross-Origin-Embedder-Policy");
  expect(read(join(work, "off"), "index.html")).toMatch(/id="og-made-with"[^>]* hidden/);
  const swVersion = (d: string) => /"version":"([0-9a-f]+)"/.exec(read(join(work, d), "sw.js"))![1];
  expect(swVersion("off")).not.toBe(swVersion("a"));
  const req = await webBuild(project, entry, join(work, "req"), false, settings({ isolation: "required", fit: "letterbox", aspect: "16:9" }));
  expect(req.ok).toBe(true);
  expect(JSON.parse(read(join(work, "req"), "origingame.runtime.json"))).toEqual({ version: 1, crossOriginIsolated: true });
  expect(read(join(work, "req"), "_headers")).toContain("Cross-Origin-Embedder-Policy: require-corp");
  // A dev build: every probe member, no service worker.
  expect(req.manifest!.build).toBe("dev");
  expect(req.manifest!.probe.members).toContain("commands");
  expect(existsSync(join(work, "req", "sw.js"))).toBe(false);
  expect(JSON.parse(/<script type="application\/json" id="og-config">(.*?)<\/script>/.exec(read(join(work, "req"), "index.html"))![1]!).aspect).toBeCloseTo(16 / 9, 6);
}, 120_000);

test("a project icon must be a square PNG of at least 192 px", async () => {
  const copy = join(work, "iconed");
  cpSync(project, copy, { recursive: true });
  writeFileSync(join(copy, "icon.png"), new Uint8Array([1, 2, 3]));
  const bad = await webBuild(copy, join(copy, "src", "main.ts"), join(work, "icon-bad"), true, settings({ icon: "icon.png" }));
  expect(bad.ok).toBe(false);
  expect(bad.reason).toMatch(/icon\.png is not a PNG/);
  cpSync(join(pkg, "shell", "brand", "icon-512.png"), join(copy, "icon.png"));
  const good = await webBuild(copy, join(copy, "src", "main.ts"), join(work, "icon-good"), true, settings({ icon: "icon.png" }));
  expect(good.ok).toBe(true);
  expect(good.manifest!.icons).toEqual([{ src: "icons/icon.png", sizes: "512x512", type: "image/png", source: "project" }]);
}, 120_000);
