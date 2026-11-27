import { file } from '@origingame/engine/author';
// Effekseer node 0.4.1 "Blur"
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
    0.5,
    1.1667
  ],
  "speed": [
    48.0,
    48.0
  ],
  "size": [
    96.0,
    96.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.75
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
        0.9914,
        0.0
      ]
    ],
    [
      0.2,
      [
        1.0,
        1.0,
        0.9914,
        0.1882
      ]
    ],
    [
      0.2969,
      [
        1.0,
        1.0,
        0.9914,
        0.2456
      ]
    ],
    [
      0.4,
      [
        1.0,
        1.0,
        0.9914,
        0.2824
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.9914,
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
