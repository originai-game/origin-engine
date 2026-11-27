import { file } from '@origingame/engine/author';
// Effekseer node 0 "Drill"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/drillshi"
    },
    "channel": "rgba",
    "uv": {
      "offset": [
        0.5,
        0.0
      ],
      "scroll": [
        3.75,
        0.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
