import { file } from '@origingame/engine/author';
// Effekseer node 0.11.6 "Blur"
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
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    90.0,
    90.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
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
      0.0625,
      [
        1.0,
        1.0,
        1.0,
        0.4688
      ]
    ],
    [
      0.125,
      [
        1.0,
        1.0,
        1.0,
        0.875
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
