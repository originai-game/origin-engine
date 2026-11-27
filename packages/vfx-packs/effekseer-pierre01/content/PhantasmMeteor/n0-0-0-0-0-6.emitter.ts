import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.6 "Blur"
export default file.particles({
  "maxParticles": 230,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 23,
      "interval": 0.0667
    }
  ],
  "duration": 1.4833,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    20.0,
    20.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.5
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
      0.0667,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      0.6667,
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
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/ExperimentalBlur"
  },
  "blend": "alpha",
  "distortion": 0.02
});
