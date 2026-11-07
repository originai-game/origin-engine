import { realpathSync } from "node:fs";
import { isAbsolute, relative, resolve, sep } from "node:path";
import { ImportGraph } from "./import-graph.ts";

declare const Bun: {
  spawnSync(argv: string[], options: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

/** Compare against a Git tree, including staged, unstaged and untracked sources. */
export async function affected(root: string, entry: string, suites: readonly string[], base: string): Promise<{ selected: string[]; reason: string }> {
  root = realpathSync(root);
  const git = (...args: string[]) => {
    const result = Bun.spawnSync(["git", ...args], { cwd: root });
    if (result.exitCode !== 0) throw new Error(new TextDecoder().decode(result.stderr).trim());
    return new TextDecoder().decode(result.stdout);
  };
  // Resolve first: a revision must never become a Git option or pathspec.
  const revision = git("rev-parse", "--verify", "--end-of-options", `${base}^{tree}`).trim();
  const worktree = realpathSync(git("rev-parse", "--show-toplevel").trim());
  // Root-relative with `/` separators on every OS, as ImportGraph names its inputs.
  const changed = new Set([
    ...git("diff", "--name-only", "--no-renames", "-z", revision, "--").split("\0"),
    ...git("ls-files", "--others", "--exclude-standard", "--full-name", "-z").split("\0"),
  ].filter(Boolean).map(file => relative(root, resolve(worktree, file)).split(sep).join("/")));
  if (changed.size === 0) return { selected: [], reason: "no changed inputs" };
  // Assets, resolver configuration, native code and paths outside this project
  // can affect every run. Unknown inputs never justify skipping a suite.
  if ([...changed].some(file => file === ".." || file.startsWith("../") || isAbsolute(file)
    || !/\.(?:[cm]?[jt]s|[jt]sx)$/.test(file) || file.endsWith(".d.ts"))) {
    return { selected: [...suites], reason: "unclassified inputs; all suites selected" };
  }
  const graph = new ImportGraph(root);
  const game = graph.inputs(entry).catch(() => null);
  const selected: string[] = [];
  for (const suite of suites) {
    try {
      const inputs = await graph.inputs(suite);
      if (suite.endsWith(".og.ts")) {
        const inputsOfGame = await game;
        if (inputsOfGame === null) {
          selected.push(suite);
          continue;
        }
        for (const file of inputsOfGame) inputs.add(file);
      }
      if ([...changed].some(file => inputs.has(file))) selected.push(suite);
    } catch {
      // Runtime reads and unresolved imports make this suite unclassifiable.
      selected.push(suite);
    }
  }
  return { selected, reason: "source dependencies (unresolved suites always selected)" };
}

if (import.meta.main) {
  const [root, entry, base, ...suites] = process.argv.slice(2);
  if (!root || !entry || !base) throw new Error("affected requires root, game entry, base revision and suite paths");
  process.stdout.write(JSON.stringify(await affected(root, entry, suites, base)) + "\n");
}
