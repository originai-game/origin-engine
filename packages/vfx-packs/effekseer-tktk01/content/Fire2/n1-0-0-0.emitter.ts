import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0 "湯気単体"
export default file.particles({
  "maxParticles": 4096,
  "shape": {
    "kind": "point"
  },
  "spread": 70.5288,
  "rate": 240.0,
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.25,
    0.75
  ],
  "speed": [
    3.0,
    11.3578
  ],
  "size": [
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
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
      1.0,
      0.0
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
      0.0312,
      [
        1.0,
        1.0,
        1.0,
        0.0625
      ]
    ],
    [
      0.0469,
      [
        1.0,
        1.0,
        1.0,
        0.0645
      ]
    ],
    [
      0.2969,
      [
        1.0,
        1.0,
        1.0,
        0.024
      ]
    ],
    [
      0.6667,
      [
        1.0,
        1.0,
        1.0,
        0.0133
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
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Fog3002"
  },
  "blend": "additive"
});
