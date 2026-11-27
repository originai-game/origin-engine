import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -360,
    360
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 22,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.25,
    "die": false,
    "friction": 0.5
  },
  "color": [
    0.800000011920929,
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
  "drag": 0.15000000596046448,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "heightOverLife": [
    [
      0,
      0.10000000149011612
    ],
    [
      1,
      0.10000000149011612
    ]
  ],
  "lifetime": [
    1.399999976158142,
    2
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 26,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "tetrahedron"
  },
  "rotation": [
    0,
    360
  ],
  "roughness": 0.05000000074505806,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.019999999552965164
    ]
  },
  "size": [
    0.07000000029802322,
    0.17000000178813934
  ],
  "speed": [
    0.4000000059604645,
    2.4000000953674316
  ],
  "spread": 60
});
