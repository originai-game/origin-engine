// Splits a bound engine wasm into its primary and one module per optional
// capability (docs/web-export.md §Modules), with Binaryen's wasm-split
// (pinned by scripts/install-tools.sh):
//
//   bun scripts/wasm-split.ts <bound dir> <out dir>
//
// <bound dir> holds wasm-bindgen's output with its name section
// (origin-engine.js and origin-engine_bg.wasm, or the -lanes pair). <out dir>
// gets the primary under the same name and `<dir>/<module>.<hash12>.wasm`,
// each without its name section, and `<dir>/modules.json` ({ module: { file,
// bytes, sha256 } }); <dir> is `modules`, or `modules-lanes` for the lanes
// module, which shares its directory with the single-threaded one. A function goes to the first module of wasm-modules.json whose
// pattern its path matches, unless a game using no module runs it
// (wasm-profile.ts): those stay in the primary, so such a game loads none.
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { withoutNames } from "./wasm-symbols.ts";

const here = dirname(fileURLToPath(import.meta.url));
/** The bindgen module's features (wasm-split needs them named). */
const FEATURES = ["--enable-threads", "--enable-bulk-memory", "--enable-mutable-globals", "--enable-sign-ext", "--enable-nontrapping-float-to-int", "--enable-reference-types", "--enable-multivalue"];

