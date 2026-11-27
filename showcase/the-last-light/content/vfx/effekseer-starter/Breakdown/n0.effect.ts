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
        0.733299970626831,
        1
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "subtractive",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/ShockWave5"),
    "channel": "rgba",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        0,
        0.9375
      ],
      "tile": [
        2,
        2
      ],
      "rotate": 0
    }
  }
});
