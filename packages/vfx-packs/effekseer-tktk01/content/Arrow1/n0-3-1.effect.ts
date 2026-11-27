import { file } from '@origingame/engine/author';
// Effekseer node 0.3.1 "矢中心glow"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
    },
    "channel": "rgba",
    "uv": {}
  },
  "blend": "additive",
  "doubleSided": true
});
