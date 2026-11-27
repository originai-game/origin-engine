import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0.6 "柱glow"
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
