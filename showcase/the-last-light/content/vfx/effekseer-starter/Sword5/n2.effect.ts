import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        1
      ],
      [
        0.16670000553131104,
        1
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/Aurora_B"),
    "channel": "rgba",
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
