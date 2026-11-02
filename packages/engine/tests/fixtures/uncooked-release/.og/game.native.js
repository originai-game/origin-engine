// @bun
// packages/engine/tests/fixtures/uncooked-release/src/main.ts
import { og, defineResource, defineSystem, f64, u32 } from "/home/user/workspace/repo/packages/engine/src/index.ts";

// packages/engine/src/node/png.ts
function zlib() {
  return globalThis.process.getBuiltinModule("node:zlib");
}
var CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0;n < 256; n++) {
    let c = n;
    for (let k = 0;k < 8; k++)
      c = c & 1 ? 3988292384 ^ c >>> 1 : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();
function chunk(type, data) {
  const out = new Uint8Array(12 + data.length);
  const view = new DataView(out.buffer);
  view.setUint32(0, data.length);
  for (let i = 0;i < 4; i++)
    out[4 + i] = type.charCodeAt(i);
  out.set(data, 8);
  let c = 4294967295;
  for (let i = 4;i < 8 + data.length; i++)
    c = CRC[(c ^ out[i]) & 255] ^ c >>> 8;
  view.setUint32(8 + data.length, (c ^ 4294967295) >>> 0);
  return out;
}
function encodePng(img) {
  const stride = img.width * 3;
  const raw = new Uint8Array((stride + 1) * img.height);
  for (let y = 0;y < img.height; y++) {
    raw[y * (stride + 1)] = 1;
    for (let x = 0;x < img.width; x++) {
      for (let k = 0;k < 3; k++) {
        const v = img.rgba[(y * img.width + x) * 4 + k];
        const left = x > 0 ? img.rgba[(y * img.width + x - 1) * 4 + k] : 0;
        raw[y * (stride + 1) + 1 + x * 3 + k] = v - left & 255;
      }
    }
  }
  const ihdr = new Uint8Array(13);
  const v = new DataView(ihdr.buffer);
  v.setUint32(0, img.width);
  v.setUint32(4, img.height);
  ihdr.set([8, 2, 0, 0, 0], 8);
  const parts = [new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", ihdr), chunk("IDAT", zlib().deflateSync(raw, { level: 9 })), chunk("IEND", new Uint8Array(0))];
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  parts.reduce((at, p) => (out.set(p, at), at + p.length), 0);
  return out;
}

// packages/engine/tests/fixtures/uncooked/glb.ts
function glb(json, bin) {
  const pad = (n) => (4 - n % 4) % 4;
  const raw = Uint8Array.from(JSON.stringify(json), (c) => c.charCodeAt(0));
  const text = new Uint8Array(raw.length + pad(raw.length)).fill(32);
  text.set(raw);
  const body = new Uint8Array(bin.length + pad(bin.length));
  body.set(bin);
  const out = new Uint8Array(12 + 8 + text.length + 8 + body.length);
  const v = new DataView(out.buffer);
  v.setUint32(0, 1179937895, true);
  v.setUint32(4, 2, true);
  v.setUint32(8, out.length, true);
  v.setUint32(12, text.length, true);
  v.setUint32(16, 1313821514, true);
  out.set(text, 20);
  v.setUint32(20 + text.length, body.length, true);
  v.setUint32(24 + text.length, 5130562, true);
  out.set(body, 28 + text.length);
  return out;
}
function pack(parts) {
  const at = [];
  let n = 0;
  for (const p of parts) {
    at.push(n);
    n += p.length + (4 - p.length % 4) % 4;
  }
  const bin = new Uint8Array(n);
  parts.forEach((p, i) => bin.set(p, at[i]));
  return { bin, at };
}
var bytes = (a) => new Uint8Array(a.buffer, a.byteOffset, a.byteLength);
function gridGlb(n) {
  const side = n + 1;
  const pos = new Float32Array(side * side * 3);
  for (let j = 0;j < side; j++) {
    for (let i = 0;i < side; i++) {
      const k = (j * side + i) * 3;
      pos[k] = i / n * 2 - 1;
      pos[k + 1] = 0.05 * Math.sin(i * 0.7) * Math.cos(j * 0.5);
      pos[k + 2] = j / n * 2 - 1;
    }
  }
  const idx = new Uint32Array(n * n * 6);
  for (let j = 0, k = 0;j < n; j++) {
    for (let i = 0;i < n; i++, k += 6) {
      const a = j * side + i;
      idx.set([a, a + side, a + 1, a + 1, a + side, a + side + 1], k);
    }
  }
  const { bin, at } = pack([bytes(pos), bytes(idx)]);
  return glb({
    asset: { version: "2.0" },
    scenes: [{ nodes: [0] }],
    nodes: [{ mesh: 0 }],
    meshes: [{ primitives: [{ attributes: { POSITION: 0 }, indices: 1 }] }],
    buffers: [{ byteLength: bin.length }],
    bufferViews: [
      { buffer: 0, byteOffset: at[0], byteLength: pos.byteLength },
      { buffer: 0, byteOffset: at[1], byteLength: idx.byteLength }
    ],
    accessors: [
      { bufferView: 0, componentType: 5126, count: side * side, type: "VEC3", min: [-1, -0.05, -1], max: [1, 0.05, 1] },
      { bufferView: 1, componentType: 5125, count: idx.length, type: "SCALAR" }
    ]
  }, bin);
}

// packages/engine/tests/fixtures/uncooked-release/src/main.ts
var Loads = defineResource("Loads", { picture: f64(), grid: f64(), shown: u32(), released: u32() });
var size = 1024;
var picture = encodePng({ width: size, height: size, rgba: new Uint8Array(size * size * 4).map((_, i) => i * 7 & 255) });
var Cycle = defineSystem("Cycle", { writes: [Loads, og.Transform, og.Sprite, og.Model] }, (_q, _t, world) => {
  const l = world.resource(Loads);
  if (!l.ok)
    return;
  if (world.tick === 10) {
    const p = og.load.imageData(world, "drawings/wall.png", picture, {});
    const g = og.load.gltfData(world, "creations/field.glb", gridGlb(256), {});
    if (!p.ok || !g.ok)
      throw new Error(p.ok ? g.ok ? "" : g.reason : p.reason);
    l.value.picture = p.value;
    l.value.grid = g.value;
  }
  const [p, g] = [l.value.picture, l.value.grid];
  if (world.tick > 10 && !l.value.shown && og.load.status(world, p) === "ready" && og.load.status(world, g) === "ready") {
    l.value.shown = world.tick;
    const info = og.load.imageInfo(world, p);
    const s = world.spawn({ name: "wall", position: { x: -1.2, y: 1, z: 0 } });
    if (info.ok && s.ok)
      world.insert(s.value, og.Sprite, { texture: info.value.texture, width: 1.6, height: 1.6 });
    const m = world.spawn({ name: "field", position: { x: 1.2, y: 0, z: 0 } });
    if (m.ok)
      world.insert(m.value, og.Model, { asset: g });
  }
  if (world.tick === 40) {
    for (const a of [p, g]) {
      const r = og.load.release(world, a);
      if (!r.ok)
        throw new Error(r.reason);
    }
    l.value.released = world.tick;
  }
});
var main_default = og.game({
  tick: 60,
  seed: 1,
  systems: [Cycle],
  resources: [Loads],
  start(world) {
    const cam = world.resource(og.Camera);
    if (cam.ok) {
      cam.value.position = { x: 0, y: 2, z: 4 };
      cam.value.quaternion = og.transform.lookAt({ x: 0, y: 2, z: 4 }, { x: 0, y: 0.5, z: 0 });
    }
  }
});
export {
  main_default as default
};

//# debugId=1AD02BA0B6EC104564756E2164756E21
//# sourceMappingURL=game.native.js.map
