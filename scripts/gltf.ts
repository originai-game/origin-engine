// Binary glTF writing for the repository's offline fixture/preview generators.
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

export class Gltf {
  buf: number[] = [];
  views: Record<string, any>[] = [];
  accessors: Record<string, any>[] = [];
  bytes(raw: Uint8Array): number {
    while (this.buf.length % 4) this.buf.push(0);
    this.views.push({ buffer: 0, byteOffset: this.buf.length, byteLength: raw.length });
    for (const b of raw) this.buf.push(b);
    return this.views.length - 1;
  }
  add(
    data: number[],
    fmt: string,
    count: number,
    ctype: number,
    atype: string,
    minmax?: number[][] | null,
    target?: number | null,
  ): number {
    const size = fmt === "H" ? 2 : 4,
      raw = Buffer.alloc(data.length * size);
    data.forEach((v, i) =>
      fmt === "f" ? raw.writeFloatLE(v, i * size) : fmt === "H" ? raw.writeUInt16LE(v, i * size) : raw.writeUInt32LE(v, i * size),
    );
    const view = this.bytes(raw);
    if (target) this.views[view]!.target = target;
    const a: Record<string, any> = { bufferView: view, componentType: ctype, count, type: atype };
    if (minmax) [a.min, a.max] = minmax;
    this.accessors.push(a);
    return this.accessors.length - 1;
  }
  write(path: string, doc: Record<string, any>): number {
    while (this.buf.length % 4) this.buf.push(0);
    doc.buffers = [{ byteLength: this.buf.length }];
    doc.bufferViews = this.views;
    doc.accessors = this.accessors;
    // Stable key order, like the original fixture generators.
    const sorted = (v: any): any =>
      Array.isArray(v)
        ? v.map(sorted)
        : v && typeof v === "object"
          ? Object.fromEntries(
              Object.keys(v)
                .sort()
                .map((k) => [k, sorted(v[k])]),
            )
          : v;
    let js = Buffer.from(JSON.stringify(sorted(doc)));
    js = Buffer.concat([js, Buffer.alloc((4 - (js.length % 4)) % 4, 32)]);
    const header = Buffer.alloc(20),
      bin = Buffer.alloc(8);
    header.writeUInt32LE(0x46546c67);
    header.writeUInt32LE(2, 4);
    header.writeUInt32LE(28 + js.length + this.buf.length, 8);
    header.writeUInt32LE(js.length, 12);
    header.writeUInt32LE(0x4e4f534a, 16);
    bin.writeUInt32LE(this.buf.length);
    bin.writeUInt32LE(0x004e4942, 4);
    const glb = Buffer.concat([header, js, bin, Buffer.from(this.buf)]);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, glb);
    return glb.length;
  }
}
export const FACES = [
  [
    [1, 0, 0],
    [0, 1, 0],
    [0, 0, 1],
  ],
  [
    [-1, 0, 0],
    [0, 0, 1],
    [0, 1, 0],
  ],
  [
    [0, 1, 0],
    [0, 0, 1],
    [1, 0, 0],
  ],
  [
    [0, -1, 0],
    [1, 0, 0],
    [0, 0, 1],
  ],
  [
    [0, 0, 1],
    [1, 0, 0],
    [0, 1, 0],
  ],
  [
    [0, 0, -1],
    [0, 1, 0],
    [1, 0, 0],
  ],
];
export const cross = (u: number[], v: number[]) => [
  u[1]! * v[2]! - u[2]! * v[1]!,
  u[2]! * v[0]! - u[0]! * v[2]!,
  u[0]! * v[1]! - u[1]! * v[0]!,
];
export const bounds = (a: number[]): number[][] =>
  [Math.min, Math.max].map((f) => [0, 1, 2].map((k) => f(...a.filter((_, i) => i % 3 === k))));
