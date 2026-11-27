import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.6 "柱glow"
export default file.particles({
  "maxParticles": 5,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5.0,
      1.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 6,
      "interval": 0.1167
    }
  ],
  "duration": 0.6,
  "looping": false,
  "lifetime": [
    0.2667,
    0.2667
  ],
  "speed": [
    22.9412,
    30.0
  ],
  "speedOverLife": [
    [
      0.0,
      0.7442
    ],
    [
      0.25,
      0.6243
    ],
    [
      0.5,
      0.8001
    ],
    [
      0.75,
      1.2758
    ],
    [
      1.0,
      2.0369
    ]
  ],
  "size": [
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0938,
      0.8139
    ],
    [
      0.2031,
      0.6317
    ],
    [
      0.3125,
      0.4826
    ],
    [
      0.4219,
      0.3618
    ],
    [
      0.5312,
      0.2645
    ],
    [
      0.6562,
      0.176
    ],
    [
      1.0,
      0.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.0938,
      1.3376
    ],
    [
      0.2031,
      2.1573
    ],
    [
      0.3125,
      2.8282
    ],
    [
      0.4219,
      3.3718
    ],
    [
      0.5312,
      3.8095
    ],
    [
      0.6562,
      4.208
    ],
    [
      1.0,
      5.0
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      -1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-tktk01/Light4/n0-0-0-0-6.effect"
  }
});
