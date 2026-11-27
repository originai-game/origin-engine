import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.0 "Hit"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.0,
      3.0,
      3.0
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.05
    }
  ],
  "duration": 0.9667,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    21.0,
    21.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7143
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.0143
    ],
    [
      1.0,
      0.0048
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle01"
  },
  "blend": "additive"
});
