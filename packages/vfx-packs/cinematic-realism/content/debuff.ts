import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 1.600000023841858,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.20000000298023224
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0.8999999761581421,
      "offset": [
        0,
        1.899999976158142,
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
        "kind": "trail",
        "color": [
          1.2000000476837158,
          0.5,
          1.7999999523162842,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/tracer-body"),
        "lifetime": 0.44999998807907104,
        "orbit": [
          0.44999998807907104,
          -1.2000000476837158
        ],
        "rise": -1.7999999523162842,
        "width": [
          0.03500000014901161,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0.8999999761581421,
      "offset": [
        0,
        1.899999976158142,
        0
      ],
      "rotate": [
        0,
        180,
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
        "kind": "trail",
        "color": [
          1.2000000476837158,
          0.5,
          1.7999999523162842,
          1
        ],
        "effect": ref("base/vfx/cinematic-realism/tracer-body"),
        "lifetime": 0.44999998807907104,
        "orbit": [
          0.44999998807907104,
          -1.2000000476837158
        ],
        "rise": -1.7999999523162842,
        "width": [
          0.03500000014901161,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 1,
      "offset": [
        0,
        0.8999999761581421,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/frost-mist")
      }
    },
    {
      "at": 0.5,
      "life": 0.5,
      "offset": [
        0,
        0.30000001192092896,
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
        "kind": "glow",
        "color": [
          0.8999999761581421,
          0.3499999940395355,
          1.399999976158142,
          0.44999998807907104
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              0.30000001192092896
            ],
            [
              0.4000000059604645,
              0.8999999761581421
            ],
            [
              1,
              0.4000000059604645
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 1.2000000476837158,
      "offset": [
        0,
        0.800000011920929,
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
        "kind": "light",
        "color": [
          0.699999988079071,
          0.4000000059604645,
          1
        ],
        "distance": 4,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              0.4000000059604645,
              1.2000000476837158
            ],
            [
              1,
              0
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
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
        "kind": "sound",
        "pitch": [
          0.550000011920929,
          0.6000000238418579
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/swell")
      }
    }
  ]
});
