import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0 "粒収縮"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.2,
    0.3667
  ],
  "speed": [
    3.5294,
    14.1176
  ],
  "speedOverLife": [
    [
      0.0,
      1.4215
    ],
    [
      0.25,
      1.0803
    ],
    [
      0.5,
      0.893
    ],
    [
      0.75,
      0.8662
    ],
    [
      1.0,
      0.9967
    ]
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0278,
        0.0,
        0.0278,
        1.0
      ]
    ],
    [
      0.1176,
      [
        0.0278,
        0.0,
        0.0278,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0278,
        0.0,
        0.0278,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3112,
        0.0,
        0.3112,
        1.0
      ]
    ],
    [
      0.1176,
      [
        0.3112,
        0.0,
        0.3112,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.3112,
        0.0,
        0.3112,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
