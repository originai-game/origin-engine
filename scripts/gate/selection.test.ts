import { afterEach, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { appendFileSync, mkdtempSync, mkdirSync, realpathSync, rmSync, writeFileSync } from "node:fs";
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
    ".gitignore": "target/\n",
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
  for (const [name, contents] of Object.entries(files)) {
    mkdirSync(join(root, name, ".."), { recursive: true });
    writeFileSync(join(root, name), contents);
  }
  const edges: Record<string, string[]> = { "og-a": [], "og-b": [], "og-c": ["og-a"], "og-cli": [], "og-host-node": [] };
  const names = Object.keys(edges);
  writeFileSync(join(root, "cargo.json"), JSON.stringify({
    packages: names.map((name) => ({ id: name, name, manifest_path: join(root, "crates", name, "Cargo.toml") })),
    workspace_members: names,
    resolve: { nodes: names.map((id) => ({ id, dependencies: edges[id] })) },
  }));
  const git = (...args: string[]) => {
    const run = spawnSync("git", args, { cwd: root, encoding: "utf8" });
    if (run.status !== 0) throw new Error(run.stderr);
    return run.stdout.trim();
  };
  git("init", "-q");
  const tree = () => (git("add", "-A"), git("write-tree"));
  const inputs = (tree: string, ...args: string[]) =>
    spawnSync("python3", [script, ...args], {
      cwd: root, encoding: "utf8",
      env: { ...process.env, GATE_LOGS: root, OG_TARGET_DIR: join(root, "target"), OG_GATE_TREE: tree, FORCE_ALL: "0" },
    });
  const green = tree();
  mkdirSync(join(root, "target/og-gate/green"), { recursive: true });
  writeFileSync(join(root, "target/og-gate/green/key"), green + "\n");
  return {
    crates: (name: string) => inputs(green, "crates", name).stdout.trim().split(" "),
    /** Whether og-c's tests are skipped after `file` changed. */
    skipsAfter(file: string) {
      appendFileSync(join(root, file), "// changed\n");
      const run = inputs(tree(), "unchanged", "key", ...this.crates("og-c"));
      git("checkout", "-q", green, "--", "."), git("read-tree", green);
      return run.status === 0;
    },
  };
}

test("a binary's inputs are its own closure and the sources that closure names by path", () => {
  const ws = workspace();
  expect(ws.crates("og-c")).toEqual(["crates/og-a", "crates/og-b/src/shared.rs", "crates/og-c"]);
  expect(ws.crates("og-cli")).toEqual(["crates/og-cli", "crates/og-host-node"]);
  expect(ws.skipsAfter("crates/og-cli/src/main.rs")).toBe(true);
  expect(ws.skipsAfter("crates/og-b/src/other.rs")).toBe(true);
  expect(ws.skipsAfter("AGENTS.md")).toBe(true);
  expect(ws.skipsAfter("crates/og-b/src/shared.rs")).toBe(false);
  expect(ws.skipsAfter("crates/og-a/src/lib.rs")).toBe(false);
  // A file that is not Rust source has no proven reader: it runs everything.
  expect(ws.skipsAfter("crates/og-b/data.txt")).toBe(false);
});
