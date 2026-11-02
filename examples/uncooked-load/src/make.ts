// Files made in code at run time, as a generator or a player's editor
// would make them: a PNG and a binary glTF (.glb).

// ── PNG ─────────────────────────────────────────────────────────────────────

const crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});

function crc32(bytes: Uint8Array): number {
  let c = 0xffffffff;
  for (const b of bytes) c = crcTable[(c ^ b) & 0xff]! ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function concat(parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

function u32be(n: number): Uint8Array {
  return new Uint8Array([n >>> 24, (n >>> 16) & 0xff, (n >>> 8) & 0xff, n & 0xff]);
}

const ascii = (s: string) => Uint8Array.from(s, (c) => c.charCodeAt(0));

function chunk(type: string, data: Uint8Array): Uint8Array {
  const body = concat([ascii(type), data]);
  return concat([u32be(data.length), body, u32be(crc32(body))]);
}

/** An RGBA8 image as a PNG: rows unfiltered, deflate's stored blocks (no compression). */
export function png(width: number, height: number, rgba: Uint8Array): Uint8Array {
  const raw = new Uint8Array(height * (1 + width * 4));
  for (let y = 0; y < height; y++) raw.set(rgba.subarray(y * width * 4, (y + 1) * width * 4), y * (1 + width * 4) + 1);
  const blocks: Uint8Array[] = [new Uint8Array([0x78, 0x01])];
  for (let at = 0; at < raw.length || at === 0; at += 65535) {
    const part = raw.subarray(at, at + 65535);
    const last = at + 65535 >= raw.length ? 1 : 0;
    blocks.push(new Uint8Array([last, part.length & 0xff, part.length >>> 8, ~part.length & 0xff, (~part.length >>> 8) & 0xff]), part);
  }
  let a = 1;
  let b = 0;
  for (const v of raw) {
    a = (a + v) % 65521;
    b = (b + a) % 65521;
  }
  blocks.push(u32be(((b << 16) | a) >>> 0));
  const header = concat([u32be(width), u32be(height), new Uint8Array([8, 6, 0, 0, 0])]);
  return concat([new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]), chunk("IHDR", header), chunk("IDAT", concat(blocks)), chunk("IEND", new Uint8Array(0))]);
}

/** An RGBA8 image drawn by `paint(u, v)`, which answers linear-ish 0..1 r, g, b, a. */
export function image(width: number, height: number, paint: (u: number, v: number) => [number, number, number, number]): Uint8Array {
  const rgba = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const c = paint((x + 0.5) / width, (y + 0.5) / height);
      for (let k = 0; k < 4; k++) rgba[(y * width + x) * 4 + k] = Math.round(Math.min(1, Math.max(0, c[k]!)) * 255);
    }
  }
  return png(width, height, rgba);
}

// ── glTF ────────────────────────────────────────────────────────────────────

/** `points` through a Catmull-Rom spline, `steps` samples a span: a smooth lathe profile. */
export function smooth(points: Array<[number, number]>, steps: number): Array<[number, number]> {
  const at = (i: number) => points[Math.min(points.length - 1, Math.max(0, i))]!;
  const out: Array<[number, number]> = [];
  for (let i = 0; i < points.length - 1; i++) {
    const [p0, p1, p2, p3] = [at(i - 1), at(i), at(i + 1), at(i + 2)];
    for (let s = 0; s < steps; s++) {
      const t = s / steps;
      const f = (k: 0 | 1) =>
        0.5 * (2 * p1[k] + (p2[k] - p0[k]) * t + (2 * p0[k] - 5 * p1[k] + 4 * p2[k] - p3[k]) * t * t + (3 * p1[k] - p0[k] - 3 * p2[k] + p3[k]) * t * t * t);
      out.push([Math.max(0, f(0)), f(1)]);
    }
  }
  out.push(points[points.length - 1]!);
  return out;
}

const bytesOf = (a: Float32Array | Uint32Array) => new Uint8Array(a.buffer, a.byteOffset, a.byteLength);
const pad4 = (n: number) => (4 - (n % 4)) % 4;

