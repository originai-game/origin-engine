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

// The gate's path-aware lane skips this file: it spawns og processes for
// minutes (docs/ci.md §Lanes). `scripts/check.sh --all`, CI and a plain
// `bun run test` run it (OG_GATE_ALL, or no OG_GATE at all).
const SLOW = process.env.OG_GATE === "1" && !process.env.OG_GATE_ALL;

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
  if (SLOW) return;
  rmSync(work, { recursive: true, force: true });
  mkdirSync(work, { recursive: true });
});

test.skipIf(SLOW)("the same inputs give the same bytes, wherever they are written", async () => {
  // The project's cooked bundle (in the test, not a hook: a hook's 5 s
  // would not cover a cold cook).
  // An incremental cook: free when cooked/ is current, and a cooker
  // version bump re-cooks instead of reading stale output.
  const r = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo });
  if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
  const a = await webBuild(project, entry, join(work, "a"), true, settings());
  const b = await webBuild(project, entry, join(work, "deeper", "b"), true, settings());
  expect(a.ok && b.ok).toBe(true);
  expect(b.manifest!.version).toBe(a.manifest!.version);
  expect(b.manifest!.files).toEqual(a.manifest!.files);
  expect(read(join(work, "deeper", "b"), "og-build.json")).toBe(read(join(work, "a"), "og-build.json"));
}, 120_000);

test.skipIf(SLOW)("og-build.json names every file beside it, with bytes and sha256, and the Player's facts", async () => {
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
  // The Player transports: what the platform reads to open the page at the top level.
  expect(m.player).toEqual({ protocol: 2, transports: ["frame", "page"] });
  expect(html).toContain('<meta name="og-player" content="frame page">');
  expect(html).toContain('data-motion="genesis"');
  expect(html).toContain("Made with Origin Engine");
  // The service worker precaches what the page fetches, under a cache named by content.
  const sw = read(dir, "sw.js");
  const precache = JSON.parse(/^const OG_SW=(\{.*?\});/.exec(sw)![1]!) as { version: string; files: string[]; single: string[]; lanes: string[] };
  expect(precache.files).toContain("cooked/.cook.ogt");
  // The two engine variants: the worker precaches the one its page boots, never both.
  expect(precache.single).toEqual(["origin-engine.js", "origin-engine_bg.wasm"]);
  expect(precache.lanes).toContain("origin-engine-lanes.js");
  expect(precache.lanes).toContain("origin-engine-lanes_bg.wasm");
  expect(precache.lanes.some((f) => f.startsWith("snippets/"))).toBe(true);
  for (const f of [...precache.single, ...precache.lanes]) expect(precache.files).not.toContain(f);
  // The lanes variant ships beside the single-threaded engine, and the page knows its size.
  expect(m.lanes).toEqual({ glue: "origin-engine-lanes.js", wasm: "origin-engine-lanes_bg.wasm" });
  for (const p of ["origin-engine-lanes.js", "origin-engine-lanes_bg.wasm"]) expect(paths).toContain(p);
  const config = JSON.parse(/<script type="application\/json" id="og-config">(.*?)<\/script>/.exec(html)![1]!);
  expect(config.expect.lanes).toBe(m.files.find((f: { path: string }) => f.path === "origin-engine-lanes_bg.wasm").bytes);
  expect(precache.files).not.toContain("_headers");
  expect(precache.files).not.toContain("og-build.json");
  const web = JSON.parse(read(dir, "manifest.webmanifest"));
  expect(web.display).toBe("standalone");
  expect(web.icons.map((i: { sizes: string }) => i.sizes)).toEqual(["192x192", "512x512"]);
}, 60_000);

test.skipIf(SLOW)("a publish build ships the engine variant of the game's profile (docs/web-export.md §Variants)", async () => {
  const same = (a: string, b: string) => {
    const [x, y] = [new Uint8Array(readFileSync(a)), new Uint8Array(readFileSync(b))];
    return x.length === y.length && x.every((v, i) => v === y[i]);
  };
  // runner is a 2d game: the 2D module, as origin-engine_bg.wasm.
  const two = JSON.parse(read(join(work, "a"), "og-build.json"));
  expect([two.game.profile, two.variant]).toEqual(["2d", "2d"]);
  expect(same(join(work, "a", "origin-engine_bg.wasm"), join(pkg, "dist", "web-publish-2d", "origin-engine_bg.wasm"))).toBe(true);
  expect(same(join(work, "a", "origin-engine.js"), join(pkg, "dist", "web-publish-2d", "origin-engine.js"))).toBe(true);
  // The same project declared 3d takes the 3D module; the lanes variant is the one module of both.
  const out = join(work, "three-d");
  const b = await webBuild(project, entry, out, true, { ...settings(), profile: "3d" });
  expect(b.ok).toBe(true);
  expect([b.manifest!.game.profile, b.manifest!.variant]).toEqual(["3d", "3d"]);
  expect(same(join(out, "origin-engine_bg.wasm"), join(pkg, "dist", "web-publish", "origin-engine_bg.wasm"))).toBe(true);
  expect(same(join(out, "origin-engine-lanes_bg.wasm"), join(work, "a", "origin-engine-lanes_bg.wasm"))).toBe(true);
  // The two variants differ (each has only its profile's physics), and a dev build takes the full module.
  expect(same(join(pkg, "dist", "web-publish", "origin-engine_bg.wasm"), join(pkg, "dist", "web-publish-2d", "origin-engine_bg.wasm"))).toBe(false);
  const dev = await webBuild(project, entry, join(work, "dev"), false, settings());
  expect(dev.ok && dev.manifest!.variant === "full").toBe(true);
}, 120_000);

