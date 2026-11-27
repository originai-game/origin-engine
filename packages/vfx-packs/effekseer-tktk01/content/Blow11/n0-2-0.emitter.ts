import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0 "glow"
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
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.0,
    6.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0478,
        0.0
      ]
    ],
    [
      0.0667,
      [
        1.0,
        0.0478,
        0.0478,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0478,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
