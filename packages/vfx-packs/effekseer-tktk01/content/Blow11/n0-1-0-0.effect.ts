import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0 "レンズフレアglow"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/actionlines"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        1.0,
        0.9524
      ]
    }
  },
  "blend": "additive"
});
