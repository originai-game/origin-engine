import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 3.200000047683716,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0.800000011920929
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0.800000011920929,
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
        14,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/fw-rise")
      }
    },
    {
      "at": 0,
      "life": 0.800000011920929,
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
        14,
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
          2.4000000953674316,
          1.600000023841858,
          0.699999988079071,
          0.699999988079071
        ],
        "size": 0.3499999940395355
      }
    },
    {
      "at": 0.800000011920929,
      "life": 0,
      "offset": [
        0,
        11.199999809265137,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/fw-burst")
      }
    },
    {
      "at": 1.2999999523162842,
      "life": 0,
      "offset": [
        0,
        10.600000381469727,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/fw-glitter")
      }
    },
    {
      "at": 0.800000011920929,
      "life": 0.20000000298023224,
      "offset": [
        0,
        11.199999809265137,
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
          2.5999999046325684,
          2,
          1.2000000476837158,
          0.699999988079071
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              2
            ],
            [
              1,
              5
            ]
          ]
        }
      }
    },
    {
      "at": 0.800000011920929,
      "life": 0.6000000238418579,
      "offset": [
        0,
        11.199999809265137,
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
          1,
          0.800000011920929,
          0.5
        ],
        "distance": 30,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              20
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
          1.600000023841858,
          1.7999999523162842
        ],
        "volume": 0.5,
        "asset": ref("base/vfx/cinematic-realism/audio/whoosh")
      }
    },
    {
      "at": 0.800000011920929,
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
          1.2999999523162842,
          1.5
        ],
        "volume": 0.6000000238418579,
        "asset": ref("base/vfx/cinematic-realism/audio/blast-small")
      }
    }
  ]
});
