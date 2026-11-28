/** Same-input receipts for terminal test work; never a substitute for products. */
import { createHash } from "node:crypto";
import { closeSync, openSync, readSync, readFileSync, readdirSync, realpathSync, renameSync, lstatSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { execFileSync } from "node:child_process";

export function fileHash(path: string): string {
  const fd = openSync(path, "r"), digest = createHash("sha256"), chunk = Buffer.alloc(1024 * 1024);
  try {
    let n: number;
    while ((n = readSync(fd, chunk, 0, chunk.length, null))) digest.update(chunk.subarray(0, n));
  } finally { closeSync(fd); }
  return digest.digest("hex");
}
/** Preserve existing receipt keys: sorted keys, Python JSON spacing and ASCII escaping. */
export function receiptJson(value: unknown): string {
  if (Array.isArray(value)) return "[" + value.map(receiptJson).join(", ") + "]";
  if (value && typeof value === "object") return "{" + Object.keys(value).sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b))).map(k => receiptJson(k) + ": " + receiptJson((value as Record<string, unknown>)[k])).join(", ") + "}";
  return JSON.stringify(value).replace(/[\u007f-\uffff]/g, c => "\\u" + c.charCodeAt(0).toString(16).padStart(4, "0"));
}
function command(...args: string[]): string {
  return execFileSync(args[0], args.slice(1), { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], timeout: 10000 }).trim();
}
export function fingerprint(scope: string): string | null {
  const env = process.env;
  if (process.platform !== "linux" || env.OG_RENDERER === "none" || ["OG_GFX_CAPTURE", "OG_BIN", "OG_ENGINE_PACKAGE"].some(k => env[k])) return null;
  const run = JSON.parse(readFileSync(join(env.GATE_LOGS!, "run.json"), "utf8"));
  if (!run.force || run.skipped.length || !run.tree) return null;
  if (command("git", "rev-parse", "HEAD^{tree}") !== run.tree || command("git", "status", "--porcelain")) return null;
  const gpu = command("nvidia-smi", "--query-gpu=name,uuid,driver_version", "--format=csv,noheader").split(/\r?\n/);
  if (gpu.length !== 1 || !gpu[0].startsWith("NVIDIA L4,")) return null;
  const runtime: Record<string, string[]> = {};
  for (const [tool, version] of [["bun", "--revision"], ["rustc", "-vV"], ["cargo", "--version"], ["tsrs", "--version"], ["wasm-bindgen", "--version"]]) {
    const path = Bun.which(tool);
    if (!path) return null;
    runtime[tool] = [realpathSync(path), fileHash(path), command(tool, version)];
  }
  if (scope === "browser") {
    const chrome = realpathSync(command("bun", "-e", 'import {chromePath} from "./packages/engine/tests/web/cdp.ts"; console.log(chromePath())'));
    runtime.chrome = [chrome, fileHash(chrome), command(chrome, "--version")];
    for (const tool of ["wasm-split", "rustfilt"]) {
      const path = Bun.which(tool);
      if (!path) return null;
      runtime[tool] = [fileHash(path), command(path, "--version")];
    }
  }
  const products: Record<string, string> = {};
  const roots = ["packages/engine/native", ...readdirSync("packages/engine/dist").filter(n => n.startsWith("web")).sort().map(n => join("packages/engine/dist", n))];
  if (roots.length < 2) return null;
  for (const root of roots) {
    if (realpathSync(root) !== resolve(root) || !statSync(root).isDirectory()) return null;
    for (const name of new Bun.Glob("**/*").scanSync({ cwd: root, dot: true, onlyFiles: false, followSymlinks: false })) {
      const path = join(root, name), entry = lstatSync(path);
      if (entry.isSymbolicLink()) return null;
      if (entry.isFile()) products[path] = fileHash(path);
    }
  }
  if (!Object.keys(products).some(n => n.endsWith(".node")) || !Object.keys(products).some(n => n.endsWith(".wasm"))) return null;
  const volatile = new Set(["GATE_LOGS", "OG_GATE_RESUME", "OG_GATE_RAN", "OG_GPU_SLOT", "OG_LOCK_IDENTITIES", "_", "SHLVL"]);
  const scratch = join(process.cwd(), ".og/tmp");
  for (const name of ["TMPDIR", "TMP", "TEMP"]) {
    const value = resolve(env[name] ?? "");
    if (dirname(value) === scratch && basename(value).startsWith("run.")) volatile.add(name);
  }
  const inputs = [1, scope, run, process.cwd(), Object.fromEntries(Object.entries(env).filter(([k]) => !volatile.has(k))), runtime, gpu, products, readFileSync("/proc/sys/kernel/random/boot_id", "utf8").trim()];
  return createHash("sha256").update(receiptJson(inputs)).digest("hex");
}
function main(): number {
  const [action, scope, name, key] = process.argv.slice(2);
  if (!["browser", "bun-test"].includes(scope)) return 1;
  const logs = process.env.GATE_LOGS!;
  if (action === "key") {
    const key = fingerprint(scope);
    if (key) console.log(key);
    return 0;
  }
  if (!/^[a-z0-9-]+$/.test(name) || !/^[a-f0-9]{64}$/.test(key)) return 1;
  const relative = `${scope}-${name}.receipt`;
  if (action === "reuse") {
    const source = process.env.OG_GATE_RESUME;
    if (!source || !/^[0-9]+-[0-9]+$/.test(source) || readFileSync(join(dirname(logs), source, relative), "utf8").trim() !== key) return 1;
    console.log(`REUSED ${scope}:${name} from ${source} (exact input receipt)`);
  } else if (action !== "pass") return 1;
  const receipt = join(logs, relative), pending = receipt.replace(/\.receipt$/, ".part");
  writeFileSync(pending, key + "\n");
  renameSync(pending, receipt);
  return 0;
}
if (import.meta.main) {
  try { process.exitCode = main(); } catch { process.exitCode = 1; }
}
