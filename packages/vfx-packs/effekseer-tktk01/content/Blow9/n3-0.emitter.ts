import { file } from '@origingame/engine/author';
// Effekseer node 3.0 "背景glow"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.1667,
    0.5
  ],
  "speed": [
    3.0,
    6.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1308
    ],
    [
      0.25,
      1.3814
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      0.5423
    ],
    [
      1.0,
      0.4657
    ]
  ],
  "size": [
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2
    ],
    [
      0.0938,
      0.3489
    ],
    [
      0.2031,
      0.4946
    ],
    [
      0.3125,
      0.6139
    ],
    [
      0.4219,
      0.7105
    ],
    [
      0.6562,
      0.8592
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
        0.1275,
        0.1275,
        0.1275,
        0.0784
      ]
    ],
    [
      1.0,
      [
        0.1275,
        0.1275,
        0.1275,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.4479,
        0.4479,
        0.4479,
        0.0784
      ]
    ],
    [
      1.0,
      [
        0.4479,
        0.4479,
        0.4479,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
