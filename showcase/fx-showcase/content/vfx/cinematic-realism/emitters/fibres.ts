import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -300,
    300
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 8,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.30000001192092896,
    0.28999999165534973,
    0.27000001072883606,
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
      0.800000011920929,
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
  "drag": 2.5,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -3,
    0
  ],
  "heightOverLife": [
    [
      0,
      6
    ],
    [
      1,
      6
    ]
  ],
  "lifetime": [
    0.800000011920929,
    1.399999976158142
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 10,
  "rate": 0,
  "render": {
    "kind": "mesh",
    "mesh": "box"
  },
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.019999999552965164
  },
  "size": [
    0.0020000000949949026,
    0.004000000189989805
  ],
  "speed": [
    0.800000011920929,
    2.4000000953674316
  ],
  "wind": 1
});
