import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "alpha",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/icicle2"),
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
