import { file } from '@origingame/engine/author';
// Effekseer node 0.5.0.8 "Blur"
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
      "cycles": 8,
      "interval": 0.05
    }
  ],
  "duration": 0.3667,
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
    43.2,
    43.2
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
        0.251
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
