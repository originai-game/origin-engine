import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.03999999910593033
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 32,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.3499999940395355,
    "die": false,
    "friction": 0.30000001192092896
  },
  "colorOverLife": [
    [
      0,
      [
        2.4000000953674316,
        1.399999976158142,
        0.4000000059604645,
        1
      ]
    ],
    [
      0.25,
      [
        2,
        0.8500000238418579,
        0.15000000596046448,
        1
      ]
    ],
    [
      1,
      [
        0.4000000059604645,
        0.07999999821186066,
        0.009999999776482582,
        0
      ]
    ]
  ],
  "drag": 1.2999999523162842,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    0.25,
    0.699999988079071
  ],
  "looping": false,
  "maxParticles": 36,
  "rate": 0,
  "shape": {
    "kind": "cone",
    "angleDegrees": 65,
    "radius": 0.03999999910593033
  },
  "size": [
    0.02800000086426735,
    0.05000000074505806
  ],
  "speed": [
    2,
    5
  ],
  "trail": {
    "head": true,
    "lifetime": 0.11999999731779099,
    "width": 0.5
  }
});
