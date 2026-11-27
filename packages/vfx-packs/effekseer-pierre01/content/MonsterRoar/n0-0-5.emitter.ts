import { file } from '@origingame/engine/author';
// Effekseer node 0.0.5 "Blur"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.5
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 18,
      "interval": 0.0833
    }
  ],
  "duration": 1.4333,
  "looping": false,
  "lifetime": [
    0.3333,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.0,
    6.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6667
    ],
    [
      0.2031,
      0.7894
    ],
    [
      0.4219,
      0.8794
    ],
    [
      0.6562,
      0.9413
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
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.1255
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
  "rotation": [
    -3.0,
    3.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/ExperimentalBlur"
  },
  "blend": "alpha",
  "distortion": 0.01
});
