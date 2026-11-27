import { file } from '@origingame/engine/author';
// Effekseer node 0.1.5 "SplashVariation"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre01/textures/Splash"
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
  "blend": "alpha"
});
