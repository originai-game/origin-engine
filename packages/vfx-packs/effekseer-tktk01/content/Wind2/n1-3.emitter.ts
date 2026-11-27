import { file } from '@origingame/engine/author';
// Effekseer node 1.3 "衝撃グロー拡大"
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
    30.0,
    30.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0333
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.0333
    ],
    [
      1.0,
      0.6667
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        0.2195,
        0.5882
      ]
    ],
    [
      0.5,
      [
        0.1166,
        1.0,
        0.1166,
        0.2941
      ]
    ],
    [
      1.0,
      [
        0.0478,
        1.0,
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
