import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.1 "glow大"
export default file.particles({
  "maxParticles": 938,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      0.75,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 6000.0,
  "duration": 0.125,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.45,
    2.85
  ],
  "speedOverLife": [
    [
      0.0,
      2.5537
    ],
    [
      0.2031,
      1.6773
    ],
    [
      0.3906,
      1.0819
    ],
    [
      0.5938,
      0.6882
    ],
    [
      0.8125,
      0.5564
    ],
    [
      0.8281,
      0.4652
    ],
    [
      0.8438,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    2.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.5,
      1.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.1961
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.1961
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
