import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        0
      ],
      [
        0.13330000638961792,
        1
      ],
      [
        0.666700005531311,
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
