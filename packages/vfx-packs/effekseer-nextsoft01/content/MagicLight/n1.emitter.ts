import { file } from '@origingame/engine/author';
// Effekseer node 1 "ya2"
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.4165,
      0.163,
      0.001
    ]
  },
  "spread": 3.9169,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 40,
      "interval": 0.0167
    }
  ],
  "duration": 0.6667,
  "looping": false,
  "lifetime": [
    0.1667,
    0.1667
  ],
  "speed": [
    63.6254,
    69.1564
  ],
  "size": [
    1.0,
    1.0
  ],
  "heightOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.25,
      1.5438
    ],
    [
      0.5,
      2.15
    ],
    [
      0.75,
      2.8187
    ],
    [
      1.0,
      3.55
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.7042,
      0.064,
      0.7071
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        1.0,
        0.1569
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.1569
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/SaintArrow"
  },
  "blend": "additive"
});
