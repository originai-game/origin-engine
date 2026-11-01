# glTF format fixtures

Small files for `tests/gltf_formats.rs`, one per extension or texture format
the cook reads. Every file here is made for this repository from nothing but
the scripts below and the tools named: no third-party art. They are released
under CC0 1.0 (public domain dedication).

All geometry is one box, from (-1, 0, -0.5) to (2, 3, 0.5), 24 vertices and 36
indices, so a test can check the decoded positions against that box. The
texture is 8×8 with four quadrants: blue (top left), red (top right), white
(bottom left), lime (bottom right).

| File | Made by | Exercises |
|---|---|---|
| `tex.png` | ImageMagick (`magick -size 4x4 xc:blue xc:red +append ( xc:white xc:lime +append ) -append tex.png`) | the reference texture |
| `tex.webp`, `tex.tga` | `magick tex.png -define webp:lossless=true tex.webp`, `magick tex.png tex.tga` | WebP and TGA sources |
| `tex.dds`, `tex-bc1.dds` | `magick tex.png -define dds:compression=none -define dds:mipmaps=0 tex.dds`, the same with `dds:compression=dxt1` for `tex-bc1.dds` | raw and BC1 DDS sources |
| `tex.psd` | `node psd.mjs tex.psd` (below) | a one-layer RGBA PSD with a PackBits composite |
| `box.gltf` | `node box.mjs` (below) | the plain reference box |
| `draco.glb` | `gltf-transform draco box.gltf draco.glb` (glTF Transform 4.5.1) | `KHR_draco_mesh_compression` |
| `meshopt.glb` | `gltf-transform meshopt box.gltf meshopt.glb` | `EXT_meshopt_compression` with filters, `KHR_mesh_quantization` |
| `quantized.glb` | `gltfpack -i box.gltf -o quantized.glb` (gltfpack 1.3) | `KHR_mesh_quantization`, `KHR_texture_transform` (dequantising uvs) |
| `gltfpack-cc.glb` | `gltfpack -cc -i box.gltf -o gltfpack-cc.glb` | meshopt with quantisation, as gltfpack ships it |
| `basisu.gltf`, `webp.gltf`, `extensions.gltf` | `node ext.mjs` (below) | `KHR_texture_basisu`, `EXT_texture_webp`; transform, unlit, lights, instancing, variants |

The glb and DDS recipes reproduce the committed bytes exactly; the PNG's
pixels match (ImageMagick stamps dates into its chunks). `basisu.gltf`'s image is the KTX2 (ETC1S, BasisLZ) the cook makes from
`tex.png`, taken out of its `.ogb`. The scripts run in a directory with the
inputs in `out/` and write there.

## box.mjs

```js
import fs from 'fs';
const lo=[-1,0,-0.5], hi=[2,3,0.5];
const P=[],N=[],U=[],I=[];
// faces: axis, sign
const faces=[[0,1],[0,-1],[1,1],[1,-1],[2,1],[2,-1]];
for(const [a,s] of faces){
  const b=(a+1)%3, c=(a+2)%3; const base=P.length/3;
  const corners=[[0,0],[1,0],[1,1],[0,1]];
  for(const [u,v] of corners){
    const p=[0,0,0]; p[a]= s>0?hi[a]:lo[a]; p[b]= u?hi[b]:lo[b]; p[c]= v?hi[c]:lo[c];
    P.push(...p); const n=[0,0,0]; n[a]=s; N.push(...n); U.push(u*0.5+(a*0.1), v*0.25+(s>0?0:0.5));
  }
  if(s>0) I.push(base,base+1,base+2,base,base+2,base+3); else I.push(base,base+2,base+1,base,base+3,base+2);
}
const f32=a=>Buffer.from(new Float32Array(a).buffer), u16=a=>Buffer.from(new Uint16Array(a).buffer);
const parts=[f32(P),f32(N),f32(U),u16(I)]; let off=0; const views=[];
for(const p of parts){views.push({buffer:0,byteOffset:off,byteLength:p.length}); off+=p.length; }
const bin=Buffer.concat(parts);
const png=fs.readFileSync('out/tex.png').toString('base64');
const g={asset:{version:'2.0',generator:'origin-engine fixture (box.mjs)'},scene:0,scenes:[{nodes:[0]}],nodes:[{mesh:0,name:'box'}],
meshes:[{name:'box',primitives:[{attributes:{POSITION:0,NORMAL:1,TEXCOORD_0:2},indices:3,material:0}]}],
materials:[{name:'tex',pbrMetallicRoughness:{baseColorTexture:{index:0},metallicFactor:0,roughnessFactor:0.7}}],
textures:[{source:0}],images:[{uri:'data:image/png;base64,'+png}],
accessors:[{bufferView:0,componentType:5126,count:24,type:'VEC3',min:lo,max:hi},{bufferView:1,componentType:5126,count:24,type:'VEC3'},{bufferView:2,componentType:5126,count:24,type:'VEC2'},{bufferView:3,componentType:5123,count:36,type:'SCALAR'}],
bufferViews:views,buffers:[{byteLength:bin.length,uri:'data:application/octet-stream;base64,'+bin.toString('base64')}]};
fs.writeFileSync('out/box.gltf',JSON.stringify(g,null,1));
```

