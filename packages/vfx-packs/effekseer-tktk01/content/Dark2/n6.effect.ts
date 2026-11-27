import { file } from '@origingame/engine/author';
// Effekseer node 6 "魔法陣衝撃用グロー"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave_ring001"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        8.0,
        0.7812
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
