import { file } from '@origingame/engine/author';
// Effekseer node 1.1.3.0.0 "glow"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 250,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.375,
    4.125
  ],
  "speedOverLife": [
    [
      0.0,
      7.033
    ],
    [
      0.0625,
      5.5907
    ],
    [
      0.0938,
      4.9562
    ],
    [
      0.1406,
      4.1807
    ],
    [
      0.1875,
      3.6167
    ],
    [
      0.2031,
      1.0564
    ],
    [
      0.2188,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.1166,
        0.0,
        0.0104,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.0,
        0.0104,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3587,
        0.0,
        0.1166,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.3587,
        0.0,
        0.1166,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
