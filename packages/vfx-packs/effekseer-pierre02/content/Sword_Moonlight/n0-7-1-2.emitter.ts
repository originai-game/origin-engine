import { file } from '@origingame/engine/author';
// Effekseer node 0.7.1.2 "Blur"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 3,
      "interval": 0.1167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.6667,
    1.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    9.0,
    9.0
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalNoise"
  },
  "blend": "alpha",
  "distortion": 0.02
});
