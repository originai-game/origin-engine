import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "剣からの炎"
export default file.particles({
  "maxParticles": 4096,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2222,
      0.2222,
      0.2222
    ]
  },
  "spread": 0.0,
  "rate": 15000.0,
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.25,
    0.5833
  ],
  "speed": [
    12.2667,
    12.2667
  ],
  "size": [
    0.8889,
    0.8889
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0469,
      [
        1.0,
        1.0,
        1.0,
        0.0031
      ]
    ],
    [
      0.2812,
      [
        1.0,
        1.0,
        1.0,
        0.1098
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0796
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Fog3002"
  },
  "blend": "additive"
});
