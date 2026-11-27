import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0 "glow"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave_ring001"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        10.0,
        1.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
