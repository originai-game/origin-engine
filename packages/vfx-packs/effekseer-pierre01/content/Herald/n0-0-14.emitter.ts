import { file } from '@origingame/engine/author';
// Effekseer node 0.0.14 "DistortionEye"
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
    1.5,
    1.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.0752,
    4.0752
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2454
    ],
    [
      0.125,
      0.4944
    ],
    [
      0.25,
      0.6937
    ],
    [
      0.375,
      0.8433
    ],
    [
      0.5,
      0.9433
    ],
    [
      0.625,
      0.9935
    ],
    [
      0.75,
      0.9941
    ],
    [
      1.0,
      0.8461
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
      0.1333,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.8222,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
    -300.0,
    300.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Halo"
  },
  "blend": "alpha",
  "distortion": 0.006
});
