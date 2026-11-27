import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.5 "Burst1"
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
    0.2,
    0.2
  ],
  "speed": [
    30.0,
    30.0
  ],
  "size": [
    41.5692,
    41.5692
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
        0.9574,
        0.7519,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.9574,
        0.7519,
        1.0,
        0.0523
      ]
    ],
    [
      1.0,
      [
        0.9574,
        0.7519,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Burst_2"
  },
  "blend": "additive"
});
