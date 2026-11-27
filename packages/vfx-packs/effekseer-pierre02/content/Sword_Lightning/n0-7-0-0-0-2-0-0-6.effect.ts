import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.0.2.0.0.6 "CloudLighting_Variation3"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke_Lighting"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.5,
        0.5
      ],
      "offset": [
        0.0,
        0.5
      ]
    }
  },
  "blend": "additive"
});
