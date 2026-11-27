import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.3.1 "Ring_ｈｅａｄ"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk02/textures/Aura"
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
        1.0,
        0.0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});
