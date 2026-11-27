import { file } from '@origingame/engine/author';
// Effekseer node 0.8.0.0.0.6 "Blur"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.6589,
      0.5271
    ]
  },
  "spread": 32.9053,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.4167,
    1.0833
  ],
  "speed": [
    3.5797,
    12.6562
  ],
  "speedOverLife": [
    [
      0.0,
      2.3953
    ],
    [
      0.2031,
      1.6178
    ],
    [
      0.3906,
      1.0751
    ],
    [
      0.625,
      0.6548
    ],
    [
      0.875,
      0.5225
    ],
    [
      0.8906,
      0.2038
    ],
    [
      0.9062,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    25.3125,
    25.3125
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6
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
      0.1111,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      0.2812,
      [
        1.0,
        1.0,
        1.0,
        0.2568
      ]
    ],
    [
      0.4444,
      [
        1.0,
        1.0,
        1.0,
        0.3137
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalBlur"
  },
  "blend": "alpha",
  "distortion": 0.02
});
