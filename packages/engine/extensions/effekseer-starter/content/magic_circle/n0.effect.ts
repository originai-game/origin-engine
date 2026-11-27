import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "alpha",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/ribbon01_trans"),
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
        7.61899995803833,
        1
      ],
      "rotate": 0
    }
  }
});
