import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.5
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "additive",
  "color": [
    1,
    0.8500000238418579,
    0.6499999761581421,
    1
  ],
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/realistic-starter/textures/ring"),
    "channel": "red",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        0,
        0
      ],
      "tile": [
        1,
        1
      ],
      "rotate": 0
    }
  }
});