## ext.mjs

```js
import fs from 'fs';
const box = JSON.parse(fs.readFileSync('out/box.gltf'));
const b64 = u => u.split(';base64,')[1];
const bin = Buffer.from(b64(box.buffers[0].uri), 'base64');
const img = (f, mime) => `data:${mime};base64,` + fs.readFileSync(f).toString('base64');
const gen = 'origin-engine fixture (fixtures/gltf/README.md)';
// One textured box with its texture in an image extension.
function imageExt(ext, file, mime) {
  const g = structuredClone(box);
  g.asset.generator = gen;
  g.extensionsUsed = [ext]; g.extensionsRequired = [ext];
  g.textures = [{ extensions: { [ext]: { source: 0 } } }];
  g.images = [{ uri: img(file, mime) }];
  return g;
}
fs.writeFileSync('out/basisu.gltf', JSON.stringify(imageExt('KHR_texture_basisu', 'out/tex.ktx2', 'image/ktx2')));
fs.writeFileSync('out/webp.gltf', JSON.stringify(imageExt('EXT_texture_webp', 'out/tex.webp', 'image/webp')));

// The other extensions on the same box geometry.
const g = structuredClone(box);
g.asset.generator = gen;
g.extensionsUsed = ['KHR_texture_transform', 'KHR_materials_unlit', 'KHR_lights_punctual', 'EXT_mesh_gpu_instancing', 'KHR_materials_variants'];
g.extensionsRequired = ['EXT_mesh_gpu_instancing'];
const tt = (o) => ({ index: 0, extensions: { KHR_texture_transform: o } });
g.materials = [
  { name: 'baked', pbrMetallicRoughness: { baseColorTexture: tt({ offset: [0.25, 0.5], scale: [0.5, 0.25], rotation: 0.5 }) } },
  { name: 'carried', pbrMetallicRoughness: { baseColorTexture: tt({ scale: [2, 3], rotation: 0.25 }) }, normalTexture: { index: 0 } },
  { name: 'unlit', pbrMetallicRoughness: { baseColorFactor: [0.2, 0.9, 0.4, 1] }, extensions: { KHR_materials_unlit: {} } },
  { name: 'red', pbrMetallicRoughness: { baseColorFactor: [1, 0, 0, 1] } },
];
const prim = (material, extra) => ({ attributes: { POSITION: 0, NORMAL: 1, TEXCOORD_0: 2 }, indices: 3, material, ...extra });
g.meshes = [
  { name: 'baked', primitives: [prim(0)] },
  { name: 'variant', primitives: [prim(1, { extensions: { KHR_materials_variants: { mappings: [{ material: 3, variants: [1] }, { material: 1, variants: [0] }] } } })] },
  { name: 'instanced', primitives: [prim(2)] },
];
// Instance transforms: three translations along x.
const inst = Buffer.from(new Float32Array([0, 0, 0, 5, 0, 0, 10, 0, 0]).buffer);
const pad = Buffer.alloc((4 - bin.length % 4) % 4);
const all = Buffer.concat([bin, pad, inst]);
g.bufferViews.push({ buffer: 0, byteOffset: bin.length + pad.length, byteLength: inst.length });
g.accessors.push({ bufferView: g.bufferViews.length - 1, componentType: 5126, count: 3, type: 'VEC3' });
g.buffers = [{ byteLength: all.length, uri: 'data:application/octet-stream;base64,' + all.toString('base64') }];
g.extensions = {
  KHR_lights_punctual: { lights: [
    { name: 'bulb', type: 'point', color: [1, 0.5, 0.25], intensity: 20 },
    { name: 'cone', type: 'spot', intensity: 40, range: 5, spot: { innerConeAngle: 0.2, outerConeAngle: 0.6 } },
    { name: 'sun', type: 'directional', intensity: 3 },
  ] },
  KHR_materials_variants: { variants: [{ name: 'base' }, { name: 'red' }] },
};
g.nodes = [
  { name: 'baked', mesh: 0 },
  { name: 'variant', mesh: 1, translation: [0, 0, 4] },
  { name: 'crates', mesh: 2, translation: [0, 0, -4], extensions: { EXT_mesh_gpu_instancing: { attributes: { TRANSLATION: g.accessors.length - 1 } } } },
  { name: 'bulb', translation: [0, 4, 0], extensions: { KHR_lights_punctual: { light: 0 } } },
  { name: 'cone', translation: [2, 4, 0], rotation: [-0.7071068, 0, 0, 0.7071068], extensions: { KHR_lights_punctual: { light: 1 } } },
  { name: 'sun', extensions: { KHR_lights_punctual: { light: 2 } } },
];
g.scenes = [{ nodes: [0, 1, 2, 3, 4, 5] }];
fs.writeFileSync('out/extensions.gltf', JSON.stringify(g));
```

