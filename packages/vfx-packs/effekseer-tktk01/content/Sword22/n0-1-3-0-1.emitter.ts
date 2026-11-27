import { file } from '@origingame/engine/author';
// Effekseer node 0.1.3.0.1 "glow大"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    8.1,
    9.9
  ],
  "size": [
    5.0,
    5.0
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
        0.0,
        0.0,
        0.0,
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.1961
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.1961
      ]
    ]
  ],
  "gravity": [
    0.0,
    0.0,
    -14.4
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
