import { file } from '@origingame/engine/author';
// Effekseer node 0.6.1.2.1 "Burst1"
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
    120.0,
    120.0
  ],
  "size": [
    166.2769,
    166.2769
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
        0.0523
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Burst_2"
  },
  "blend": "additive"
});
