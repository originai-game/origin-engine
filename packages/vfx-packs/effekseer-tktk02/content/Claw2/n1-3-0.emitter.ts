import { file } from '@origingame/engine/author';
// Effekseer node 1.3.0 "Glow1"
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
    0.25,
    0.25
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
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        1.0,
        1.0,
        0.2353
      ]
    ],
    [
      0.1406,
      [
        1.0,
        1.0,
        1.0,
        0.2919
      ]
    ],
    [
      0.2656,
      [
        1.0,
        1.0,
        1.0,
        0.9941
      ]
    ],
    [
      0.6719,
      [
        1.0,
        1.0,
        1.0,
        0.7412
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
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
