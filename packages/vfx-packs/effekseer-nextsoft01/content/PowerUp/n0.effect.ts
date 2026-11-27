import { file } from '@origingame/engine/author';
// Effekseer node 0 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Shock_wave_ring004"
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
        0.0667,
        1.0
      ],
      [
        0.7333,
        1.0
      ],
      [
        0.9062,
        0.2727
      ],
      [
        0.9531,
        0.1121
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
