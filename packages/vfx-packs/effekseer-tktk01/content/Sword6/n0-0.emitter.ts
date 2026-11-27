import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "メイン斬撃"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 10
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.0,
    1.0
  ],
  "heightOverLife": [
    [
      0.0,
      6.0
    ],
    [
      1.0,
      6.0
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
        0.9075,
        0.2632,
        0.9406,
        0.1961
      ]
    ],
    [
      0.5,
      [
        0.9075,
        0.2632,
        0.9406,
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.9075,
        0.2632,
        0.9406,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
