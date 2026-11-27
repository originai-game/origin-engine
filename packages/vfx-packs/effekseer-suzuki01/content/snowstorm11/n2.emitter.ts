import { file } from '@origingame/engine/author';
// Effekseer node 2 "particle"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      18.823,
      21.8282,
      43.2345
    ]
  },
  "spread": 30.9638,
  "rate": 120.0,
  "duration": 1.6667,
  "looping": false,
  "lifetime": [
    4.0,
    4.0
  ],
  "speed": [
    8.4853,
    18.0
  ],
  "size": [
    0.1,
    0.1
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle02"
  },
  "blend": "additive"
});
