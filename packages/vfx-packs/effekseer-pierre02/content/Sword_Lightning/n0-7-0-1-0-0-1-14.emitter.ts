import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1.14 "Blur"
export default file.particles({
  "maxParticles": 83,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 11,
      "cycles": 2,
      "interval": 0.1
    }
  ],
  "duration": 0.1167,
  "looping": false,
  "lifetime": [
    0.4667,
    0.4667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    75.6,
    75.6
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7407
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
      0.2143,
      [
        1.0,
        1.0,
        1.0,
        0.1255
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
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalBlur"
  },
  "blend": "alpha",
  "distortion": 0.01
});
