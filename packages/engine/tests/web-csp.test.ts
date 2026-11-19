// A strict Content-Security-Policy serves a built page (docs/web-export.md
// §Hosting): a platform page's `script-src 'self'`, no inline style and no
// `blob:` Worker. index.html holds no inline script or style, the shell's
// CSS and install hook and the decoders' Workers are files of the build, and
// the lanes pool's helper starts its Workers from its own URL on the page's
// origin. Fails when the shell's CSS or install hook goes back inline, a
// decoder's file is not written or not what the Worker runs, or
// scripts/rayon-worker.sh stops rewriting wasm-bindgen-rayon's `blob:` start,
// or the Basis transcoder is evaluated from text again (no 'unsafe-eval')
// (tests/web/export.ts loads such a page in Chrome under that policy).
import { expect, test } from "bun:test";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { settingsOf, webBuild } from "../src/node/web-build.ts";
import { decoderSource } from "../src/web/audio-decoder.ts";
import { av1WorkerSource } from "../src/web/video.ts";
import { loadBasisTranscoder, VENDORED } from "../src/web/basis.ts";
import { cookProject } from "./support/og-bin.ts";
import { slow } from "./support/slow.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const NO_DEV_WASM = !existsSync(join(pkg, "dist", "web", "origin-engine_bg.wasm"));
const NO_PUBLISH_WASM = !existsSync(join(pkg, "dist", "web-publish-2d", "origin-engine_bg.wasm"));

