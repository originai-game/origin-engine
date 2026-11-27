import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0 "Node"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
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
    30.0,
    30.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3333
    ],
    [
      1.0,
      1.0
    ]
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
        0.0,
        0.0,
        0.1961
      ]
    ],
    [
      0.2656,
      [
        1.0,
        0.0119,
        0.0026,
        0.144
      ]
    ],
    [
      0.5156,
      [
        1.0,
        0.0511,
        0.0111,
        0.095
      ]
    ],
    [
      0.7656,
      [
        1.0,
        0.122,
        0.0265,
        0.046
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
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
