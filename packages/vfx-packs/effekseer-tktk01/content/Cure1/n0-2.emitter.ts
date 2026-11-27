import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "光メイン右下"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    2.8284,
    2.8284
  ],
  "size": [
    0.75,
    0.75
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
        0.8751,
        0.9158,
        0.2468,
        0.7843
      ]
    ],
    [
      0.25,
      [
        0.9055,
        0.9364,
        0.3838,
        0.6373
      ]
    ],
    [
      0.5,
      [
        0.9364,
        0.9574,
        0.5542,
        0.4902
      ]
    ],
    [
      0.75,
      [
        0.9679,
        0.9786,
        0.7593,
        0.3431
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.1961
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
