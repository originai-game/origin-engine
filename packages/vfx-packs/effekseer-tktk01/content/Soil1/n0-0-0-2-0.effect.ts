import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2.0 "着弾時衝撃リング"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave_ring0011"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0938,
        1.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