declare const Bun: {
  spawnSync(cmd: string[], o: { stdout: "pipe"; stderr: "pipe" }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

function run(cmd: string, args: string[]): string {
  const r = Bun.spawnSync([cmd, ...args], { stdout: "pipe", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`${cmd} ${args.slice(0, 3).join(" ")} failed: ${new TextDecoder().decode(r.stderr)}`);
  return new TextDecoder().decode(r.stdout);
}

/** Each function's name, in index order, from the name section. */
export function functionNames(wasm: Uint8Array): string[] {
  let at = 8;
  const leb = () => {
    let r = 0;
    for (let s = 0; ; s += 7) {
      const b = wasm[at++]!;
      r += (b & 0x7f) * 2 ** s;
      if (!(b & 0x80)) return r;
    }
  };
  const names: string[] = [];
  while (at < wasm.length) {
    const id = wasm[at++];
    const size = leb();
    const end = at + size;
    if (id === 0) {
      const n = leb();
      const section = new TextDecoder().decode(wasm.subarray(at, at + n));
      at += n;
      if (section === "name") {
        while (at < end) {
          const sub = wasm[at++];
          const subEnd = leb() + at;
          if (sub === 1) {
            for (let k = leb(); k > 0; k--) {
              leb();
              const l = leb();
              names.push(new TextDecoder().decode(wasm.subarray(at, at + l)));
              at += l;
            }
          }
          at = subEnd;
        }
      }
    }
    at = end;
  }
  if (names.length === 0) throw new Error("the module has no name section: split bindgen's output before wasm-symbols strips it");
  return names;
}

const CRATE = /([A-Za-z_][A-Za-z0-9_]*)\[[0-9a-f]{6,}\]/g;
const STD = new Set(["core", "alloc", "std", "hashbrown"]);

/** The module a function belongs to by its name, or null for the primary. */
export function moduleOf(name: string, rules: Array<[string, RegExp[]]>): string | null {
  if (/^og_[a-z0-9_]+( multivalue shim| externref shim)?$/.test(name)) return null;
  for (const m of name.matchAll(CRATE)) {
    if (STD.has(m[1]!)) continue;
    const path = name.slice(m.index).replace(/\[[0-9a-f]{6,}\]/g, "");
    for (const [module, patterns] of rules) if (patterns.some((p) => p.test(path))) return module;
    return null;
  }
  return null;
}

/** Where a module's split modules sit beside it. */
export const moduleDir = (lanes: boolean) => (lanes ? "modules-lanes" : "modules");

export function splitEngine(bound: string, out: string): Record<string, { file: string; bytes: number; sha256: string }> {
  const lanes = existsSync(join(bound, "origin-engine-lanes_bg.wasm"));
  const stem = lanes ? "origin-engine-lanes" : "origin-engine";
  const dir = moduleDir(lanes);
  const wasmFile = join(bound, `${stem}_bg.wasm`);
  const named = new Uint8Array(readFileSync(wasmFile));
  const table = JSON.parse(readFileSync(join(here, "wasm-modules.json"), "utf8")) as { modules: Record<string, string[]> };
  const rules = Object.entries(table.modules).map(([m, ps]) => [m, ps.map((p) => new RegExp(p))] as [string, RegExp[]]);
  const work = mkdtempSync(join(tmpdir(), "og-split-"));
  try {
    // What a game using no module runs stays in the primary.
    const instrumented = join(work, `${stem}_bg.wasm`);
    run("wasm-split", ["--instrument", "-g", ...FEATURES, wasmFile, "-o", instrumented]);
    writeFileSync(join(work, `${stem}.js`), readFileSync(join(bound, `${stem}.js`)));
    // The lanes glue imports wasm-bindgen-rayon's Worker helper from beside it.
    if (existsSync(join(bound, "snippets"))) cpSync(join(bound, "snippets"), join(work, "snippets"), { recursive: true });
    const profile = join(work, "core.prof");
    run("bun", [join(here, "wasm-profile.ts"), join(work, `${stem}.js`), instrumented, profile]);
    const keep = new Set<string>();
    let keeping = false;
    for (const line of run("wasm-split", ["--print-profile=" + profile, wasmFile, ...FEATURES]).split("\n")) {
      if (line.startsWith("Keeping functions")) keeping = true;
      else if (line.startsWith("Splitting out")) keeping = false;
      else if (keeping && line.startsWith("+ ")) keep.add(line.slice(2));
    }
    const groups = new Map<string, string[]>();
    for (const name of functionNames(named)) {
      if (keep.has(name)) continue;
      const m = moduleOf(name, rules);
      if (m) (groups.get(m) ?? groups.set(m, []).get(m)!).push(name);
    }
    const manifest = join(work, "manifest.txt");
    writeFileSync(manifest, [...groups].map(([m, fns]) => `${m}\n${fns.join("\n")}\n`).join("\n"));
    const split = join(work, "split");
    mkdirSync(split);
    run("wasm-split", ["-g", "--multi-split", wasmFile, "--manifest", manifest, "--out-prefix", `${split}/`, "-o", join(split, "primary.wasm"), "--export-prefix=%", ...FEATURES]);
    rmSync(join(out, dir), { recursive: true, force: true });
    mkdirSync(join(out, dir), { recursive: true });
    writeFileSync(join(out, `${stem}_bg.wasm`), readFileSync(join(split, "primary.wasm")));
    const listed: Record<string, { file: string; bytes: number; sha256: string }> = {};
    for (const m of [...groups.keys()].sort()) {
      const bytes = withoutNames(new Uint8Array(readFileSync(join(split, `${m}.wasm`))));
      // Named by content: the release serves it as
      // /downloads/engine/<version>/wasm/<variant>/<module>.<hash12>.wasm.
      const sha256 = createHash("sha256").update(bytes).digest("hex");
      const file = `${dir}/${m}.${sha256.slice(0, 12)}.wasm`;
      writeFileSync(join(out, file), bytes);
      listed[m] = { file, bytes: bytes.length, sha256 };
    }
    writeFileSync(join(out, dir, "modules.json"), `${JSON.stringify(listed, null, 2)}\n`);
    return listed;
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  const [bound, out] = process.argv.slice(2);
  if (!bound || !out) throw new Error("usage: bun scripts/wasm-split.ts <bound dir> <out dir>");
  const listed = splitEngine(bound, out);
  for (const [m, f] of Object.entries(listed)) console.log(`${m} ${f.bytes} ${f.sha256.slice(0, 12)}`);
}
