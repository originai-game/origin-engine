#!/usr/bin/env bun
/** Schedule a dependency DAG; commands and test selection live beside this file. */
import { closeSync, existsSync, mkdirSync, openSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { execFileSync, spawnSync } from "node:child_process";

export const NEEDS: Record<string, string[]> = {
  preflight: [], fmt: [], clippy: ["fmt"], "rust-build": ["clippy", "preflight"],
  "native-ready": ["rust-build"], drift: ["rust-build"], "api-surface": ["native-ready"],
  // og-cli's style check bundles a game with the package's own addon.
  "rust-cpu": ["rust-build", "native-ready"], "rust-gpu": ["rust-build"],
  "bun-quick": ["preflight"], "engine-typecheck": ["rust-build"], "wasm-check": ["preflight"],
  "wasm-build": ["preflight"], "wasm-bindgen": ["wasm-build"], "web-ready": ["native-ready", "wasm-bindgen"],
  "bun-test": ["web-ready", "api-surface", "drift", "bun-quick", "engine-typecheck"],
  fixtures: ["native-ready"], "doc-tests": ["rust-build"], showcase: ["native-ready"], looks: ["native-ready"],
  browser: ["web-ready", "api-surface", "drift", "engine-typecheck"], "web-reference": ["web-ready"],
  safari: ["wasm-bindgen", "rust-build"], "check-conformance": [], "shape-wasm32": ["preflight"],
};
export const QUICK = ["fmt", "clippy", "rust-build", "rust-cpu", "rust-gpu", "drift", "api-surface", "bun-quick", "engine-typecheck", "wasm-check"];
export const FULL = [...QUICK, "wasm-build", "wasm-bindgen", "bun-test", "fixtures", "showcase", "looks", "doc-tests", "browser", "shape-wasm32"];
// These consumers may cook the same projects: serialize only when selected.
export const PROJECT_STEPS = new Set(["bun-test", "fixtures", "showcase", "looks", "browser", "web-reference"]);
// Bun's hrtime starts at process launch. An epoch origin plus a monotonic
// offset lets the scheduler compare markers written by separate Bun processes.
const monotonic = () => (performance.timeOrigin + performance.now()) / 1000;
type Env = Record<string, string | undefined>;
interface Timing {
  started_unix_ns: number; launch_monotonic?: number; wall_s?: number; resource_wait_s?: number | null;
  command_s?: number | null; log_bytes?: number; product_bytes?: number | null; exit_code?: number; cancelled?: boolean;
}
function descendants(root: string): string[] {
  if (!existsSync(root)) return [];
  return [...new Bun.Glob("**/*").scanSync({ cwd: root, dot: true, onlyFiles: false, followSymlinks: false })].map(name => join(root, name));
}
function outputBytes(step: string, env: Env): number | null {
  const wasm = env.OG_WASM_OUT ?? "target/og-wasm";
  let paths: string[];
  if (step === "wasm-build") paths = existsSync(join(wasm, "raw")) ? readdirSync(join(wasm, "raw")).filter(n => n.endsWith(".wasm")).map(n => join(wasm, "raw", n)) : [];
  else if (step === "wasm-bindgen") paths = ["bindgen", "dev", "split"].flatMap(d => descendants(join(wasm, d)));
  else if (["native-ready", "web-ready"].includes(step)) {
    const roots = step === "native-ready" ? ["packages/engine/native"] : existsSync("packages/engine/dist") ? readdirSync("packages/engine/dist").filter(n => n.startsWith("web")).map(n => join("packages/engine/dist", n)) : [];
    paths = roots.flatMap(descendants);
  } else if (step === "rust-build") {
    try {
      const data = JSON.parse(readFileSync(join(env.GATE_LOGS!, "binaries.json"), "utf8"));
      paths = Object.values(data["rust-binaries"]).map((b: any) => b["binary-path"]);
    } catch { return null; }
  } else return null;
  return [...new Set(paths)].reduce((sum, p) => sum + (existsSync(p) && statSync(p).isFile() ? statSync(p).size : 0), 0);
}
function recordTiming(row: Timing, logs: string, step: string, duration: number, rc: number, env: Env, cancelled = false): void {
  function marker(name: string): number | null {
    try { const n = Number(readFileSync(join(logs, `${step}.${name}-start`), "utf8").trim()); return Number.isFinite(n) ? n : null; } catch { return null; }
  }
  const start = marker("resource"), command = marker("command");
  let products: number | null = null;
  try { if (!cancelled && rc === 0) products = outputBytes(step, env); } catch { /* A removed product has no size. */ }
  Object.assign(row, { wall_s: duration, resource_wait_s: start !== null && command !== null ? Math.max(0, command - start) : null,
    command_s: command !== null ? Math.max(0, row.launch_monotonic! + duration - command) : null,
    log_bytes: statSync(join(logs, `${step}.log`)).size, product_bytes: products, exit_code: rc, cancelled });
  delete row.launch_monotonic;
}
function stopGroup(proc: Bun.Subprocess, signal: NodeJS.Signals): void {
  if (process.platform === "win32") { spawnSync("taskkill", ["/T", "/F", "/PID", String(proc.pid)], { stdio: "ignore" }); return; }
  try { process.kill(-proc.pid, signal); } catch (e) { if ((e as NodeJS.ErrnoException).code !== "ESRCH") throw e; }
}
export async function main(): Promise<number> {
  let args = process.argv.slice(2);
  if (args.some(a => ["--help", "-h", "--list"].includes(a))) {
    console.log("scripts/check.sh [quick|full] [--all | --resume RUN_ID]\nscripts/check.sh --only STEP...");
    console.log("Steps: " + Object.keys(NEEDS).join(" "));
    return 0;
  }
  let resume: string | undefined;
  if (args.includes("--resume")) {
    const i = args.indexOf("--resume");
    if (!/^[0-9]+-[0-9]+$/.test(args[i + 1] ?? "")) throw new Error("--resume requires a gate run directory name (timestamp-pid)");
    if (args.includes("--all") || process.env.CI) throw new Error("--resume is incompatible with a fresh forced run (--all or CI)");
    resume = args[i + 1]; args.splice(i, 2);
  }
  const force = Boolean(resume || args.includes("--all") || process.env.CI), only = args.includes("--only");
  args = args.filter(a => !["--all", "--only"].includes(a));
  const lane = ["quick", "full"].includes(args[0]) ? args.shift()! : "full";
  const wanted = new Set(args.length ? args : lane === "quick" ? QUICK : FULL);
  let unknown = [...wanted].filter(s => !Object.hasOwn(NEEDS, s));
  if (unknown.length) throw new Error("Unknown steps: " + unknown.sort().join(", "));
  if (wanted.has("bun-test")) { wanted.add("rust-cpu"); wanted.add("rust-gpu"); }
  let size = -1;
  while (size !== wanted.size) { size = wanted.size; for (const s of wanted) for (const d of NEEDS[s]) wanted.add(d); }
  for (const step of ["rust-cpu", "rust-gpu", "fixtures", "showcase", "looks"]) NEEDS[step].push(...["drift", "api-surface", "engine-typecheck"].filter(s => wanted.has(s)));
  const skip = (process.env.OG_GATE_SKIP ?? "").split(",").map(s => s.trim()).filter(Boolean);
  unknown = [...new Set(skip)].filter(s => !Object.hasOwn(NEEDS, s));
  if (unknown.length) throw new Error("OG_GATE_SKIP names unknown steps: " + unknown.sort().join(", "));
  const unlandable = skip.filter(s => s !== "rust-gpu").sort();
  if (process.env.OG_GATE_LANDING && unlandable.length) throw new Error(`landing: OG_GATE_SKIP dropped steps a landing runs (${unlandable.join(", ")}); only rust-gpu may be skipped for a landing`);
  const skipped = Object.keys(NEEDS).filter(s => wanted.has(s) && skip.includes(s));
  if (["browser", "wasm-build", "wasm-check"].every(s => wanted.has(s))) NEEDS["wasm-check"].push("wasm-build");
  if (resume && skip.length) throw new Error("--resume is incompatible with OG_GATE_SKIP");
  const jobs = Number(process.env.OG_GATE_JOBS ?? "2"), threads = Number(process.env.OG_GATE_THREADS ?? "2");
  if (!Number.isInteger(jobs) || !Number.isInteger(threads) || jobs < 1 || threads < 1) throw new Error("OG_GATE_JOBS and OG_GATE_THREADS must be positive");
  const runs = join(process.env.OG_EVIDENCE_DIR!, "gate/runs");
  if (resume && !existsSync(join(runs, resume, "run.json"))) throw new Error(`--resume: no run ${resume} in ${runs}`);
  const logs = join(runs, `${BigInt(Date.now()) * 1000000n}-${process.pid}`);
  mkdirSync(logs, { recursive: true });
  const env: Env = { ...process.env, GATE_LOGS: logs, OG_GATE_LANE: lane, FORCE_ALL: String(Number(force)), OG_GATE_ALL: lane === "full" || force ? "1" : "",
    OG_WASM_PROFILE: "wasm-test", OG_BIN: join(process.env.OG_TARGET_DIR!, "debug/og"),
    OG_WASM_VARIANTS: wanted.has("browser") || wanted.has("safari") ? "all" : wanted.has("web-reference") ? "reference" : "dev",
    OG_GATE_STEPS: [...wanted].filter(s => !skipped.includes(s)).sort().join(" "), OG_GATE_RESUME: resume ?? "" };
  delete env.OG_WASM_STALE;
  const cargo = openSync(join(logs, "cargo.json"), "w");
  try { execFileSync("cargo", ["metadata", "--locked", "--format-version=1"], { stdio: ["inherit", cargo, "inherit"] }); } finally { closeSync(cargo); }
  const laneRecord = join(process.env.OG_TARGET_DIR ?? "target", "og-gate/green/lane.full");
  const whole = lane === "full" && force && !only && !skipped.length;
  if (whole) rmSync(laneRecord, { force: true });
  const active = new Map<string, { proc: Bun.Subprocess; log: number; begin: number; rc: number | null }>();
  const timings: Record<string, Timing> = {}, completed = new Map(skipped.map(s => [s, `SKIPPED ${s} (OG_GATE_SKIP)`]));
  const started = monotonic();
  console.log(`gate ${lane}: ${jobs} steps × ${threads} jobs; logs ${logs}`);
  for (const s of skipped) console.log(completed.get(s));
  writeFileSync(join(logs, "run.json"), JSON.stringify({ head: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(), tree: env.OG_GATE_TREE ?? null, steps: [...wanted].sort(), skipped, force }));
  let interrupted = false;
  const interrupt = () => { interrupted = true; };
  process.on("SIGTERM", interrupt); process.on("SIGINT", interrupt);
  try {
    while (completed.size < wanted.size) {
      if (interrupted) return 130;
      for (const step of Object.keys(NEEDS)) {
        if (!wanted.has(step) || completed.has(step) || active.has(step) || active.size >= jobs) continue;
        if (!NEEDS[step].every(d => completed.has(d))) continue;
        if (PROJECT_STEPS.has(step) && [...active.keys()].some(s => PROJECT_STEPS.has(s))) continue;
        const log = openSync(join(logs, `${step}.log`), "w"), begin = monotonic();
        const proc = Bun.spawn(["bash", "scripts/gate/step.sh", step], { env: { ...env, OG_GATE_RAN: " " + [...completed].filter(([, v]) => v.startsWith("PASS ")).map(([s]) => s).join(" ") + " " }, stdout: log, stderr: log, detached: true });
        const row = { proc, log, begin, rc: null as number | null };
        void proc.exited.then(rc => { row.rc = rc; });
        active.set(step, row);
        timings[step] = { started_unix_ns: Date.now() * 1e6, launch_monotonic: begin };
        console.log(`--> ${step}`);
      }
      for (const [step, { log, begin, rc }] of active) {
        if (rc === null) continue;
        closeSync(log); active.delete(step);
        const duration = monotonic() - begin;
        recordTiming(timings[step], logs, step, duration, rc, env);
        process.stdout.write(readFileSync(join(logs, `${step}.log`)));
        const reused = resume && rc === 4 && step === "bun-test" && existsSync(join(logs, "bun-test-suite.receipt"));
        const status = reused ? "REUSED" : rc === 0 ? "PASS" : rc === 3 ? "SKIPPED" : "FAIL";
        completed.set(step, `${status} ${step} (${duration.toFixed(1)}s)`);
        console.log(completed.get(step));
        if (status === "FAIL") return 1;
      }
      if (!active.size && completed.size < wanted.size && ![...wanted].some(s => !completed.has(s) && NEEDS[s].every(d => completed.has(d)))) throw new Error("No runnable step in gate DAG");
      await Bun.sleep(100);
    }
    if (whole && !resume && env.OG_GATE_TREE && [...wanted].every(s => completed.get(s)!.startsWith("PASS "))) {
      mkdirSync(join(laneRecord, ".."), { recursive: true }); writeFileSync(laneRecord, env.OG_GATE_TREE + "\n");
    }
    let note = skipped.length ? `; NOT RUN (OG_GATE_SKIP): ${skipped.join(", ")}` : "";
    if (resume) note += `; continuation of ${resume} (see per-suite reuse records)`;
    console.log(`gate PASSED (${completed.size - skipped.length}/${wanted.size} steps, ${(monotonic() - started).toFixed(1)}s${note})`);
    return 0;
  } finally {
    for (const { proc } of active.values()) stopGroup(proc, "SIGTERM");
    for (const [step, { proc, log, begin }] of active) {
      let timer: ReturnType<typeof setTimeout> | undefined;
      try { await Promise.race([proc.exited, new Promise<void>(resolve => { timer = setTimeout(() => { stopGroup(proc, "SIGKILL"); resolve(); }, 10000); })]); }
      finally { clearTimeout(timer); }
      const rc = await proc.exited;
      closeSync(log); recordTiming(timings[step], logs, step, monotonic() - begin, rc, env, true);
    }
    process.off("SIGTERM", interrupt); process.off("SIGINT", interrupt);
    const summary = Object.keys(NEEDS).filter(s => completed.has(s)).map(s => completed.get(s)).join("\n") + "\n";
    writeFileSync(join(logs, "timings.json"), JSON.stringify(timings, null, 2) + "\n");
    writeFileSync(join(logs, "summary.txt"), summary);
    writeFileSync(join(process.env.OG_EVIDENCE_DIR!, "check-summary.txt"), summary);
  }
}
if (import.meta.main) process.exitCode = await main();
