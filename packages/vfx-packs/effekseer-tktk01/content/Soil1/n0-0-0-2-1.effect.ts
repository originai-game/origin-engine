import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2.1 "着弾時衝撃リング柱"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave_ring002"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0938,
        0.8594
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
