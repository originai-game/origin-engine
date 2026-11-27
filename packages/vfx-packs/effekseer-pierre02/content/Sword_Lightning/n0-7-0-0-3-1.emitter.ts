import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.3.1 "Blur"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5333,
    0.5333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    160.0,
    160.0
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
      0.0938,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.5312,
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
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalBlur"
  },
  "blend": "alpha",
  "distortion": 0.02
});
