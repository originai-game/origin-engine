import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "魔方陣"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-nextsoft01/textures/MagicCircle1"
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
        0.06,
        1.0
      ],
      [
        0.84,
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
