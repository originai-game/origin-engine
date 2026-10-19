# Compressed textures at runtime (owed row 32)

The runtime loaders now hand the renderer a cooked Basis texture in the
encoding the attached renderer takes for its kind
(`og_render::RenderInfo::texture_formats`, chosen per texture by
`og_cook::runtime::CookedTexture::target_format`), every cooked mip level:

| GPU family | colour, linear data | normal maps | where |
|---|---|---|---|
| BC | BC7 | BC5 (z rebuilt in the shader) | desktop GPUs, Apple silicon, lavapipe (its only family) |
| ASTC | ASTC 4x4 | ASTC 4x4 | Apple and most Android (and `RenderDesc::mobile()`) |
| ETC2 | ETC2 RGBA8 | EAC RG11 | other OpenGL ES 3 / WebGL2 devices |
| none | RGBA8 level 0, mips built on the GPU | | also pixel art, MSDF, lossless cooks, sizes not in whole 4x4 blocks |

Natively og-runtime passes the backend's formats to og-cook's basisu
transcoder (og-host-node's `OffscreenBackend`, so `og test` and the
executor addon take this path). On the web the engine Worker fetches the
bundle, attaches the renderer, asks the wasm for each texture's target
(`og_web_texture_target`), transcodes with Binomial's transcoder and hands
every level over (`og_web_put_texture_encoded`); the shell's `ready`
reports what was used (`textureFormat`: `bc7+bc5+rgba8` for fx-showcase in
Chrome 150 on both WebGPU and WebGL2, the `rgba8` being its 4x1 toon ramp).

## A cook fix the BC5 and EAC paths needed

Basis transcoders take BC5's and EAC RG11's second channel from **alpha**
(basisu's default channels 0 and 3; the Rust crate hard-codes them, and the
JS binding ignores the channel arguments for UASTC), and the cook had left a
normal map's alpha opaque: BC5 read y = 1 everywhere and the stone frame
missed by RMSE 0.076 (crates/og-cook/tests/compressed.rs, before the fix).
The cook now keeps x y z in RGB and copies y into alpha for `kind normal`
(`has_alpha` 1); its settings string says "normal y in alpha", so every
project recooks its normal maps. A map cooked before (alpha opaque, `has_alpha`
0) goes as colour (BC7 or ETC2) instead of a two-channel format. The toktx
fallback no longer passes `--normal_mode`, which kept only x in RGB.

## GPU memory and load time

Native, release build, Apple M4 Pro, Metal (`cargo test --release -p og-cook
--test compressed -- --test-threads=1 --nocapture`; GPU bytes are
`FrameStats::texture_bytes`, every level as allocated):

Tolerance against the RGBA8 frame: channel 32, outliers 0.5 %, RMSE 0.012. The normal map flipped (the error to catch): RMSE 0.02249.

| family | formats | GPU bytes (every level) | transcode ms (both) | upload ms (median over 5 fresh scenes of the first frame less a steady one; RGBA8 builds its mips on the GPU) | RMSE vs RGBA8 | outliers |
|---|---|---|---|---|---|---|
| rgba8 | rgba8 + rgba8 | 2796200 | 16.08 | 1.42 | - | - |
| bc | bc7 + bc5 | 699104 | 7.39 | 0.41 | 0.00540 | 0.0000 % |
| astc | astc4x4 + astc4x4 | 699104 | 4.66 | 4.18 | 0.00513 | 0.0000 % |
| etc2 | etc2 + eacrg11 | 699104 | 4.99 | 0.67 | 0.00541 | 0.0000 % |

Upload is a couple of milliseconds either way on these two 512x512 maps and
of the same order as the frame-to-frame noise (two more runs: RGBA8 2.01 and
1.42 ms; block formats 0.4-0.75 ms with single runs at 4-5.5 ms); the
compressed paths skip the GPU mip build. Transcoding every level to a block
format costs about what RGBA8 level 0 did (4-7 ms for both maps).

The whole template (og-tests' `gltf-reference` scene with the cooked
trickster and dropship, its 1024², 512² and 256² maps, plus the ground's
RGBA8 checker in every row):

| family | GPU bytes | models loaded (transcoding included) | RMSE vs `gltf-reference` (metal) | outliers |
|---|---|---|---|---|
| rgba8 (before) | 8,738,128 | 25.6 ms | 0.00508 | 0.010 % |
| bc (BC7 + BC5) | 3,233,188 | 24.1 ms | 0.01432 | 0.649 % |
| astc | 3,233,188 | 23.1 ms | 0.01432 | 0.650 % |
| etc2 (ETC2 + EAC RG11) | 3,233,188 | 27.7 ms | 0.01435 | 0.655 % |

Web: the Worker's transcoding of the template's five Basis textures with the
vendored Binomial transcoder (bun 1.3 on the M4 Pro, median of 5; the
browser runs the same wasm), the script below:

| family | textures | ms | bytes to the GPU |
|---|---|---|---|
| rgba8 (before: every level transcoded, level 0 handed over, the GPU builds the chain) | 5 | 16.2 | 10,136,228 |
| bc | 5 | 16.6 | 2,534,192 |
| astc | 5 | 13.6 | 2,534,192 |
| etc2 | 5 | 14.6 | 2,534,192 |

So: 4.0x less GPU memory for the textures (2.7x for the gltf scene, whose
RGBA8 ground stays), load time unchanged within noise.

## Tolerances, and why

