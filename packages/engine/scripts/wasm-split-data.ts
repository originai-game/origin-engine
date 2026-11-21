// A split module's data (docs/web-export.md §Modules): the read-only data
// only its functions reach goes with them. wasm-split moves functions and
// leaves every data segment in the primary, so a module's tables, strings
// and shader text would download with every game.
//
// The raw wasm is linked with `--emit-relocs` (crates/og-host-web/build.rs,
// for a publish build), so its `linking` section names every
// data symbol with its place and size and its `reloc.CODE` and `reloc.DATA`
// sections say which function and which data refer to which symbol. A
// `.rodata` symbol whose every referrer, through any chain of data, is one
// module's function belongs to that module; a symbol no relocation names,
// or one the primary or two modules reach, stays. wasm-bindgen moves no
// data (its output's segments sit where the raw module's do), so the
// addresses hold for the bound module the split reads.
//
// The primary keeps its segments with those bytes zeroed, as a module's
// functions are its placeholders until it loads; the module gets an active
// data segment over its imported memory that writes them back when it is
// instantiated, before any of its functions runs. Read-only bytes only:
// a lanes pool Worker instantiates the module again and writes the same
// bytes over the shared memory, which changes nothing a thread reads.

/** A byte range of the engine's memory, `[from, to)`. */
export interface Range {
  readonly from: number;
  readonly to: number;
}

interface DataSymbol {
  readonly name: string;
  readonly segment: number;
  readonly from: number;
  readonly to: number;
  readonly exported: boolean;
}

/** A reader over a wasm module's bytes. */
class Reader {
  at: number;
  readonly w: Uint8Array;
  constructor(w: Uint8Array, at = 0) {
    this.w = w;
    this.at = at;
  }
  u8(): number {
    return this.w[this.at++]!;
  }
  leb(): number {
    let r = 0;
    for (let s = 0; ; s += 7) {
      const b = this.w[this.at++]!;
      r += (b & 0x7f) * 2 ** s;
      if (!(b & 0x80)) return r;
    }
  }
  sleb(): number {
    let r = 0n;
    let s = 0n;
    let b: number;
    do {
      b = this.w[this.at++]!;
      r |= BigInt(b & 0x7f) << s;
      s += 7n;
    } while (b & 0x80);
    if (b & 0x40) r -= 1n << s;
    return Number(r);
  }
  str(): string {
    const n = this.leb();
    const s = new TextDecoder().decode(this.w.subarray(this.at, this.at + n));
    this.at += n;
    return s;
  }
}

interface Section {
  readonly id: number;
  readonly name: string;
  /** The payload: after the id and size (and a custom section's name). */
  readonly start: number;
  readonly end: number;
  /** The whole section, id included. */
  readonly head: number;
}

function sections(w: Uint8Array): Section[] {
  if (![0, 97, 115, 109, 1, 0, 0, 0].every((b, i) => w[i] === b)) throw new Error("invalid wasm header");
  const out: Section[] = [];
  const r = new Reader(w, 8);
  while (r.at < w.length) {
    const head = r.at;
    const id = r.u8();
    const size = r.leb();
    const end = r.at + size;
    const name = id === 0 ? r.str() : "";
    out.push({ id, name, start: id === 0 ? r.at : end - size, end, head });
    r.at = end;
  }
  return out;
}

interface Segment {
  /** Where it lands in memory: an active segment's offset, a passive one's `__wasm_init_memory` destination. */
  readonly address: number;
  /** Its bytes' place in the file. */
  readonly at: number;
  readonly length: number;
}

/** A function's name to its index, from the name section. */
function functionIndex(w: Uint8Array, all: Section[]): Map<string, number> {
  const names = all.find((s) => s.id === 0 && s.name === "name");
  if (!names) throw new Error("the wasm has no name section");
  const out = new Map<string, number>();
  const n = new Reader(w, names.start);
  while (n.at < names.end) {
    const sub = n.u8();
    const end = n.leb() + n.at;
    if (sub === 1) for (let k = n.leb(); k > 0; k--) {
      const i = n.leb();
      out.set(n.str(), i);
    }
    n.at = end;
  }
  return out;
}

