import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "additive",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/ShockWave5"),
    "channel": "rgba",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        0,
        0.9375
      ],
      "tile": [
        1,
        1
      ],
      "rotate": 0
    }
  }
});
