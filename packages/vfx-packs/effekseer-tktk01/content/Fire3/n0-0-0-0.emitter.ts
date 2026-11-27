import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "熱線尻尾"
export default file.particles({
  "maxParticles": 4096,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.7627,
      0.001,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 34800.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.1667,
    0.1667
  ],
  "speed": [
    0.9159,
    3.6637
  ],
  "size": [
    0.7051,
    0.7051
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      1.0,
      1.0
    ]
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
        0.0603
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Fog3002"
  },
  "blend": "additive"
});
