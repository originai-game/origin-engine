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
