import { file } from '@origingame/engine/author';
// Effekseer node 0.6.0.2.0.0.0 "WindTracker"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Line_2"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.5,
        2.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
