import { file } from '@origingame/engine/author';
// Effekseer node 0 "Smoke"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.001,
      2.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 40,
      "interval": 0.0167
    }
  ],
  "duration": 0.6667,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    1.591,
    6.364
  ],
  "size": [
    1.5,
    1.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7519,
        0.1166,
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.7519,
        0.1166,
        0.502
      ]
    ],
    [
      0.875,
      [
        1.0,
        0.7519,
        0.1166,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7519,
        0.1166,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/smoke_01"
  },
  "sheet": {
    "columns": 4,
    "rows": 4,
    "frames": 16,
    "mode": {
      "kind": "life",
      "cycles": 0.8333
    }
  },
  "blend": "additive"
});
