import { file } from '@origingame/engine/author';
// Effekseer node 0.3.5 "glow"
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
    16.0,
    16.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.0781,
      0.6318
    ],
    [
      0.1719,
      0.7559
    ],
    [
      0.2656,
      0.8475
    ],
    [
      0.375,
      0.9198
    ],
    [
      0.5781,
      0.9812
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
        0.2195,
        0.2195,
        0.502
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.1166,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
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
