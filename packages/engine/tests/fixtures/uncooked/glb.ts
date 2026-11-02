// Generated .glb files for the uncooked-load tests (og.load.gltfData).

/** A .glb of `json` with `bin` as its binary chunk, each padded to 4 bytes. */
export function glb(json: object, bin: Uint8Array): Uint8Array {
  const pad = (n: number) => (4 - (n % 4)) % 4;
  // ASCII JSON (game code has no TextEncoder).
  const raw = Uint8Array.from(JSON.stringify(json), (c) => c.charCodeAt(0));
  const text = new Uint8Array(raw.length + pad(raw.length)).fill(0x20);
  text.set(raw);
  const body = new Uint8Array(bin.length + pad(bin.length));
  body.set(bin);
  const out = new Uint8Array(12 + 8 + text.length + 8 + body.length);
  const v = new DataView(out.buffer);
  v.setUint32(0, 0x46546c67, true);
  v.setUint32(4, 2, true);
  v.setUint32(8, out.length, true);
  v.setUint32(12, text.length, true);
  v.setUint32(16, 0x4e4f534a, true);
  out.set(text, 20);
  v.setUint32(20 + text.length, body.length, true);
  v.setUint32(24 + text.length, 0x004e4942, true);
  out.set(body, 28 + text.length);
  return out;
}

/** Byte arrays laid end to end, each at a multiple of 4; their offsets. */
function pack(parts: Uint8Array[]): { bin: Uint8Array; at: number[] } {
  const at: number[] = [];
  let n = 0;
  for (const p of parts) {
    at.push(n);
    n += p.length + ((4 - (p.length % 4)) % 4);
  }
  const bin = new Uint8Array(n);
  parts.forEach((p, i) => bin.set(p, at[i]!));
  return { bin, at };
}

const bytes = (a: Float32Array | Uint16Array | Uint32Array) => new Uint8Array(a.buffer, a.byteOffset, a.byteLength);

/**
 * A textured quad (two triangles over x 0..1, y 0..1, its texture
 * coordinates the corner's x and y) sampling `png`, on a root node, and a
 * child node 5 m along x drawing it again. `extra` replaces top-level keys.
 */
export function quadGlb(png: Uint8Array, extra: Record<string, unknown> = {}): Uint8Array {
  const { bin, at } = pack([
    bytes(new Float32Array([0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0])),
    bytes(new Float32Array([0, 0, 1, 0, 1, 1, 0, 1])),
    bytes(new Uint16Array([0, 1, 2, 0, 2, 3])),
    png,
  ]);
  return glb(
    {
      asset: { version: "2.0" },
      scene: 0,
      scenes: [{ nodes: [0] }],
      nodes: [{ mesh: 0, children: [1] }, { mesh: 0, translation: [5, 0, 0] }],
      meshes: [{ primitives: [{ attributes: { POSITION: 0, TEXCOORD_0: 1 }, indices: 2, material: 0 }] }],
      materials: [{ pbrMetallicRoughness: { baseColorTexture: { index: 0 }, metallicFactor: 0 } }],
      textures: [{ source: 0 }],
      images: [{ bufferView: 3, mimeType: "image/png" }],
      buffers: [{ byteLength: bin.length }],
      bufferViews: [
        { buffer: 0, byteOffset: at[0], byteLength: 48 },
        { buffer: 0, byteOffset: at[1], byteLength: 32 },
        { buffer: 0, byteOffset: at[2], byteLength: 12 },
        { buffer: 0, byteOffset: at[3], byteLength: png.length },
      ],
      accessors: [
        { bufferView: 0, componentType: 5126, count: 4, type: "VEC3", min: [0, 0, 0], max: [1, 1, 0] },
        { bufferView: 1, componentType: 5126, count: 4, type: "VEC2" },
        { bufferView: 2, componentType: 5123, count: 6, type: "SCALAR" },
      ],
      ...extra,
    },
    bin,
  );
}

/** A flat grid of `n` x `n` quads (2n² triangles) over x and z, -1..1, untextured. */
export function gridGlb(n: number): Uint8Array {
  const side = n + 1;
  const pos = new Float32Array(side * side * 3);
  for (let j = 0; j < side; j++) {
    for (let i = 0; i < side; i++) {
      const k = (j * side + i) * 3;
      pos[k] = (i / n) * 2 - 1;
      pos[k + 1] = 0.05 * Math.sin(i * 0.7) * Math.cos(j * 0.5);
      pos[k + 2] = (j / n) * 2 - 1;
    }
  }
  const idx = new Uint32Array(n * n * 6);
  for (let j = 0, k = 0; j < n; j++) {
    for (let i = 0; i < n; i++, k += 6) {
      const a = j * side + i;
      idx.set([a, a + side, a + 1, a + 1, a + side, a + side + 1], k);
    }
  }
  const { bin, at } = pack([bytes(pos), bytes(idx)]);
  return glb(
    {
      asset: { version: "2.0" },
      scenes: [{ nodes: [0] }],
      nodes: [{ mesh: 0 }],
      meshes: [{ primitives: [{ attributes: { POSITION: 0 }, indices: 1 }] }],
      buffers: [{ byteLength: bin.length }],
      bufferViews: [
        { buffer: 0, byteOffset: at[0], byteLength: pos.byteLength },
        { buffer: 0, byteOffset: at[1], byteLength: idx.byteLength },
      ],
      accessors: [
        { bufferView: 0, componentType: 5126, count: side * side, type: "VEC3", min: [-1, -0.05, -1], max: [1, 0.05, 1] },
        { bufferView: 1, componentType: 5125, count: idx.length, type: "SCALAR" },
      ],
    },
    bin,
  );
}
