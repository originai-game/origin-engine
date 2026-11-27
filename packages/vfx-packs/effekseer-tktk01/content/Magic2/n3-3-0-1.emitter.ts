import { file } from '@origingame/engine/author';
// Effekseer node 3.3.0.1 "gliow大"
export default file.particles({
  "maxParticles": 313,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.375,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 250
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.3,
    3.15
  ],
  "speedOverLife": [
    [
      0.0,
      2.9755
    ],
    [
      0.1875,
      1.889
    ],
    [
      0.3594,
      1.179
    ],
    [
      0.5312,
      0.766
    ],
    [
      0.7031,
      0.6502
    ],
    [
      0.7188,
      0.1398
    ],
    [
      0.7344,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    2.4609,
    2.4609
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.5625,
      1.0
    ],
    [
      0.5781,
      1.0
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
        0.0023,
        0.0104,
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0023,
        0.0104,
        0.1961
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0254,
        0.1166,
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0254,
        0.1166,
        0.1961
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
