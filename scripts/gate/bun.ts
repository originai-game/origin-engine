import { mkdirSync, readdirSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";
import { ImportGraph } from "./imports";

const root = process.cwd();
const target = process.env.OG_TARGET_DIR;
const tree = process.env.OG_GATE_TREE;
const logs = process.env.GATE_LOGS;
if (!target || !logs) throw new Error("Run Bun checks through scripts/check.sh");
const full = process.argv[2] === "full";
const graph = new ImportGraph(root);
const green = join(target, "og-gate", "green");
mkdirSync(green, { recursive: true });
let unchecked = false;

async function run(command: string[], cwd = root): Promise<void> {
  const child = Bun.spawn(command, { cwd, stdout: "inherit", stderr: "pipe" });
  const stderr = new Response(child.stderr).text();
  const rc = await child.exited;
  const diagnostic = await stderr;
  process.stderr.write(diagnostic);
  if (/WARNING .*missing GPU:/.test(diagnostic)) unchecked = true;
  if (rc !== 0) throw new Error(`${command.join(" ")} exited ${rc}`);
}

for (const entry of readdirSync("packages", { withFileTypes: true })) {
  if (!entry.isDirectory()) continue;
  const pkg = `packages/${entry.name}`;
  if ((entry.name === "engine") !== full) continue;
  const manifestFile = Bun.file(join(pkg, "package.json"));
  // A package without a manifest or a test script still has its test files
  // run (packages/launcher has neither): a test file is never skipped for
  // what its package does not declare.
  const manifest = await manifestFile.exists() ? await manifestFile.json() : {};
  const script: unknown = manifest.scripts?.test;
  const testCommand = typeof script === "string" ? script.lastIndexOf("bun test") : -1;
  if (typeof script === "string" && testCommand < 0) {
    await run(["bun", "run", "test"], resolve(pkg));
    continue;
  }
  // Preserve validation/prepack commands from the package's own contract.
  // engine preparation and typecheck are done once by step_bun_test.
  const prepare = typeof script === "string" ? script.slice(0, testCommand).replace(/\s*&&\s*$/, "") : "";
  if (!full && prepare.trim()) await run(["bash", "-eu", "-c", prepare], resolve(pkg));
  const files = [...new Bun.Glob("{tests,extensions}/**/*.{test,spec}.{ts,tsx,js}").scanSync(pkg)].sort();
  const dependencies = new Map<string, Set<string>>();
  const classified = new Set<string>();
  for (const file of files) {
    try {
      const inputs = await graph.inputs(`${pkg}/${file}`);
      dependencies.set(file, inputs);
      for (const input of inputs) classified.add(input);
    } catch {
      // Unresolved or computed imports cannot justify a skipped test.
    }
  }
  const selected: string[] = [];
  for (const file of files) {
    const key = `bun.${new Bun.CryptoHasher("sha256").update(`${pkg}/${file}`).digest("hex")}`;
    const stamp = Bun.file(join(green, key));
    let affected = true;
    if (process.env.FORCE_ALL !== "1" && tree && await stamp.exists()) {
      const diff = Bun.spawnSync(["git", "diff", "--no-renames", "--name-only", "-z", (await stamp.text()).trim(), tree]);
      if (diff.exitCode === 0) {
        const changed = diff.stdout.toString().split("\0").filter(Boolean);
        try {
          const inputs = dependencies.get(file);
          if (!inputs) throw new Error("unclassified imports");
          // Imports do not describe filesystem reads, subprocesses, generated
          // bundles or native modules. Keep these callers conservative.
          let runtime = false;
          for (const input of inputs) {
            if (!/\.[cm]?[jt]sx?$/.test(input)) continue;
            if (/\b(readFile|readFileSync|Bun\.(file|spawn)|spawn|exec|fetch|require)\s*\(|\bimport\s*\(/.test(await Bun.file(input).text())) runtime = true;
          }
          affected = changed.some(path => runtime || inputs.has(path) || !/\.[cm]?[jt]sx?$/.test(path)
            // Only a source reached by a resolved graph can be classified.
            || !classified.has(path));
        } catch {
          affected = true;
        }
      }
    }
    if (affected) selected.push(file);
    else console.log(`SKIPPED ${pkg}/${file}: inputs unchanged`);
  }
  if (!selected.length) continue;
  for (const file of selected) {
    const key = `bun.${new Bun.CryptoHasher("sha256").update(`${pkg}/${file}`).digest("hex")}`;
    rmSync(join(green, key), { force: true });
  }
  await run(["bun", "test", `--parallel=${process.env.OG_GATE_THREADS ?? "2"}`,
    `--timeout=${process.env.OG_BUN_TEST_TIMEOUT ?? "30000"}`,
    `--timings=${join(logs, `bun-${entry.name}.json`)}`, "--update-timings",
    ...selected.map(file => `./${file}`)], resolve(pkg));
  if (tree && !unchecked) for (const file of selected) {
    const key = `bun.${new Bun.CryptoHasher("sha256").update(`${pkg}/${file}`).digest("hex")}`;
    await Bun.write(join(green, key), `${tree}\n`);
  }
}
if (unchecked) {
  console.error("GPU tests unchecked; no passing input snapshots recorded");
  process.exit(3);
}
