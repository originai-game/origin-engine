import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.1.0.0 "glow"
export default file.particles({
  "maxParticles": 94,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.125,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.0833,
    1.0833
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
      1.0,
      [
        0.1166,
        0.0,
        0.0104,
        1.0
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
      1.0,
      [
        0.3587,
        0.0,
        0.1166,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
