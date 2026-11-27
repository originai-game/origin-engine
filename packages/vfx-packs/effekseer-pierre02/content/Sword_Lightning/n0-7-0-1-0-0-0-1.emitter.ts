import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.0.1 "ExperimentalBlur"
export default file.particles({
  "maxParticles": 47,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 12.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.1167,
    0.2833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    80.0,
    80.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8
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
      0.25,
      [
        1.0,
        1.0,
        1.0,
        0.1882
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalBlur"
  },
  "blend": "alpha",
  "distortion": 0.025
});
