import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "additive",
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/hisigata0"),
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
        0.5
      ],
      "rotate": 0
    }
  }
});
