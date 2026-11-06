// `og build`'s web half on the runner archetype: the exported directory is
// the same bytes for the same inputs wherever it is written, og-build.json
// describes exactly the files beside it, the service worker's cache is named
// by their content, and project.ogt's publish settings decide isolation, the
// badge and the icon. (The page itself is driven in Chrome by tests/web/.)
import { beforeAll, expect, test } from "bun:test";
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ALIASED_FILES, DEFAULT_PUBLISH, downloadLines, downloadsOf, hashedName, headersFile, HASHED_FILES, serviceWorkerLists, settingsOf, webBuild, type ProjectSettings, type SizeRow } from "../src/node/web-build.ts";
import { mapStack } from "../src/sourcemap.ts";
import { ogBin } from "./support/og-bin.ts";
import { slow } from "./support/slow.ts";

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
 * scripts/rayon-worker.sh (web finding #8). The engine's own
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

test("the lanes pool's Worker starts the glue with one object, so a lanes page logs no deprecated-parameters warning (web finding #8)", () => {
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
}, slow(120_000));

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
  for (const p of ["index.html", "sw.js", "manifest.webmanifest", "_headers", "cooked/.cook.ogt", "icons/icon-512.png"]) expect(paths).toContain(p);
  // Content-hashed names (docs/web-export.md §Content-hashed names): each
  // file the page fetches by name is served as <stem>.<12 hex of its sha256>.<ext>,
  // and the page's configuration and og-build.json name it so.
  const names = m.names as Record<string, string>;
  // The AV1 fallback ships when the package has it (scripts/build-av1.sh).
  const av1 = existsSync(join(pkg, "dist/av1/og-av1.wasm"));
  expect(Object.keys(names).sort()).toEqual(HASHED_FILES.filter((f) => av1 || f !== "av1/og-av1.wasm").sort());
  for (const [plain, served] of Object.entries(names)) {
    expect(served).toBe(hashedName(plain, hash(served)));
    expect(paths).toContain(served);
    // A plain copy stays only where the platform still reads the plain name (docs/platform-asks.md #18),
    // and play.js as the page's entry.
    expect([plain, paths.includes(plain)]).toEqual([plain, plain === "play.js" || (ALIASED_FILES as readonly string[]).includes(plain)]);
  }
  expect(m.worker).toBe(names["game.js"]);
  expect(m.wasm).toBe(names["origin-engine_bg.wasm"]);
  expect(paths).not.toContain("og-build.json");
  expect(m.engine.name).toBe("origin-engine");
  expect(m.build).toBe("publish");
  expect(m.probe.members).toEqual(["hello", "tick", "state", "pick", "entity", "input", "capture"]);
  expect(m.crossOriginIsolation).toBe("preferred");
  expect(m.game).toEqual({ name: "Runner", id: "runner", profile: "2d" });
  const html = read(dir, "index.html");
  // The entry keeps its name (origin-agent's deploy and Stage checks read `./play.js`): a loader of the hashed shell.
  expect(html).toContain('<script type="module" src="./play.js"></script>');
  expect(html).toContain(`<link rel="modulepreload" href="./${names["play.js"]}">`);
  expect(read(dir, "play.js")).toBe(`import "./${names["play.js"]}";\n`);
  expect(JSON.parse(/<script type="application\/json" id="og-config">(.*?)<\/script>/.exec(html)![1]!).files).toEqual(names);
  // Hashed files are cached for a year; the entry points are revalidated.
  const headers = read(dir, "_headers");
  for (const served of Object.values(names)) expect(headers).toContain(`/${served}\n  Cache-Control: public, max-age=31536000, immutable`);
  expect(headers).toContain("/sw.js\n  Cache-Control: no-cache");
  expect(headers).toContain("/play.js\n  Cache-Control: no-cache");
  expect(html).toContain(`<meta name="og-engine" content="${m.engine.version}">`);
  // The Player transports: what the platform reads to open the page at the top level.
  expect(m.player).toEqual({ protocol: 2, transports: ["frame", "page"] });
  expect(html).toContain('<meta name="og-player" content="frame page">');
  expect(html).toContain('data-motion="genesis"');
  expect(html).toContain("Made with Origin Engine");
  // The service worker precaches what the page fetches, under a cache named by content.
  const sw = read(dir, "sw.js");
  const precache = JSON.parse(/^const OG_SW=(\{.*?\});/.exec(sw)![1]!) as { version: string; files: string[]; single: string[]; lanes: string[]; hashes: Record<string, string> };
  expect(precache.files).toContain("cooked/.cook.ogt");
  expect(precache.files).toContain(names["game.js"]);
  expect(precache.files).toContain("play.js");
  // The plain copies are fetched by no page: not precached.
  for (const plain of ALIASED_FILES) expect(precache.files).not.toContain(plain);
  // Each precached file's content, which the next build's worker carries over by.
  for (const f of precache.files.filter((f) => f !== "./")) expect(precache.hashes[f]).toBe(hash(f).slice(0, 16));
  // The two engine variants: the worker precaches the one its page boots, never both.
  expect(precache.single).toEqual([names["origin-engine.js"], names["origin-engine_bg.wasm"]]);
  expect(precache.lanes).toContain(names["origin-engine-lanes.js"]);
  expect(precache.lanes).toContain(names["origin-engine-lanes_bg.wasm"]);
  expect(precache.lanes.some((f) => f.startsWith("snippets/"))).toBe(true);
  // The pool Worker's helper as published: its init call takes one object (web finding #8).
  expect(positionalInits(dir)).toEqual([]);
  for (const f of [...precache.single, ...precache.lanes]) expect(precache.files).not.toContain(f);
  // The lanes variant ships beside the single-threaded engine, and the page knows its size.
  expect(m.lanes).toEqual({ glue: names["origin-engine-lanes.js"], wasm: names["origin-engine-lanes_bg.wasm"] });
  const config = JSON.parse(/<script type="application\/json" id="og-config">(.*?)<\/script>/.exec(html)![1]!);
  expect(config.expect.lanes).toBe(m.files.find((f: { path: string }) => f.path === m.lanes.wasm).bytes);
  expect(precache.files).not.toContain("_headers");
  expect(precache.files).not.toContain("og-build.json");
  // The AV1 fallback is content-hashed too, so a new decoder is never served
  // from an old cache, and is still cached only when the engine fetches it.
  if (av1) {
    expect(names["av1/og-av1.wasm"]).toMatch(/^av1\/og-av1\.[0-9a-f]{12}\.wasm$/);
    expect(paths).not.toContain("av1/og-av1.wasm");
    const lazy = (JSON.parse(/^const OG_SW=(\{.*?\});/.exec(sw)![1]!) as { lazy: string[] }).lazy;
    expect(lazy).toContain(names["av1/og-av1.wasm"]);
    expect(precache.files).not.toContain(names["av1/og-av1.wasm"]);
  }
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
  expect(same(join(work, "a", two.lanes.wasm), join(pkg, "dist", "web-publish-2d", "origin-engine-lanes_bg.wasm"))).toBe(true);
  expect(same(join(work, "a", two.lanes.glue), join(pkg, "dist", "web-publish-2d", "origin-engine-lanes.js"))).toBe(true);
  // The same project declared 3d takes the 3D module and the 3D lanes module.
  const out = join(work, "three-d");
  const b = await webBuild(project, entry, out, true, { ...settings(), profile: "3d" });
  expect(b.ok).toBe(true);
  expect([b.manifest!.game.profile, b.manifest!.variant]).toEqual(["3d", "3d"]);
  expect(same(join(out, "origin-engine_bg.wasm"), join(pkg, "dist", "web-publish", "origin-engine_bg.wasm"))).toBe(true);
  expect(same(join(out, b.manifest!.lanes!.wasm), join(pkg, "dist", "web-publish", "origin-engine-lanes_bg.wasm"))).toBe(true);
  // The variants differ (each has only its profile's physics), single-threaded
  // and lanes alike, and a dev build takes the full module.
  expect(same(join(pkg, "dist", "web-publish", "origin-engine_bg.wasm"), join(pkg, "dist", "web-publish-2d", "origin-engine_bg.wasm"))).toBe(false);
  expect(b.manifest!.lanes!.wasm).not.toBe(two.lanes.wasm);
  const dev = await webBuild(project, entry, join(work, "dev"), false, settings());
  expect(dev.ok && dev.manifest!.variant === "full").toBe(true);
}, slow(120_000));

