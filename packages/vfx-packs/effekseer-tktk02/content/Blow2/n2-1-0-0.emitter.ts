import { file } from '@origingame/engine/author';
// Effekseer node 2.1.0.0 "Glow"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.25,
    2.25
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.1094,
      [
        1.0,
        0.4733,
        0.1818,
        0.3706
      ]
    ],
    [
      0.2188,
      [
        1.0,
        0.4151,
        0.148,
        0.498
      ]
    ],
    [
      0.4531,
      [
        1.0,
        0.3044,
        0.088,
        0.1206
      ]
    ],
    [
      0.6719,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
