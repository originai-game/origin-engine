import { file } from '@origingame/engine/author';
// Effekseer node 0.1.3 "リング外枠"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave_ring0011"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        8.0,
        1.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
