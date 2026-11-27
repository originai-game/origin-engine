import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2 "Wave_Distortion"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 2.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    3.3,
    3.7
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    22.5882,
    25.4118
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5667
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
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0714,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.8571,
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
    60.0,
    120.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Wind"
  },
  "blend": "alpha",
  "distortion": 0.025
});
