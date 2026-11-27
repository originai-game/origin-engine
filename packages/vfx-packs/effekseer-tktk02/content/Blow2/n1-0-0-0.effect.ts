import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0 "giza_uti"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk02/textures/Shock_wave_ring0011"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        1.0
      ]
    }
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.3,
        1.0
      ],
      [
        1.0,
        0.0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});
