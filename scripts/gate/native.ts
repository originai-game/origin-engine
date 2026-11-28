/** Discover source files and GPU requirements from the actual libtest binaries. */
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, openSync, closeSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { delimiter, join, resolve } from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { LOGS, GREEN, ROOT, crateInputs, mark, metadata, unaffected } from "./inputs.ts";

interface Binary { "binary-id": string; "binary-path": string; "package-id": string }
interface Test { name: string; source?: string | null; gpu: boolean; ignored: boolean }
interface Listing { "rust-binaries": Record<string, Binary>; "rust-build-meta": any }
const hash = (id: string) => createHash("sha256").update(id).digest("hex").slice(0, 20);
const buildKey = (id: string) => "rust.build." + hash(id);
const packageName = (entry: Binary) => entry["package-id"].split("/").at(-1)!.split("#")[0];
function excluded(): [string[], string[]] {
  const crates: string[] = [], bins: string[] = [], path = join(ROOT, "scripts/gate-exclude.txt");
  if (existsSync(path)) for (let line of readFileSync(path, "utf8").split(/\r?\n/)) {
    line = line.split("#", 1)[0].trim();
    if (!line) continue;
    if (line.startsWith("bin ")) bins.push(line.split(/\s+/)[1]); else crates.push(line);
  }
  return [crates, bins];
}
function build(): void {
  const [exCrates, exBins] = excluded();
  if (process.env.OG_GATE_LANE === "quick") exCrates.push("og-tests");
  const meta = metadata();
  const allowed = new Set(meta.packages.filter(p => meta.workspace_members.includes(p.id) && p.targets.some(t => t.kind.includes("test")) && !exCrates.includes(p.name)).map(p => p.name));
  let stored: Record<string, Binary>;
  try { stored = JSON.parse(readFileSync(join(GREEN, "rust.build.list.binaries.json"), "utf8"))["rust-binaries"]; } catch { stored = {}; }
  const listed = new Set(Object.values(stored).map(packageName)), reuse: Record<string, Binary> = {}, affected = new Set<string>();
  for (const [id, entry] of Object.entries(stored)) {
    const pkg = packageName(entry);
    if (!allowed.has(pkg) || exBins.includes(entry["binary-id"])) continue;
    let reusable = false;
    try { reusable = statSync(join(GREEN, `${buildKey(id)}.tests.json`)).isFile() && statSync(entry["binary-path"]).isFile() && unaffected(buildKey(id), crateInputs([pkg])); } catch { /* Missing inputs rebuild. */ }
    if (reusable) reuse[id] = entry; else affected.add(pkg);
  }
  for (const pkg of allowed) if (!listed.has(pkg)) affected.add(pkg);
  let listing: Listing;
  if (!affected.size && Object.keys(reuse).length) listing = JSON.parse(readFileSync(join(GREEN, "rust.build.list.binaries.json"), "utf8"));
  else {
    const args = ["nextest", "list", "--locked", "--list-type", "binaries-only", "--message-format", "json"];
    args.push(...(affected.size ? [...affected].sort().flatMap(p => ["-p", p]) : ["--workspace", ...exCrates.sort().map(c => `--exclude=${c}`)]));
    const out = openSync(join(LOGS, "binaries.json"), "w");
    try { execFileSync("cargo", args, { stdio: ["inherit", out, "inherit"] }); } finally { closeSync(out); }
    listing = JSON.parse(readFileSync(join(LOGS, "binaries.json"), "utf8"));
    for (const [id, entry] of Object.entries(reuse)) listing["rust-binaries"][id] ??= entry;
  }
  writeFileSync(join(LOGS, "binaries.json"), JSON.stringify(listing));
  const binaries = Object.fromEntries(Object.entries(listing["rust-binaries"]).filter(([, e]) => allowed.has(packageName(e)) && !exBins.includes(e["binary-id"])));
  listing["rust-binaries"] = binaries;
  const buildMeta = listing["rust-build-meta"], env: NodeJS.ProcessEnv = { ...process.env, OG_RENDERER: "none" };
  delete env.OG_GPU_REPORT_DIR; delete env.OG_REFERENCE_REPORT_DIR;
  const loader = process.platform === "darwin" ? "DYLD_FALLBACK_LIBRARY_PATH" : process.platform === "win32" ? "PATH" : "LD_LIBRARY_PATH";
  const libraries = buildMeta["linked-paths"].map((p: string) => resolve(buildMeta["target-directory"], p));
  libraries.push(...buildMeta["base-output-directories"].map((p: string) => resolve(buildMeta["target-directory"], p, "deps")), buildMeta.platforms.host.libdir.path);
  if (process.platform === "win32") libraries.push(resolve(buildMeta.platforms.host.libdir.path, "../../../..", "bin"));
  env[loader] = [...libraries, env[loader] ?? ""].join(delimiter);
  // Build once: later steps run these products, never cargo for them.
  execFileSync("cargo", ["build", "--locked", "-q", "-p", "og-host-node", "-p", "og-cli"], { stdio: "inherit" });
  const discovered: Record<string, { binary: Binary; tests: Test[] }> = {};
  for (const id of Object.keys(binaries).filter(b => !(b in reuse))) {
    const binary = binaries[id];
    const output = execFileSync(binary["binary-path"], ["--list", "--format=json", "-Zunstable-options"], { env, encoding: "utf8" });
    const tests: Test[] = [];
    for (const line of output.trimEnd().split(/\r?\n/).filter(Boolean)) {
      const record = JSON.parse(line);
      if (record.type !== "test" || record.event !== "discovered") continue;
      const gpu = (record.ignore_message ?? "").includes("OG_RENDERER=none");
      tests.push({ name: record.name, source: record.source_path ?? null, gpu, ignored: Boolean(record.ignore) && !gpu });
    }
    discovered[id] = { binary, tests };
    mkdirSync(GREEN, { recursive: true });
    writeFileSync(join(GREEN, `${buildKey(id)}.tests.json`), JSON.stringify(tests)); mark(buildKey(id));
  }
  for (const id of Object.keys(reuse)) discovered[id] = { binary: binaries[id], tests: JSON.parse(readFileSync(join(GREEN, `${buildKey(id)}.tests.json`), "utf8")) };
  writeFileSync(join(LOGS, "discovered.json"), JSON.stringify(discovered, null, 2));
  writeFileSync(join(GREEN, "rust.build.list.binaries.json"), JSON.stringify(listing));
}
export function selection(id: string, tests: Test[], gpu: boolean): string {
  // Match Python re.escape, then escape the filterset's slash delimiter.
  const escape = (s: string) => s.replace(/[ \t\n\r\v\f#&~.\-+*?^${}()|\[\]\\/]/g, "\\$&");
  const names = (kind: boolean) => tests.filter(t => t.gpu === kind).map(t => escape(t.name)).join("|");
  const mine = names(gpu), other = names(!gpu);
  if (!other) return `binary_id(=${id})`;
  if ([...other].length < [...mine].length) return `(binary_id(=${id}) & not test(/^(?:${other})$/))`;
  return `(binary_id(=${id}) & test(/^(?:${mine})$/))`;
}
export function passed(junit: string, expected: Record<string, number>): Set<string> {
  // Read JUnit without a permissive HTML parser: a truncated/malformed report
  // must never certify a binary. Count direct testcase children, as nextest emits.
  try {
    const xml = readFileSync(junit, "utf8"), stack: { tag: string; attrs: Record<string, string>; cases: number }[] = [];
    const green = new Set<string>();
    let at = 0, roots = 0;
    const decode = (s: string) => s.replace(/&(#x[\da-fA-F]+|#\d+|amp|lt|gt|quot|apos);/g, (_, e: string) => e.startsWith("#") ? String.fromCodePoint(parseInt(e.slice(e[1] === "x" ? 2 : 1), e[1] === "x" ? 16 : 10)) : ({ amp: "&", lt: "<", gt: ">", quot: '"', apos: "'" }[e]!));
    const finish = () => {
      const node = stack.pop()!;
      if (node.tag === "testsuite" && node.attrs.failures === "0" && node.attrs.errors === "0" && node.cases === expected[node.attrs.name]) green.add(node.attrs.name);
    };
    const tokens = /<!--[\s\S]*?-->|<\?[^]*?\?>|<!\[CDATA\[[\s\S]*?\]\]>|<\/(\w[\w:.-]*)\s*>|<([\w:.-]+)((?:\s+[\w:.-]+\s*=\s*(?:"[^"<]*"|'[^'<]*'))*)\s*(\/?)>/g;
    for (const m of xml.matchAll(tokens)) {
      const text = xml.slice(at, m.index);
      if (text.includes("<") || (!stack.length && text.trim()) || /&(?!(?:#x[\da-fA-F]+|#\d+|amp|lt|gt|quot|apos);)/.test(text)) return new Set();
      at = m.index! + m[0].length;
      if (m[0].startsWith("<![CDATA[")) { if (!stack.length) return new Set(); continue; }
      if (m[0].startsWith("<!") || m[0].startsWith("<?")) continue;
      if (m[1]) { if (stack.at(-1)?.tag !== m[1]) return new Set(); finish(); continue; }
      if (!stack.length && ++roots !== 1) return new Set();
      if (m[2] === "testcase" && stack.at(-1)?.tag === "testsuite") stack.at(-1)!.cases++;
      const attrs: Record<string, string> = {};
      for (const a of m[3].matchAll(/([\w:.-]+)\s*=\s*(?:"([^"]*)"|'([^']*)')/g)) {
        const value = a[2] ?? a[3];
        if (a[1] in attrs || /&(?!(?:#x[\da-fA-F]+|#\d+|amp|lt|gt|quot|apos);)/.test(value)) return new Set();
        attrs[a[1]] = decode(value);
      }
      stack.push({ tag: m[2], attrs, cases: 0 });
      if (m[4]) finish();
    }
    return stack.length || roots !== 1 || xml.slice(at).trim() ? new Set() : green;
  } catch { return new Set(); }
}
function run(kind: string): number {
  const discovered: Record<string, { binary: Binary; tests: Test[] }> = JSON.parse(readFileSync(join(LOGS, "discovered.json"), "utf8"));
  const packages = new Map(metadata().packages.map(p => [p.id, p.name]));
  const groups: string[] = [], keys: Record<string, string> = {}, expected: Record<string, number> = {};
  for (const [id, entry] of Object.entries(discovered)) {
    const tests = entry.tests.filter(t => t.gpu === (kind === "gpu"));
    if (!tests.length) continue;
    const key = `rust.${kind}.${hash(id)}`;
    if (unaffected(key, crateInputs([packages.get(entry.binary["package-id"])!]))) { console.log(`SKIPPED ${id} ${kind}: dependency inputs unchanged`); continue; }
    rmSync(join(GREEN, key), { force: true }); keys[id] = key;
    expected[id] = tests.filter(t => !t.ignored).length;
    groups.push(selection(id, entry.tests, kind === "gpu"));
  }
  if (!groups.length) return 3;
  const reports = join(LOGS, `${kind}-gpu-reports`), moved = join(LOGS, `${kind}-reference-mismatches`);
  mkdirSync(reports); mkdirSync(moved);
  const env: NodeJS.ProcessEnv = { ...process.env, OG_GPU_REPORT_DIR: reports, OG_REFERENCE_REPORT_DIR: moved };
  if (kind === "cpu") env.OG_RENDERER = "none"; else env.OG_RENDERER ??= "hardware";
  let threads = process.env.OG_GATE_THREADS!, gpuConfig: string | undefined;
  if (kind === "gpu") {
    threads = process.env.OG_GPU_CAPACITY!; gpuConfig = join(LOGS, "gpu-nextest.toml");
    writeFileSync(gpuConfig, readFileSync(join(ROOT, ".config/nextest.toml"), "utf8") + `\n[[profile.gpu.overrides]]\nfilter = 'all()'\nthreads-required = ${threads}\n`);
    console.log(`GPU memory pool: ${threads} GiB credits; undeclared tests reserve all`);
  }
  const args = ["nextest", "run", "--cargo-metadata", join(LOGS, "cargo.json"), "--binaries-metadata", join(LOGS, "binaries.json"), "--profile", kind, "--test-threads", threads, "-E", groups.join(" | ")];
  if (gpuConfig) args.push("--config-file", gpuConfig);
  if (kind === "gpu") args.push("--no-tests=warn");
  const junit = join(process.env.OG_TARGET_DIR!, "nextest", kind, "junit.xml");
  rmSync(junit, { force: true });
  const result = spawnSync("cargo", args, { env, stdio: "inherit" });
  if (result.error) throw result.error;
  const rc = result.status ?? 1;
  if (existsSync(junit)) copyFileSync(junit, join(LOGS, `rust-${kind}.junit.xml`));
  let count = 0;
  for (const name of readdirSync(moved).filter(n => n.endsWith(".txt")).sort()) {
    const text = readFileSync(join(moved, name), "utf8"); count += text.split("REFERENCE MISMATCH ").length - 1; process.stdout.write(text);
  }
  if (count) console.log(`${count} reference frames moved: reported, not failed; review each actual and diff PNG named above`);
  const unchecked = readdirSync(reports).filter(n => n.endsWith(".txt"));
  if (kind === "gpu" && unchecked.length) {
    for (const name of unchecked) process.stdout.write(readFileSync(join(reports, name)));
    console.log("GPU checks are unchecked; no passing snapshot recorded"); return rc || 3;
  }
  const green = rc ? passed(junit, expected) : new Set(Object.keys(keys));
  for (const [id, key] of Object.entries(keys)) if (green.has(id)) mark(key);
  return rc;
}
if (import.meta.main) {
  if (process.argv[2] === "build") build(); else process.exitCode = run(process.argv[2]);
}