test.skipIf(SLOW)("publish settings: isolation, the badge and the service worker's version", async () => {
  const off = await webBuild(project, entry, join(work, "off"), true, settings({ isolation: "off", madeWith: false }));
  expect(off.ok).toBe(true);
  expect(existsSync(join(work, "off", "origingame.runtime.json"))).toBe(false);
  expect(read(join(work, "off"), "_headers")).not.toContain("Cross-Origin-Embedder-Policy");
  expect(read(join(work, "off"), "index.html")).toMatch(/id="og-made-with"[^>]* hidden/);
  // Without isolation the page can never share memory: no lanes variant.
  expect(off.manifest!.lanes).toBeNull();
  expect(existsSync(join(work, "off", "origin-engine-lanes_bg.wasm"))).toBe(false);
  expect(existsSync(join(work, "off", "snippets"))).toBe(false);
  expect(JSON.parse(/<script type="application\/json" id="og-config">(.*?)<\/script>/.exec(read(join(work, "off"), "index.html"))![1]!).expect.lanes).toBe(0);
  const swVersion = (d: string) => /"version":"([0-9a-f]+)"/.exec(read(join(work, d), "sw.js"))![1];
  expect(swVersion("off")).not.toBe(swVersion("a"));
  const req = await webBuild(project, entry, join(work, "req"), false, settings({ isolation: "required", fit: "letterbox", aspect: "16:9" }));
  expect(req.ok).toBe(true);
  expect(JSON.parse(read(join(work, "req"), "origingame.runtime.json"))).toEqual({ version: 1, crossOriginIsolated: true });
  expect(read(join(work, "req"), "_headers")).toContain("Cross-Origin-Embedder-Policy: require-corp");
  // A dev build: every probe member, no service worker.
  expect(req.manifest!.build).toBe("dev");
  expect(req.manifest!.probe.members).toContain("commands");
  expect(req.manifest!.lanes).toBeNull();
  expect(existsSync(join(work, "req", "sw.js"))).toBe(false);
  // A dev (Stage) build declares the same transports: the Stage page reads them too.
  expect(req.manifest!.player).toEqual({ protocol: 2, transports: ["frame", "page"] });
  expect(read(join(work, "req"), "index.html")).toContain('<meta name="og-player" content="frame page">');
  expect(JSON.parse(/<script type="application\/json" id="og-config">(.*?)<\/script>/.exec(read(join(work, "req"), "index.html"))![1]!).aspect).toBeCloseTo(16 / 9, 6);
}, 120_000);

test.skipIf(SLOW)("a project icon must be a square PNG of at least 192 px", async () => {
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

test.skipIf(SLOW)("og build never replaces the project, an ancestor of it or a directory it did not write", async () => {
  const guarded = join(work, "guarded");
  mkdirSync(join(guarded, "notes"), { recursive: true });
  writeFileSync(join(guarded, "project.ogt"), "type project\n");
  writeFileSync(join(guarded, "notes", "keep.txt"), "mine\n");
  const other = join(work, "other-project");
  mkdirSync(other, { recursive: true });
  writeFileSync(join(other, "project.ogt"), "type project\n");
  writeFileSync(join(other, "og-build.json"), "{}\n");
  const cases: Array<[string, RegExp]> = [
    [guarded, /is the project or contains it/],
    [work, /is the project or contains it/],
    [join(guarded, "notes"), /holds no og-build\.json/],
    [other, /holds a project\.ogt/],
  ];
  for (const [out, reason] of cases) {
    const b = await webBuild(guarded, join(guarded, "main.ts"), out, true, settings());
    expect([out, b.ok]).toEqual([out, false]);
    expect(b.reason).toMatch(reason);
  }
  expect(readFileSync(join(guarded, "notes", "keep.txt"), "utf8")).toBe("mine\n");
  expect(existsSync(join(guarded, "project.ogt"))).toBe(true);
  expect(existsSync(join(other, "project.ogt"))).toBe(true);
});
