import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -900,
    900
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 18,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.25,
    "die": false,
    "friction": 0.6000000238418579
  },
  "color": [
    0.75,
    0.6000000238418579,
    0.4000000059604645,
    1
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
      0.8500000238418579,
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
  "drag": 0.800000011920929,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "heightOverLife": [
    [
      0,
      9
    ],
    [
      1,
      9
    ]
  ],
  "lifetime": [
    0.699999988079071,
    1.2000000476837158
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 20,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "box"
  },
  "rotation": [
    0,
    360
  ],
  "roughness": 0.800000011920929,
  "shape": {
    "kind": "cone",
    "angleDegrees": 50,
    "radius": 0.019999999552965164
  },
  "size": [
    0.0012000000569969416,
    0.003000000026077032
  ],
  "speed": [
    2,
    5
  ]
});
