import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "Burst"
export default file.particles({
  "maxParticles": 70,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8,
      "cycles": 7,
      "interval": 0.05
    }
  ],
  "duration": 0.3167,
  "looping": false,
  "lifetime": [
    0.1333,
    0.5333
  ],
  "speed": [
    3.0,
    3.0
  ],
  "size": [
    10.74,
    10.74
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7449
    ],
    [
      0.5,
      0.9004
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7519,
        0.5356,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.7519,
        0.5356,
        0.0471
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.7519,
        0.5356,
        0.0627
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7519,
        0.5356,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    0.0,
    3.6
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Burst_2"
  },
  "blend": "additive"
});
