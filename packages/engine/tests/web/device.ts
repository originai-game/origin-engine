// `og run --device chrome` end to end: the og binary cooks and exports
// templates/game's publish build, serves it with COOP/COEP on 127.0.0.1, opens
// it in headless Chrome over the DevTools protocol, taps the play gate and
// times the first frame and the played frames. Checked: the report and
// device.json exist; the first frame is after navigation; the display
// interval and engine work percentiles are positive and ordered; the device
// section has the user agent, DPR, viewport, cores and the tier; the page
// was cross-origin isolated and the server sent COOP/COEP on the page and
// the wasm. The first run lands on WebGPU, a second with --webgl on the
// WebGL2 fallback. A Chrome path that does not exist is refused in one
// line, exit 1.
//
//   bun tests/web/device.ts [--project <dir>] [--out <dir>] [--chrome <path>] [--seconds N]
//
// --project (relative to the repository) runs another project's publish
// build instead of templates/game's: the example games' own device runs,
// examples/brasswick and examples/meridian-loop (release row F4), whose
// default output is dist/test-device/<project's name>.
//
// Writes <out>/{webgpu,webgl2}/{report.md,device.json} and
// <out>/device-result.json (default dist/test-device). Exits 1 when a check
// fails. A missing Chrome is an error (cdp.ts chromePath), never a skip.

import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromePath } from "./cdp.ts";
import { ogBin } from "../support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "pipe"; stderr: "pipe" }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const args = process.argv.slice(2);
const opt = (name: string) => (args.includes(name) ? args[args.indexOf(name) + 1] : undefined);
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const repo = join(pkg, "..", "..");
const projectRel = opt("--project");
const out = resolve(opt("--out") ?? join(pkg, "dist", "test-device", ...(projectRel ? [projectRel.split("/").pop()!] : [])));
const seconds = opt("--seconds") ?? "3";
const chrome = chromePath(args);
const og = ogBin();
const project = join(repo, projectRel ?? join("templates", "game"));

