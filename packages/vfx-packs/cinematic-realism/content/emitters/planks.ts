import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -360,
    360
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 14,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.30000001192092896,
    "die": false,
    "friction": 0.6000000238418579
  },
  "color": [
    0.41999998688697815,
    0.3100000023841858,
    0.20000000298023224,
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
  "drag": 0.30000001192092896,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "heightOverLife": [
    [
      0,
      5
    ],
    [
      1,
      5
    ]
  ],
  "lifetime": [
    1.600000023841858,
    2.4000000953674316
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 16,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "box"
  },
  "rotation": [
    0,
    360
  ],
  "roughness": 0.8500000238418579,
  "shape": {
    "kind": "sphere",
    "radius": 0.30000001192092896,
    "surface": false
  },
  "size": [
    0.05000000074505806,
    0.09000000357627869
  ],
  "speed": [
    2,
    5
  ],
  "spread": 180
});
