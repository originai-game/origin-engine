import { file } from '@origingame/engine/author';
// Effekseer node 1.2.0.1.2 "Glow"
export default file.particles({
  "maxParticles": 39,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      0.001,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 40,
      "interval": 0.0167
    }
  ],
  "duration": 0.6667,
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
      0.1719,
      [
        1.0,
        0.4704,
        0.1801,
        0.3804
      ]
    ],
    [
      0.3281,
      [
        1.0,
        0.4151,
        0.148,
        0.498
      ]
    ],
    [
      0.6719,
      [
        1.0,
        0.3066,
        0.0892,
        0.1235
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
