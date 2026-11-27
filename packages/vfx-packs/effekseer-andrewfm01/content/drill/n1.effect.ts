import { file } from '@origingame/engine/author';
// Effekseer node 1 "Drill"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/drillshi"
    },
    "channel": "rgba",
    "uv": {
      "offset": [
        0.125,
        0.0
      ],
      "scroll": [
        15.4688,
        0.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
