import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "ヘックス模様"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/FractalNoise_2"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        2.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