test.skipIf(SLOW)("source maps: a dev build serves them linked; a publish build writes them under sourcemaps/, never served (docs/web-export.md §Source maps)", async () => {
  const pub = join(work, "a");
  const m = JSON.parse(read(pub, "og-build.json"));
  expect(m.sourcemaps.served).toBe(false);
  expect(m.sourcemaps.maps.map((x: { map: string }) => x.map)).toEqual(["sourcemaps/game.js.map", "sourcemaps/play.js.map", "sourcemaps/audio-worklet.js.map"]);
  for (const { bundle, file, map } of m.sourcemaps.maps as { bundle: string; file: string; map: string }[]) {
    expect(file).toBe(m.names[bundle]);
    expect(read(pub, file)).not.toContain("sourceMappingURL");
    const raw = JSON.parse(read(pub, map)) as { sources: string[]; sourcesContent: (string | null)[] };
    // Named relative to the project (the engine's files by package), with no machine's path in them.
    expect(raw.sources.some((s) => s.startsWith("/") || /^[A-Za-z]:/.test(s))).toBe(false);
    expect(JSON.stringify(raw)).not.toContain(repo);
  }
  expect(JSON.parse(read(pub, m.sourcemaps.maps[0].map)).sources).toContain("src/main.ts");
  // Not part of what is served: not in files or version, not precached.
  expect(m.files.some((f: { path: string }) => f.path.endsWith(".map"))).toBe(false);
  expect(read(pub, "sw.js")).not.toContain("sourcemaps/");
  // The build id the shell's crash reports carry.
  expect(JSON.parse(/<script type="application\/json" id="og-config">(.*?)<\/script>/s.exec(read(pub, "index.html"))![1]!).buildId).toBe(m.sourcemaps.build);

  const dev = join(work, "maps-dev");
  expect((await webBuild(project, entry, dev, false, settings())).ok).toBe(true);
  const d = JSON.parse(read(dev, "og-build.json"));
  expect(d.sourcemaps.served).toBe(true);
  // A dev build keeps every plain name (og dev, the tests and its linked maps read them).
  expect(d.names).toEqual({});
  expect(d.worker).toBe("game.js");
  expect(read(dev, "game.js")).toContain("//# sourceMappingURL=game.js.map");
  expect(existsSync(join(dev, "game.js.map"))).toBe(true);
}, slow(120_000));

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
}, slow(120_000));

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
    // A streamed texture: its mip tail boots, its whole chain (#mips) is fetched when big on screen.
    row("cooked/tex/rock.png.ogb", 60), row("cooked/tex/rock.png.mips.ogb", 900), row("cooked/models/hero.glb.tex0#mips.ogb", 800),
    // A streamed video: its meta and poster boot, its units and the AV1 fallback never do.
    row("cooked/clips/intro.ogt.ogb", 2), row("cooked/clips/intro.ogt.poster.ogb", 40), row("cooked/clips/intro.ogt.stream.ogb", 9000), row("cooked/clips/intro.ogt.stream720.ogb", 4000), row("av1/og-av1.wasm", 311),
  ];
  const d = downloadsOf(files);
  // The lanes page: its own module and the rayon snippet, the shell, the transcoder and the first group (the song's first chunk only).
  expect(d.boot.lanes!.compressed).toBe(26 + 3100 + 1 + 10 + 20 + 30 + 17 + 200 + 9 + 150 + 300 + 2 + 40 + 60);
  expect(d.boot.single.compressed).toBe(25 + 3000 + 10 + 20 + 30 + 17 + 200 + 9 + 150 + 300 + 2 + 40 + 60);
  expect(d.boot.single.engine.files).toBe(2);
  expect(d.later.packs).toEqual({ files: 1, bytes: 2100, compressed: 700 });
  expect(d.later.chunks.files).toBe(2);
  expect(d.later.mips).toEqual({ files: 2, bytes: 5100, compressed: 1700 });
  expect(d.later.worklet.compressed).toBe(1);
  expect(d.later.pwa.files).toBe(3);
  expect(d.later.videos).toEqual({ files: 2, bytes: 39000, compressed: 13000 });
  expect(d.later.av1.compressed).toBe(311);
  // The service worker installs neither stream nor the fallback decoder: a
  // page fetches one stream (the 720p rendition only on the fallback), and
  // each is cached when fetched.
  const sw = serviceWorkerLists(files.map((f) => f.path), new Set(["cooked/stream/vale.ogb"]));
  expect(sw.files).toContain("cooked/clips/intro.ogt.poster.ogb");
  expect(sw.files).toContain("cooked/tex/rock.png.ogb");
  for (const f of ["cooked/clips/intro.ogt.stream.ogb", "cooked/clips/intro.ogt.stream720.ogb", "av1/og-av1.wasm", "cooked/stream/vale.ogb", "cooked/music/song.wav.chunk1.ogb", "cooked/tex/rock.png.mips.ogb", "cooked/models/hero.glb.tex0#mips.ogb"]) {
    expect(sw.files).not.toContain(f);
    expect(sw.lazy).toContain(f);
  }
  expect(sw.single).toEqual(["origin-engine.js", "origin-engine_bg.wasm"]);
  const text = downloadLines(d, "brotli-11");
  expect(text).toContain("boot, lanes page (isolated)");
  expect(text).toContain(`${d.boot.single.compressed} B`);
  // A publish build's content-hashed files count as their own names, and their plain copies not at all.
  const hashed = files.flatMap((f) => {
    if (!(HASHED_FILES as readonly string[]).includes(f.path)) return [f];
    const served = { ...f, path: hashedName(f.path, "0123456789abcdef") };
    return (ALIASED_FILES as readonly string[]).includes(f.path) ? [f, served] : [served];
  });
  expect(downloadsOf(hashed)).toEqual(d);
  // A dev build ships no lanes module: one page.
  expect(downloadsOf(files.filter((f) => !/lanes|snippets/.test(f.path))).boot.lanes).toBeNull();
});

