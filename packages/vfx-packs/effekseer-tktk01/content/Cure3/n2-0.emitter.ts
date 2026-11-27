import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "ほし"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.1667,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.5,
    0.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.1562,
      0.9252
    ],
    [
      0.3281,
      0.7803
    ],
    [
      0.4688,
      0.6281
    ],
    [
      0.8281,
      0.1898
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0278,
        0.7969,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0278,
        0.7969,
        1.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3112,
        0.7969,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.3112,
        0.7969,
        1.0,
        1.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -720.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
