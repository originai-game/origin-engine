import { file } from '@origingame/engine/author';
// Effekseer node 0 "Node"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    5.25,
    5.25
  ],
  "speedOverLife": [
    [
      0.0,
      0.2582
    ],
    [
      0.125,
      0.0448
    ],
    [
      0.25,
      0.0091
    ],
    [
      0.375,
      0.1609
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      1.0269
    ],
    [
      0.75,
      1.7411
    ],
    [
      1.0,
      3.6952
    ]
  ],
  "size": [
    1.0,
    1.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
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
        0.2674,
        0.586,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.2674,
        0.586,
        1.0,
        1.0
      ]
    ],
    [
      0.95,
      [
        0.2674,
        0.586,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2674,
        0.586,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Photon"
  },
  "blend": "alpha"
});
