/** Optional, bounded cache of bindgen or split output trees under the gate lock. */
import { createHash } from "node:crypto";
import { cpSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { spawnSync } from "node:child_process";
import { fileHash, receiptJson } from "./continuation.ts";

export function manifest(path: string): Record<string, string> {
  const stat = lstatSync(path);
  if (stat.isFile()) return { ".": fileHash(path) };
  if (!stat.isDirectory()) throw new Error("missing or linked product");
  const files: Record<string, string> = {};
  for (const name of new Bun.Glob("**/*").scanSync({ cwd: path, dot: true, onlyFiles: false, followSymlinks: false })) {
    const file = join(path, name), entry = lstatSync(file);
    if (!entry.isFile() && !entry.isDirectory()) throw new Error("unexpected product type");
    if (entry.isFile()) files[relative(path, file)] = fileHash(file);
  }
  if (!Object.keys(files).length) throw new Error("empty product");
  return files;
}
export function key(kind: string, variant: string, source: string, command: string[]): string {
  const tools = kind === "bindgen" ? ["wasm-bindgen"] : ["bun", "wasm-split", "rustfilt"];
  const hashes = tools.map(t => fileHash(Bun.which(t) ?? ""));
  // Splitting executes engine TS: retain the complete source snapshot.
  const tree = kind === "split" ? process.env.OG_GATE_TREE : null;
  if (kind === "split" && !tree) throw new Error("no source snapshot for splitter");
  const env = Object.fromEntries(Object.entries(process.env).filter(([k]) => /^(BUN_|WASM_|WBG_)/.test(k)));
  return createHash("sha256").update(receiptJson([1, kind, variant, source, command, hashes, manifest(source), tree, env])).digest("hex");
}
function main(): number {
  const [kind, variant, source, output, ...command] = process.argv.slice(2);
  if (!["bindgen", "split"].includes(kind) || !/^[a-zA-Z0-9_-]+$/.test(variant)) throw new Error("invalid product");
  const cache = join(process.env.OG_TARGET_DIR!, "og-wasm-products", kind, variant), product = join(cache, "product");
  let wanted: string | null = null;
  try { wanted = key(kind, variant, source, command); } catch { /* Unknown inputs run normally. */ }
  if (wanted) {
    try {
      const saved = JSON.parse(readFileSync(join(cache, "receipt.json"), "utf8"));
      if (receiptJson(saved) === receiptJson({ key: wanted, files: manifest(product) })) {
        cpSync(product, output, { recursive: true });
        console.log(`${kind} ${variant}: cache hit`);
        return 0;
      }
    } catch {
      // A failed restore may have left a partial directory.
      rmSync(output, { recursive: true, force: true });
    }
  }
  console.log(`${kind} ${variant}: cache miss`);
  const result = spawnSync(command[0], command.slice(1), { stdio: "inherit" });
  if (result.error) throw result.error;
  const rc = result.status ?? 1;
  if (rc || !wanted) return rc;
  let scratch: string | undefined;
  try {
    mkdirSync(dirname(cache), { recursive: true });
    scratch = mkdtempSync(join(dirname(cache), ".product-"));
    const pending = join(scratch, variant);
    cpSync(output, join(pending, "product"), { recursive: true });
    writeFileSync(join(pending, "receipt.json"), JSON.stringify({ key: wanted, files: manifest(join(pending, "product")) }));
    if (existsSync(cache)) rmSync(cache, { recursive: true });
    renameSync(pending, cache);
  } catch (error) { console.error(`${kind} ${variant}: could not cache output (${error})`); }
  finally { if (scratch) rmSync(scratch, { recursive: true, force: true }); }
  return 0;
}
if (import.meta.main) process.exitCode = main();