- **A compressed texture against its RGBA8 decode** (`compressed_textures_draw_as_their_rgba8_decode`):
  og-tests' gfx tolerance (32 of 255 per channel, 0.5 % outliers) with RMSE
  0.012. Measured 0.0051-0.0054 with no outliers on all three families. The
  same stone with its normal map read flipped, the error the frame must
  catch, is RMSE 0.0225, so the bound sits between the block formats' error
  and the smallest real mistake; colour space, channel and orientation
  errors are larger still (sRGB lost on colour: 0.063 on the gltf scene).
- **The cooked models against `gltf-reference`**: RMSE 0.02 (the bound
  template.rs already allows cooked models) and 1 % outliers (gfx's 0.5 %
  doubled). The outliers are the trickster's 1024² colour map at the
  distance it is drawn: it now samples the cook's mip chain (basisu's mip
  filter, each level ETC1S-encoded: 19-25 dB against box-filtered level 0)
  instead of mips the GPU builds from level 0. Uploading that cooked chain
  as RGBA8 measured the same RMSE (0.0143) and outliers (0.65 %) as BC7, so
  the block formats add nothing measurable; the bound is the cooked mips'.
- **The game references** keep their own tolerance (RMSE 0.01, 0.5 %
  outliers): every fx-showcase frame still passes on metal (`og test`) and
  in Chrome on WebGPU and WebGL2 (`scripts/check.sh web-reference`), so no
  reference was re-recorded. The materials station, the one with a cooked
  normal map, moved from RMSE 0.00185 to 0.00244 against its metal, WebGPU
  and WebGL2 references.

Frames: `compressed-stone-rgba8-vs-bc.png` (the stone boards, RGBA8 and
BC7 + BC5) and `compressed-gltf-reference-bc.png` (the metal reference and
the cooked models with BC textures).

## The web measurement script

Run with bun from the repository root after `og cook templates/game`
(paths relative to a copy at `tests/evidence/P2/render/`):

```ts
// bun tests/evidence/P2/render/web-transcode-time.ts (after og cook templates/game).
// Times the web host's Basis transcoding (src/web/basis.ts, Binomial's wasm
// transcoder, as the engine Worker runs it) on templates/game's cooked
// textures: before (RGBA8 level 0, the GPU then builds mips) and after (every
// level in each family's block format). Bytes are what reaches the GPU.
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { loadBasisTranscoder, transcodeKtx2, type TextureFormat } from "../../../../packages/engine/src/web/basis.ts";

const cooked = new URL("../../../../templates/game/cooked", import.meta.url).pathname;
function section(blob: Uint8Array, name: string): Uint8Array | null {
  const v = new DataView(blob.buffer, blob.byteOffset, blob.byteLength);
  const count = v.getUint32(12, true);
  const pool = 64 + 48 * count;
  for (let i = 0; i < count; i++) {
    const at = 64 + 48 * i;
    const n = new TextDecoder().decode(blob.subarray(pool + v.getUint32(at, true), pool + v.getUint32(at, true) + v.getUint32(at + 4, true)));
    if (n !== name) continue;
    const off = Number(v.getBigUint64(at + 8, true));
    return blob.subarray(off, off + Number(v.getBigUint64(at + 16, true)));
  }
  return null;
}
const files: Array<{ name: string; ktx2: Uint8Array }> = [];
const walk = (d: string) => {
  for (const e of readdirSync(d, { withFileTypes: true })) {
    const p = join(d, e.name);
    if (e.isDirectory()) walk(p);
    else if (p.endsWith(".ogb")) {
      const blob = new Uint8Array(readFileSync(p));
      // Textures carry a ktx2 section; models' embedded textures are their own blobs.
      const k = section(blob, "ktx2");
      if (k && k[0] === 0xab) files.push({ name: p.slice(cooked.length + 1), ktx2: k.slice() });
    }
  }
};
walk(cooked);
const basis = await loadBasisTranscoder();
const plans: Array<[string, (uastc: boolean) => TextureFormat]> = [
  ["rgba8 (before)", () => "rgba8"],
  ["bc", (u) => (u ? "bc5" : "bc7")],
  ["astc", () => "astc4x4"],
  ["etc2", (u) => (u ? "eacrg11" : "etc2")],
];
console.log(`| family | textures | ms (median of 5) | bytes to the GPU |`);
console.log(`|---|---|---|---|`);
for (const [name, pick] of plans) {
  const runs: number[] = [];
  let bytes = 0;
  let n = 0;
  for (let r = 0; r < 5; r++) {
    bytes = 0; n = 0;
    const t = performance.now();
    for (const f of files) {
      // Normal maps are UASTC with y in alpha; linear data UASTC too, and goes as colour.
      const isNormal = f.name.includes("_n.png") || f.name.endsWith("tex2.ogb");
      const format = name.startsWith("rgba8") ? "rgba8" : pick(isNormal);
      const out = transcodeKtx2(basis, f.ktx2, format);
      if (format === "rgba8") {
        // As the Worker did before: every level transcoded, level 0 handed over.
        // Level 0 only, the GPU builds the chain: count the chain it allocates.
        let w = out.width, h = out.height;
        for (;;) { bytes += w * h * 4; if (w === 1 && h === 1) break; w = Math.max(1, w >> 1); h = Math.max(1, h >> 1); }
      } else bytes += out.levels.reduce((s, l) => s + l.length, 0);
      n++;
    }
    runs.push(performance.now() - t);
  }
  runs.sort((a, b) => a - b);
  console.log(`| ${name} | ${n} | ${runs[2]!.toFixed(1)} | ${bytes} |`);
}
console.log(files.map((f) => f.name).join(", "));
```