/** The number of imported functions. */
function importedFunctions(w: Uint8Array, all: Section[]): number {
  const imports = all.find((s) => s.id === 2);
  if (!imports) return 0;
  let count = 0;
  const n = new Reader(w, imports.start);
  for (let k = n.leb(); k > 0; k--) {
    n.str();
    n.str();
    const kind = n.u8();
    if (kind === 0) {
      n.leb();
      count++;
    } else if (kind === 1) {
      n.u8();
      const f = n.leb();
      n.leb();
      if (f & 1) n.leb();
    } else if (kind === 2) {
      const f = n.leb();
      n.leb();
      if (f & 1) n.leb();
    } else if (kind === 3) {
      n.u8();
      n.u8();
    } else if (kind === 4) {
      n.u8();
      n.leb();
    }
  }
  return count;
}

/** Each function's body in the code section, by index. */
function bodiesOf(w: Uint8Array, all: Section[]): Array<{ index: number; from: number; to: number }> {
  const code = all.find((s) => s.id === 10)!;
  const out: Array<{ index: number; from: number; to: number }> = [];
  const c = new Reader(w, code.start);
  let index = importedFunctions(w, all);
  for (let k = c.leb(); k > 0; k--, index++) {
    const size = c.leb();
    out.push({ index, from: c.at, to: c.at + size });
    c.at += size;
  }
  return out;
}

/**
 * A lanes module's passive segments' destinations: `__wasm_init_memory`
 * copies each with `i32.const dest; i32.const 0; i32.const len;
 * memory.init seg`.
 */
function initDestinations(w: Uint8Array, all: Section[]): Map<number, number> {
  const out = new Map<number, number>();
  const index = functionIndex(w, all).get("__wasm_init_memory");
  if (index === undefined) return out;
  const body = bodiesOf(w, all).find((b) => b.index === index);
  if (!body) return out;
  const r = new Reader(w, body.from);
  const consts: number[] = [];
  while (r.at < body.to) {
    const op = r.u8();
    if (op === 0x41) consts.push(r.sleb());
    else if (op === 0xfc) {
      const sub = r.leb();
      if (sub === 8) {
        const seg = r.leb();
        r.u8();
        const dest = consts[consts.length - 3];
        if (dest !== undefined) out.set(seg, dest);
      } else if (sub === 9) r.leb();
      else if (sub === 10) {
        r.u8();
        r.u8();
      } else if (sub === 11) r.u8();
      consts.length = 0;
    }
    // Other opcodes only end a run of constants; the pattern needs nothing else decoded exactly.
    else if (op !== 0x41) consts.length = op === 0x0b || op === 0x1a ? 0 : consts.length;
  }
  return out;
}

/** The memory's data segments: each one's address and its bytes' place in the file. */
function segments(w: Uint8Array, data: Section, all: Section[]): Segment[] {
  const r = new Reader(w, data.start);
  const out: Segment[] = [];
  let passive: Map<number, number> | null = null;
  for (let n = r.leb(), i = 0; n > 0; n--, i++) {
    const mode = r.leb();
    let address = -1;
    if (mode === 0 || mode === 2) {
      if (mode === 2) r.leb();
      if (r.u8() !== 0x41) throw new Error("a data segment's offset is not an i32.const");
      address = r.sleb();
      if (r.u8() !== 0x0b) throw new Error("a data segment's offset is not one i32.const");
    }
    const length = r.leb();
    if (mode === 1) {
      passive ??= initDestinations(w, all);
      address = passive.get(i) ?? -1;
    }
    out.push({ address, at: r.at, length });
    r.at += length;
  }
  return out;
}

