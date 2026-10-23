// Lists a module's data segments, largest first, with a preview of each.
//   bun data-segs.ts NAMED.wasm [N]
import { readFileSync } from "node:fs";
const wasm = readFileSync(process.argv[2]);
const top = Number(process.argv[3] ?? 40);
let at = 8;
function leb(): number {
  let r = 0, s = 0;
  for (;;) { const b = wasm[at++]; r += (b & 0x7f) * 2 ** s; if (!(b & 0x80)) return r; s += 7; }
}
const segs: { i: number; off: number; data: Buffer }[] = [];
const names = new Map<number, string>();
while (at < wasm.length) {
  const id = wasm[at++]; const size = leb(); const end = at + size;
  if (id === 11) {
    const n = leb();
    for (let i = 0; i < n; i++) {
      const flags = leb();
      let off = -1;
      if (flags === 0 || flags === 2) {
        if (flags === 2) leb();
        const op = wasm[at++];
        if (op === 0x41) { // i32.const signed leb
          let r = 0, s = 0, b;
          do { b = wasm[at++]; r |= (b & 0x7f) << s; s += 7; } while (b & 0x80);
          off = r;
        }
        while (wasm[at++] !== 0x0b) {}
      }
      const len = leb();
      segs.push({ i, off, data: wasm.subarray(at, at + len) });
      at += len;
    }
  } else if (id === 0) {
    const nl = leb(); const sec = wasm.subarray(at, at + nl).toString(); at += nl;
    if (sec === "name") {
      while (at < end) {
        const sub = wasm[at++]; const ss = leb(); const se = at + ss;
        if (sub === 9) { for (let n = leb(); n > 0; n--) { const idx = leb(); const l = leb(); names.set(idx, wasm.subarray(at, at + l).toString()); at += l; } }
        at = se;
      }
    }
  }
  at = end;
}
segs.sort((a, b) => b.data.length - a.data.length);
let total = 0; for (const s of segs) total += s.data.length;
console.log(`${segs.length} segments, ${total} B`);
for (const s of segs.slice(0, top)) {
  const pv = s.data.subarray(0, 100).toString("latin1").replace(/[^\x20-\x7e]/g, ".");
  console.log(`${s.data.length}\t${names.get(s.i) ?? s.i}\t${pv}`);
}
