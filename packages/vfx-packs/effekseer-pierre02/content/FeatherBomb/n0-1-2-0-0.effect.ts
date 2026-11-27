import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2.0.0 "SparkEmitter"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Feather"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.25,
        0.25
      ],
      "offset": [
        0.0,
        0.5
      ]
    }
  },
  "blend": "alpha"
});
