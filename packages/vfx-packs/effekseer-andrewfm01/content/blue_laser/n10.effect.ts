import { file } from '@origingame/engine/author';
// Effekseer node 10 "Laser Base"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Shock_wave_ring002"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        1.0,
        0.5
      ],
      "scroll": [
        0.0,
        7.5
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
