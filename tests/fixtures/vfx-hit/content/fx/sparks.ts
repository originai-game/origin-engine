import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.03500000014901161
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 70,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.3499999940395355,
    "die": false,
    "friction": 0.5
  },
  "colorOverLife": [
    [
      0,
      [
        8,
        6,
        4,
        1
      ]
    ],
    [
      0.30000001192092896,
      [
        6,
        2.4000000953674316,
        0.6000000238418579,
        1
      ]
    ],
    [
      1,
      [
        2,
        0.4000000059604645,
        0.05000000074505806,
        0
      ]
    ]
  ],
  "drag": 1.2000000476837158,
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    -9,
    0
  ],
  "lifetime": [
    0.25,
    0.6499999761581421
  ],
  "looping": false,
  "maxParticles": 128,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 0.10000000149011612,
    "surface": false
  },
  "size": [
    0.05000000074505806,
    0.10000000149011612
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      1,
      0.30000001192092896
    ]
  ],
  "speed": [
    4,
    10
  ],
  "spread": 180,
  "texture": ref("base/textures/spark")
});
