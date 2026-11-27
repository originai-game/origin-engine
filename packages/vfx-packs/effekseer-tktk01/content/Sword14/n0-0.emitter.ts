import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "ナイフ"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 8,
      "interval": 0.0667
    }
  ],
  "duration": 0.4833,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    30.0,
    30.0
  ],
  "speedOverLife": [
    [
      0.0,
      3.6895
    ],
    [
      0.25,
      1.5124
    ],
    [
      0.375,
      0.7867
    ],
    [
      0.5,
      0.3172
    ],
    [
      0.625,
      0.1037
    ],
    [
      0.75,
      0.1464
    ],
    [
      0.875,
      0.4452
    ],
    [
      1.0,
      0.9788
    ]
  ],
  "size": [
    2.0,
    2.0
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
        1.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
    -90.0,
    -90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/naihu"
  },
  "blend": "additive"
});
