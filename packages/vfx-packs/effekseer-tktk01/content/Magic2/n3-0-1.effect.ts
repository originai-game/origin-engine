import { file } from '@origingame/engine/author';
// Effekseer node 3.0.1 "リング柱"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave_ring0011"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        1.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
