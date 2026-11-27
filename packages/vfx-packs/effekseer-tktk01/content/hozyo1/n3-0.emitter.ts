import { file } from '@origingame/engine/author';
// Effekseer node 3.0 "気の上昇"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.2083,
  "looping": false,
  "lifetime": [
    0.3333,
    0.6667
  ],
  "speed": [
    1.0,
    11.0
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
        0.5924,
        0.1275,
        0.0784
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.4409,
        0.1275,
        0.0588
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.3135,
        0.1275,
        0.0392
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.2093,
        0.1275,
        0.0196
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1275,
        0.1275,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
