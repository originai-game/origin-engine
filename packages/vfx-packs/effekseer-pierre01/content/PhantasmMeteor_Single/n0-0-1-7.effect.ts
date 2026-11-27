import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.7 "CosmosSmoke_White"
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
      ]
    }
  },
  "blend": "additive"
});
