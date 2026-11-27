import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 7,
      "cycles": 1,
      "interval": 0,
      "time": 0.05000000074505806
    }
  ],
  "color": [
    1,
    1,
    1,
    0
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
      0.20000000298023224,
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
  "distortion": 0.014000000432133675,
  "drag": 1,
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    1.5,
    0
  ],
  "lifetime": [
    1.2000000476837158,
    2
  ],
  "looping": false,
  "maxParticles": 8,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 2,
    "surface": false
  },
  "size": [
    5,
    7
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      1.399999976158142
    ]
  ],
  "speed": [
    0.5,
    1.2000000476837158
  ]
});
