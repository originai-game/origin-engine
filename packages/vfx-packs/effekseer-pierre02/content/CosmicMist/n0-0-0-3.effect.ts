import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.3 "CosmosSmoke_White"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
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
