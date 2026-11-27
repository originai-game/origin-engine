import { file } from '@origingame/engine/author';
// Effekseer node 0.3 "GlowCore"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
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
        0.1429,
        1.0
      ],
      [
        0.3594,
        1.0
      ],
      [
        0.5714,
        1.0
      ],
      [
        0.7812,
        0.5104
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
