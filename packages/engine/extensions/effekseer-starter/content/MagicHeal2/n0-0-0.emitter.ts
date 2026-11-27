import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "upright",
    "axis": [
      0,
      1,
      0
    ]
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
      "cycles": 9,
      "interval": 0.16670000553131104,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.5860000252723694,
        1,
        0.5860000252723694,
        0
      ]
    ],
    [
      0.06669999659061432,
      [
        0.5860000252723694,
        1,
        0.5860000252723694,
        0.6274999976158142
      ]
    ],
    [
      0.8666999936103821,
      [
        0.5860000252723694,
        1,
        0.5860000252723694,
        0.6274999976158142
      ]
    ],
    [
      1,
      [
        0.5860000252723694,
        1,
        0.5860000252723694,
        0
      ]
    ]
  ],
  "duration": 1.350000023841858,
  "heightOverLife": [
    [
      0,
      0.550000011920929
    ],
    [
      1,
      0.550000011920929
    ]
  ],
  "lifetime": [
    2.5,
    2.5
  ],
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    2,
    2
  ],
  "speed": [
    2.799999952316284,
    2.799999952316284
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Line01")
});
