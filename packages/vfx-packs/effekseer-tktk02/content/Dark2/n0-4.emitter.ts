import { file } from '@origingame/engine/author';
// Effekseer node 0.4 "Glow"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 6,
      "interval": 0.0667
    }
  ],
  "duration": 0.35,
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
    7.5,
    7.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1333
    ],
    [
      0.0781,
      0.3618
    ],
    [
      0.1719,
      0.5769
    ],
    [
      0.2656,
      0.7356
    ],
    [
      0.375,
      0.8609
    ],
    [
      0.4688,
      0.9269
    ],
    [
      0.5781,
      0.9675
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
        0.1166,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.0963,
        0.0356,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0104,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
