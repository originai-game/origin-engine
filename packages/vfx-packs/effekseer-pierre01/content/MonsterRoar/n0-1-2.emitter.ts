import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2 "Blur"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      1.0,
      0.001
    ]
  },
  "spread": 180.0,
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
    0.45,
    0.45
  ],
  "speed": [
    0.7857,
    3.1427
  ],
  "size": [
    32.0,
    32.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0938,
      0.8255
    ],
    [
      0.2031,
      0.6547
    ],
    [
      0.3125,
      0.515
    ],
    [
      0.4219,
      0.4017
    ],
    [
      0.6562,
      0.2275
    ],
    [
      1.0,
      0.0625
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
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        0.502
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
  "distortion": 0.02
});
