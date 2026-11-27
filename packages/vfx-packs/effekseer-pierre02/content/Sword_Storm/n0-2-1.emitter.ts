import { file } from '@origingame/engine/author';
// Effekseer node 0.2.1 "Blur"
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 13,
      "interval": 0.0167
    }
  ],
  "duration": 0.2167,
  "looping": false,
  "lifetime": [
    0.4167,
    1.0833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    72.0,
    72.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6667
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
        0.0471
      ]
    ],
    [
      0.2812,
      [
        1.0,
        1.0,
        1.0,
        0.0963
      ]
    ],
    [
      0.4444,
      [
        1.0,
        1.0,
        1.0,
        0.1176
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalNoise"
  },
  "blend": "alpha",
  "distortion": 0.02
});
