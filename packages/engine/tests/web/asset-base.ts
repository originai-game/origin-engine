// A package served from an asset base (docs/web-export.md §Asset base), in
// headless Chrome: a fixture is built with an asset base (webBuild's
// `publish.assets`, as `og build --assets` gives it), its page served from
// 127.0.0.1 and its cooked/ from a CDN on localhost (another origin, CORS
// `*`). The Worker's boot (src/web/boot.ts preloadBundle, run here on the
// page with dist/web's glue) must fetch the manifest against the page
// configuration's integrity and every file against the manifest's own: the
// intact package arrives whole, every file from the CDN; a package with one
// byte of one file changed fails that file alone, naming its integrity;
// a changed manifest fails the boot. A loader that ignores the integrity
// takes the changed bytes; one that checks only the manifest takes the
// changed file.
//
//   bun tests/web/asset-base.ts [--chrome <path>]
//
// Needs Chrome (cdp.ts), dist/web (`bun run build`) and og (the cook).
// Exits 1 on a failure.

import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild, DEFAULT_PUBLISH } from "../../src/node/web-build.ts";
import { Browser, chromePath } from "./cdp.ts";
import { serve } from "./serve.ts";
import { cookProject } from "../support/og-bin.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
  serve(o: { port: number; hostname: string; fetch(req: Request): Response | Promise<Response> }): { port: number; stop(): void };
};

const args = process.argv.slice(2);
const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..", "..");
const repo = join(pkg, "..", "..");
const glueFile = join(pkg, "dist", "web", "origin-engine.js");
if (!existsSync(glueFile) || !readFileSync(glueFile, "utf8").includes("og_web_bundle_integrity")) {
  throw new Error(`${glueFile} has no og_web_bundle_integrity: build the web host (\`bun run build\` in packages/engine)`);
}

// The CDN: the build's cooked/, answering any origin. `/tampered/` serves
// the same package with one byte of `victim` flipped, `/forged/` with the
// manifest changed.
const project = join(repo, "tests", "fixtures", "side-scroller");
cookProject(project, repo);
let cooked = "";
let victim = "";
const cdn = Bun.serve({
  port: 0,
  hostname: "127.0.0.1",
  fetch(req) {
    const path = decodeURIComponent(new URL(req.url).pathname);
    const cors = { "access-control-allow-origin": "*", "cross-origin-resource-policy": "cross-origin" };
    const m = /^\/(intact|tampered|forged)\/cooked\/(.+)$/.exec(path);
    const file = m ? resolve(cooked, m[2]!) : "";
    if (!m || !file.startsWith(cooked) || !existsSync(file)) return new Response("not found", { status: 404, headers: cors });
    const bytes = new Uint8Array(readFileSync(file));
    if ((m[1] === "tampered" && m[2] === victim) || (m[1] === "forged" && m[2] === ".cook.ogt")) bytes[bytes.length - 1] ^= 0x01;
    return new Response(new Blob([bytes]), { headers: cors });
  },
});
const base = (variant: string) => `http://localhost:${cdn.port}/${variant}`;

const dist = join(pkg, "dist", "test-asset-base");
const settings = { ...settingsOf(project), publish: { ...DEFAULT_PUBLISH, gate: "none" as const, assets: base("intact") } };
const built = await webBuild(project, join(project, "src", "main.ts"), dist, false, settings);
if (!built.ok) throw new Error(built.reason);
cooked = join(dist, "cooked");
const config = JSON.parse(/<script type="application\/json" id="og-config">(.*?)<\/script>/s.exec(readFileSync(join(dist, "index.html"), "utf8"))![1]!) as { assets?: { base: string; manifest: string } };
const listed = JSON.parse(readFileSync(join(dist, "og-build.json"), "utf8")) as { assets: { base: string; origin: string } | null; files: Array<{ path: string; bytes: number }> };
// The first group's blobs, the largest of them the one changed.
const firstGroup = listed.files.filter((f) => f.path.startsWith("cooked/") && f.path.endsWith(".ogb") && !f.path.startsWith("cooked/stream/")).sort((a, b) => b.bytes - a.bytes);
victim = firstGroup[0]!.path.slice("cooked/".length);

