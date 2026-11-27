import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0 "Particle"
export default file.particles({
  "maxParticles": 127,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 60.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    -0.4,
    0.6
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.4045,
        1.0,
        0.2508,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.4045,
        1.0,
        0.2508,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle02"
  },
  "blend": "additive"
});
