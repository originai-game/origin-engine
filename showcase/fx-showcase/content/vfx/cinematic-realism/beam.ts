import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "additive",
  "main": {
    "texture": ref("base/vfx/cinematic-realism/textures/streak"),
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
        1,
        1
      ],
      "rotate": 0
    }
  }
});
