import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.1.0 "LineMain"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.875,
    1.875
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.2195,
        0.0
      ]
    ],
    [
      0.1429,
      [
        1.0,
        0.2195,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.2195,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5295,
        0.5295,
        0.0
      ]
    ],
    [
      0.1429,
      [
        1.0,
        0.5295,
        0.5295,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5295,
        0.5295,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
