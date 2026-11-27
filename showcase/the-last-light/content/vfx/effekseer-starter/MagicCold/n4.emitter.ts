import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
      "cycles": 30,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.21950000524520874,
        1,
        1,
        0.7843000292778015
      ]
    ],
    [
      0.25,
      [
        0.35740000009536743,
        1,
        1,
        0.5881999731063843
      ]
    ],
    [
      0.5,
      [
        0.5325999855995178,
        1,
        1,
        0.3921999931335449
      ]
    ],
    [
      0.75,
      [
        0.746399998664856,
        1,
        1,
        0.19609999656677246
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "duration": 0.5,
  "heightOverLife": [
    [
      0,
      0.33329999446868896
    ],
    [
      1,
      0.666700005531311
    ]
  ],
  "lifetime": [
    0.666700005531311,
    0.666700005531311
  ],
  "looping": false,
  "maxParticles": 38,
  "randomColorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.7843000292778015
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2,
      0.0010000000474974513,
      2
    ]
  },
  "size": [
    3,
    3
  ],
  "sizeOverLife": [
    [
      0,
      0.33329999446868896
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    2.121299982070923,
    8.485300064086914
  ],
  "spread": 180,
  "texture": ref("base/vfx/effekseer-starter/textures/Fog3001")
});
