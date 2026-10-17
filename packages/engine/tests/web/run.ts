// The hello world in headless Chrome: builds the page and the engine Worker,
// serves them with COOP/COEP (cross-origin isolation, so input crosses the
// SharedArrayBuffer ring), runs Chrome for Testing headless and prints the
// page's result as JSON. Needs `bun run build` first (dist/web).
//
//   bun tests/web/run.ts [--webgl] [--no-isolation] [--chrome <path>]

import { fileURLToPath } from "node:url";
import { existsSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";

declare const Bun: {
  build(o: { entrypoints: string[]; outdir: string; target: string; format: string }): Promise<{ success: boolean; logs: unknown[] }>;
  serve(o: { port: number; fetch(req: Request): Response | Promise<Response> }): { port: number; stop(): void };
  file(path: string): Blob & { exists(): Promise<boolean> };
  spawn(cmd: string[], o?: { stdout?: string; stderr?: string }): { kill(): void; exited: Promise<number> };
};

const args = process.argv.slice(2);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const out = join(pkg, "dist", "test-web");
const chrome =
  args[args.indexOf("--chrome") + 1] && args.includes("--chrome")
    ? args[args.indexOf("--chrome") + 1]!
    : (process.env.CHROME ?? `${process.env.HOME}/.agent-browser/browsers/chrome-150.0.7871.115/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`);
if (!existsSync(join(pkg, "dist", "web", "origin-engine_bg.wasm"))) {
  console.error("no dist/web: run `bun run build` in packages/engine first");
  process.exit(1);
}
for (const entry of ["page.ts", "hello.worker.ts"]) {
  const r = await Bun.build({ entrypoints: [join(here, entry)], outdir: out, target: "browser", format: "esm" });
  if (!r.success) {
    console.error(r.logs);
    process.exit(1);
  }
}
const isolate = !args.includes("--no-isolation");
const page = `<!doctype html><html><head><meta charset="utf-8"><title>Origin Engine hello</title>
<style>html,body{margin:0;height:100%;background:#14161b}canvas{display:block;width:100vw;height:100vh}</style></head>
<body><canvas></canvas><script type="module" src="./page.js"></script></body></html>`;
let resolve!: (v: unknown) => void;
const result = new Promise((r) => (resolve = r));
const types: Record<string, string> = { ".js": "text/javascript", ".wasm": "application/wasm", ".html": "text/html" };
const server = Bun.serve({
  port: 0,
  async fetch(req) {
    const url = new URL(req.url);
    const headers: Record<string, string> = { "cache-control": "no-store" };
    if (isolate) Object.assign(headers, { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp" });
    if (url.pathname === "/result") {
      resolve(await req.json());
      return new Response("ok", { headers });
    }
    if (url.pathname === "/") return new Response(page, { headers: { ...headers, "content-type": "text/html" } });
    const path = url.pathname.startsWith("/web/") ? join(pkg, "dist", url.pathname) : join(out, url.pathname);
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
const timeout = new Promise((r) => setTimeout(() => r({ error: "timed out after 60 s" }), 60_000));
const answer = await Promise.race([result, timeout]);
browser.kill();
await browser.exited;
server.stop();
rmSync(profile, { recursive: true, force: true });
console.log(JSON.stringify(answer, null, 2));
process.exit((answer as { error?: string }).error ? 1 : 0);
