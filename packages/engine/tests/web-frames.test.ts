// Remote <iframe>s are on demand (docs/web-export.md §Variants): `og build
// --publish` ships the engine modules that have them only to a game whose
// project.ogt lists `frames`. A game without them must not download their
// code: the Worker's document fetcher (og_web_set_remote_text_fetcher), the
// frame budgets and the nesting walk ("frames nest at most"); its engine
// refuses a remote src instead. A game that lists them gets all of it.
// Fails when a frames feature leaks into the plain publish or lanes
// modules (scripts/wasm-crates.txt) or og build picks the wrong directory.
import { expect, test } from "bun:test";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../src/node/web-build.ts";
import { cookProject } from "./support/og-bin.ts";
import { slow } from "./support/slow.ts";
import { FRAME_HOSTS, packFrameHosts } from "../src/node/frame-hosts.ts";

declare const Bun: {
  serve(options: { port: number; fetch(request: Request): Response }): { port: number; stop(close: boolean): void };
  spawn(command: string[], options: { env: Record<string, string | undefined>; stdout: "pipe"; stderr: "pipe" }): {
    exited: Promise<number>; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array>;
  };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
// The gate's dev wasm has no publish modules (OG_WASM_VARIANTS=dev); the
// full lane and release packing build them.
const NO_FRAMES_WASM = !existsSync(join(pkg, "dist", "web-publish-frames", "origin-engine_bg.wasm"));

test.skipIf(NO_FRAMES_WASM)("a frames project fetches its pinned host automatically, builds offline from cache, and refuses corrupt bytes", async () => {
  const root = mkdtempSync(join(tmpdir(), "og-frame-install-"));
  const engine = join(root, "engine");
  const project = join(root, "game");
  let requests = 0;
  let corrupt = false;
  const server = Bun.serve({ port: 0, fetch(request) {
    requests++;
    // The pin names the archive by its store address, <base>/<sha256>.
    const file = join(root, "packs", `${new URL(request.url).pathname.slice(1)}.tar.gz`);
    const bytes = new Uint8Array(readFileSync(file));
    if (corrupt) bytes[bytes.length - 1] ^= 1;
    return new Response(bytes);
  } });
  try {
    mkdirSync(engine, { recursive: true });
    for (const entry of ["src", "shell", "types", "schema", "vendor", "package.json"]) cpSync(join(pkg, entry), join(engine, entry), { recursive: true });
    for (const host of FRAME_HOSTS) cpSync(join(pkg, "dist", host), join(engine, "dist", host), { recursive: true });
    packFrameHosts(join(engine, "dist"), join(root, "packs"), `http://127.0.0.1:${server.port}`);
    for (const host of FRAME_HOSTS) rmSync(join(engine, "dist", host), { recursive: true });
    cpSync(join(repo, "tests/fixtures/browser-host"), project, { recursive: true });
    cookProject(project, repo);
    const runner = join(root, "build.ts");
    writeFileSync(runner, `import {webBuild,settingsOf} from ${JSON.stringify(join(engine, "src/node/web-build.ts"))};
const result=await webBuild(${JSON.stringify(project)},${JSON.stringify(join(project, "src/main.ts"))},${JSON.stringify(join(root, "out"))},true,{...settingsOf(${JSON.stringify(project)}),frames:["https://events.example.com"]});
console.log(JSON.stringify(result)); process.exit(result.ok ? 0 : 1);`);
    const run = async (cache: string) => {
      const child = Bun.spawn(["bun", runner], { env: { ...process.env, ORIGIN_ENGINE_CACHE: cache }, stdout: "pipe", stderr: "pipe" });
      const [code, stdout, stderr] = await Promise.all([child.exited, new Response(child.stdout).text(), new Response(child.stderr).text()]);
      return { code, stdout, stderr };
    };
    const cache = join(root, "cache");
    const first = await run(cache);
    expect(first.code, first.stderr + first.stdout).toBe(0);
    expect(requests).toBe(1);
    const manifest = JSON.parse(readFileSync(join(root, "out/og-build.json"), "utf8"));
    expect(manifest.frames).toEqual(["https://events.example.com"]);
    expect(readFileSync(join(root, "out", manifest.wasm))).toEqual(readFileSync(join(pkg, "dist", manifest.variant === "2d" ? "web-publish-2d-frames" : "web-publish-frames", "origin-engine_bg.wasm")));
    corrupt = true;
    const bad = await run(join(root, "empty-cache"));
    expect(bad.code).toBe(1);
    expect(bad.stdout).toContain("hash or length differs");
    server.stop(true);
    const cached = await run(cache);
    expect(cached.code, cached.stderr + cached.stdout).toBe(0);
    const offline = await run(join(root, "offline-cache"));
    expect(offline.code).toBe(1);
    expect(offline.stdout).toContain("offline or download refused");
  } finally {
    server.stop(true);
    rmSync(root, { recursive: true, force: true });
  }
}, slow(180_000));

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
