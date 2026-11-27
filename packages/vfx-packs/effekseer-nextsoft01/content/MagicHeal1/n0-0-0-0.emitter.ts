import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "リボンから出るキラキラ"
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
    0.48,
    0.72
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8333
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2468,
        1.0,
        0.4368,
        1.0
      ]
    ],
    [
      0.1562,
      [
        0.3154,
        1.0,
        0.4972,
        0.8673
      ]
    ],
    [
      0.5938,
      [
        0.5029,
        1.0,
        0.6482,
        0.5702
      ]
    ],
    [
      0.7344,
      [
        0.6084,
        1.0,
        0.7273,
        0.4296
      ]
    ],
    [
      0.8667,
      [
        0.7611,
        1.0,
        0.8368,
        0.248
      ]
    ],
    [
      0.9062,
      [
        0.8213,
        1.0,
        0.8788,
        0.1279
      ]
    ],
    [
      0.9375,
      [
        0.8748,
        1.0,
        0.9155,
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
        0.4902,
        1.0,
        0.8264,
        1.0
      ]
    ],
    [
      0.5938,
      [
        0.6853,
        1.0,
        0.8988,
        1.0
      ]
    ],
    [
      0.7344,
      [
        0.7569,
        1.0,
        0.9232,
        1.0
      ]
    ],
    [
      0.8667,
      [
        0.8552,
        1.0,
        0.9553,
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
