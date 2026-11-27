import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "炎メイン"
export default file.particles({
  "maxParticles": 1250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.3282,
      2.2188,
      0.001
    ]
  },
  "spread": 11.3099,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1000
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.8485,
    1.3416
  ],
  "size": [
    1.0,
    1.0
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
        0.0478,
        0.0104,
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0111,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.1961
      ]
    ],
    [
      0.2656,
      [
        1.0,
        0.122,
        0.0262,
        0.144
      ]
    ],
    [
      0.5156,
      [
        1.0,
        0.2271,
        0.0486,
        0.095
      ]
    ],
    [
      0.7656,
      [
        1.0,
        0.3686,
        0.0786,
        0.046
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1139,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "angularVelocity": [
    -600.0,
    600.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
