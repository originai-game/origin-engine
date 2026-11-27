import { file } from '@origingame/engine/author';
// Effekseer node 2 "Node"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      0.001,
      1.0
    ]
  },
  "spread": 25.2394,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.6667,
    1.0
  ],
  "speed": [
    3.9799,
    3.9799
  ],
  "size": [
    0.3,
    0.3
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.2195,
        1.0,
        0.7843
      ]
    ],
    [
      0.1094,
      [
        0.0,
        0.1501,
        1.0,
        0.5355
      ]
    ],
    [
      0.2188,
      [
        0.0,
        0.1198,
        1.0,
        0.4068
      ]
    ],
    [
      0.3438,
      [
        0.0,
        0.1105,
        1.0,
        0.3636
      ]
    ],
    [
      0.625,
      [
        0.0,
        0.1227,
        1.0,
        0.4197
      ]
    ],
    [
      0.75,
      [
        0.0,
        0.1176,
        1.0,
        0.397
      ]
    ],
    [
      0.875,
      [
        0.0,
        0.0923,
        1.0,
        0.2738
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0478,
        1.0,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -10.8,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Foam003"
  },
  "blend": "additive"
});