/**
 * A vase lathed from `profile` (radius, height pairs, bottom to top) in
 * `segments` around y, its texture coordinates u around and v down the
 * profile, glazed with `texture` (a PNG) embedded in the file.
 */
export function vase(profile: Array<[number, number]>, segments: number, texture: Uint8Array): Uint8Array {
  const ring = segments + 1;
  const positions = new Float32Array(profile.length * ring * 3);
  const normals = new Float32Array(profile.length * ring * 3);
  const uvs = new Float32Array(profile.length * ring * 2);
  for (let i = 0; i < profile.length; i++) {
    const [r, y] = profile[i]!;
    // The profile's slope gives the normal's lean.
    const [r0, y0] = profile[Math.max(0, i - 1)]!;
    const [r1, y1] = profile[Math.min(profile.length - 1, i + 1)]!;
    const dr = r1 - r0;
    const dy = y1 - y0;
    const len = Math.hypot(dr, dy) || 1;
    for (let s = 0; s < ring; s++) {
      const t = (s / segments) * Math.PI * 2;
      const k = i * ring + s;
      positions.set([Math.cos(t) * r, y, Math.sin(t) * r], k * 3);
      normals.set([(Math.cos(t) * dy) / len, -dr / len, (Math.sin(t) * dy) / len], k * 3);
      uvs.set([s / segments, 1 - i / (profile.length - 1)], k * 2);
    }
  }
  const indices = new Uint32Array((profile.length - 1) * segments * 6);
  for (let i = 0, k = 0; i < profile.length - 1; i++) {
    for (let s = 0; s < segments; s++, k += 6) {
      const a = i * ring + s;
      indices.set([a, a + ring, a + 1, a + 1, a + ring, a + ring + 1], k);
    }
  }
  const parts = [bytesOf(positions), bytesOf(normals), bytesOf(uvs), bytesOf(indices), texture];
  const at: number[] = [];
  let n = 0;
  for (const p of parts) {
    at.push(n);
    n += p.length + pad4(p.length);
  }
  const bin = new Uint8Array(n);
  parts.forEach((p, i) => bin.set(p, at[i]!));
  const view = (i: number) => ({ buffer: 0, byteOffset: at[i]!, byteLength: parts[i]!.length });
  const top = Math.max(...profile.map((p) => p[1]));
  const wide = Math.max(...profile.map((p) => p[0]));
  const json = {
    asset: { version: "2.0", generator: "uncooked-load example" },
    scene: 0,
    scenes: [{ nodes: [0] }],
    nodes: [{ name: "vase", mesh: 0 }],
    meshes: [{ name: "vase", primitives: [{ attributes: { POSITION: 0, NORMAL: 1, TEXCOORD_0: 2 }, indices: 3, material: 0 }] }],
    materials: [{ name: "glaze", doubleSided: true, pbrMetallicRoughness: { baseColorTexture: { index: 0 }, metallicFactor: 0, roughnessFactor: 0.35 } }],
    textures: [{ source: 0 }],
    images: [{ bufferView: 4, mimeType: "image/png" }],
    buffers: [{ byteLength: bin.length }],
    bufferViews: [view(0), view(1), view(2), view(3), view(4)],
    accessors: [
      { bufferView: 0, componentType: 5126, count: positions.length / 3, type: "VEC3", min: [-wide, 0, -wide], max: [wide, top, wide] },
      { bufferView: 1, componentType: 5126, count: normals.length / 3, type: "VEC3" },
      { bufferView: 2, componentType: 5126, count: uvs.length / 2, type: "VEC2" },
      { bufferView: 3, componentType: 5125, count: indices.length, type: "SCALAR" },
    ],
  };
  const raw = ascii(JSON.stringify(json));
  const text = new Uint8Array(raw.length + pad4(raw.length)).fill(0x20);
  text.set(raw);
  return concat([u32le(0x46546c67), u32le(2), u32le(12 + 8 + text.length + 8 + bin.length), u32le(text.length), u32le(0x4e4f534a), text, u32le(bin.length), u32le(0x004e4942), bin]);
}

function u32le(n: number): Uint8Array {
  return new Uint8Array([n & 0xff, (n >>> 8) & 0xff, (n >>> 16) & 0xff, n >>> 24]);
}
