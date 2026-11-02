// @bun
// src/main.ts
import { og, defineResource, defineSystem, f64, u32 } from "/home/user/workspace/repo/packages/engine/src/index.ts";

// src/make.ts
var crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0;k < 8; k++)
    c = c & 1 ? 3988292384 ^ c >>> 1 : c >>> 1;
  return c >>> 0;
});
function crc32(bytes) {
  let c = 4294967295;
  for (const b of bytes)
    c = crcTable[(c ^ b) & 255] ^ c >>> 8;
  return (c ^ 4294967295) >>> 0;
}
function concat(parts) {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}
function u32be(n) {
  return new Uint8Array([n >>> 24, n >>> 16 & 255, n >>> 8 & 255, n & 255]);
}
var ascii = (s) => Uint8Array.from(s, (c) => c.charCodeAt(0));
function chunk(type, data) {
  const body = concat([ascii(type), data]);
  return concat([u32be(data.length), body, u32be(crc32(body))]);
}
function png(width, height, rgba) {
  const raw = new Uint8Array(height * (1 + width * 4));
  for (let y = 0;y < height; y++)
    raw.set(rgba.subarray(y * width * 4, (y + 1) * width * 4), y * (1 + width * 4) + 1);
  const blocks = [new Uint8Array([120, 1])];
  for (let at = 0;at < raw.length || at === 0; at += 65535) {
    const part = raw.subarray(at, at + 65535);
    const last = at + 65535 >= raw.length ? 1 : 0;
    blocks.push(new Uint8Array([last, part.length & 255, part.length >>> 8, ~part.length & 255, ~part.length >>> 8 & 255]), part);
  }
  let a = 1;
  let b = 0;
  for (const v of raw) {
    a = (a + v) % 65521;
    b = (b + a) % 65521;
  }
  blocks.push(u32be((b << 16 | a) >>> 0));
  const header = concat([u32be(width), u32be(height), new Uint8Array([8, 6, 0, 0, 0])]);
  return concat([new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", header), chunk("IDAT", concat(blocks)), chunk("IEND", new Uint8Array(0))]);
}
function image(width, height, paint) {
  const rgba = new Uint8Array(width * height * 4);
  for (let y = 0;y < height; y++) {
    for (let x = 0;x < width; x++) {
      const c = paint((x + 0.5) / width, (y + 0.5) / height);
      for (let k = 0;k < 4; k++)
        rgba[(y * width + x) * 4 + k] = Math.round(Math.min(1, Math.max(0, c[k])) * 255);
    }
  }
  return png(width, height, rgba);
}
var bytesOf = (a) => new Uint8Array(a.buffer, a.byteOffset, a.byteLength);
var pad4 = (n) => (4 - n % 4) % 4;
function vase(profile, segments, texture) {
  const ring = segments + 1;
  const positions = new Float32Array(profile.length * ring * 3);
  const normals = new Float32Array(profile.length * ring * 3);
  const uvs = new Float32Array(profile.length * ring * 2);
  for (let i = 0;i < profile.length; i++) {
    const [r, y] = profile[i];
    const [r0, y0] = profile[Math.max(0, i - 1)];
    const [r1, y1] = profile[Math.min(profile.length - 1, i + 1)];
    const dr = r1 - r0;
    const dy = y1 - y0;
    const len = Math.hypot(dr, dy) || 1;
    for (let s = 0;s < ring; s++) {
      const t = s / segments * Math.PI * 2;
      const k = i * ring + s;
      positions.set([Math.cos(t) * r, y, Math.sin(t) * r], k * 3);
      normals.set([Math.cos(t) * dy / len, -dr / len, Math.sin(t) * dy / len], k * 3);
      uvs.set([s / segments, 1 - i / (profile.length - 1)], k * 2);
    }
  }
  const indices = new Uint32Array((profile.length - 1) * segments * 6);
  for (let i = 0, k = 0;i < profile.length - 1; i++) {
    for (let s = 0;s < segments; s++, k += 6) {
      const a = i * ring + s;
      indices.set([a, a + ring, a + 1, a + 1, a + ring, a + ring + 1], k);
    }
  }
  const parts = [bytesOf(positions), bytesOf(normals), bytesOf(uvs), bytesOf(indices), texture];
  const at = [];
  let n = 0;
  for (const p of parts) {
    at.push(n);
    n += p.length + pad4(p.length);
  }
  const bin = new Uint8Array(n);
  parts.forEach((p, i) => bin.set(p, at[i]));
  const view = (i) => ({ buffer: 0, byteOffset: at[i], byteLength: parts[i].length });
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
      { bufferView: 3, componentType: 5125, count: indices.length, type: "SCALAR" }
    ]
  };
  const raw = ascii(JSON.stringify(json));
  const text = new Uint8Array(raw.length + pad4(raw.length)).fill(32);
  text.set(raw);
  return concat([u32le(1179937895), u32le(2), u32le(12 + 8 + text.length + 8 + bin.length), u32le(text.length), u32le(1313821514), text, u32le(bin.length), u32le(5130562), bin]);
}
function u32le(n) {
  return new Uint8Array([n & 255, n >>> 8 & 255, n >>> 16 & 255, n >>> 24]);
}

