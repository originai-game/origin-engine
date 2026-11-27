import { file } from '@origingame/engine/author';
// Effekseer node 0.1.11 "HeatHaze"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      0.25,
      0.25
    ]
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
    3.0,
    3.0
  ],
  "speed": [
    0.6,
    0.6
  ],
  "size": [
    16.0,
    16.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.75
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
      0.0833,
      [
        1.0,
        1.0,
        1.0,
        0.3137
      ]
    ],
    [
      0.5,
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
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/ExperimentalNoise"
  },
  "blend": "alpha",
  "distortion": 0.01
});
