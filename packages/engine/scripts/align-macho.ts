// Works around the macOS 27 linker (ld-27037) leaving a dylib's LINKEDIT
// string table 4 bytes off 8-byte alignment after an odd-length indirect
// symbol table, which dyld then refuses ("mis-aligned LINKEDIT string pool").
// Run on an unsigned Mach-O (codesign --remove-signature first): it pads the
// string table onto an 8-byte boundary, fixes LC_SYMTAB and __LINKEDIT, and
// leaves re-signing to the caller. A file that is already aligned is untouched.
//
//   bun scripts/align-macho.ts <file>

import { readFileSync, writeFileSync } from "node:fs";

const path = process.argv[2];
if (!path) {
  console.error("usage: bun scripts/align-macho.ts <mach-o file>");
  process.exit(2);
}
const file = readFileSync(path);
const view = new DataView(file.buffer, file.byteOffset, file.byteLength);
if (view.getUint32(0, true) !== 0xfeedfacf) {
  console.error(`${path}: not a 64-bit little-endian Mach-O`);
  process.exit(1);
}
const ncmds = view.getUint32(16, true);
let at = 32;
let symtab = -1;
let linkedit = -1;
let signed = false;
for (let i = 0; i < ncmds; i++) {
  const cmd = view.getUint32(at, true);
  const size = view.getUint32(at + 4, true);
  if (cmd === 0x2) symtab = at;
  if (cmd === 0x1d) signed = true;
  if (cmd === 0x19) {
    const name = new TextDecoder().decode(file.subarray(at + 8, at + 24)).replace(/\0+$/, "");
    if (name === "__LINKEDIT") linkedit = at;
  }
  at += size;
}
if (symtab < 0 || linkedit < 0) {
  console.error(`${path}: no LC_SYMTAB or __LINKEDIT`);
  process.exit(1);
}
if (signed) {
  console.error(`${path}: remove the code signature first (codesign --remove-signature)`);
  process.exit(1);
}
const stroff = view.getUint32(symtab + 16, true);
const pad = (8 - (stroff % 8)) % 8;
if (pad === 0) process.exit(0);
const strsize = view.getUint32(symtab + 20, true);
if (stroff + strsize !== file.byteLength) {
  console.error(`${path}: the string table is not last in the file; cannot pad it in place`);
  process.exit(1);
}
const out = new Uint8Array(file.byteLength + pad);
out.set(file.subarray(0, stroff), 0);
out.set(file.subarray(stroff), stroff + pad);
const o = new DataView(out.buffer);
o.setUint32(symtab + 16, stroff + pad, true);
const filesize = o.getBigUint64(linkedit + 48, true);
o.setBigUint64(linkedit + 48, filesize + BigInt(pad), true);
const vmsize = o.getBigUint64(linkedit + 32, true);
if (filesize + BigInt(pad) > vmsize) o.setBigUint64(linkedit + 32, vmsize + 0x4000n, true);
writeFileSync(path, out);
console.log(`${path}: string table moved ${pad} bytes onto 8-byte alignment`);
