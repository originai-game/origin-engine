import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.0 "Fire"
export default file.particles({
  "maxParticles": 233,
  "shape": {
    "kind": "point"
  },
  "spread": 0.4302,
  "rate": 60.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    8.8363,
    10.4307
  ],
  "speedOverLife": [
    [
      0.0,
      1.7075
    ],
    [
      0.4062,
      1.1725
    ],
    [
      0.8125,
      0.8508
    ],
    [
      0.8281,
      0.7029
    ],
    [
      0.8438,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.3333,
    1.3333
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.083,
      0.0,
      0.9965
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        0.7529
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Fire"
  },
  "sheet": {
    "columns": 2,
    "rows": 2,
    "frames": 4,
    "mode": {
      "kind": "off"
    }
  },
  "blend": "additive"
});