const SYMBOL_TABLE = 8;
const SEGMENT_INFO = 5;
const UNDEFINED = 0x10;
const EXPORTED = 0x20;
const EXPLICIT_NAME = 0x40;
/** Relocation types that carry an addend. */
const ADDEND = new Set([3, 4, 5, 8, 9, 11, 14, 15, 16, 17, 21, 22, 23, 25]);
/** Relocation types whose index is a data symbol (memory addresses). */
const MEMORY = new Set([3, 4, 5, 11, 14, 15, 16, 17, 21, 23, 25]);

/**
 * The bytes each module alone reaches, from `raw` (linked with
 * `--emit-relocs`) and `owner` (a function's name to its module; absent:
 * the primary). Read-only data only. Each module's ranges are sorted and
 * disjoint.
 */
export function moduleData(raw: Uint8Array, owner: (fn: string) => string | null): Map<string, Range[]> {
  const all = sections(raw);
  const linking = all.find((s) => s.id === 0 && s.name === "linking");
  if (!linking) throw new Error("the raw wasm has no linking section: link it with --emit-relocs (scripts/gate/commands.sh wasm_build)");
  const code = all.find((s) => s.id === 10)!;
  const data = all.find((s) => s.id === 11);
  if (!data) return new Map();
  const segs = segments(raw, data, all);
  // The linking section: the output segments' names and every symbol.
  const r = new Reader(raw, linking.start);
  if (r.leb() !== 2) throw new Error("linking section version is not 2");
  const segNames: string[] = [];
  const symbols: Array<{ kind: number; index: number; data?: DataSymbol }> = [];
  while (r.at < linking.end) {
    const id = r.u8();
    const end = r.leb() + r.at;
    if (id === SEGMENT_INFO) {
      for (let n = r.leb(); n > 0; n--) {
        segNames.push(r.str());
        r.leb();
        r.leb();
      }
    } else if (id === SYMBOL_TABLE) {
      for (let n = r.leb(); n > 0; n--) {
        const kind = r.u8();
        const flags = r.leb();
        if (kind === 1) {
          const name = r.str();
          if (flags & UNDEFINED) {
            symbols.push({ kind, index: -1 });
            continue;
          }
          const seg = r.leb();
          const offset = r.leb();
          const size = r.leb();
          const s = segs[seg];
          if (!s || s.address < 0) {
            symbols.push({ kind, index: -1 });
            continue;
          }
          const from = s.address + offset;
          symbols.push({ kind, index: seg, data: { name, segment: seg, from, to: from + size, exported: (flags & EXPORTED) !== 0 } });
        } else if (kind === 3) {
          symbols.push({ kind, index: r.leb() });
        } else {
          const index = r.leb();
          if (!(flags & UNDEFINED) || flags & EXPLICIT_NAME) r.str();
          symbols.push({ kind, index });
        }
      }
    }
    r.at = end;
  }
  // Each function's body, by its place in the code section, and its owner.
  const byIndex = new Map([...functionIndex(raw, all)].map(([name, i]) => [i, name]));
  const bodies = bodiesOf(raw, all).map((b) => ({ from: b.from - code.start, to: b.to - code.start, owner: owner(byIndex.get(b.index) ?? "") }));
  const bodyAt = (offset: number) => {
    let lo = 0;
    let hi = bodies.length - 1;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      const b = bodies[mid]!;
      if (offset < b.from) hi = mid - 1;
      else if (offset >= b.to) lo = mid + 1;
      else return b;
    }
    return null;
  };
  // The data symbols by address, to find the one a data relocation sits in.
  const defined = symbols.flatMap((s, i) => (s.data ? [{ i, d: s.data }] : [])).sort((a, b) => a.d.from - b.d.from);
  const symbolAt = (address: number) => {
    let found: number | null = null;
    let lo = 0;
    let hi = defined.length - 1;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (defined[mid]!.d.from <= address) {
        lo = mid + 1;
        if (address < defined[mid]!.d.to) found = defined[mid]!.i;
      } else hi = mid - 1;
    }
    return found;
  };
  const PRIMARY = "\0primary";
  const fromCode = new Map<number, Set<string>>();
  const fromData = new Map<number, Set<number>>();
  const add = <K, V>(m: Map<K, Set<V>>, k: K, v: V) => (m.get(k) ?? m.set(k, new Set()).get(k)!).add(v);
  for (const s of all) {
    if (s.id !== 0 || !s.name.startsWith("reloc.")) continue;
    const rr = new Reader(raw, s.start);
    const target = all[rr.leb()]!;
    for (let k = rr.leb(); k > 0; k--) {
      const type = rr.u8();
      const offset = rr.leb();
      const index = rr.leb();
      if (ADDEND.has(type)) rr.sleb();
      if (!MEMORY.has(type) || !symbols[index]?.data) continue;
      if (target.id === 10) {
        const b = bodyAt(offset);
        add(fromCode, index, b?.owner ?? PRIMARY);
      } else if (target.id === 11) {
        // The address the relocation patches: inside the segment holding it.
        const at = target.start + offset;
        const seg = segs.find((g) => g.address >= 0 && at >= g.at && at < g.at + g.length);
        const from = seg ? symbolAt(seg.address + (at - seg.at)) : null;
        if (from === null) add(fromCode, index, PRIMARY);
        else add(fromData, index, from);
      } else add(fromCode, index, PRIMARY);
    }
  }
  // Each symbol's owners, to a fixpoint over data referring to data.
  const owners = new Map<number, Set<string>>();
  for (const { i, d } of defined) {
    const o = new Set(fromCode.get(i) ?? []);
    // The segment names follow the symbol table in the section.
    if (d.exported || !(segNames[d.segment] ?? "").startsWith(".rodata") || (!fromCode.has(i) && !fromData.has(i))) o.add(PRIMARY);
    owners.set(i, o);
  }
  for (let changed = true; changed; ) {
    changed = false;
    for (const [i, refs] of fromData) {
      const o = owners.get(i);
      if (!o) continue;
      for (const j of refs) {
        for (const x of owners.get(j) ?? [PRIMARY]) {
          if (!o.has(x)) {
            o.add(x);
            changed = true;
          }
        }
      }
    }
  }
  // A byte moves when every symbol over it has the same one module.
  const kept: Range[] = [];
  const moved = new Map<string, Range[]>();
  for (const { i, d } of defined) {
    if (d.to <= d.from) continue;
    const o = owners.get(i)!;
    if (o.size === 1 && !o.has(PRIMARY)) {
      const m = [...o][0]!;
      (moved.get(m) ?? moved.set(m, []).get(m)!).push({ from: d.from, to: d.to });
    } else kept.push({ from: d.from, to: d.to });
  }
  const out = new Map<string, Range[]>();
  const keptMerged = merge(kept);
  for (const [m, ranges] of moved) {
    const left = subtract(merge(ranges), keptMerged);
    if (left.length) out.set(m, left);
  }
  // Two modules never claim one byte: a symbol over both has two owners.
  return out;
}

