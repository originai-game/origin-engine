// A game's error points at its TypeScript (docs/web-export.md §Source maps):
// a native bundle's stack names the .ts line as bun runs it (og run, og dev,
// og test and playtests load bundles made by bundleGame), and a minified
// browser bundle's stack, mapped with the map the build wrote, names it too
// (the crash card on a dev page, the platform on a stored report).
import { beforeAll, expect, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { bundleGame } from "../src/node/build.ts";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";
import { mapStack, SourceMap, stackFrames } from "../src/sourcemap.ts";
import { useSourceMap } from "../src/spawn-site.ts";

const fixtures = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "stack-maps");

beforeAll(() => loadNativeHost());

test("a system's throw in a native bundle names the game's .ts line, not the bundle's", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-maps-"));
  try {
    const out = join(dir, "game.native.js");
    const b = await bundleGame(join(fixtures, "main.ts"), out, "native");
    expect(b.ok).toBe(true);
    expect(b.map).toBe(`${out}.map`);
    expect(readFileSync(out, "utf8")).toContain("//# sourceMappingURL=game.native.js.map");
    const game = ((await import(pathToFileURL(out).href)) as { default: GameToken }).default;
    const created = createEngine(game);
    if (!created.ok) throw new Error(created.reason);
    const advanced = created.value.advance(5);
    expect(advanced.ok).toBe(false);
    const reason = advanced.ok ? "" : advanced.reason;
    expect(reason).toContain("the well is dry at depth 3");
    // `throw new Error` is line 10 of main.ts, inside drawWater.
    expect(reason).toMatch(/drawWater \(.*stack-maps\/main\.ts:10:\d+\)/);
    expect(reason).not.toContain("game.native.js:");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("a minified bundle's stack, its map kept apart, maps back to the .ts line", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-maps-"));
  try {
    // As a publish build writes it: minified, the map elsewhere and not linked.
    const out = join(dir, "play.js");
    const mapFile = join(dir, "sourcemaps", "play.js.map");
    const b = await bundleGame(join(fixtures, "script.ts"), out, "script", true, { root: dirname(fixtures), link: false, file: mapFile });
    expect(b.ok).toBe(true);
    expect(existsSync(`${out}.map`)).toBe(false);
    expect(readFileSync(out, "utf8")).not.toContain("sourceMappingURL");
    const raw = JSON.parse(readFileSync(mapFile, "utf8")) as { sources: string[] };
    // Named relative to the root given, never by an absolute path of the machine that built it.
    expect(raw.sources).toEqual(["stack-maps/script.ts"]);

    let stack = "";
    try {
      ((await import(pathToFileURL(out).href)) as { default: () => void }).default();
    } catch (e) {
      stack = (e as Error).stack ?? "";
    }
    expect(stack).toContain("play.js:1:");
    // What a crash report carries: frames only, the location cut to the file name.
    const frames = stackFrames(`Error: the lantern is out\n${stack.split("\n").slice(1).join("\n")}`);
    expect(frames.length).toBeGreaterThan(0);
    for (const f of frames) expect(f).not.toContain(dir);
    expect(frames[0]).toMatch(/\(play\.js:1:\d+\)$/);

    const mapped = mapStack(frames.join("\n"), { "play.js": readFileSync(mapFile, "utf8") });
    expect(mapped.split("\n")[0]).toMatch(/\(stack-maps\/script\.ts:7:\d+\)$/);
    expect(mapped).toMatch(/stack-maps\/script\.ts:11:\d+/);
    // A frame of a file without a map stays as it was.
    expect(mapStack("    at x (other.js:1:5)", { "play.js": readFileSync(mapFile, "utf8") })).toBe("    at x (other.js:1:5)");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

/** The probe's source for the entity drawn at the middle of a 64×64 frame of `game` (describe, then pick). */
function spawnedSource(game: GameToken, root?: string): unknown {
  const created = createEngine(game, root === undefined ? {} : { root });
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  try {
    expect(engine.advance(1).ok).toBe(true);
    const seen = engine.probe.describe() as unknown as { entities: Array<{ id: string; point: { x: number; y: number } }> };
    const crate = seen.entities.find((e) => e.id === "crate");
    if (!crate) throw new Error(`no crate in ${JSON.stringify(seen)}`);
    const at = engine.probe.pick(crate.point.x, crate.point.y) as { entity: string; source: unknown } | null;
    expect(at?.entity).toBe("crate");
    return at!.source;
  } finally {
    engine.destroy();
  }
}

test("an entity spawned in code has the game's .ts line as its source, through bun's mapping or a bundle's map", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-maps-"));
  try {
    // As og run loads a game: bun maps the linked bundle's frames to the
    // sources, which are named relative to the project.
    const linked = join(dir, "linked", "game.native.js");
    expect((await bundleGame(join(fixtures, "spawn.ts"), linked, "native")).ok).toBe(true);
    const native = ((await import(pathToFileURL(linked).href)) as { default: GameToken }).default;
    // `world.spawn(` is line 14 of spawn.ts; the engine's own frames above it are skipped.
    expect(spawnedSource(native, fixtures)).toBe("spawn.ts:14");

    // As the web Worker sees its bundle: no mapping of its own, the map
    // handed over (useSourceMap) with sources relative to the project.
    const out = join(dir, "kept", "game.js");
    const mapFile = join(dir, "kept", "maps", "game.js.map");
    expect((await bundleGame(join(fixtures, "spawn.ts"), out, "native", false, { root: dirname(fixtures), link: false, file: mapFile })).ok).toBe(true);
    const web = ((await import(pathToFileURL(out).href)) as { default: GameToken }).default;
    expect(spawnedSource(web)).toBeNull();
    useSourceMap(out, new SourceMap(readFileSync(mapFile, "utf8")));
    expect(spawnedSource(web)).toBe("stack-maps/spawn.ts:14");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("frames in every browser's format: V8, JavaScriptCore and SpiderMonkey", () => {
  // Generated line 1: "AAAA" maps column 0 to source 0 line 0 column 0; ",KAEE" column 5 to line 2 column 2.
  const map = new SourceMap({ version: 3, sources: ["src/main.ts"], mappings: "AAAA,KAEE", names: [] });
  expect(map.original(1, 1)).toEqual({ source: "src/main.ts", line: 1, column: 1, name: null });
  expect(map.original(1, 9)).toEqual({ source: "src/main.ts", line: 3, column: 3, name: null });
  expect(map.original(2, 1)).toBeNull();
  const maps = { "game.js": map };
  expect(mapStack("    at f (https://x.example/g/game.js?v=1:1:7)", maps)).toBe("    at f (src/main.ts:3:3)");
  expect(mapStack("    at https://x.example/game.js:1:2", maps)).toBe("    at src/main.ts:1:1");
  expect(mapStack("f@https://x.example/game.js:1:6", maps)).toBe("f@src/main.ts:3:3");
  expect(stackFrames("TypeError: x is 3\nf@https://x.example/a/b/game.js:1:6\n    at g (/home/me/game.js:1:2)\nnot a frame")).toEqual(["f@game.js:1:6", "at g (game.js:1:2)"]);
});