test("a crash in a content-hashed bundle maps through the bundle's own map, and _headers caches only hashed files for good", () => {
  // `a` at line 1 column 1 of game.js is src/main.ts line 3 column 5 (VLQ "AAEI").
  const map = { version: 3, sources: ["src/main.ts"], names: [], mappings: "AAEI" };
  const frame = "    at a (https://play.example/g/r/game.0123456789ab.js:1:1)";
  expect(mapStack(frame, { "game.js": map })).toBe("    at a (src/main.ts:3:5)");
  // A plain name that merely looks hashed is not taken for another bundle's.
  expect(mapStack("    at a (https://play.example/g/r/other.0123456789ab.js:1:1)", { "game.js": map })).toBe("    at a (https://play.example/g/r/other.0123456789ab.js:1:1)");
  const h = headersFile("off", ["game.0123456789ab.js"]);
  expect(h).toContain("/game.0123456789ab.js\n  Cache-Control: public, max-age=31536000, immutable");
  expect(h.match(/immutable/g)?.length).toBe(1);
  expect(hashedName("basis/basis_transcoder.wasm", "abcdef0123456789")).toBe("basis/basis_transcoder.abcdef012345.wasm");
  expect(hashedName("origin-engine_bg.wasm", "abcdef0123456789")).toBe("origin-engine_bg.abcdef012345.wasm");
});
