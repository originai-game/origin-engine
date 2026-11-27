import { file } from '@origingame/engine/author';
// Effekseer node 9.0 "Splash"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "sphere",
    "radius": 0.0
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 30,
      "interval": 0.0167
    }
  ],
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.5,
    2.5
  ],
  "heightOverLife": [
    [
      0.0,
      0.4167
    ],
    [
      1.0,
      0.4167
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
        1.0,
        0.0478,
        0.0104,
        1.0
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.0478,
        0.0104,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.1166,
        1.0
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.5356,
        0.1166,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line2"
  },
  "blend": "additive"
});
