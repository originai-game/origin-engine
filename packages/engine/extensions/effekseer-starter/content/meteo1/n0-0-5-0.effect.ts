import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "additive",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/Fire4"),
    "channel": "rgba",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        0,
        0.46880000829696655
      ],
      "tile": [
        2,
        0.5
      ],
      "rotate": 0
    }
  }
});
