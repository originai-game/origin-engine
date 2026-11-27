import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.0.0.0.0.0.0 "06"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre01/textures/Tentacle"
    },
    "channel": "rgba",
    "uv": {}
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        1.0
      ],
      [
        0.8333,
        1.0
      ],
      [
        1.0,
        0.0
      ]
    ]
  },
  "blend": "alpha",
  "doubleSided": true
});
