import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "BaseLight"
export default file.particles({
  "maxParticles": 5,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.6667,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    4.0,
    4.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    28.0,
    28.0
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
        0.2195,
        0.0055,
        0.0104,
        0.0
      ]
    ],
    [
      0.0625,
      [
        0.2195,
        0.0055,
        0.0104,
        1.0
      ]
    ],
    [
      0.8125,
      [
        0.2195,
        0.0055,
        0.0104,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0055,
        0.0104,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "alpha"
});
