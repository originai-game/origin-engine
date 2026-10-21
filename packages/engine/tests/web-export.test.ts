// `og build`'s web half on the runner archetype: the exported directory is
// the same bytes for the same inputs wherever it is written, og-build.json
// describes exactly the files beside it, the service worker's cache is named
// by their content, and project.ogt's publish settings decide isolation, the
// badge and the icon. (The page itself is driven in Chrome by tests/web/.)
import { beforeAll, expect, test } from "bun:test";
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { DEFAULT_PUBLISH, downloadLines, downloadsOf, settingsOf, webBuild, type ProjectSettings, type SizeRow } from "../src/node/web-build.ts";
import { ogBin } from "./support/og-bin.ts";

// The gate's path-aware lane skips this file: it spawns og processes for
// minutes (docs/ci.md §Lanes). `scripts/check.sh --all`, CI and a plain
// `bun run test` run it (OG_GATE_ALL, or no OG_GATE at all).
const SLOW = process.env.OG_GATE === "1" && !process.env.OG_GATE_ALL;

declare module "node:fs" {
  export function cpSync(from: string, to: string, options?: { recursive?: boolean }): void;
  export function readdirSync(path: string): string[];
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

/**
 * The wasm-bindgen init calls in a build's `snippets/` (the scripts
 * wasm-bindgen copies from crates: wasm-bindgen-rayon's pool Worker
 * helper) that pass positional arguments, which wasm-bindgen 0.2.129's
 * glue warns about on every call ("using deprecated parameters …; pass a
 * single object instead"): the helper made one per pool Worker until
 * scripts/rayon-worker.sh (examples/WEB-FINDINGS.md #8). The engine's own
 * calls (src/web/host.ts) pass one object.
 */
function positionalInits(dir: string): string[] {
  dir = join(dir, "snippets");
  const found: string[] = [];
  const walk = (d: string) => {
    for (const name of readdirSync(d)) {
      const p = join(d, name);
      if (statSync(p).isDirectory()) walk(p);
      else if (name.endsWith(".js")) {
        // A call of the glue's default export or initSync whose argument is not one object literal.
        for (const m of readFileSync(p, "utf8").matchAll(/\b(?:default|initSync)\(\s*(?![{)\s])[^)]*\)/g)) found.push(`${p.slice(dir.length + 1)}: ${m[0]}`);
      }
    }
  };
  if (existsSync(dir)) walk(dir);
  return found;
}

test("the lanes pool's Worker starts the glue with one object, so a lanes page logs no deprecated-parameters warning (WEB-FINDINGS #8)", () => {
  let helpers = 0;
  for (const variant of ["web-publish", "web-publish-2d"]) {
    const dir = join(pkg, "dist", variant);
    if (!existsSync(join(dir, "origin-engine-lanes.js"))) continue;
    const snippets = join(dir, "snippets");
    const helper = existsSync(snippets) ? readdirSync(snippets).map((s) => join(snippets, s, "src", "workerHelpers.no-bundler.js")).filter((f) => existsSync(f)) : [];
    expect([variant, helper.length]).toEqual([variant, 1]);
    helpers += helper.length;
    expect([variant, positionalInits(dir)]).toEqual([variant, []]);
    expect(readFileSync(helper[0]!, "utf8")).toContain("pkg.default({ module_or_path: data.module, memory: data.memory })");
  }
  // A package without the lanes variants has nothing to check, but the gate's build.sh always ships both.
  if (process.env.OG_GATE === "1" && process.env.OG_WASM_STALE !== "1") expect(helpers).toBe(2);
});

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
  // The pool Worker's helper as published: its init call takes one object (WEB-FINDINGS #8).
  expect(positionalInits(dir)).toEqual([]);
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
  // Its lanes variant is the 2D lanes module, from the same directory (owed row 71).
  expect(same(join(work, "a", "origin-engine-lanes_bg.wasm"), join(pkg, "dist", "web-publish-2d", "origin-engine-lanes_bg.wasm"))).toBe(true);
  expect(same(join(work, "a", "origin-engine-lanes.js"), join(pkg, "dist", "web-publish-2d", "origin-engine-lanes.js"))).toBe(true);
  // The same project declared 3d takes the 3D module and the 3D lanes module.
  const out = join(work, "three-d");
  const b = await webBuild(project, entry, out, true, { ...settings(), profile: "3d" });
  expect(b.ok).toBe(true);
  expect([b.manifest!.game.profile, b.manifest!.variant]).toEqual(["3d", "3d"]);
  expect(same(join(out, "origin-engine_bg.wasm"), join(pkg, "dist", "web-publish", "origin-engine_bg.wasm"))).toBe(true);
  expect(same(join(out, "origin-engine-lanes_bg.wasm"), join(pkg, "dist", "web-publish", "origin-engine-lanes_bg.wasm"))).toBe(true);
  // The variants differ (each has only its profile's physics), single-threaded
  // and lanes alike, and a dev build takes the full module.
  expect(same(join(pkg, "dist", "web-publish", "origin-engine_bg.wasm"), join(pkg, "dist", "web-publish-2d", "origin-engine_bg.wasm"))).toBe(false);
  expect(same(join(out, "origin-engine-lanes_bg.wasm"), join(work, "a", "origin-engine-lanes_bg.wasm"))).toBe(false);
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

test("og build prints what each page downloads, not the directory's total", () => {
  const row = (path: string, compressed: number): SizeRow => ({ path, bytes: compressed * 3, compressed, codec: "brotli-11" });
  const files = [
    row("index.html", 10), row("play.js", 20), row("game.js", 30), row("audio-worklet.js", 1), row("sw.js", 2), row("manifest.webmanifest", 1), row("icons/icon-192.png", 5),
    row("origin-engine.js", 25), row("origin-engine_bg.wasm", 3000), row("origin-engine-lanes.js", 26), row("origin-engine-lanes_bg.wasm", 3100), row("snippets/x/workerHelpers.no-bundler.js", 1),
    row("basis/basis_transcoder.js", 17), row("basis/basis_transcoder.wasm", 200), row("basis/LICENSE", 4), row("og-build.json", 6), row("_headers", 1),
    row("cooked/.cook.ogt", 9), row("cooked/music/song.wav.ogb", 150), row("cooked/music/song.wav.chunk1.ogb", 140), row("cooked/music/song.wav.chunk2.ogb", 140), row("cooked/stream/vale.ogb", 700), row("cooked/models/hero.glb.ogb", 300),
  ];
  const d = downloadsOf(files);
  // The lanes page: its own module and the rayon snippet, the shell, the transcoder and the first group (the song's first chunk only).
  expect(d.boot.lanes!.compressed).toBe(26 + 3100 + 1 + 10 + 20 + 30 + 17 + 200 + 9 + 150 + 300);
  expect(d.boot.single.compressed).toBe(25 + 3000 + 10 + 20 + 30 + 17 + 200 + 9 + 150 + 300);
  expect(d.boot.single.engine.files).toBe(2);
  expect(d.later.packs).toEqual({ files: 1, bytes: 2100, compressed: 700 });
  expect(d.later.chunks.files).toBe(2);
  expect(d.later.worklet.compressed).toBe(1);
  expect(d.later.pwa.files).toBe(3);
  const text = downloadLines(d, "brotli-11");
  expect(text).toContain("boot, lanes page (isolated)");
  expect(text).toContain(`${d.boot.single.compressed} B`);
  // A dev build ships no lanes module: one page.
  expect(downloadsOf(files.filter((f) => !/lanes|snippets/.test(f.path))).boot.lanes).toBeNull();
});
