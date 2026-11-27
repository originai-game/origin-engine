import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1 "Distortion"
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 14,
      "interval": 0.0333
    }
  ],
  "duration": 0.45,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    7.0,
    7.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3571
    ],
    [
      0.1094,
      0.4963
    ],
    [
      0.2188,
      0.6136
    ],
    [
      0.3281,
      0.7111
    ],
    [
      0.4531,
      0.801
    ],
    [
      0.7031,
      0.9238
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
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion6"
  },
  "blend": "additive",
  "distortion": 0.004
});
