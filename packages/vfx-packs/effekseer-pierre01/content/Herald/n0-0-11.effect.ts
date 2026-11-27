import { file } from '@origingame/engine/author';
// Effekseer node 0.0.11 "CosmosSmokeVariation"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
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
