import { file } from '@origingame/engine/author';
// Effekseer node 0.6.0.0.7.0.0 "粒子glow"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 7200.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.4,
    0.4
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0,
        1.0,
        0.502
      ]
    ],
    [
      0.5833,
      [
        0.2195,
        0.0,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
