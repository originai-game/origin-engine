import { file } from '@origingame/engine/author';
// Effekseer node 8 "Wave"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Shock_wave_ring001_long"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        2.8125
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
