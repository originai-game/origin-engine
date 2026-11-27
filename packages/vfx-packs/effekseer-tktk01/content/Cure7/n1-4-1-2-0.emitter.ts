import { file } from '@origingame/engine/author';
// Effekseer node 1.4.1.2.0 "粒子グロー"
export default file.particles({
  "maxParticles": 32,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 240.0,
  "duration": 0.1042,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.05,
    0.35
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
    0.125,
    0.125
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.1275,
        0.5882
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.1275,
        0.5882
      ]
    ]
  ],
  "rotation": [
    45.0,
    45.0
  ],
  "angularVelocity": [
    308.5714,
    617.1428
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
