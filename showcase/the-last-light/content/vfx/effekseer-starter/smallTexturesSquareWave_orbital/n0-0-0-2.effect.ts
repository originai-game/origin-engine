import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "additive",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/squareWave"),
    "channel": "rgba",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        3.75,
        0
      ],
      "tile": [
        8,
        1
      ],
      "rotate": 0
    }
  }
});
