import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "blend": "alpha",
  "bursts": [
    {
      "count": 48,
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
    0.75,
    0.8500000238418579,
    0.949999988079071,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.8999999761581421
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0.699999988079071
      ]
    ]
  ],
  "drag": 0.30000001192092896,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.5,
    0.949999988079071
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 64,
  "rate": 0,
  "roughness": 0.15000000596046448,
  "round": true,
  "shape": {
    "kind": "cone",
    "angleDegrees": 22,
    "radius": 0.11999999731779099
  },
  "size": [
    0.019999999552965164,
    0.04500000178813934
  ],
  "speed": [
    2.5,
    5.5
  ],
  "spread": 12
});
