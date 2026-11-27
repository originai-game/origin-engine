import { file } from '@origingame/engine/author';
// Effekseer node 3 "魔方陣"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-nextsoft01/textures/MagicCircle4"
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
        0.25,
        1.0
      ],
      [
        0.875,
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
