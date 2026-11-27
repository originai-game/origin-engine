import { file } from '@origingame/engine/author';
// Effekseer node 0 "Core"
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
    1.6667,
    1.6667
  ],
  "speed": [
    60.0,
    60.0
  ],
  "size": [
    4.0,
    4.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.2195,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        1.0,
        1.0
      ]
    ]
  ],
  "angularVelocity": [
    1200.0,
    1200.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle01"
  },
  "blend": "additive"
});
