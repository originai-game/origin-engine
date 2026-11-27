import { file } from '@origingame/engine/author';
// Effekseer node 1.2 "Glow1"
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
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    20.0,
    20.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.2188,
      0.7703
    ],
    [
      0.4531,
      0.5642
    ],
    [
      0.7031,
      0.3811
    ],
    [
      1.0,
      0.2
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.125,
      [
        0.4443,
        0.0359,
        1.0,
        0.1255
      ]
    ],
    [
      0.25,
      [
        0.3762,
        0.0275,
        1.0,
        0.251
      ]
    ],
    [
      0.5,
      [
        0.2888,
        0.0175,
        1.0,
        0.251
      ]
    ],
    [
      0.7188,
      [
        0.2469,
        0.0131,
        1.0,
        0.1412
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0104,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle_Soft"
  },
  "blend": "additive"
});
