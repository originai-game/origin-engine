import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "粒の子ノード"
export default file.particles({
  "maxParticles": 2633,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.15,
      0.15
    ]
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.9482,
    3.7928
  ],
  "size": [
    1.0,
    1.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.4219,
      0.9625
    ],
    [
      0.5312,
      0.9156
    ],
    [
      0.625,
      0.8395
    ],
    [
      0.7344,
      0.6949
    ],
    [
      0.8281,
      0.5118
    ],
    [
      0.9219,
      0.2636
    ],
    [
      1.0,
      0.0
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
        0.1011,
        0.7969,
        0.8355,
        0.1961
      ]
    ],
    [
      0.4219,
      [
        0.1011,
        0.7969,
        0.8355,
        0.1887
      ]
    ],
    [
      0.625,
      [
        0.1011,
        0.7969,
        0.8355,
        0.1646
      ]
    ],
    [
      0.8281,
      [
        0.1011,
        0.7969,
        0.8355,
        0.1004
      ]
    ],
    [
      1.0,
      [
        0.1011,
        0.7969,
        0.8355,
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
