import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -700,
    700
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 76,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.30000001192092896,
    "die": false,
    "friction": 0.4000000059604645
  },
  "color": [
    0.8199999928474426,
    0.8999999761581421,
    0.9399999976158142,
    0.6000000238418579
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
      0.8999999761581421,
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
  "drag": 0.25,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "heightOverLife": [
    [
      0,
      0.30000001192092896
    ],
    [
      1,
      0.30000001192092896
    ]
  ],
  "lifetime": [
    1,
    1.7000000476837158
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 84,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "tetrahedron"
  },
  "rotation": [
    0,
    360
  ],
  "roughness": 0.05999999865889549,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.019999999552965164
    ]
  },
  "size": [
    0.014999999664723873,
    0.04500000178813934
  ],
  "speed": [
    0.800000011920929,
    3.5999999046325684
  ],
  "spread": 80
});
