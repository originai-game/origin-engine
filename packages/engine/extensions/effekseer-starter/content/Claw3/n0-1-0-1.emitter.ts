import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 5,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.16670000553131104,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.33329999446868896,
      [
        1,
        1,
        1,
        1
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
  "distortion": 0.004000000189989805,
  "duration": 0.016699999570846558,
  "lifetime": [
    0.5,
    0.5
  ],
  "looping": false,
  "maxParticles": 13,
  "rate": 0,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ]
    ]
  },
  "size": [
    10,
    10
  ],
  "sizeOverLife": [
    [
      0,
      0.10000000149011612
    ],
    [
      0.10939999669790268,
      0.2948000133037567
    ],
    [
      0.21879999339580536,
      0.45899999141693115
    ],
    [
      0.3280999958515167,
      0.5954999923706055
    ],
    [
      0.4530999958515167,
      0.7214000225067139
    ],
    [
      0.5781000256538391,
      0.8191999793052673
    ],
    [
      0.7031000256538391,
      0.8932999968528748
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Distortion6")
});
