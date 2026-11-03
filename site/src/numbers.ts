// The numbers wall. Every figure is counted from this checkout at build time
// or read from a measurement in the showcase archive (showcase/manifest.jsonl),
// with its source shown beside it. A figure that has not been measured yet is
// a `pending` slot: drawn as a marked placeholder, refused by --final.
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const root = join(process.cwd(), "..");
const git = (...args: string[]) => execFileSync("git", ["-C", root, ...args], { encoding: "utf8" }).trim();

interface Entry {
  id: string;
  kind: string;
  title: string;
  date: string;
  commit: string;
  machine?: string | null;
  excluded?: boolean;
  metrics?: { name: string; value: number | string; unit: string }[];
}
const manifest: Entry[] = readFileSync(join(root, "showcase/manifest.jsonl"), "utf8")
  .split("\n")
  .filter(Boolean)
  .map((l) => JSON.parse(l))
  .filter((e: Entry) => !e.excluded);

/** The newest benchmark entry carrying this metric (by date, then by manifest order). */
function measured(metric: string, where?: (e: Entry) => boolean) {
  const hits = manifest.filter((e) => e.kind === "benchmark" && e.metrics?.some((m) => m.name === metric) && (!where || where(e)));
  hits.sort((a, b) => a.date.localeCompare(b.date));
  const e = hits.at(-1);
  if (!e) throw new Error(`numbers: no measurement of ${metric}`);
  return { value: Number(e.metrics!.find((m) => m.name === metric)!.value), entry: e };
}

function walk(dir: string, test: (path: string) => boolean, acc: string[] = []): string[] {
  for (const d of readdirSync(dir, { withFileTypes: true })) {
    if (d.name === "node_modules" || d.name === "target" || d.name.startsWith(".")) continue;
    const p = join(dir, d.name);
    if (d.isDirectory()) walk(p, test, acc);
    else if (test(p)) acc.push(p);
  }
  return acc;
}

const head = git("rev-parse", "--short=9", "HEAD");
const headDate = git("log", "-1", "--format=%cs");
const first = git("log", "--reverse", "--format=%cs", "--max-parents=0").split("\n")[0];
const commits = Number(git("rev-list", "--count", "HEAD"));
const days = Math.round((Date.parse(headDate) - Date.parse(first)) / 86_400_000);
const crates = readdirSync(join(root, "crates"), { withFileTypes: true }).filter((d) => d.isDirectory()).length;
const apiFns = readdirSync(join(root, "crates/og-api/api"))
  .filter((f) => f.endsWith(".ogt"))
  .reduce((n, f) => n + (readFileSync(join(root, "crates/og-api/api", f), "utf8").match(/^\s*fn \w/gm)?.length ?? 0), 0);
const references = walk(root, (p) => /[\\/]references[\\/]metal[\\/][^\\/]+\.png$/.test(p)).length;
const skills = readdirSync(join(root, "packages/knowledge/skills"), { withFileTypes: true }).filter((d) => d.isDirectory()).length;
const cliCommands = new Set(
  [...readFileSync(join(root, "crates/og-cli/src/main.rs"), "utf8").matchAll(/\[\s*"([a-z][a-z-]*)",\s*rest @ \.\.\]/g)].map((m) => m[1]),
);
cliCommands.delete("help");
cliCommands.delete("version");

const tests = measured("tests");
const size = measured("publish3d");
const webFrame = manifest.find((e) => e.title === "Perf baseline: showcase (webgpu-chrome-apple-m4-pro)")!;
const rollback = manifest.find((e) => e.title.startsWith("Rollback vs lockstep"))!;

export type Num = {
  key: string;
  value: string;
  unit?: { en: string; zh: string };
  label: { en: string; zh: string };
  source: { en: string; zh: string };
  pending?: boolean;
};

const fmt = (n: number) => n.toLocaleString("en-US");
const counted = { en: `Counted at ${head}, ${headDate}`, zh: `统计于 ${head}，${headDate}` };
const from = (e: Entry) => ({
  en: `${e.title} · ${e.machine ?? "macrunner"} · ${e.commit.slice(0, 9)}, ${e.date}`,
  zh: `${e.title} · ${e.machine ?? "macrunner"} · ${e.commit.slice(0, 9)}，${e.date}`,
});

export const numbers: Num[] = [
  {
    key: "days",
    value: fmt(days),
    unit: { en: "days", zh: "天" },
    label: { en: `from the first commit to this build, ${fmt(commits)} commits, written by AI agents`, zh: `从第一次提交到本页构建，共 ${fmt(commits)} 次提交，由 AI agent 编写` },
    source: { en: `git log: ${first} → ${headDate}`, zh: `git log：${first} → ${headDate}` },
  },
  { key: "tests", value: fmt(tests.value), label: { en: "tests in the gate", zh: "门禁中的测试" }, source: from(tests.entry) },
  { key: "refs", value: fmt(references), label: { en: "reference frames compared pixel by pixel", zh: "逐像素比对的参考帧" }, source: counted },
  { key: "api", value: fmt(apiFns), label: { en: "API functions, one IDL for wasm and native", zh: "个 API 函数，一份 IDL 生成 wasm 与原生两端" }, source: counted },
  {
    key: "wasm",
    value: (size.value / 2 ** 20).toFixed(1),
    unit: { en: "MiB", zh: "MiB" },
    label: { en: "the whole 3D engine, brotli, as a player downloads it", zh: "完整 3D 引擎的下载体积（brotli）" },
    source: from(size.entry),
  },
  {
    key: "frame",
    value: String(webFrame.metrics!.find((m) => m.name === "frame_ms")!.value),
    unit: { en: "ms", zh: "ms" },
    label: { en: "per frame for the lit showcase scene in Chrome on WebGPU, 1280×720, Apple M4 Pro", zh: "光照展示场景在 Chrome WebGPU 下每帧耗时，1280×720，Apple M4 Pro" },
    source: from(webFrame),
  },
  {
    key: "rollback",
    value: fmt(Number(rollback.metrics!.find((m) => m.name === "rollbacks-peer0")!.value)),
    label: { en: "rollbacks at 80 ms latency and 5 % loss, ending on the lockstep hash", zh: "次回滚（80 ms 延迟、5% 丢包），最终哈希与锁步一致" },
    source: from(rollback),
  },
  { key: "crates", value: fmt(crates), label: { en: "Rust crates behind one TypeScript API", zh: "个 Rust crate，对外只有一套 TypeScript API" }, source: counted },
  { key: "cli", value: fmt(cliCommands.size), label: { en: "og commands: build, run, probe, test, cook, polish…", zh: "个 og 命令：build、run、probe、test、cook、polish…" }, source: counted },
  { key: "skills", value: fmt(skills), label: { en: "agent skills shipped with the engine", zh: "个随引擎发布的 agent 技能" }, source: counted },
  {
    key: "hero-frame",
    value: "—",
    label: { en: "hero scene frame time on a mid-range laptop", zh: "首屏场景在中端笔记本上的帧时间" },
    source: { en: "Measured at the wave's end", zh: "波次结束时实测" },
    pending: true,
  },
  {
    key: "first-frame",
    value: "—",
    label: { en: "from navigation to the first frame", zh: "从打开页面到第一帧" },
    source: { en: "Measured at the wave's end", zh: "波次结束时实测" },
    pending: true,
  },
];
