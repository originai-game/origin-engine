import { file } from '@origingame/engine/author';
// Effekseer node 0.2.1 "Node"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        -0.0382,
        0.4966,
        5.6782
      ],
      [
        -0.6415,
        4.319,
        3.664
      ],
      [
        -0.869,
        5.6114,
        -0.4966
      ],
      [
        -0.5875,
        3.6167,
        -4.3662
      ],
      [
        0.0382,
        -0.4966,
        -5.6782
      ],
      [
        0.6415,
        -4.319,
        -3.664
      ],
      [
        0.869,
        -5.6114,
        0.4966
      ],
      [
        0.5875,
        -3.6167,
        4.3662
      ]
    ]
  },
  "spread": 20.8442,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 7,
      "interval": 0.8333
    }
  ],
  "duration": 5.0167,
  "looping": false,
  "lifetime": [
    5.4667,
    5.4667
  ],
  "speed": [
    0.5546,
    1.1505
  ],
  "size": [
    0.8,
    0.8
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
      0.1119,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.8985,
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-hato01/textures/particle2"
  },
  "blend": "additive"
});