const boot = await Bun.build({ entrypoints: [join(pkg, "src", "web", "boot.ts")], target: "browser", format: "esm" });
if (!boot.success) throw new Error(`bundling boot.ts: ${boot.logs.map(String).join("\n")}`);
// Inlined in the page's module (a module script must not be served as HTML).
const bootCode = (await boot.outputs[0]!.text()).replaceAll("</script", "<\\/script");
const page = `<!doctype html><html><head><meta charset="utf-8"><title>asset base</title></head><body>
<script type="module">
import init, * as glue from "/game/origin-engine.js";
${bootCode}
globalThis.__boot = async (root, integrity) => {
  const puts = [];
  const fails = [];
  const g = {
    og_web_bundle_files: glue.og_web_bundle_files,
    og_web_bundle_integrity: glue.og_web_bundle_integrity,
    og_web_put_file: (path, bytes) => { puts.push(path); glue.og_web_put_file(path, bytes); },
    og_web_fail_file: (path, reason) => { fails.push({ path, reason }); glue.og_web_fail_file(path, reason); },
  };
  try {
    const r = await preloadBundle(g, root, () => {}, undefined, integrity);
    return { files: r.files, failed: r.failed, puts, fails, checked: r.integrity ? Object.keys(r.integrity).length : 0 };
  } catch (e) { return { error: String(e && e.stack || e), puts, fails }; }
};
try { await init(); globalThis.__ready = true; } catch (e) { globalThis.__error = String(e && e.stack || e); }
</script></body></html>`;
const server = serve({ dist, isolate: false, extra: { "/boot.html": page } });
const browser = await Browser.launch({ chrome: chromePath(args), width: 640, height: 480 });
const log: string[] = [];
let failures = 0;
const check = (ok: boolean, what: string, detail?: unknown) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${!ok && detail !== undefined ? `: ${JSON.stringify(detail).slice(0, 600)}` : ""}`);
  if (!ok) failures++;
};
type Booted = { files: number; failed: number; puts: string[]; fails: Array<{ path: string; reason: string }>; checked: number; error?: string };
try {
  check(config.assets?.base === base("intact") && /^sha384-[A-Za-z0-9+/]{64}$/.test(config.assets?.manifest ?? ""), "the page's configuration names the asset base and its manifest's integrity", config.assets);
  check(listed.assets?.base === base("intact") && listed.assets?.origin === `http://localhost:${cdn.port}`, "og-build.json names the asset base and the origin a CSP must allow", listed.assets);
  const tab = await browser.open(`http://127.0.0.1:${server.port}/boot.html`, log);
  await tab.until("globalThis.__ready || globalThis.__error", 60_000);
  const failed = await tab.eval<string | undefined>("globalThis.__error");
  if (failed) throw new Error(`the page did not load: ${failed}`);
  const run = (variant: string) => tab.eval<Booted>(`globalThis.__boot(${JSON.stringify(base(variant))}, ${JSON.stringify(config.assets!.manifest)})`);

  const intact = await run("intact");
  check(!intact.error && intact.failed === 0 && intact.files === firstGroup.length && intact.files > 2 && intact.fails.length === 0, `the intact package boots whole (${intact.files} files)`, intact);
  check(intact.puts.length > 1 && intact.puts.every((p) => p.startsWith(`${base("intact")}/cooked/`)), "every file came from the asset base", intact.puts.filter((p) => !p.startsWith(base("intact"))));
  check(intact.checked >= intact.files, `every file had its integrity (${intact.checked} listed)`, intact.checked);

  const tampered = await run("tampered");
  check(!tampered.error && tampered.failed === 1 && tampered.files === intact.files - 1, `one changed byte in ${victim} fails that file alone`, tampered);
  check(tampered.fails.length === 1 && tampered.fails[0]!.path === `${base("tampered")}/cooked/${victim}` && /integrity/.test(tampered.fails[0]!.reason), "its failure names the file and its integrity", tampered.fails);

  const forged = await run("forged");
  check(!forged.error && forged.files === 0 && forged.failed === 1 && forged.fails[0]?.path === `${base("forged")}/cooked/.cook.ogt`, "a changed manifest fails the boot before any file", forged);
} catch (e) {
  failures++;
  console.error(e instanceof Error ? e.stack : String(e));
  if (log.length > 0) console.error(log.slice(-20).join("\n"));
} finally {
  await browser.close();
  server.stop();
  cdn.stop();
}
process.exit(failures === 0 ? 0 : 1);
