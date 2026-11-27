import { file } from '@origingame/engine/author';
// Effekseer node 2.2 "中心グロー"
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
    14.0,
    14.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2143
    ],
    [
      0.0781,
      0.4214
    ],
    [
      0.1719,
      0.6164
    ],
    [
      0.2656,
      0.7603
    ],
    [
      0.375,
      0.8739
    ],
    [
      0.4688,
      0.9337
    ],
    [
      0.5781,
      0.9705
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
        1.0,
        0.0478,
        0.0104,
        0.3922
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
