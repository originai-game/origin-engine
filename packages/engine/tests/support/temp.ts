import { mkdirSync, mkdtempSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** Caller removes this directory after its children exit, including failed launches. */
export function testTempDir(prefix: string): string {
  const root = fileURLToPath(new URL("../../../../.og/tmp/", import.meta.url));
  const inherited = process.env.TMPDIR;
  const base = inherited && resolve(inherited).startsWith(join(root, "run.")) ? inherited : root;
  mkdirSync(base, { recursive: true });
  return mkdtempSync(join(base, prefix));
}
