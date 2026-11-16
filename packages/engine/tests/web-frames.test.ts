// Remote <iframe>s are on demand (docs/web-export.md §Variants): `og build
// --publish` ships the engine modules that have them only to a game whose
// project.ogt lists `frames`. A game without them must not download their
// code: the Worker's document fetcher (og_web_set_remote_text_fetcher), the
// frame budgets and the nesting walk ("frames nest at most"); its engine
// refuses a remote src instead. A game that lists them gets all of it.
// Fails when a frames feature leaks into the plain publish or lanes
// modules (scripts/wasm-crates.txt) or og build picks the wrong directory.
import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../src/node/web-build.ts";
import { cookProject } from "./support/og-bin.ts";
import { slow } from "./support/slow.ts";

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
// The gate's dev wasm has no publish modules (OG_WASM_VARIANTS=dev); the
// full lane and release packing build them.
const NO_FRAMES_WASM = !existsSync(join(pkg, "dist", "web-publish-frames", "origin-engine_bg.wasm"));

test.skipIf(NO_FRAMES_WASM)("a game without `frames` ships an engine without remote <iframe>s, and one that lists them an engine with them", async () => {
  const project = join(repo, "tests", "fixtures", "browser-host");
  cookProject(project, repo);
  const shipped = async (frames: string[]) => {
    const dist = join(repo, "target", "evidence", "web-frames", frames.length > 0 ? "frames" : "none");
    const b = await webBuild(project, join(project, "src", "main.ts"), dist, true, { ...settingsOf(project), frames });
    if (!b.ok) throw new Error(b.reason);
    const m = b.manifest!;
    expect(m.frames).toEqual(frames);
    // The single-threaded module and, beside it, the lanes one.
    const modules = [m.wasm, ...(m.lanes ? [m.lanes.wasm] : [])];
    expect(modules.length).toBe(2);
    return modules.map((file) => {
      const bytes = new Uint8Array(readFileSync(join(dist, file)));
      const exports = WebAssembly.Module.exports(new WebAssembly.Module(bytes)).map((e) => e.name);
      const text = new TextDecoder("latin1").decode(bytes);
      return {
        fetcher: exports.includes("og_web_set_remote_text_fetcher"),
        walk: text.includes("frames nest at most"),
        refusal: text.includes("remote <iframe>s are not built into this engine"),
      };
    });
  };
  const without = { fetcher: false, walk: false, refusal: true };
  const withFrames = { fetcher: true, walk: true, refusal: false };
  expect(await shipped([])).toEqual([without, without]);
  expect(await shipped(["https://events.example.com"])).toEqual([withFrames, withFrames]);
}, slow(180_000));
