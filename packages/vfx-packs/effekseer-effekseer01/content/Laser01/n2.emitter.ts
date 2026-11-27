import { file } from '@origingame/engine/author';
// Effekseer node 2 "LaserCore"
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
      "count": 1,
      "cycles": 4,
      "interval": 0.1667
    }
  ],
  "duration": 0.5167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    11.0,
    11.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0909
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        1.0,
        0.2195,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0478,
        1.0,
        0.2195,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle02"
  },
  "blend": "additive"
});
