import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 10,
  "looping": true,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 5,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        -90
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "trail",
        "color": [
          0.0012000000569969416,
          0.899399995803833,
          0.3391999900341034,
          0
        ],
        "effect": ref("base/vfx/effekseer-starter/Laser03/n0-0.effect"),
        "lifetime": 5,
        "orbit": [
          2,
          1.666700005531311
        ],
        "rise": 0,
        "width": [
          0,
          1
        ]
      }
    },
    {
      "at": 0,
      "life": 1.666700005531311,
      "offset": [
        0,
        2,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Laser03/n0-0-0.emitter")
      }
    },
    {
      "at": 0,
      "life": 5,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        120,
        0,
        -90
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "trail",
        "color": [
          0.40450000762939453,
          1,
          0.2508000135421753,
          0
        ],
        "effect": ref("base/vfx/effekseer-starter/Laser03/n1-0.effect"),
        "lifetime": 5,
        "orbit": [
          2,
          1.666700005531311
        ],
        "rise": 0,
        "width": [
          0,
          1
        ]
      }
    },
    {
      "at": 0,
      "life": 1.666700005531311,
      "offset": [
        0,
        -1,
        1.732100009918213
      ],
      "rotate": [
        120,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Laser03/n1-0-0.emitter")
      }
    },
    {
      "at": 0,
      "life": 5,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        -120,
        0,
        -90
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "trail",
        "color": [
          0.0006000000284984708,
          0.8751000165939331,
          0.9322999715805054,
          0
        ],
        "effect": ref("base/vfx/effekseer-starter/Laser03/n2-0.effect"),
        "lifetime": 5,
        "orbit": [
          2,
          1.666700005531311
        ],
        "rise": 0,
        "width": [
          0,
          1
        ]
      }
    },
    {
      "at": 0,
      "life": 1.666700005531311,
      "offset": [
        0,
        -1,
        -1.732100009918213
      ],
      "rotate": [
        -120,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Laser03/n2-0-0.emitter")
      }
    }
  ]
});
