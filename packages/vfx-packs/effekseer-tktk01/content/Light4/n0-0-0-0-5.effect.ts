import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.5 "消滅柱"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        1.0,
        0.5
      ]
    }
  },
  "blend": "additive"
});
