import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.05000000074505806
  },
  "bursts": [
    {
      "count": 12,
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
        1
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        1
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
  "drag": 2.4000000953674316,
  "duration": 0.10000000149011612,
  "lifetime": [
    0.6000000238418579,
    0.8999999761581421
  ],
  "looping": false,
  "maxParticles": 32,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 4,
    "surface": false
  },
  "size": [
    4,
    6
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      1,
      0.20000000298023224
    ]
  ],
  "speed": [
    90,
    260
  ],
  "spread": 180
});
