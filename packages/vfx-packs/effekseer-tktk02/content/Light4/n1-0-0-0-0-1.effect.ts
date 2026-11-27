import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.0.1 "ReimsColor"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        1.0,
        2.0
      ],
      "offset": [
        0.0,
        -1.0
      ]
    }
  },
  "blend": "additive"
});
