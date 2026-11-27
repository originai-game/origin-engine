import { file } from '@origingame/engine/author';
// Effekseer node 0.3.0.0.2 "上半分glow"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/hisigata0"
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
