import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0 "リボンから出るキラキラ"
export default file.particles({
  "maxParticles": 32,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 25,
      "interval": 0.0333
    }
  ],
  "duration": 0.8167,
  "looping": false,
  "lifetime": [
    0.8333,
    1.1667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.4,
    0.4
  ],
  "heightOverLife": [
    [
      0.0,
      1.5
    ],
    [
      1.0,
      1.5
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2468,
        0.7205,
        0.2468,
        1.0
      ]
    ],
    [
      0.1562,
      [
        0.3154,
        0.7548,
        0.3154,
        0.8673
      ]
    ],
    [
      0.5938,
      [
        0.5029,
        0.8346,
        0.5029,
        0.5702
      ]
    ],
    [
      0.7344,
      [
        0.6084,
        0.8738,
        0.6084,
        0.4296
      ]
    ],
    [
      0.8667,
      [
        0.7611,
        0.926,
        0.7611,
        0.248
      ]
    ],
    [
      0.9062,
      [
        0.8213,
        0.9455,
        0.8213,
        0.1279
      ]
    ],
    [
      0.9375,
      [
        0.8748,
        0.9622,
        0.8748,
        0.0588
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
  "randomColorOverLife": [
    [
      0.0,
      [
        0.4985,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.5938,
      [
        0.691,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.7344,
      [
        0.7614,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.8667,
      [
        0.858,
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
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Particle02"
  },
  "blend": "additive"
});
