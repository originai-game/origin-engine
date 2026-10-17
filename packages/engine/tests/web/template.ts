// The template in headless Chrome, through `og build`'s web output: builds
// templates/game (cooking it first when cooked/ is missing) with
// src/node/web-build.ts, serves the output with COOP/COEP, loads the play
// page and calls the probe published on globalThis.__runtime_probe__ over the
// request ring: hello, capture (answered once the readback lands), dump world.
// Prints the result as JSON and writes the captured frame.
//
//   bun tests/web/template.ts [--webgl] [--publish] [--entry <file>] [--png <out.png>] [--chrome <path>]

import { existsSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { webBuild } from "../../src/node/web-build.ts";

declare const Bun: {
  serve(o: { port: number; fetch(req: Request): Response | Promise<Response> }): { port: number; stop(): void };
  file(path: string): Blob & { exists(): Promise<boolean> };
  spawn(cmd: string[], o?: { stdout?: string; stderr?: string }): { kill(): void; exited: Promise<number> };
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string; env?: Record<string, string | undefined> }): { exitCode: number; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "game");
const chrome =
  opt("--chrome") ??
  process.env.CHROME ??
  `${process.env.HOME}/.agent-browser/browsers/chrome-150.0.7871.115/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`;
if (!existsSync(chrome)) {
  console.error(`no Chrome at ${chrome}: pass --chrome <path> or set CHROME`);
  process.exit(1);
}
if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
  const r = Bun.spawnSync(["cargo", "run", "--release", "-q", "-p", "og-cook", "--bin", "og-cook", "--", project], { cwd: repo, stdout: "ignore", stderr: "pipe" });
  if (r.exitCode !== 0) {
    console.error(`cooking ${project} failed: ${new TextDecoder().decode(r.stderr)}`);
    process.exit(1);
  }
}
const entry = resolve(opt("--entry") ?? (existsSync(join(project, "src", "main.ts")) ? join(project, "src", "main.ts") : join(pkg, "examples", "hello.ts")));
const out = join(pkg, "dist", "test-template");
const built = await webBuild(project, entry, out, args.includes("--publish"));
if (!built.ok) {
  console.error(built.reason);
  process.exit(1);
}

const harness = `<script type="module">
const post = (body) => fetch("/result", { method: "POST", body: JSON.stringify(body) });
try {
  const t0 = performance.now();
  while (!globalThis.__runtime_probe__) {
    if (performance.now() - t0 > 30000) throw new Error("the probe was never published");
    await new Promise((r) => setTimeout(r, 50));
  }
  const p = globalThis.__runtime_probe__;
  const ready = performance.now() - t0;
  await new Promise((r) => setTimeout(r, 1500));
  const hello = await p.hello();
  const c0 = performance.now();
  const capture = await p.capture();
  const captureMs = performance.now() - c0;
  const dump = await p.dump({ what: "world" });
  await post({ crossOriginIsolated, readyMs: Math.round(ready), hello, captureMs: Math.round(captureMs), capture, dump });
} catch (e) {
  await post({ error: String(e && e.stack || e) });
}
</script>`;
let resolveResult!: (v: unknown) => void;
const result = new Promise((r) => (resolveResult = r));
const types: Record<string, string> = { ".js": "text/javascript", ".wasm": "application/wasm", ".html": "text/html", ".ogb": "application/octet-stream" };
const server = Bun.serve({
  port: 0,
  async fetch(req) {
    const url = new URL(req.url);
    const headers: Record<string, string> = {
      "cache-control": "no-store",
      "cross-origin-opener-policy": "same-origin",
      "cross-origin-embedder-policy": "require-corp",
    };
    if (url.pathname === "/result") {
      resolveResult(await req.json());
      return new Response("ok", { headers });
    }
    if (url.pathname === "/" || url.pathname === "/index.html") {
      const page = (await Bun.file(join(out, "index.html")).text()).replace("</body>", `${harness}</body>`);
      return new Response(page, { headers: { ...headers, "content-type": "text/html" } });
    }
    const path = join(out, decodeURIComponent(url.pathname));
    const file = Bun.file(path);
    if (!(await file.exists())) return new Response("not found", { status: 404, headers });
    const ext = path.slice(path.lastIndexOf("."));
    return new Response(file, { headers: { ...headers, "content-type": types[ext] ?? "application/octet-stream" } });
  },
});
const query = args.includes("--webgl") ? "?webgl=1" : "";
const profile = join(out, `profile-${Date.now()}`);
const browser = Bun.spawn(
  [
    chrome, "--headless=new", "--no-first-run", "--no-default-browser-check", `--user-data-dir=${profile}`,
    "--window-size=1280,720", "--use-angle=metal", "--enable-unsafe-webgpu", "--enable-features=WebGPU",
    "--ignore-gpu-blocklist", "--disable-gpu-vsync", "--remote-debugging-port=0",
    `http://127.0.0.1:${server.port}/${query}`,
  ],
  { stdout: "ignore", stderr: "ignore" },
);
const timeout = new Promise((r) => setTimeout(() => r({ error: "timed out after 90 s" }), 90_000));
const answer = (await Promise.race([result, timeout])) as { error?: string; capture?: { png: string; width: number; height: number; tick: number } | null };
browser.kill();
await browser.exited;
server.stop();
rmSync(profile, { recursive: true, force: true });
const png = opt("--png");
if (answer.capture && png) {
  const bin = atob(answer.capture.png);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  writeFileSync(png, bytes);
}
if (answer.capture) (answer.capture as { png: string }).png = `<${answer.capture.png.length} base64 chars>`;
console.log(JSON.stringify({ entry, build: { total: built.total, probe: built.probe }, ...answer }, null, 2));
process.exit(answer.error || !answer.capture ? 1 : 0);
