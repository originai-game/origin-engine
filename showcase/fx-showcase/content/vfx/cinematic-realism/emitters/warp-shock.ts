import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 1,
      "cycles": 1,
      "interval": 0,
      "time": 0
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
  "distortion": 0.029999999329447746,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.6000000238418579,
    0.6000000238418579
  ],
  "looping": false,
  "maxParticles": 1,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    1.2000000476837158,
    1.2000000476837158
  ],
  "sizeOverLife": [
    [
      0,
      0.30000001192092896
    ],
    [
      1,
      4
    ]
  ],
  "speed": [
    0,
    0
  ]
});
