import { file } from '@origingame/engine/author';
// Effekseer node 0 "Fire"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.3,
      0.3,
      0.3
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 15,
      "interval": 0.0167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.2,
    0.8
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0478,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.0478,
        0.0478,
        0.251
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.0478,
        0.0478,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0478,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.2195,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        0.2195,
        0.502
      ]
    ],
    [
      0.6667,
      [
        1.0,
        1.0,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Fire3"
  },
  "sheet": {
    "columns": 5,
    "rows": 5,
    "frames": 15,
    "mode": {
      "kind": "fps",
      "fps": 30.0
    }
  },
  "blend": "additive"
});
