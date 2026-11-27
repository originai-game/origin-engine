import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.4.5 "Blur"
export default file.particles({
  "maxParticles": 1125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.75,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 25,
      "cycles": 36,
      "interval": 0.0167
    }
  ],
  "duration": 0.6,
  "looping": false,
  "lifetime": [
    0.5333,
    0.8333
  ],
  "speed": [
    11.25,
    18.75
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
      0.0732,
      [
        1.0,
        1.0,
        1.0,
        0.3765
      ]
    ],
    [
      0.6341,
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
