import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "剣からの炎尻尾"
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
  "spread": 26.5254,
  "rate": 30000.0,
  "duration": 0.2917,
  "looping": false,
  "lifetime": [
    0.25,
    0.5833
  ],
  "speed": [
    10.1333,
    16.1151
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
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Fog3002"
  },
  "blend": "additive"
});
