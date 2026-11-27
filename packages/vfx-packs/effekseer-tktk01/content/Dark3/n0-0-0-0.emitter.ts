import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "glow"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.375,
    1.125
  ],
  "speedOverLife": [
    [
      0.0,
      2.4833
    ],
    [
      0.2031,
      1.6514
    ],
    [
      0.3906,
      1.0794
    ],
    [
      0.6094,
      0.6707
    ],
    [
      0.8438,
      0.5418
    ],
    [
      0.8594,
      0.1941
    ],
    [
      0.875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.3125,
    0.4375
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.1166,
        0.0,
        0.0104,
        1.0
      ]
    ],
    [
      0.5156,
      [
        0.1166,
        0.0,
        0.0494,
        0.4844
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.0,
        0.1166,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3587,
        0.0,
        0.1166,
        1.0
      ]
    ],
    [
      0.5,
      [
        0.3587,
        0.0,
        0.2195,
        0.5
      ]
    ],
    [
      1.0,
      [
        0.3587,
        0.0,
        0.3587,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
