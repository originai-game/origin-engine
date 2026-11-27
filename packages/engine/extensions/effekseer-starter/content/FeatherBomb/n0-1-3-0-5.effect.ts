import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "alpha",
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/Splash"),
    "channel": "rgba",
    "uv": {
      "offset": [
        0.5,
        0
      ],
      "polar": false,
      "scroll": [
        0,
        0
      ],
      "tile": [
        0.5,
        0.5
      ],
      "rotate": 0
    }
  }
});
