import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "additive",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/FractalNoise_2"),
    "channel": "rgba",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        0,
        0
      ],
      "tile": [
        2,
        2
      ],
      "rotate": 0
    }
  }
});
