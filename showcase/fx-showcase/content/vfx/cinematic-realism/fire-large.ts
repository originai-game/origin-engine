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
        "asset": ref("base/vfx/cinematic-realism/emitters/flame-large")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0.6000000238418579,
        0.25,
        0.20000000298023224
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
        "asset": ref("base/vfx/cinematic-realism/emitters/flame-small")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        -0.5,
        0.25,
        -0.30000001192092896
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
        "seed": 5,
        "asset": ref("base/vfx/cinematic-realism/emitters/flame-small")
      }
    },
    {
      "at": 0,
      "life": 0,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/fire-smoke-heavy")
      }
    },
    {
      "at": 0,
      "life": 0,
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
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/cinematic-realism/emitters/fire-embers")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0.30000001192092896,
        1.399999976158142,
        0.20000000298023224
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
        "seed": 9,
        "asset": ref("base/vfx/cinematic-realism/emitters/fire-embers")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        1.7999999523162842,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/heat-rise")
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
        "kind": "glow",
        "color": [
          1.600000023841858,
          0.800000011920929,
          0.25,
          0.11999999731779099
        ],
        "size": 2.5999999046325684
      }
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        1.399999976158142,
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
        "distance": 14,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              6
            ],
            [
              0.20000000298023224,
              7.5
            ],
            [
              0.44999998807907104,
              5.5
            ],
            [
              0.699999988079071,
              7
            ],
            [
              1,
              6
            ]
          ]
        }
      }
    }
  ]
});
