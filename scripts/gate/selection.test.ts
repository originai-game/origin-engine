import { afterEach, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const script = join(import.meta.dir, "inputs.py");
const roots: string[] = [];
afterEach(() => {
  for (const root of roots.splice(0)) rmSync(root, { recursive: true });
});

/** A scratch workspace: og-c depends on og-a, og-a includes one og-b source by path. */
function workspace() {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "og-gate-inputs-")));
  roots.push(root);
  const files: Record<string, string> = {
    "AGENTS.md": "guidance\n",
    "crates/og-a/src/lib.rs": '#[path = "../../og-b/src/shared.rs"]\nmod shared;\n',
    "crates/og-b/src/lib.rs": "mod other;\n",
    "crates/og-b/src/shared.rs": "pub fn shared() {}\n",
    "crates/og-b/src/other.rs": "pub fn other() {}\n",
    "crates/og-b/data.txt": "read by nobody the graph names\n",
    "crates/og-c/src/lib.rs": "pub fn c() {}\n",
    "crates/og-cli/src/main.rs": "fn main() {}\n",
    "crates/og-host-node/src/lib.rs": "pub fn addon() {}\n",
  };
  const names = Object.keys(files);
  for (const name of names) {
    mkdirSync(join(root, name, ".."), { recursive: true });
    writeFileSync(join(root, name), files[name]);
  }
  const edges: Record<string, string[]> = { "og-a": [], "og-b": [], "og-c": ["og-a"], "og-cli": [], "og-host-node": [] };
  const crates = Object.keys(edges);
  writeFileSync(join(root, "cargo.json"), JSON.stringify({
    packages: crates.map((name) => ({ id: name, name, manifest_path: join(root, "crates", name, "Cargo.toml") })),
    workspace_members: crates,
    resolve: { nodes: crates.map((id) => ({ id, dependencies: edges[id] })) },
  }));
  // The gate compares two trees, so the scratch repository holds every tree
  // the test needs, written by one `git fast-import`: the green one, and one
  // per file with that file alone changed. Nothing edits the working tree
  // afterwards, so the checks are independent and run side by side. The test
  // starts few processes because a loaded gate makes each one slow.
  const green = "refs/heads/green";
  const changed = (file: string) => `refs/heads/changed-${names.indexOf(file)}`;
  const blob = (file: string, contents: string) => `M 100644 inline ${file}\ndata ${contents.length}\n${contents}\n`;
  const commit = (ref: string, body: string) => `commit ${ref}\ncommitter gate <gate@localhost> 0 +0000\ndata 0\n${body}\n`;
  const git = (args: string[], input = "") => {
    const run = spawnSync("git", args, { cwd: root, encoding: "utf8", input });
    if (run.status !== 0) throw new Error(run.stderr);
  };
  git(["init", "-q"]);
  git(["fast-import", "--quiet"], commit(green, names.map((file) => blob(file, files[file])).join("")) +
    names.map((file) => commit(changed(file), `from ${green}\n` + blob(file, files[file] + "// changed\n"))).join(""));
  mkdirSync(join(root, "target/og-gate/green"), { recursive: true });
  writeFileSync(join(root, "target/og-gate/green/key"), green + "\n");
  const env = (tree: string) =>
    ({ ...process.env, GATE_LOGS: root, OG_TARGET_DIR: join(root, "target"), OG_GATE_TREE: tree, FORCE_ALL: "0" });
  const inputs = (name: string) =>
    spawnSync("python3", [script, "crates", name], { cwd: root, encoding: "utf8", env: env(green) }).stdout.trim().split(" ");
  return {
    crates: inputs,
    /** Of `changes`, each made alone, the files that leave og-c's tests skipped. */
    async skipsAfter(changes: string[]) {
      const paths = inputs("og-c");
      const codes = await Promise.all(changes.map((file) =>
        Bun.spawn(["python3", script, "unchanged", "key", ...paths], { cwd: root, env: env(changed(file)) }).exited));
      return changes.filter((_, index) => codes[index] === 0);
    },
  };
}

test("a binary's inputs are its own closure and the sources that closure names by path", async () => {
  const ws = workspace();
  expect(ws.crates("og-c")).toEqual(["crates/og-a", "crates/og-b/src/shared.rs", "crates/og-c"]);
  expect(ws.crates("og-cli")).toEqual(["crates/og-cli", "crates/og-host-node"]);
  expect(await ws.skipsAfter([
    "crates/og-cli/src/main.rs",
    "crates/og-b/src/other.rs",
    "AGENTS.md",
    "crates/og-b/src/shared.rs",
    "crates/og-a/src/lib.rs",
    // A file that is not Rust source has no proven reader: it runs everything.
    "crates/og-b/data.txt",
  ])).toEqual(["crates/og-cli/src/main.rs", "crates/og-b/src/other.rs", "AGENTS.md"]);
});

test("after a failed run only the binaries that ran every test and failed none are green", () => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "og-gate-passed-")));
  roots.push(root);
  const suite = (name: string, failures: number, cases: number) =>
    `<testsuite name="${name}" tests="${cases}" failures="${failures}" errors="0">` +
    '<testcase name="t" classname="c"/>'.repeat(cases) + "</testsuite>";
  writeFileSync(join(root, "junit.xml"),
    `<testsuites>${suite("og-a::tests", 0, 3)}${suite("og-b::tests", 1, 3)}${suite("og-c::tests", 0, 2)}</testsuites>`);
  const expected = { "og-a::tests": 3, "og-b::tests": 3, "og-c::tests": 3, "og-d::tests": 1 };
  const run = spawnSync("python3", ["-c",
    `import json, native; print(json.dumps(sorted(native.passed('junit.xml', ${JSON.stringify(expected)}))))`], {
    cwd: root, encoding: "utf8",
    env: { ...process.env, PYTHONPATH: import.meta.dir, GATE_LOGS: root, OG_TARGET_DIR: join(root, "target") },
  });
  // og-b failed a test, og-c stopped before its third, og-d never started.
  expect(JSON.parse(run.stdout)).toEqual(["og-a::tests"]);
});