/** Sorted, merged ranges. */
function merge(ranges: Range[]): Range[] {
  const s = [...ranges].sort((a, b) => a.from - b.from);
  const out: Range[] = [];
  for (const r of s) {
    const last = out[out.length - 1];
    if (last && r.from <= last.to) out[out.length - 1] = { from: last.from, to: Math.max(last.to, r.to) };
    else out.push(r);
  }
  return out;
}

/** `a` without the bytes of `b` (both merged). */
function subtract(a: Range[], b: Range[]): Range[] {
  const out: Range[] = [];
  let j = 0;
  for (const r of a) {
    let from = r.from;
    while (j < b.length && b[j]!.to <= from) j++;
    let k = j;
    while (from < r.to) {
      const cut = b[k];
      if (!cut || cut.from >= r.to) {
        out.push({ from, to: r.to });
        break;
      }
      if (cut.from > from) out.push({ from, to: cut.from });
      from = Math.max(from, cut.to);
      k++;
    }
  }
  return out;
}

/** The bytes of `ranges` in `wasm`'s data segments, zeroed there; each range's bytes. */
export function takeData(wasm: Uint8Array, ranges: Range[]): Uint8Array[] {
  const all = sections(wasm);
  const data = all.find((s) => s.id === 11);
  if (!data) throw new Error("the primary has no data section");
  const segs = segments(wasm, data, all);
  return ranges.map(({ from, to }) => {
    // A passive (lanes) segment is written at its __wasm_init_memory address, which is its linked one.
    const seg = segs.find((g) => g.address >= 0 && from >= g.address && to <= g.address + g.length);
    if (!seg) throw new Error(`data ${from}..${to} is not inside one segment of the primary`);
    const at = seg.at + (from - seg.address);
    const bytes = wasm.slice(at, at + (to - from));
    wasm.fill(0, at, at + (to - from));
    return bytes;
  });
}

