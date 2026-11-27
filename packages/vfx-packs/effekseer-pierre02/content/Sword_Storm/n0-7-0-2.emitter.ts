import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.2 "Blur"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 25,
      "interval": 0.0917
    }
  ],
  "duration": 2.2167,
  "looping": false,
  "lifetime": [
    0.6,
    0.6
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    30.0,
    30.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.75
    ],
    [
      0.6719,
      0.9291
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
      0.0833,
      [
        1.0,
        1.0,
        1.0,
        0.3765
      ]
    ],
    [
      0.5833,
      [
        1.0,
        1.0,
        1.0,
        0.3765
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
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalBlur"
  },
  "blend": "alpha",
  "distortion": 0.02
});
