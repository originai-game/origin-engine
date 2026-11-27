import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "地面リングカラー"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/7irohannten"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        0.4961
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
