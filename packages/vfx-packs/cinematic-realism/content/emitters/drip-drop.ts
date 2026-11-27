import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.03500000014901161
  },
  "blend": "alpha",
  "bursts": [
    {
      "count": 1,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0,
    "die": true,
    "friction": 0
  },
  "color": [
    1.100000023841858,
    1.149999976158142,
    1.2000000476837158,
    1
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
      0.05000000074505806,
      [
        1,
        1,
        1,
        0.8500000238418579
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0.8500000238418579
      ]
    ]
  ],
  "duration": 1.5,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.6700000166893005,
    0.6700000166893005
  ],
  "maxParticles": 4,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    0.02199999988079071,
    0.02199999988079071
  ],
  "speed": [
    0,
    0
  ],
  "subEmitters": [
    {
      "count": 9,
      "emitter": ref("base/vfx/cinematic-realism/emitters/drip-plip"),
      "inheritVelocity": 0,
      "trigger": "death"
    }
  ]
});
