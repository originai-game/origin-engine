// Host tools ask the restricted Rust reader, never import author TS in Bun.
import { spawnSync } from "node:child_process";
import { readProjectOutput } from "./project-output.ts";

export interface StringTable {
  readonly id: string;
  readonly path: string;
  readonly locale: string;
  readonly base: boolean;
  readonly direction: "ltr" | "rtl";
  readonly fallback: string | null;
  readonly fonts: readonly string[];
  readonly entries: Readonly<Record<string, string>>;
}

export function readStringTables(root: string, builtins = false, og = process.env.OG_BIN || "og"): readonly StringTable[] {
  const result = spawnSync(og, ["i18n", "tables", root, "--json", ...(builtins ? ["--builtins"] : [])], { encoding: "utf8", maxBuffer: Infinity });
  return readProjectOutput<{tables: StringTable[]}>(result).tables;
}

// Bundling registers identities only; do not evaluate every locale or transfer
// every message merely to intercept a runtime import in a large content tree.
export function readStringTableIndex(root: string, og = process.env.OG_BIN || "og"): readonly Pick<StringTable, "id" | "path">[] {
  const result = spawnSync(og, ["i18n", "tables", root, "--index", "--json"], { encoding: "utf8", maxBuffer: Infinity });
  return readProjectOutput<{tables: Pick<StringTable, "id" | "path">[]}>(result).tables;
}
