import { file } from '@origingame/engine/author';
// Effekseer node 0.5.0.1 "Burst1"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.2,
    0.2
  ],
  "speed": [
    12.0,
    12.0
  ],
  "size": [
    16.6277,
    16.6277
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
        0.2195,
        1.0,
        0.5356,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.2195,
        1.0,
        0.5356,
        0.0523
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Burst_2"
  },
  "blend": "additive"
});
