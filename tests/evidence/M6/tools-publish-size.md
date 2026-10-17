# og build --publish: the template's size

`og build . --publish` on a fresh copy of `templates/game` (engine at this revision,
`packages/engine/dist/web-publish` from `scripts/build.sh`; the wasm is wasm-bindgen output,
not wasm-opt'd: CI's size gate measures the `wasm-opt -Oz` build separately).
Compressed column: brotli -q 11.

| Part | Bytes | Brotli |
|---|---:|---:|
| `origin-engine_bg.wasm` (engine: ECS, renderer, physics 2D+3D, audio, UI, probe trimmed to publish) | 10621593 | 2256666 |
| `game.js` (the game and the TypeScript runtime, minified) | 86112 | 19736 |
| `cooked/` (11 files: models, textures, font, sound, scene, manifest) | 1738395 | 1534663 |
| everything in `dist/` (also the page, the glue, the Basis transcoder) | 13234950 | 4055621 |

Probe: publish build, 7 members (hello, tick, state, pick, entity, input, capture).

Full table:

```
og cook: 7 assets, 10 outputs (0 cooked, 10 up to date), 1735738 bytes in cooked/, 45 ms
file                                               bytes  brotli-11
basis/LICENSE                                      11347       3061
basis/NOTICE                                         998        408
basis/README.md                                      999        529
basis/basis_transcoder.js                          57529      13699
basis/basis_transcoder.wasm                       527333     203605
cooked/.cook.ogt                                    2657        854
cooked/audio/blip.wav.ogb                           9144       8378
cooked/fonts/NotoSansSC-Regular-subset.ttf.ogb    555854     534856
cooked/models/dropship.glb.ogb                    167001      67703
cooked/models/trickster.glb.ogb                   387288     340096
cooked/models/trickster.glb.tex0.ogb              175229     175234
cooked/models/trickster.glb.tex1.ogb               71716      63754
cooked/models/trickster.glb.tex2.ogb              183174     166581
cooked/scenes/main.ogt.ogb                          1174        458
cooked/textures/stone.png.ogb                      22132      21599
cooked/textures/stone_n.png.ogb                   163026     155150
game.js                                            86112      19736
index.html                                           469        175
origin-engine.js                                  182403      20109
origin-engine_bg.wasm                           10621593    2256666
play.js                                             7772       2970
total                                           13234950    4055621
probe: publish build, 7 members: hello, tick, state, pick, entity, input, capture
```
