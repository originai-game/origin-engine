import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.4 "FireVariation"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.5,
        0.5
      ],
      "offset": [
        0.5,
        0.5
      ]
    }
  },
  "blend": "additive"
});
