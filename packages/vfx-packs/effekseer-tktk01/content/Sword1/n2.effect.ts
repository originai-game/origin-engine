import { file } from '@origingame/engine/author';
// Effekseer node 2 "斬撃後の爆風"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Flame01"
    },
    "channel": "rgba",
    "uv": {}
  },
  "blend": "additive",
  "doubleSided": true
});
