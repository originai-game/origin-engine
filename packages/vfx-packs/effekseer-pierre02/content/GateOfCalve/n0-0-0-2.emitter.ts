import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2 "Wave_Distortion"
export default file.particles({
  "maxParticles": 450,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 2.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    1.3,
    1.7
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.6471,
    6.3529
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
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.3765
      ]
    ],
    [
      0.6667,
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
    60.0,
    120.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Wind"
  },
  "blend": "alpha",
  "distortion": 0.025
});