## psd.mjs

```js
// A spec-correct 8x8 RGB + alpha PSD: one layer (count -1: the composite's
// fourth channel is its transparency), the composite PackBits-compressed.
import fs from 'fs';
const W = 8, H = 8;
const px = (x, y) => x < 4 && y < 4 ? [0, 0, 255] : x >= 4 && y < 4 ? [255, 0, 0] : x < 4 ? [255, 255, 255] : [0, 255, 0];
const planes = [0, 1, 2, 3].map(c => { const p = []; for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) p.push(c == 3 ? 255 : px(x, y)[c]); return p; });
const out = [];
const u16 = v => out.push(v >> 8 & 255, v & 255), u32 = v => { u16(v >>> 16); u16(v & 65535); }, bytes = b => out.push(...b);
bytes(Buffer.from('8BPS')); u16(1); bytes([0, 0, 0, 0, 0, 0]); u16(4); u32(H); u32(W); u16(8); u16(3);
u32(0); u32(0);
// Layer record: rect, 4 channels raw.
const rec = [];
const r16 = v => rec.push(v >> 8 & 255, v & 255), r32 = v => { r16(v >>> 16); r16(v & 65535); };
r32(0); r32(0); r32(H); r32(W); r16(4);
for (const id of [0, 1, 2, 0xffff]) { r16(id); r32(2 + W * H); }
rec.push(...Buffer.from('8BIMnorm')); rec.push(255, 0, 0, 0);
r32(12); r32(0); r32(0); rec.push(2, 76, 49, 0);
for (const c of [0, 1, 2, 3]) { r16(0); rec.push(...planes[c]); }
const info = [0xff, 0xff, ...rec]; // count -1
if (info.length % 2) info.push(0);
u32(4 + info.length + 4); u32(info.length); bytes(info); u32(0);
// Composite: PackBits, each row one literal run.
u16(1);
for (let i = 0; i < 4 * H; i++) u16(1 + W);
for (const p of planes) for (let y = 0; y < H; y++) { out.push(W - 1); bytes(p.slice(y * W, y * W + W)); }
fs.writeFileSync(process.argv[2], Buffer.from(out));
```
