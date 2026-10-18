// The packed engine on the platform's E2B executor (billable, one short run):
// installs @origingame/engine and @origingame/engine-linux-x64 from the
// tarballs scripts/pack.sh made into a copy of the runner archetype inside
// the `origin-runner-probe` sandbox, cooks it with the packed og, runs
// `og run --headless` with a screenshot and a world dump on Mesa lavapipe,
// and downloads them. Reads E2B_API_KEY from the platform checkout's .env,
// as tools/bench/e2b-run.ts does.
//
//   bun scripts/e2b-addon.ts [--out <dir>] [--template origin-runner-probe]
//
// Prints a JSON report and writes <out>/sandbox-frame.png, sandbox-world.json, sandbox-run.json.

import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

declare module "node:fs" {
  export function readdirSync(path: string): string[];
}
declare module "node:path" {
  export function relative(from: string, to: string): string;
}

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const out = opt("--out") ?? join(pkg, "dist", "e2b");
const template = opt("--template") ?? "origin-runner-probe";
const env = readFileSync(process.env.OG_PLATFORM_ENV ?? "/Volumes/app/origingame/.env", "utf8");
const apiKey = env.match(/^E2B_API_KEY=(.+)$/m)?.[1]?.trim();
if (!apiKey) throw new Error("no E2B_API_KEY in the platform .env (OG_PLATFORM_ENV)");
const e2b = process.env.OG_E2B_SDK ?? join(repo, "tools", "bench", "node_modules", "e2b");
if (!existsSync(e2b)) throw new Error(`no e2b SDK at ${e2b}: run bun install in tools/bench`);
const { Sandbox } = (await import(e2b)) as { Sandbox: { create(t: string, o: Record<string, unknown>): Promise<any> } };
const version = (JSON.parse(readFileSync(join(pkg, "package.json"), "utf8")) as { version: string }).version;
const tarballs = [`origingame-engine-${version}.tgz`, `origingame-engine-linux-x64-${version}.tgz`].map((t) => join(pkg, "dist", "pack", t));
for (const t of tarballs) if (!existsSync(t)) throw new Error(`${t} is missing: sh scripts/pack.sh --linux-x64`);

function walk(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    if (["cooked", "dist", "og-out", ".og", "node_modules"].includes(n)) return [];
    const p = join(dir, n);
    return statSync(p).isDirectory() ? walk(p) : [p];
  });
}
const game = join(repo, "templates", "archetypes", "runner");
const files = walk(game).map((p) => ({ path: `/tmp/og/game/${relative(game, p)}`, data: readFileSync(p).buffer as ArrayBuffer }));
files.push(...tarballs.map((t) => ({ path: `/tmp/og/${t.split("/").pop()}`, data: readFileSync(t).buffer as ArrayBuffer })));

const report: Record<string, unknown> = { template };
const t0 = performance.now();
const sb = await Sandbox.create(template, { apiKey, timeoutMs: 600_000, metadata: { purpose: "og-engine-addon" } });
report.sandbox = sb.sandboxId;
report.createMs = Math.round(performance.now() - t0);
const sh = async (cmd: string, envs: Record<string, string> = {}) => {
  const r = await sb.commands.run(cmd, { timeoutMs: 300_000, envs }).catch((e: { result?: unknown }) => e.result ?? { exitCode: -1, stdout: "", stderr: String(e) });
  return { exitCode: r.exitCode as number, stdout: String(r.stdout ?? "").trim(), stderr: String(r.stderr ?? "").trim() };
};
try {
  await sb.files.write(files);
  report.machine = (await sh("uname -m; (. /etc/os-release; echo $PRETTY_NAME); ldd --version | head -1; nproc; which bun node; bun --version; ls /usr/share/vulkan/icd.d/ 2>/dev/null")).stdout.split("\n");
  const icd = (await sh("ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -1")).stdout;
  const vk = icd ? { VK_ICD_FILENAMES: icd, WGPU_BACKEND: "vulkan" } : {};
  // A project that depends on the two tarballs, as npm would install them from the registry.
  await sb.files.write([{ path: "/tmp/og/game/package.json", data: JSON.stringify({ name: "runner", private: true, type: "module", dependencies: { "@origingame/engine": `file:../${tarballs[0]!.split("/").pop()}`, "@origingame/engine-linux-x64": `file:../${tarballs[1]!.split("/").pop()}` } }) }]);
  const install = await sh("cd /tmp/og/game && bun install 2>&1 | tail -5");
  report.install = install;
  const og = "/tmp/og/game/node_modules/.bin/og";
  report.version = await sh(`${og} --version`);
  report.doctor = await sh(`cd /tmp/og/game && ${og} doctor`, vk);
  const cook = await sh(`cd /tmp/og/game && ${og} cook .`);
  report.cook = cook.stdout.split("\n").slice(-2);
  const r0 = performance.now();
  const run = await sh(`cd /tmp/og/game && ${og} run . --headless --size 640x360 --out out --commands "0.5:key_down:Space;0.6:key_up:Space;1:screenshot:frame.png;1:dump_world:world.json;1.05:quit" --json`, vk);
  report.runMs = Math.round(performance.now() - r0);
  report.run = { exitCode: run.exitCode, stdout: run.stdout.split("\n").slice(-6), stderr: run.stderr.split("\n").slice(-6) };
  mkdirSync(out, { recursive: true });
  if (run.exitCode === 0) {
    const png = (await sb.files.read("/tmp/og/game/out/frame.png", { format: "bytes" })) as Uint8Array;
    writeFileSync(join(out, "sandbox-frame.png"), png);
    const world = (await sb.files.read("/tmp/og/game/out/world.json")) as string;
    writeFileSync(join(out, "sandbox-world.json"), world);
    const w = JSON.parse(world) as { tick: number; hash: string; entities: number };
    report.world = { tick: w.tick, hash: w.hash, entities: w.entities };
    report.frameBytes = png.length;
  }
} finally {
  await sb.kill();
}
mkdirSync(out, { recursive: true });
writeFileSync(join(out, "sandbox-run.json"), JSON.stringify(report, null, 1) + "\n");
console.log(JSON.stringify(report, null, 1));
process.exit((report.run as { exitCode: number } | undefined)?.exitCode === 0 ? 0 : 1);
