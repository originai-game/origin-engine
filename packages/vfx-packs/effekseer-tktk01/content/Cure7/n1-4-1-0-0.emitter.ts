import { file } from '@origingame/engine/author';
// Effekseer node 1.4.1.0.0 "星グロー"
export default file.particles({
  "maxParticles": 32,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.1042,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    1.875
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
    0.7812,
    0.7812
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.25,
      1.0
    ],
    [
      0.5,
      0.0
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
        1.0,
        1.0,
        0.3112,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        0.4368,
        0.1961
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        0.586,
        0.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.586,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
