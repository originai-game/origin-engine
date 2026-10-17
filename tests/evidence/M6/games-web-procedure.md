# M6 games: world hash after 600 ticks on both hosts (procedure)

Result: `games-world-hash-600-two-hosts.json`. Runner 692d75bc9ee5997b and match3
ae3f57f7c49523a1 on the native host (twice each) and in headless Chrome for Testing
150.0.7871.115 (WebGPU, cross-origin isolated): equal.

Native: `og run <game> --headless --commands "<script>;10:dump_world:w600.json;10:screenshot:native-600.png;10.05:quit"`
(the script strings are in the JSON).

Web: `og build <game> --out <dist>`, then `WINDOW=1280,863 bun drive.ts <dist> <out> "<script>"`
(the window size gives a 1280x720 surface; at 1280,720 headless Chrome's page is 1280x577).
The page loads with `?autoplay=0`; the harness schedules the script at tick 0 through
`probe.commands`, advances 1 tick (title frame), 599 more, captures, then `probe.dump({what:"world"})`.

Note: the layout (HUD extents, match3 pointer mapping) is derived from the render
target and is world state, so hashes compare only at the same surface size.

drive.ts:

```ts
// Drives one built archetype (og build output) in headless Chrome for Testing 150:
// serves the dist with COOP/COEP, injects a harness into index.html that waits for
// the probe, schedules the same command script the native run uses, advances 600
// ticks through the probe, dumps the world (its hash) and captures frames, then
// POSTs everything back. Writes <out>/web-*.png and <out>/web-result.json.
//
//   bun drive.ts <dist> <out> '<script>' [extra-steps-json]
// extra-steps-json: [{ "advance": n, "capture": "name" }, ...] taken after the 600-tick dump.
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [dist, out, script, extraJson] = process.argv.slice(2) as [string, string, string, string | undefined];
const extra = JSON.parse(extraJson ?? "[]") as Array<{ commands?: string; advance?: number; capture?: string }>;
mkdirSync(out, { recursive: true });
const chrome = process.env.CHROME ?? `${process.env.HOME}/.agent-browser/browsers/chrome-150.0.7871.115/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`;

const harness = `<script type="module">
const post = (body) => fetch("/result", { method: "POST", body: JSON.stringify(body) });
const steps = ${JSON.stringify(extra)};
try {
  const t0 = performance.now();
  while (!globalThis.__runtime_probe__) {
    if (performance.now() - t0 > 30000) throw new Error("the probe was never published");
    await new Promise((r) => setTimeout(r, 50));
  }
  const p = globalThis.__runtime_probe__;
  const hello = await p.hello();
  const tick0 = await p.tick();
  // The script is scheduled at tick 0, as og run schedules --commands; tick 1 is still the title.
  const scheduled = await p.commands(${JSON.stringify(script)});
  await p.advance(1);
  const title = await p.capture();
  const advanced = await p.advance(599);
  const frame600 = await p.capture();
  const dump = await p.dump({ what: "world" });
  const afterDump = await p.capture();
  const state = await p.state();
  const afterState = await p.capture();
  const more = [];
  for (const s of steps) {
    if (s.commands) more.push({ scheduled: await p.commands(s.commands) });
    if (s.advance) more.push({ advanced: await p.advance(s.advance) });
    if (s.capture) more.push({ name: s.capture, capture: await p.capture(), state: await p.state() });
  }
  await post({ crossOriginIsolated, hello, tick0, title, scheduled, advanced, hash: dump.hash, tick: dump.tick, entities: dump.entities, state, frame600, afterDump, afterState, more });
} catch (e) {
  await post({ error: String((e && e.stack) || e) });
}
</script>`;

let done!: (v: unknown) => void;
const result = new Promise((r) => (done = r));
const types: Record<string, string> = { ".js": "text/javascript", ".wasm": "application/wasm", ".html": "text/html" };
const headers = { "cache-control": "no-store", "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp" };
const server = Bun.serve({
  port: 0,
  async fetch(req) {
    const url = new URL(req.url);
    if (url.pathname === "/result") {
      done(await req.json());
      return new Response("ok", { headers });
    }
    const path = join(dist, url.pathname === "/" ? "index.html" : decodeURIComponent(url.pathname));
    const file = Bun.file(path);
    if (!(await file.exists())) return new Response("not found", { status: 404, headers });
    if (path.endsWith("index.html")) {
      const html = (await file.text()).replace("</body>", harness + "</body>");
      return new Response(html, { headers: { ...headers, "content-type": "text/html" } });
    }
    const ext = path.slice(path.lastIndexOf("."));
    return new Response(file, { headers: { ...headers, "content-type": types[ext] ?? "application/octet-stream" } });
  },
});
const profile = join(out, `.profile-${Date.now()}`);
const browser = Bun.spawn(
  [chrome, "--headless=new", "--no-first-run", "--no-default-browser-check", `--user-data-dir=${profile}`,
    `--window-size=${process.env.WINDOW ?? "1280,720"}`, "--hide-scrollbars", "--use-angle=metal", "--enable-unsafe-webgpu", "--enable-features=WebGPU",
    "--ignore-gpu-blocklist", "--disable-gpu-vsync", "--remote-debugging-port=0",
    `http://127.0.0.1:${server.port}/?autoplay=0`],
  { stdout: "ignore", stderr: "ignore" },
);
const answer = (await Promise.race([result, new Promise((r) => setTimeout(() => r({ error: "timed out after 90 s" }), 90_000))])) as Record<string, any>;
browser.kill();
await browser.exited;
server.stop();
await Bun.$`rm -rf ${profile}`.quiet();
if (answer.error) {
  console.error(answer.error);
  process.exit(1);
}
const png = (name: string, c: { png: string } | undefined) => c && writeFileSync(join(out, `web-${name}.png`), Buffer.from(c.png, "base64"));
png("title", answer.title);
png("600", answer.frame600);
png("600-after-dump", answer.afterDump);
png("600-after-state", answer.afterState);
for (const m of answer.more ?? []) if (m.capture) png(m.name, m.capture);
const strip = (c: any) => (c ? { tick: c.tick, width: c.width, height: c.height } : c);
const summary = {
  crossOriginIsolated: answer.crossOriginIsolated,
  hello: answer.hello,
  tick0: answer.tick0,
  scheduled: answer.scheduled,
  advanced: answer.advanced,
  tick: answer.tick,
  hash: answer.hash,
  entities: answer.entities,
  facts: answer.state?.facts,
  frames: { title: strip(answer.title), "600": strip(answer.frame600), ...Object.fromEntries((answer.more ?? []).filter((m: any) => m.capture).map((m: any) => [m.name, { ...strip(m.capture), facts: m.state?.facts }])) },
};
writeFileSync(join(out, "web-result.json"), JSON.stringify(summary, null, 2));
console.log(JSON.stringify(summary, null, 2));
```
