import { file } from '@origingame/engine/author';
// Effekseer node 1 "ギザギザリング"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave_ring0011"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        0.4
      ],
      "scroll": [
        0.0,
        1.05
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
