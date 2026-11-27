import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "additive",
  "color": [
    0.800000011920929,
    4,
    1.600000023841858,
    1
  ],
  "intensity": 1.5,
  "main": {
    "texture": ref("base/textures/noise"),
    "channel": "red",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        -2,
        0
      ],
      "tile": [
        3,
        1
      ],
      "rotate": 0
    }
  }
});