/** What a page's `script-src 'self'` and `style-src 'self'` refuse in its HTML. */
function inline(html: string): string[] {
  const scripts = [...html.matchAll(/<script\b([^>]*)>/g)].filter((m) => !/\bsrc=/.test(m[1]!) && !/type="application\/json"/.test(m[1]!));
  return [...scripts.map((m) => m[0]), ...[...html.matchAll(/<style\b[^>]*>|\sstyle="/g)].map((m) => m[0])];
}

test.skipIf(NO_DEV_WASM)(
  "a built page holds no inline script or style, and its shell CSS and decoder Workers are its own files",
  async () => {
    const root = mkdtempSync(join(tmpdir(), "og-csp-"));
    try {
      const project = join(root, "game");
      cpSync(join(repo, "tests/fixtures/browser-host"), project, { recursive: true });
      cookProject(project, repo);
      const builds = [{ publish: false, out: join(root, "dev") }, ...(NO_PUBLISH_WASM ? [] : [{ publish: true, out: join(root, "publish") }])];
      for (const { publish, out } of builds) {
        const b = await webBuild(project, join(project, "src/main.ts"), out, publish, settingsOf(project));
        expect(b.ok, b.reason).toBe(true);
        const html = readFileSync(join(out, "index.html"), "utf8");
        expect(inline(html)).toEqual([]);
        const names = b.manifest!.names;
        const served = (f: string) => names[f] ?? f;
        expect(html).toContain(`<link rel="stylesheet" href="./${served("shell.css")}">`);
        expect(readFileSync(join(out, served("shell.css")), "utf8")).toContain("#og-gate");
        expect(readFileSync(join(out, served("audio-decoder.js")), "utf8")).toBe(decoderSource());
        if (existsSync(join(out, served("av1/og-av1.wasm")))) expect(readFileSync(join(out, served("av1/av1-worker.js")), "utf8")).toBe(av1WorkerSource());
        if (publish) {
          // The PWA's hook runs from the head before the shell's module.
          expect(html.indexOf(`<script src="./${served("install.js")}"></script>`)).toBeGreaterThan(-1);
          expect(html.indexOf(`src="./${served("install.js")}"`)).toBeLessThan(html.indexOf('type="module"'));
          expect(readFileSync(join(out, served("install.js")), "utf8")).toContain("__ogInstallPrompt");
          expect(served("shell.css")).toMatch(/^shell\.[0-9a-f]{12}\.css$/);
        }
      }
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  },
  slow(120_000),
);

/** wasm-bindgen-rayon's helper as Cargo.lock pins it, from Cargo's registry (none before a build fetched it). */
function pinnedHelper(): string | null {
  const lock = readFileSync(join(repo, "Cargo.lock"), "utf8");
  const version = /name = "wasm-bindgen-rayon"\nversion = "([^"]+)"/.exec(lock)?.[1];
  const registry = join(process.env["CARGO_HOME"] ?? join(homedir(), ".cargo"), "registry", "src");
  if (!version || !existsSync(registry)) return null;
  for (const index of readdirSync(registry)) {
    const file = join(registry, index, `wasm-bindgen-rayon-${version}`, "src", "workerHelpers.no-bundler.js");
    if (existsSync(file)) return file;
  }
  return null;
}
const HELPER = pinnedHelper();

test.skipIf(HELPER === null)("the lanes pool's helper starts its Workers from its own URL on the page's origin, from a blob: copy elsewhere", async () => {
  const root = mkdtempSync(join(tmpdir(), "og-rayon-"));
  const saved = { Worker: globalThis.Worker, fetch: globalThis.fetch, location: (globalThis as { location?: unknown }).location };
  try {
    const snippets = join(root, "snippets", "wasm-bindgen-rayon-0000", "src");
    mkdirSync(snippets, { recursive: true });
    cpSync(HELPER!, join(snippets, "workerHelpers.no-bundler.js"));
    writeFileSync(join(root, "origin-engine-lanes.js"), "");
    const run = Bun.spawnSync(["sh", join(pkg, "scripts", "rayon-worker.sh"), root]);
    expect(run.exitCode, new TextDecoder().decode(run.stderr)).toBe(0);
    const helper = join(snippets, "workerHelpers.no-bundler.js");
    // Each run's own copy, so the module is evaluated with that run's page origin.
    const started = async (origin: string | undefined) => {
      const urls: string[] = [];
      const fetched: string[] = [];
      (globalThis as { location?: unknown }).location = origin === undefined ? undefined : { origin };
      globalThis.fetch = (async (url: string) => {
        fetched.push(String(url));
        return new Response("self.onmessage = null;");
      }) as unknown as typeof fetch;
      globalThis.Worker = class {
        readonly #listeners: Array<(e: { data: unknown }) => void> = [];
        constructor(url: string | URL) {
          urls.push(String(url));
        }
        addEventListener(_type: string, fn: (e: { data: unknown }) => void) {
          this.#listeners.push(fn);
        }
        removeEventListener() {}
        postMessage() {
          queueMicrotask(() => this.#listeners.forEach((fn) => fn({ data: { type: "wasm_bindgen_worker_ready" } })));
        }
      } as unknown as typeof Worker;
      const copy = join(snippets, `run-${urls.length}-${origin?.replace(/\W/g, "") ?? "no-location"}.js`);
      cpSync(helper, copy);
      const self = pathToFileURL(realpathSync(copy)).href;
      const m = (await import(self)) as { startWorkers(module: unknown, memory: unknown, builder: unknown): Promise<void> };
      await m.startWorkers({}, {}, { numThreads: () => 2, receiver: () => 0, mainJS: () => "", build: () => {} });
      return { urls, fetched, self };
    };
    // A file: URL's origin is "null": the page's origin when the page is a file too.
    const own = await started(new URL(pathToFileURL(helper).href).origin);
    expect(own.urls).toEqual([own.self, own.self]);
    expect(own.fetched).toEqual([]);
    const file = await started(undefined);
    expect(file.urls).toEqual([file.self, file.self]);
    expect(file.fetched).toEqual([]);
    const cdn = await started("https://page.example");
    expect(cdn.urls.every((u) => u.startsWith("blob:"))).toBe(true);
    expect(cdn.fetched).toEqual([cdn.self, cdn.self]);
    // Rewriting a helper again changes nothing.
    const once = readFileSync(helper, "utf8");
    expect(Bun.spawnSync(["sh", join(pkg, "scripts", "rayon-worker.sh"), root]).exitCode).toBe(0);
    expect(readFileSync(helper, "utf8")).toBe(once);
  } finally {
    globalThis.Worker = saved.Worker;
    globalThis.fetch = saved.fetch;
    (globalThis as { location?: unknown }).location = saved.location;
    rmSync(root, { recursive: true, force: true });
  }
});

test("the Basis transcoder loads without evaluating text, from the build's hashed file and the wasm bytes it is given", async () => {
  // As a build serves it: under a hashed name, no basis_transcoder.wasm beside it.
  const dir = mkdtempSync(join(tmpdir(), "og-basis-csp-"));
  const js = join(dir, "basis_transcoder.0123456789ab.js");
  cpSync(fileURLToPath(VENDORED.js), js);
  // What a policy without 'unsafe-eval' refuses: the Function constructor and eval.
  const real = { Function: globalThis.Function, eval: globalThis.eval };
  const refuse = () => {
    throw new EvalError("Evaluating a string as JavaScript violates the page's Content Security Policy (no 'unsafe-eval')");
  };
  globalThis.Function = new Proxy(real.Function, { construct: refuse, apply: refuse });
  globalThis.eval = refuse as typeof eval;
  try {
    const basis = await loadBasisTranscoder(pathToFileURL(js), readFileSync(fileURLToPath(VENDORED.wasm)));
    expect(typeof basis.KTX2File).toBe("function");
  } finally {
    globalThis.Function = real.Function;
    globalThis.eval = real.eval;
    rmSync(dir, { recursive: true, force: true });
  }
});
