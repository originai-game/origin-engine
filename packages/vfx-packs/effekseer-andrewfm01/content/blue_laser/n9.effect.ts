import { file } from '@origingame/engine/author';
// Effekseer node 9 "Laser Base"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Shock_wave_ring001"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        13.125
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
