import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2,
  "looping": true,
  "events": [],
  "params": [],
  "elements": [
    {
      "at": 0,
      "life": 0,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/flame-body")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        1.2000000476837158,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/fire-embers")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        1.5,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/wisp-up")
      }
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        1,
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
          0.550000011920929,
          0.2199999988079071
        ],
        "distance": 5,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.600000023841858
            ],
            [
              0.20000000298023224,
              2.0999999046325684
            ],
            [
              0.44999998807907104,
              1.399999976158142
            ],
            [
              0.699999988079071,
              1.899999976158142
            ],
            [
              1,
              1.600000023841858
            ]
          ]
        }
      }
    }
  ]
});