// src/main.ts
var Made = defineResource("Made", { vase: f64(), drawing: f64(), shown: u32() });
var glaze = image(256, 256, (u, v) => {
  const band = 0.5 + 0.5 * Math.sin(v * 38 + Math.sin(u * Math.PI * 12) * 0.9);
  const rim = v < 0.08 ? 0.45 : 1;
  return [(0.16 + 0.78 * band) * rim, (0.3 + 0.62 * band) * rim, (0.62 + 0.3 * band) * rim, 1];
});
var drawing = image(128, 128, (u, v) => {
  const x = u - 0.5;
  const y = v - 0.5;
  const r = Math.hypot(x, y);
  const ray = 0.5 + 0.5 * Math.cos(Math.atan2(y, x) * 12);
  if (r < 0.22)
    return [1, 0.78 - r, 0.2, 1];
  if (r < 0.3 + 0.16 * ray && ray > 0.55)
    return [1, 0.6, 0.15, 1];
  return [0, 0, 0, 0];
});
var profile = [
  [0, 0],
  [0.32, 0],
  [0.4, 0.08],
  [0.52, 0.32],
  [0.55, 0.5],
  [0.46, 0.78],
  [0.26, 1.02],
  [0.2, 1.2],
  [0.25, 1.32],
  [0.3, 1.38]
];
var Show = defineSystem("Show", { writes: [Made, og.Transform, og.Model, og.Sprite] }, (_q, _t, world) => {
  const m = world.resource(Made);
  if (!m.ok || m.value.shown)
    return;
  const [v, d] = [m.value.vase, m.value.drawing];
  for (const a of [v, d]) {
    if (og.load.status(world, a) === "failed")
      throw new Error(og.load.error(world, a) ?? "failed");
  }
  if (og.load.status(world, v) !== "ready" || og.load.status(world, d) !== "ready")
    return;
  m.value.shown = world.tick + 1;
  const e = world.spawn({ name: "vase", position: { x: -0.7, y: 0, z: 0 } });
  if (e.ok)
    world.insert(e.value, og.Model, { asset: v });
  const info = og.load.imageInfo(world, d);
  const s = world.spawn({ name: "drawing", position: { x: 0.9, y: 0.85, z: 0 } });
  if (info.ok && s.ok)
    world.insert(s.value, og.Sprite, { texture: info.value.texture, width: 1.1, height: 1.1 });
});
var main_default = og.game({
  tick: 60,
  seed: 1,
  systems: [Show],
  resources: [Made],
  start(world) {
    const cam = world.resource(og.Camera);
    if (cam.ok) {
      const eye = { x: 0, y: 1.3, z: 3.6 };
      cam.value.position = eye;
      cam.value.quaternion = og.transform.lookAt(eye, { x: 0, y: 0.6, z: 0 });
      cam.value.fov = 40;
      cam.value.background = { r: 0.03, g: 0.035, b: 0.05 };
    }
    const budget = { bytes: 4 * 1024 * 1024, triangles: 20000, textureSize: 1024 };
    const v = og.load.gltfData(world, "made/vase.glb", vase(profile, 48, glaze), { budget });
    const d = og.load.imageData(world, "made/sun.png", drawing, { budget });
    if (!v.ok)
      throw new Error(v.reason);
    if (!d.ok)
      throw new Error(d.reason);
    const m = world.resource(Made);
    if (m.ok)
      [m.value.vase, m.value.drawing] = [v.value, d.value];
  }
});
export {
  main_default as default
};

//# debugId=FD92D6ADF795222264756E2164756E21
//# sourceMappingURL=game.native.js.map