let failures = 0;
const checks: Array<{ check: string; ok: boolean; detail?: unknown }> = [];
function check(ok: boolean, what: string, detail?: unknown) {
  checks.push({ check: what, ok, ...(ok && detail === undefined ? {} : { detail }) });
  if (!ok) failures++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}${!ok && detail !== undefined ? `: ${JSON.stringify(detail).slice(0, 400)}` : ""}`);
}

const run = (extra: string[]) => {
  const r = Bun.spawnSync([og, "run", "--device", "chrome", project, "--chrome", chrome, "--seconds", seconds, ...extra], {
    cwd: repo,
    env: { ...process.env, OG_ENGINE_PACKAGE: pkg },
    stdout: "pipe",
    stderr: "pipe",
  });
  const decode = (b: Uint8Array) => new TextDecoder().decode(b);
  return { code: r.exitCode, stdout: decode(r.stdout), stderr: decode(r.stderr) };
};

interface Stats {
  count: number;
  mean: number;
  p50: number;
  p95: number;
  p99: number;
  max: number;
}
interface DeviceReport {
  target: string;
  scores: { firstFrame: number; frameTime: number };
  findings: Array<{ measure: string; detail: string; fix: string }>;
  fps: number;
  measured: {
    firstFrameMs: number;
    startMs: number;
    tier: string;
    frames: number;
    ticks: number;
    interval: Stats;
    work: Stats;
    device: { userAgent: string; dpr: number; viewport: { width: number; height: number }; hardwareConcurrency: number | null; crossOriginIsolated: boolean; isSecureContext: boolean };
  };
  served: Record<string, Record<string, string>>;
}

const ordered = (s: Stats) => s.count > 0 && s.p50 > 0 && s.p50 <= s.p95 && s.p95 <= s.p99 && s.p99 <= s.max && s.mean > 0;

const results: Record<string, unknown> = {};
for (const [name, extra, tiers] of [
  ["webgpu", [] as string[], ["webgpu"]],
  ["webgl2", ["--webgl"], ["webgl2"]],
] as const) {
  const dir = join(out, name);
  rmSync(dir, { recursive: true, force: true });
  const r = run(["--out", dir, ...extra]);
  check(r.code === 0, `${name}: og run --device chrome exits 0`, r.stderr.trim().split("\n").slice(-4));
  check(/^og run --device chrome: .*first frame \d+ ms, frame p50 [\d.]+ \/ p95 [\d.]+ \/ p99 [\d.]+ ms/m.test(r.stdout), `${name}: prints the summary line`, r.stdout);
  check(/serving .* at http:\/\/127\.0\.0\.1:\d+\/.*\(COOP\/COEP\)/.test(r.stderr), `${name}: prints the URLs it serves`, r.stderr.split("\n")[0]);
  const json = join(dir, "device.json");
  check(existsSync(join(dir, "report.md")) && existsSync(json), `${name}: report.md and device.json written`);
  if (!existsSync(json)) continue;
  const d = JSON.parse(readFileSync(json, "utf8")) as DeviceReport;
  const m = d.measured;
  results[name] = { tier: m.tier, firstFrameMs: m.firstFrameMs, interval: m.interval, work: m.work, fps: d.fps, scores: d.scores, findings: d.findings.map((f) => f.measure) };
  check(m.firstFrameMs > 0 && m.startMs >= m.firstFrameMs, `${name}: first frame after navigation, play after it`, { firstFrameMs: m.firstFrameMs, startMs: m.startMs });
  check((tiers as readonly string[]).includes(m.tier), `${name}: tier ${m.tier}`, tiers);
  check(ordered(m.interval) && m.interval.count === Math.min(m.frames, 4096), `${name}: display interval p50 ≤ p95 ≤ p99 ≤ max, all > 0`, m.interval);
  check(ordered(m.work), `${name}: engine work p50 ≤ p95 ≤ p99 ≤ max, all > 0`, m.work);
  check(m.ticks > 0 && d.fps > 0, `${name}: the game ticked while timed`, { ticks: m.ticks, fps: d.fps });
  const dev = m.device;
  check(/Chrome\//.test(dev.userAgent) && dev.dpr > 0 && dev.viewport.width > 0 && dev.viewport.height > 0 && (dev.hardwareConcurrency ?? 0) > 0, `${name}: device info (user agent, DPR, viewport, cores)`, dev);
  check(dev.crossOriginIsolated && dev.isSecureContext, `${name}: the page is cross-origin isolated in a secure context`, dev);
  const page = d.served["/"] ?? {};
  const wasm = d.served["/origin-engine_bg.wasm"] ?? {};
  check(
    page["cross-origin-opener-policy"] === "same-origin" && page["cross-origin-embedder-policy"] === "require-corp" && wasm["cross-origin-embedder-policy"] === "require-corp",
    `${name}: COOP/COEP served on the page and the wasm`,
    d.served,
  );
  check(typeof d.scores.firstFrame === "number" && typeof d.scores.frameTime === "number" && d.findings.every((f) => f.fix.length > 0), `${name}: scores and a fix per finding`, d);
}

// A missing prerequisite: one line naming the fix, exit 1.
{
  const r = run(["--out", join(out, "refused"), "--chrome", join(out, "no-such-chrome")]);
  const lines = r.stderr.trim().split("\n").filter((l) => l.startsWith("og run --device: ") && !l.includes("serving"));
  check(r.code === 1 && lines.length === 1 && /no Chrome at .*: point --chrome/.test(lines[0]!), "a missing Chrome: exit 1 and one line naming the fix", r.stderr);
}

mkdirSync(out, { recursive: true });
writeFileSync(join(out, "device-result.json"), JSON.stringify({ project: project.slice(repo.length + 1), chrome, seconds: Number(seconds), results, checks, failures }, null, 1) + "\n");
console.log(`${failures === 0 ? "ok" : "FAIL"}: ${checks.length - failures}/${checks.length} checks; ${join(out, "device-result.json")}`);
process.exit(failures === 0 ? 0 : 1);
