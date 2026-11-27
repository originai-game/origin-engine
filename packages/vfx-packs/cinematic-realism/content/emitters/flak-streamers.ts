import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 8,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.25,
        0.23999999463558197,
        0.23000000417232513,
        0.699999988079071
      ]
    ],
    [
      0.6000000238418579,
      [
        0.30000001192092896,
        0.28999999165534973,
        0.2800000011920929,
        0.4000000059604645
      ]
    ],
    [
      1,
      [
        0.30000001192092896,
        0.30000001192092896,
        0.30000001192092896,
        0
      ]
    ]
  ],
  "drag": 1,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -5,
    0
  ],
  "lifetime": [
    1,
    1.600000023841858
  ],
  "looping": false,
  "maxParticles": 9,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 0.20000000298023224,
    "surface": false
  },
  "size": [
    0.05000000074505806,
    0.07999999821186066
  ],
  "speed": [
    5,
    9
  ],
  "spread": 180,
  "trail": {
    "head": true,
    "lifetime": 0.699999988079071,
    "width": 1
  }
});