const lebBytes = (v: number) => {
  const out: number[] = [];
  do {
    let b = v & 0x7f;
    v = Math.floor(v / 128);
    if (v) b |= 0x80;
    out.push(b);
  } while (v);
  return out;
};
const slebBytes = (v: number) => {
  const out: number[] = [];
  for (;;) {
    let b = v & 0x7f;
    v >>= 7;
    const done = (v === 0 && !(b & 0x40)) || (v === -1 && b & 0x40);
    if (!done) b |= 0x80;
    out.push(b);
    if (done) return out;
  }
};

/**
 * `module` (a split module importing the primary's memory) with an active
 * data segment per range, writing `bytes` there when it is instantiated:
 * a data section after its code, before its custom sections.
 */
export function giveData(module: Uint8Array, ranges: Range[], bytes: Uint8Array[]): Uint8Array {
  const all = sections(module);
  if (all.some((s) => s.id === 11 || s.id === 12)) throw new Error("the split module already has data");
  const code = all.find((s) => s.id === 10);
  if (!code) throw new Error("the split module has no code");
  const body: number[] = lebBytes(ranges.length);
  const parts: Uint8Array[] = [];
  let size = 0;
  const head: number[][] = [];
  ranges.forEach((r, i) => {
    const h = [0, 0x41, ...slebBytes(r.from), 0x0b, ...lebBytes(bytes[i]!.length)];
    head.push(h);
    size += h.length + bytes[i]!.length;
  });
  size += body.length;
  const section = new Uint8Array(1 + lebBytes(size).length + size);
  let at = 0;
  section[at++] = 11;
  for (const b of lebBytes(size)) section[at++] = b;
  for (const b of body) section[at++] = b;
  ranges.forEach((_, i) => {
    section.set(head[i]!, at);
    at += head[i]!.length;
    section.set(bytes[i]!, at);
    at += bytes[i]!.length;
  });
  parts.push(module.subarray(0, code.end), section, module.subarray(code.end));
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let o = 0;
  for (const p of parts) {
    out.set(p, o);
    o += p.length;
  }
  return out;
}

/** `wasm` without the linker's `linking` and `reloc.*` sections, which only the split reads (from the raw module). */
export function withoutRelocs(wasm: Uint8Array): Uint8Array {
  const keep = sections(wasm).filter((s) => !(s.id === 0 && (s.name === "linking" || s.name.startsWith("reloc."))));
  const out = new Uint8Array(8 + keep.reduce((n, s) => n + (s.end - s.head), 0));
  out.set(wasm.subarray(0, 8));
  let at = 8;
  for (const s of keep) {
    out.set(wasm.subarray(s.head, s.end), at);
    at += s.end - s.head;
  }
  return out;
}

if (import.meta.main) {
  const [command, file] = process.argv.slice(2);
  if (command !== "strip" || !file) throw new Error("usage: bun scripts/wasm-split-data.ts strip <bound wasm>");
  const { readFileSync, writeFileSync } = await import("node:fs");
  writeFileSync(file, withoutRelocs(new Uint8Array(readFileSync(file))));
}
