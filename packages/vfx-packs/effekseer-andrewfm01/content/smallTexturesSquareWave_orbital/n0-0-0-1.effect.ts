import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "Ring Center"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/squareWave"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        16.0,
        1.0
      ],
      "scroll": [
        -3.75,
        0.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
