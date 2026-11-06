import { constants, copyFileSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";

// Generation copies are private to an admitted request. Mutable platform
// stores and output paths remain live: those are state, not prepared inputs.
const OMIT = new Set([".git", ".og", "og-out", "node_modules", "dist"]);
const MAX_BYTES = 512 * 1024 * 1024;
const MAX_FILES = 32768;

export function freezeInputs(argv: readonly string[], cwd: string, directory: string) {
  mkdirSync(directory, { recursive: true });
  const dir = mkdtempSync(join(directory, "run-"));
  const args = [...argv];
  let bytes = 0;
  let files = 0;
  const stamps = new Map<string, string>();
  const names = (path: string) => readdirSync(path).filter((name) => !OMIT.has(name) && join(path, name) !== resolve(directory)).sort();
  const stamp = (path: string) => {
    const s = statSync(path);
    const link = lstatSync(path);
    return JSON.stringify([
      s.ino,
      s.isDirectory() ? names(path) : [s.size, s.mtimeMs, s.ctimeMs],
      link.isSymbolicLink() ? [link.ino, link.mtimeMs, link.ctimeMs] : null,
    ]);
  };
  const copy = (source: string, target: string, depth = 0): void => {
    if (depth > 64 || ++files > MAX_FILES) throw new Error("session input tree exceeds its file limit");
    const stat = statSync(source);
    if (!stat.isDirectory() && !stat.isFile()) throw new Error(`session input is not a regular file or directory: ${source}`);
    const before = stamp(source);
    stamps.set(source, before);
    if (stat.isDirectory()) {
      mkdirSync(target, { recursive: true });
      for (const name of names(source)) copy(join(source, name), join(target, name), depth + 1);
    } else {
      bytes += stat.size;
      if (bytes > MAX_BYTES) throw new Error("session inputs exceed 512 MiB");
      mkdirSync(dirname(target), { recursive: true });
      copyFileSync(source, target, constants.COPYFILE_FICLONE);
    }
    if (stamp(source) !== before) throw new Error(`input changed during admission: ${source}`);
  };
  const dispose = () => rmSync(dir, { recursive: true, force: true });
  try {
    let inputRoot: string | undefined;
    for (let i = 0; i < args.length; i++) {
      const flag = args[i];
      if (!["--entry", "--root", "--platform-fake"].includes(flag!)) continue;
      const value = args[++i];
      if (!value) throw new Error(`${flag} needs a value`);
      const source = resolve(cwd, value);
      const target = join(dir, flag === "--root" ? "project" : flag === "--entry" ? join("bundle", basename(source)) : "fake.json");
      copy(source, target);
      if (flag === "--root") {
        inputRoot = target;
        args[i] = source;
      } else {
        args[i] = target;
        if (flag === "--entry" && existsSync(`${source}.map`)) {
          copy(`${source}.map`, `${target}.map`);
          const map: unknown = JSON.parse(readFileSync(`${target}.map`, "utf8"));
          if (typeof map !== "object" || map === null || !("sources" in map) || !Array.isArray(map.sources) || !map.sources.every((s): s is string | null => s === null || typeof s === "string"))
            throw new Error("session bundle has an invalid source map");
          const root = "sourceRoot" in map && typeof map.sourceRoot === "string" ? map.sourceRoot : "";
          map.sources = map.sources.map((s) => {
            if (s === null || /^[a-z][\w+.-]*:/i.test(s)) return s;
            return /^[a-z][\w+.-]*:/i.test(root) ? new URL(s, root).href : resolve(dirname(source), root, s);
          });
          if ("sourceRoot" in map) map.sourceRoot = "";
          writeFileSync(`${target}.map`, JSON.stringify(map));
        }
      }
    }
    for (const [path, before] of stamps) {
      if (stamp(path) !== before) throw new Error(`input changed during admission: ${path}`);
    }
    return { argv: args, ...(inputRoot ? { inputRoot } : {}), dispose };
  } catch (error) {
    dispose();
    throw error;
  }
}
