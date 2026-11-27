import { file } from '@origingame/engine/author';
// Effekseer node 0.7.2.1 "Burst1"
export default file.particles({
  "maxParticles": 30,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 360.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.2,
    0.2
  ],
  "speed": [
    27.0,
    27.0
  ],
  "size": [
    37.4123,
    37.4123
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5774
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
        0.7445,
        0.8122,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.7445,
        0.8122,
        1.0,
        0.0523
      ]
    ],
    [
      1.0,
      [
        0.7445,
        0.8122,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Burst_2"
  },
  "blend": "additive"
});
