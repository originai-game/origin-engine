import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3 "Ring"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk02/textures/Sonic_wave_ring005"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        1.0,
        0.7812
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
        0.3333,
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
