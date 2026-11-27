import { file } from '@origingame/engine/author';
// Effekseer node 4 "Node"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      0.001,
      2.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 30,
      "interval": 0.0167
    }
  ],
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    2.1213,
    8.4853
  ],
  "size": [
    3.0,
    3.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      1.0,
      0.6667
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        1.0,
        0.7843
      ]
    ],
    [
      0.25,
      [
        0.3574,
        1.0,
        1.0,
        0.5882
      ]
    ],
    [
      0.5,
      [
        0.5326,
        1.0,
        1.0,
        0.3922
      ]
    ],
    [
      0.75,
      [
        0.7464,
        1.0,
        1.0,
        0.1961
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
        1.0,
        1.0,
        1.0,
        0.7843
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
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Fog3001"
  },
  "blend": "additive"
});
