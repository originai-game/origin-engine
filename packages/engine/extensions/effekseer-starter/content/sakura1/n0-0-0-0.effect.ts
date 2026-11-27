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
        0.10000000149011612,
        1
      ],
      [
        0.8999999761581421,
        1
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "alpha",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/sakura-1"),
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
