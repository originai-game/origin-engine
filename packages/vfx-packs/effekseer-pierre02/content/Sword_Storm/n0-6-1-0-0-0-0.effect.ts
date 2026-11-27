import { file } from '@origingame/engine/author';
// Effekseer node 0.6.1.0.0.0.0 "WindTracker"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Line5"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.5,
        0.5
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
