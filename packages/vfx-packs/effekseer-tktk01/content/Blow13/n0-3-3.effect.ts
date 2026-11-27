import { file } from '@origingame/engine/author';
// Effekseer node 0.3.3 "リング外枠インパクト"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave_ring001"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        0.7812
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
