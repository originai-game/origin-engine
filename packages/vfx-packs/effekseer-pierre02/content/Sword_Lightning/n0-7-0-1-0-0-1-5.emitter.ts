import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1.5 "Burst1"
export default file.particles({
  "maxParticles": 42,
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
      "interval": 0.0917
    }
  ],
  "duration": 0.2,
  "looping": false,
  "lifetime": [
    0.2,
    0.2
  ],
  "speed": [
    21.0,
    21.0
  ],
  "size": [
    29.0985,
    29.0985
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5774
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
        0.7445,
        0.7818,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.7445,
        0.7818,
        1.0,
        0.0523
      ]
    ],
    [
      1.0,
      [
        0.7445,
        0.7818,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Burst_2"
  },
  "blend": "additive"
});
