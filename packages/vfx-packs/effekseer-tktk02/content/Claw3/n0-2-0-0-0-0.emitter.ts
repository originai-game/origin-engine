import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0.0.0 "smoke"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    1.25,
    1.25
  ],
  "speed": [
    1.8,
    3.0
  ],
  "speedOverLife": [
    [
      0.0,
      3.1895
    ],
    [
      0.1562,
      2.2241
    ],
    [
      0.2812,
      1.5767
    ],
    [
      0.4688,
      0.8502
    ],
    [
      0.6562,
      0.4174
    ],
    [
      0.6719,
      0.0673
    ],
    [
      0.6875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.0,
    1.0
  ],
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
      0.2667,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.7333,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/fog1"
  },
  "blend": "additive"
});
