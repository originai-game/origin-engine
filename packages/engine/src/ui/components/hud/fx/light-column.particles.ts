import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 1,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.15000000596046448,
      [
        1,
        1,
        1,
        0.8999999761581421
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.699999988079071
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    0,
    0
  ],
  "heightOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      0.20000000298023224,
      5
    ],
    [
      1,
      5
    ]
  ],
  "lifetime": [
    2,
    2
  ],
  "looping": false,
  "maxParticles": 2,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    140,
    140
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("@styles/shared/fx/ray.png")
});
