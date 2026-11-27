import { file } from '@origingame/engine/author';
// Effekseer node 7 "Bigger Laser"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/aurora01"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        -4.6875
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
