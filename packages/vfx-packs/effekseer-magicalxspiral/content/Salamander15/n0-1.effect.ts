import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/WaterForm1"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.125,
        0.5
      ],
      "scroll": [
        0.0,
        0.2344
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});
