import { file } from '@origingame/engine/author';
// Effekseer node 4 "衝撃リング"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/icicle2"
    },
    "channel": "rgba",
    "uv": {}
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.0469,
        0.0646
      ],
      [
        0.1094,
        0.2197
      ],
      [
        0.1562,
        0.3719
      ],
      [
        0.2812,
        0.8287
      ],
      [
        0.3333,
        1.0
      ],
      [
        1.0,
        1.0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});
