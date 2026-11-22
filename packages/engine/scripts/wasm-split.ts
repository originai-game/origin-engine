// Splits a bound engine wasm into its primary and one module per optional
// capability (docs/web-export.md §Modules), with Binaryen's wasm-split
// (pinned by scripts/install-tools.sh):
//
//   bun scripts/wasm-split.ts <bound dir> <out dir> <2d|3d> <raw wasm>
//
// <bound dir> holds wasm-bindgen's output with its name section
// (origin-engine.js and origin-engine_bg.wasm, or the -lanes pair). <out dir>
// gets the primary under the same name and `<dir>/<module>.<hash12>.wasm`,
// each without its name section, and `<dir>/modules.json` ({ module: { file,
// bytes, sha256 } }); <dir> is `modules`, or `modules-lanes` for the lanes
// module, which shares its directory with the single-threaded one. A function goes to the first module of wasm-modules.json whose
// pattern its path matches, unless a game using no module runs it
// (wasm-profile.ts): those stay in the primary, so such a game loads none.
// <raw wasm> is the linked module before wasm-bindgen, with its relocations
// (`--emit-relocs`): the read-only data only a module's functions reach
// goes into that module (wasm-split-data.ts).
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { devNull, tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { giveData, moduleData, takeData } from "./wasm-split-data.ts";
import { withoutNames } from "./wasm-symbols.ts";

const here = dirname(fileURLToPath(import.meta.url));
/** The bindgen module's features (wasm-split needs them named). */
const FEATURES = ["--enable-threads", "--enable-bulk-memory", "--enable-mutable-globals", "--enable-sign-ext", "--enable-nontrapping-float-to-int", "--enable-reference-types", "--enable-multivalue"];

declare const Bun: {
  spawnSync(cmd: string[], o: { stdin?: Uint8Array; stdout: "pipe"; stderr: "pipe" }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
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
/**
 * Crates whose generic instances belong with their type arguments: std's,
 * the serializers' (bincode's deserializer of a rapier2d body is
 * physics2d's code, called only from physics2d's load) and og-lanes' (its
 * `map` over dim3's body plans is physics3d's).
 */
const CARRIERS = new Set(["core", "alloc", "std", "hashbrown", "bincode", "serde_core", "og_lanes"]);

/** The module a function belongs to by its name, or null for the primary. */
export function moduleOf(name: string, rules: Array<[string, RegExp[]]>): string | null {
  if (/^og_[a-z0-9_]+( multivalue shim| externref shim)?$/.test(name)) return null;
  for (const m of name.matchAll(CRATE)) {
    if (CARRIERS.has(m[1]!)) continue;
    const path = name.slice(m.index).replace(/\[[0-9a-f]{6,}\]/g, "");
    for (const [module, patterns] of rules) if (patterns.some((p) => p.test(path))) return module;
    return null;
  }
  return null;
}

interface Table {
  modules: Record<string, string[]>;
  siblings?: Record<string, string[]>;
  roots: string[];
  entries: string[];
}

/** A name without its crates' disambiguators (`og_runtime[1a2b3c…]` → `og_runtime`), as wasm-modules.json's patterns read it. */
const plain = (name: string) => name.replace(/\[[0-9a-f]{6,}\]/g, "");

/** Binaryen's call graph of `wasmFile`: each function's direct callees. */
export function callGraph(wasmFile: string): Map<string, string[]> {
  const calls = new Map<string, string[]>();
  for (const line of run("wasm-opt", [...FEATURES, "--print-call-graph", wasmFile, "-o", devNull]).split("\n")) {
    const m = /^\s*"(.+)" -> "(.+)";/.exec(line);
    if (m) (calls.get(m[1]!) ?? calls.set(m[1]!, []).get(m[1]!)!).push(m[2]!);
  }
  return calls;
}

/** Whether a name reaches its module only through a carrier generic's type argument (`<alloc::…<og_story::…>>::new`): a body the linker may have folded with other instances. */
function generic(name: string): boolean {
  const first = CRATE.exec(name);
  CRATE.lastIndex = 0;
  return first !== null && CARRIERS.has(first[1]!);
}

/**
 * Moves out of `assigned` (function → module) each generic function
 * ([`generic`]) that a function outside its module calls: the linker folds
 * identical instances under one name, so spade's `RawVec::grow` can be the
 * body story's or the engine's code calls too, and leaving it in nav would
 * load nav there. Repeated until nothing moves (a moved function's own
 * generic callees then have a caller in the primary).
 */
export function keepShared(assigned: Map<string, string>, calls: Map<string, string[]>, siblings: Record<string, readonly string[]> = {}): void {
  const callers = new Map<string, string[]>();
  for (const [f, gs] of calls) for (const g of gs) (callers.get(g) ?? callers.set(g, []).get(g)!).push(f);
  // Sibling modules (wasm-modules.json `siblings`) are one source built per
  // variant (og-physics' rigid/sim.rs for dim2 and dim3): their
  // dimension-free functions come out byte-identical and the linker folds
  // them under one sibling's name, which the other calls. A function one
  // sibling calls in another goes to their parent, which both load.
  const parent = new Map<string, string>();
  for (const [p, kids] of Object.entries(siblings)) for (const k of kids) parent.set(k, p);
  for (let moved = true; moved; ) {
    moved = false;
    for (const [f, m] of assigned) {
      const outside = (callers.get(f) ?? []).map((c) => assigned.get(c)).filter((c) => c !== m);
      if (generic(f) && outside.length) {
        assigned.delete(f);
        moved = true;
      } else if (parent.has(m) && outside.some((c) => c !== undefined && parent.get(c) === parent.get(m))) {
        assigned.set(f, parent.get(m)!);
        moved = true;
      }
    }
  }
}

/**
 * Every function a frame, a canvas attach or a geometry call reaches by
 * direct calls (wasm-modules.json `roots`), from Binaryen's call graph,
 * stopping at its `entries`: the calls into a module the engine makes only
 * once the game uses it (each guarded in og-runtime). These stay in the
 * primary whatever their names, so no game's frame calls into a module it
 * does not use (a body the linker folded under a module type's name
 * included). An entry no function is named any more is an error: its
 * guard's boundary moved.
 */
export function framePath(wasmFile: string, names: string[], table: Table, calls: Map<string, string[]>): Set<string> {
  const roots = table.roots.map((p) => new RegExp(p));
  const entries = table.entries.map((p) => new RegExp(p));
  const missing = table.entries.filter((_, i) => !names.some((n) => entries[i]!.test(plain(n))));
  if (missing.length) throw new Error(`wasm-modules.json entries name no function of ${wasmFile}: ${missing.join(", ")} (inlined or renamed: name the guarded call that replaced it)`);
  const seen = new Set(names.filter((n) => roots.some((r) => r.test(plain(n)))));
  if (seen.size === 0) throw new Error(`wasm-modules.json roots name no function of ${wasmFile}`);
  const queue = [...seen];
  while (queue.length) {
    for (const g of calls.get(queue.pop()!) ?? []) {
      if (seen.has(g) || entries.some((e) => e.test(plain(g)))) continue;
      seen.add(g);
      queue.push(g);
    }
  }
  return seen;
}

/**
 * Rust symbol names demangled as wasm-bindgen writes them: rustc-demangle
 * keeping the hash, which is rustfilt's `-h` (scripts/install-tools.sh).
 */
function demangle(names: string[]): string[] {
  const r = Bun.spawnSync(["rustfilt", "-h"], { stdin: new TextEncoder().encode(names.join("\n") + "\n"), stdout: "pipe", stderr: "pipe" });
  if (r.exitCode !== 0) throw new Error(`rustfilt failed (scripts/install-tools.sh installs it): ${new TextDecoder().decode(r.stderr)}`);
  const out = new TextDecoder().decode(r.stdout).split("\n");
  if (out.length < names.length) throw new Error("rustfilt answered fewer names than it was given");
  return out.slice(0, names.length);
}

/** Where a module's split modules sit beside it. */
export const moduleDir = (lanes: boolean) => (lanes ? "modules-lanes" : "modules");

export function splitEngine(bound: string, out: string, raw: string): Record<string, { file: string; bytes: number; sha256: string; data: number }> {
  const lanes = existsSync(join(bound, "origin-engine-lanes_bg.wasm"));
  const stem = lanes ? "origin-engine-lanes" : "origin-engine";
  const dir = moduleDir(lanes);
  const wasmFile = join(bound, `${stem}_bg.wasm`);
  const named = new Uint8Array(readFileSync(wasmFile));
  const table = JSON.parse(readFileSync(join(here, "wasm-modules.json"), "utf8")) as Table;
  const rules = Object.entries(table.modules).map(([m, ps]) => [m, ps.map((p) => new RegExp(p))] as [string, RegExp[]]);
  const work = mkdtempSync(join(tmpdir(), "og-split-"));
  try {
    // What a game using no module runs stays in the primary.
    const instrumented = join(work, `${stem}_bg.wasm`);
    run("wasm-split", ["--instrument", "-g", ...FEATURES, wasmFile, "-o", instrumented]);
    writeFileSync(join(work, `${stem}.js`), readFileSync(join(bound, `${stem}.js`)));
    // The lanes glue imports wasm-bindgen-rayon's Worker helper from beside it.
    if (existsSync(join(bound, "snippets"))) cpSync(join(bound, "snippets"), join(work, "snippets"), { recursive: true });
    const coreProfile = join(work, "core.prof");
    run("bun", [join(here, "wasm-profile.ts"), join(work, `${stem}.js`), instrumented, coreProfile]);
    const keep = new Set<string>();
    let keeping = false;
    for (const line of run("wasm-split", ["--print-profile=" + coreProfile, wasmFile, ...FEATURES]).split("\n")) {
      if (line.startsWith("Keeping functions")) keeping = true;
      else if (line.startsWith("Splitting out")) keeping = false;
      else if (keeping && line.startsWith("+ ")) keep.add(line.slice(2));
    }
    const names = functionNames(named);
    const calls = callGraph(wasmFile);
    for (const name of framePath(wasmFile, names, table, calls)) keep.add(name);
    const assigned = new Map<string, string>();
    for (const name of names) {
      const m = keep.has(name) ? null : moduleOf(name, rules);
      if (m) assigned.set(name, m);
    }
    keepShared(assigned, calls, table.siblings);
    const groups = new Map<string, string[]>();
    for (const [name, m] of assigned) (groups.get(m) ?? groups.set(m, []).get(m)!).push(name);
    const manifest = join(work, "manifest.txt");
    writeFileSync(manifest, [...groups].map(([m, fns]) => `${m}\n${fns.join("\n")}\n`).join("\n"));
    const split = join(work, "split");
    mkdirSync(split);
    run("wasm-split", ["-g", "--multi-split", wasmFile, "--manifest", manifest, "--out-prefix", `${split}/`, "-o", join(split, "primary.wasm"), "--export-prefix=%", ...FEATURES]);
    rmSync(join(out, dir), { recursive: true, force: true });
    mkdirSync(join(out, dir), { recursive: true });
    // Each module's data out of the primary and into it.
    const primary = new Uint8Array(readFileSync(join(split, "primary.wasm")));
    // The raw module's names are mangled (v0); wasm-bindgen's are demangled, which `assigned` holds.
    const rawWasm = new Uint8Array(readFileSync(raw));
    const mangled = functionNames(rawWasm);
    const plainNames = demangle(mangled);
    const demangled = new Map(mangled.map((n, i) => [n, plainNames[i]!]));
    const data = moduleData(rawWasm, (fn) => assigned.get(demangled.get(fn) ?? fn) ?? null);
    const listed: Record<string, { file: string; bytes: number; sha256: string; data: number }> = {};
    for (const m of [...groups.keys()].sort()) {
      // Written with its name section, like the primary: pack.sh's
      // wasm-symbols.ts moves it to the symbols archive. Named and listed by
      // the bytes that ship, without it; the release serves it as
      // /downloads/engine/<version>/wasm/<variant>/<module>.<hash12>.wasm.
      const ranges = data.get(m) ?? [];
      const own = takeData(primary, ranges);
      const named = giveData(new Uint8Array(readFileSync(join(split, `${m}.wasm`))), ranges, own);
      const bytes = withoutNames(named);
      const sha256 = createHash("sha256").update(bytes).digest("hex");
      const file = `${dir}/${m}.${sha256.slice(0, 12)}.wasm`;
      writeFileSync(join(out, file), named);
      listed[m] = { file, bytes: bytes.length, sha256, data: own.reduce((n, b) => n + b.length, 0) };
    }
    writeFileSync(join(out, `${stem}_bg.wasm`), primary);
    writeFileSync(join(out, dir, "modules.json"), `${JSON.stringify(listed, null, 2)}\n`);
    return listed;
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  const [bound, out, raw] = process.argv.slice(2);
  if (!bound || !out || !raw) throw new Error("usage: bun scripts/wasm-split.ts <bound dir> <out dir> <raw wasm>");
  const listed = splitEngine(bound, out, raw);
  for (const [m, f] of Object.entries(listed)) console.log(`${m} ${f.bytes} ${f.sha256.slice(0, 12)} data ${f.data}`);
}
