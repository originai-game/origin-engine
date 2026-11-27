import { file } from '@origingame/engine/author';
// Effekseer node 1 "LaserHead"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "point"
  },
  "spread": 54.7356,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0667
    }
  ],
  "duration": 1.2833,
  "looping": false,
  "lifetime": [
    3.0,
    3.0
  ],
  "speed": [
    103.923,
    103.923
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.8434,
        1.0,
        1.0
      ]
    ],
    [
      0.8333,
      [
        1.0,
        0.8434,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8434,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle01"
  },
  "blend": "additive"
});
