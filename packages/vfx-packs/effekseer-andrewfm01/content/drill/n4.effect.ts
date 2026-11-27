import { file } from '@origingame/engine/author';
// Effekseer node 4 "Drill"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Shock_wave_ring002"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        8.0,
        10.0
      ],
      "offset": [
        1.0,
        0.0
      ],
      "scroll": [
        37.5,
        0.0
      ]
    }
  },
  "blend": "subtractive",
  "doubleSided": true
});
